/*
 * Copyright 2020, Ian Zieg
 *
 * This file is part of a program called "specsynth"
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

use std::sync::mpsc;
use std::{thread, time};
use std::time::{SystemTime, UNIX_EPOCH};

use portmidi::{MidiMessage};

use crate::context::Context;
use crate::config::{CLOCK_MULTIPLIER};
use crate::midi::{is_channel_message, parse_channel, parse_status, start_midi_channel};
use std::os::macos::raw::mode_t;
use crate::sequence_player::{SequencePlayer, Sequence};

pub fn now_millis() -> u128 {
    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    since_the_epoch.as_millis()
}

pub fn now_micros() -> u128 {
    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    since_the_epoch.as_micros()
}

pub fn average(values: &[f64]) -> f64 {
    let mut avg = 0.0;
    for i in 0..values.len() {
        avg += values[i]
    }
    avg / values.len() as f64
}

pub fn median(numbers: &[f64]) -> f64 {
    let mut sorted: Vec<f64> = numbers.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let mid = sorted.len() / 2;
    sorted[mid]
}

pub fn interquartile_mean(values: &[f64]) -> f64 {
    let mut data: Vec<f64> = values.to_vec().into_iter().filter(|v| *v > 0.0).collect();
    data.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let interquartile: Vec<f64>;
    let mut low = 0;
    let mut high = data.len();
    if data.len() >= 4 {
        low = data.len() / 4;
        high = low * 3;
    }
    interquartile = data[low..high].to_vec();

    let mut avg = 0.0;
    for i in 0..interquartile.len() {
        avg += interquartile[i]
    }
    avg / interquartile.len() as f64
}


pub fn start_clock_multiplier(clock_recv: mpsc::Receiver<u64>, clock_active: mpsc::Receiver<bool>, clock_send: mpsc::Sender<u64>) {
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
                while clock_enabled && tick_counter < CLOCK_MULTIPLIER-1  {
                    clock_send.send(tick_counter);
                    tick_counter += 1;
                }

                clock_enabled = true;
                tick_counter = 0;
            }

            if clock_enabled && tick_counter < CLOCK_MULTIPLIER-1 {
                clock_send.send(tick_counter);
                tick_counter += 1;
                thread::sleep(tick_duration);
            } else {
                // Short sleep before polling, waiting for clock to reset
                thread::sleep((time::Duration::from_micros(500)));
            }
        }

    });
}


pub fn start_midi_controller(context: &Context) {
    let midi_recv = start_midi_channel();

    let debug = context.debug;
    let midi_output = context.midi_output.to_string();
    let midi_channel = context.midi_channel;

    let mut clock_count = 0;
    let mut beat_count = 0;
    let mut last_clock_time = now_millis();
    let mut last_tick = now_millis();
    let mut bpm_history: [f64; 4] = [0.0; 4];
    let mut tick_duration_history: [f64; 48] = [0.0; 48];



    let (midi_clock_send, midi_clock_recv): (mpsc::Sender<u64>, mpsc::Receiver<u64>) = mpsc::channel();
    let (midi_state_send, midi_state_recv): (mpsc::Sender<bool>, mpsc::Receiver<bool>) = mpsc::channel();
    let (clock_reset_send, clock_reset_recv): (mpsc::Sender<bool>, mpsc::Receiver<bool>) = mpsc::channel();
    let (mult_clock_send, mult_clock_recv): (mpsc::Sender<u64>, mpsc::Receiver<u64>) = mpsc::channel();

    start_clock_multiplier(midi_clock_recv, midi_state_recv, mult_clock_send);


    thread::spawn(move || {


        let device_index: i32 = 4;

        // initialize the PortMidi context.
        let context = portmidi::PortMidi::new().unwrap();


        let mut out_port = context.device(device_index)
            .and_then(|dev| context.output_port(dev, 1024))
            .unwrap();

        let mut player1 = SequencePlayer::new(Sequence::new(4, 36));
        let mut player2 = SequencePlayer::new(Sequence::new(3, 38));
        let mut player3 = SequencePlayer::new(Sequence::new(5, 39));

        loop {
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
                    out_port.write_message(message);
                }
            }
        }
    });


    thread::spawn(move || {
        println!("Started listening on output '{}'", midi_output);

        loop {
            let (device, events) = midi_recv.recv().unwrap();
            let device_name = device.name().to_string();
            for event in events {
                if debug {
                    println!("[{}] {:?}", device, event);
                }
                let message = event.message;

                if is_channel_message(message.status) {
                    // Channel Messages
                    let channel = parse_channel(message.status);
                    let status_type = parse_status(message.status);

                    if debug {
                        println!(
                            "Channel Event: Type={} Channel={} Data1={} Data2={}",
                            status_type, channel, message.data1, message.data2,
                        );
                    }

                    if status_type == 144 {
                        // Note On
                        if midi_output == device_name && midi_channel == channel as i32 {
                            // Play a note
                        }
                    } else if status_type == 128 {
                        // Note Off
                    } else if status_type == 176 {
                        // Control Change
                        if debug {
                            println!("CC d1={} d2={}", message.data1, message.data2);
                        }

                        if midi_output == device_name
                            && message.data1 == 10
                            && channel == midi_channel as u8
                        {

                            // Perform control change 10
                        }

                        if midi_output == device_name
                            && message.data1 == 11
                            && channel == midi_channel as u8
                        {
                            // Perform control change 11
                        }
                    }
                } else {
                    // Global messages
                    if message.status == 248 {
                        // Timing Clock
                        let ppq = 24;

                        let tick = now_millis();
                        let tick_elapsed = (tick - last_tick) as f64;
                        last_tick = tick;
                        tick_duration_history[clock_count % tick_duration_history.len()] = tick_elapsed;



                        let avg_dur_ms = average(&tick_duration_history);

                        midi_clock_send.send((avg_dur_ms * 1000.0) as u64);

                        if clock_count % ppq == 0 {

                            let ms_per_beat = avg_dur_ms * (ppq as f64);
                            let ms_per_min = 60.0 * 1000.0;
                            let bpm = ms_per_min / ms_per_beat;

                            print!("{}\t",event.timestamp);
                            if beat_count > 0 {
                                print!("\t")
                            }
                            println!("beat_count={} bpm={:.1} avg_ms={:.2}", beat_count, bpm, avg_dur_ms);
                            beat_count = (beat_count + 1) % 4;

                            if beat_count == 0 {
                                clock_reset_send.send(true);
                            }
                        }



                        clock_count += 1;


                        // if clock_count % (ppq as usize / sync) == 0 {
                        //
                        // }
                    } else if message.status == 250 {
                        // Start
                        beat_count = 0;
                        clock_count = 0;
                        last_tick = now_millis();
                        print!("{}\t",event.timestamp);
                        println!("START")
                    } else if message.status == 252 {
                        // Stop
                        print!("{}\t",event.timestamp);
                        println!("STOP");
                        midi_state_send.send(false);
                        clock_reset_send.send(true);
                    }
                }
            }
        }
    });

}
