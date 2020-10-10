/*
 * Copyright 2020, Ian Zieg
 *
 * This file is part of a program called "cfgseq"
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */
extern crate portmidi;

use std::sync::mpsc::{Sender, Receiver, channel};
use std::time::{SystemTime, UNIX_EPOCH};
use std::{thread, time};
use std::ffi::OsStr;

use ansi_term::{Color, Style};
use portmidi::MidiMessage;

use crate::config::CLOCK_MULTIPLIER;
use crate::context::Context;
use crate::midi::{is_channel_message, parse_channel, parse_status, start_midi_channel};
use crate::sequence_player::{Instrument, Sequence, SequencePlayer};
use crate::performance_file::{load_performance_file, start_file_watcher};
use crate::models::{Performance};

pub fn now_millis() -> u128 {
    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    since_the_epoch.as_millis()
}

pub fn average(values: &[f64]) -> f64 {
    let mut avg = 0.0;
    for i in 0..values.len() {
        avg += values[i]
    }
    avg / values.len() as f64
}

pub fn start_clock_multiplier(
    clock_recv: Receiver<u64>,
    clock_active: Receiver<bool>,
    clock_send: Sender<u64>,
) {
    let mut clock_enabled = false;
    let mut tick_duration: time::Duration = time::Duration::from_micros(0);
    let mut tick_counter: u64 = 0;

    thread::spawn(move || {
        println!("start multiplier");
        loop {
            let clock_active_msg = clock_active.try_recv();
            if clock_active_msg.is_ok() {
                clock_enabled = clock_active_msg.unwrap();
            }

            let clock_msg = clock_recv.try_recv();
            if clock_msg.is_ok() {
                let clock_duration = clock_msg.unwrap();
                let micros = clock_duration / CLOCK_MULTIPLIER;
                tick_duration = time::Duration::from_micros(micros);

                // flush remaining ticks in case the clock arrived sooner than expected
                while clock_enabled && tick_counter < CLOCK_MULTIPLIER - 1 {
                    clock_send
                        .send(tick_counter)
                        .expect("clock_multiplier send tick failed");
                    tick_counter += 1;
                }

                clock_enabled = true;
                tick_counter = 0;
            }

            if clock_enabled && tick_counter < CLOCK_MULTIPLIER - 1 {
                clock_send
                    .send(tick_counter)
                    .expect("clock_multiplier send tick failed");
                tick_counter += 1;
                thread::sleep(tick_duration);
            } else {
                // Short sleep before polling, waiting for clock to reset
                thread::sleep(time::Duration::from_micros(100));
            }
        }
    });
}

pub fn start_performance(
    context: &Context,
    clock_reset_recv: Receiver<bool>,
    mult_clock_recv: Receiver<u64>,
) {
    let mut perf: Performance = Performance::new();
    match load_performance_file(OsStr::new(&context.performance.to_owned())) {
        Ok(initial_perf) => perf = initial_perf,
        Err(e) => {
            println!("Failed to load file: {}", e);
        }
    }

    let (perf_update_send, perf_update_recv): (Sender<Performance>, Receiver<Performance>) = channel();



    thread::spawn(move || {
        let device_index: i32 = 4;

        // initialize the PortMidi context.
        let context = portmidi::PortMidi::new().unwrap();

        let mut out_port = context
            .device(device_index)
            .and_then(|dev| context.output_port(dev, 1024))
            .unwrap();

        let mut player1 = SequencePlayer::new(Instrument::new(13), Sequence::new(4, 36));
        player1
            .sequence
            .set_steps(vec![36, 0, 0, 0, 0, 0, 36, 0, 0, 36, 0, 0, 0, 0, 36, 0]);

        let mut player2 = SequencePlayer::new(Instrument::new(13), Sequence::new(3, 38));
        player2
            .sequence
            .set_steps(vec![0, 0, 0, 0, 38, 0, 0, 0, 0, 0, 0, 0, 38, 0, 0, 0]);

        let mut player3 = SequencePlayer::new(Instrument::new(13), Sequence::new(9, 39));
        player3.sequence.set_steps(vec![
            39, 39, 39, 42, 0, 39, 0, 0, 0, 0, 39, 39, 42, 0, 0, 39, 0, 0, 0, 0,
        ]);

        loop {
            let perf_update = perf_update_recv.try_recv();
            if perf_update.is_ok() {

                println!("perf update");
            }
            let reset_msg = clock_reset_recv.try_recv();
            if reset_msg.is_ok() {
                player1.reset();
                player2.reset();
                player3.reset();
            }
            let clock_msg = mult_clock_recv.try_recv();
            if clock_msg.is_ok() {
                // println!("\t{} mult tick", clock_msg.unwrap());

                let mut messages: Vec<MidiMessage> = Vec::new();
                messages.append(player1.clock().as_mut());
                messages.append(player2.clock().as_mut());
                messages.append(player3.clock().as_mut());

                for message in messages {
                    out_port
                        .write_message(message)
                        .expect("midi write_message failed");
                }
            }
        }
    });

    start_file_watcher(&context.performance.to_owned(), perf_update_send);
}

pub fn start_controller(context: &Context) {
    let midi_recv = start_midi_channel();

    let debug = context.debug;
    let midi_output = context.midi_output.to_string();
    let midi_channel = context.midi_channel;

    let mut clock_count = 0;
    let mut beat_count = 0;
    let mut last_tick = now_millis();
    let mut tick_duration_history: [f64; 48] = [0.0; 48];

    let (midi_clock_send, midi_clock_recv): (Sender<u64>, Receiver<u64>) =
        channel();
    let (midi_state_send, midi_state_recv): (Sender<bool>, Receiver<bool>) =
        channel();
    let (clock_reset_send, clock_reset_recv): (Sender<bool>, Receiver<bool>) =
        channel();
    let (mult_clock_send, mult_clock_recv): (Sender<u64>, Receiver<u64>) =
        channel();

    start_clock_multiplier(midi_clock_recv, midi_state_recv, mult_clock_send);

    start_performance(context, clock_reset_recv, mult_clock_recv);

    thread::spawn(move || {
        println!("Started listening on output '{}'", midi_output);

        fn print_timestamp(timestamp: u32) {
            print!(
                "{}",
                Style::new().bold().paint(format!("[{:0>8}]\t", timestamp))
            );
        }

        fn log(text: String, timestamp: u32) {
            print_timestamp(timestamp);
            println!("{}", text);
        }

        loop {
            let (device, events) = midi_recv.recv().unwrap();
            // let device_name = device.name().to_string();
            for event in events {
                if debug {
                    println!("[{}] {:?}", device, event);
                }
                let message = event.message;

                // Global messages
                if message.status == 248 {
                    // Timing Clock
                    let ppq = 24;

                    let tick = now_millis();
                    let tick_elapsed = (tick - last_tick) as f64;
                    last_tick = tick;
                    tick_duration_history[clock_count % tick_duration_history.len()] =
                        tick_elapsed;

                    let avg_dur_ms = average(&tick_duration_history);

                    midi_clock_send
                        .send((avg_dur_ms * 1000.0) as u64)
                        .expect("midi_clock_send failed");

                    if clock_count % ppq == 0 {
                        let ms_per_beat = avg_dur_ms * (ppq as f64);
                        let ms_per_min = 60.0 * 1000.0;
                        let bpm = ms_per_min / ms_per_beat;

                        log(
                            format!(
                                "{}beat_count={}\tbpm={:.1} clock={:.2}ms",
                                " ".repeat(beat_count),
                                beat_count,
                                bpm,
                                avg_dur_ms
                            ),
                            event.timestamp,
                        );
                        beat_count = (beat_count + 1) % 4;

                        if beat_count == 0 {
                            clock_reset_send
                                .send(true)
                                .expect("clock_reset_send_failed");
                        }
                    }

                    clock_count += 1;
                } else if message.status == 250 {
                    // Start
                    beat_count = 0;
                    clock_count = 0;
                    last_tick = now_millis();
                    log(Color::Purple.paint("START").to_string(), event.timestamp);
                } else if message.status == 252 {
                    // Stop
                    log(Color::Cyan.paint("STOP").to_string(), event.timestamp);
                    midi_state_send.send(false).expect("midi_state_send failed");
                    clock_reset_send
                        .send(true)
                        .expect("clock_reset_send failed");
                }
            }
        }
    });



}
