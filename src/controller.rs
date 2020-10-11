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

use std::sync::mpsc::{channel, Receiver, Sender};
use std::time::{SystemTime, UNIX_EPOCH};
use std::{thread, time};

use crate::config::{CLOCK_MULTIPLIER, DEFAULT_PARTS_PER_QUARTER};
use crate::context::Context;
use crate::midi::{start_midi_listener};
use crate::models::{Controller};
use crate::performance::{start_performance};
use crate::log;


// Time --------------------------------------------------------------------------------------------


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

// Clock Multiplier --------------------------------------------------------------------------------

pub fn start_clock_multiplier(
    clock_recv: Receiver<u64>,
    clock_active: Receiver<bool>,
    clock_send: Sender<u64>,
) {
    let mut clock_enabled = false;
    let mut tick_duration: time::Duration = time::Duration::from_micros(0);
    let mut tick_counter: u64 = 0;

    thread::spawn(move || {
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

// Main Controller ---------------------------------------------------------------------------------

pub fn start_controller(context: &Context) {
    let debug = context.debug;

    let (midi_clock_send, midi_clock_recv): (Sender<u64>, Receiver<u64>) = channel();
    let (midi_state_send, midi_state_recv): (Sender<bool>, Receiver<bool>) = channel();
    let (clock_reset_send, clock_reset_recv): (Sender<bool>, Receiver<bool>) = channel();
    let (mult_clock_send, mult_clock_recv): (Sender<u64>, Receiver<u64>) = channel();
    let (ctrl_updated_send, ctrl_updated_recv): (Sender<Controller>, Receiver<Controller>) = channel();

    let midi_recv = start_midi_listener();

    start_clock_multiplier(midi_clock_recv, midi_state_recv, mult_clock_send);

    let mut ctrl_def: Controller = start_performance(context, clock_reset_recv, mult_clock_recv, ctrl_updated_send);

    thread::spawn(move || {


        let mut clock_count = 0;
        let mut beat_count = 0;
        let mut bar_count = 1;
        let mut last_tick = now_millis();
        let mut tick_duration_history: [f64; 48] = [0.0; 48];
        let mut clock_start_time = now_millis();

        loop {
            let (device, events) = midi_recv.recv().unwrap();

            let ctrl_updated_msg = ctrl_updated_recv.try_recv();
            if ctrl_updated_msg.is_ok() {
                log::success("UPDATE".to_string(), now_millis() - clock_start_time);
                ctrl_def = ctrl_updated_msg.unwrap();
            }


            let device_name = device.name().to_string();
            for event in events {
                if debug {
                    println!("[{}] {:?}", device, event);
                }
                let message = event.message;

                if device_name == ctrl_def.device {
                    // Global messages
                    if message.status == 248 {
                        // Timing Clock
                        let mut ppq = DEFAULT_PARTS_PER_QUARTER as usize;
                        if ctrl_def.ppq.is_some() {
                            ppq = ctrl_def.ppq.unwrap() as usize;
                        }

                        let tick = now_millis();
                        let tick_elapsed = (tick - last_tick) as f64;
                        last_tick = tick;
                        tick_duration_history[clock_count % tick_duration_history.len()] = tick_elapsed;

                        let avg_dur_ms = average(&tick_duration_history);


                        if clock_count % ppq == 0 {
                            let ms_per_beat = avg_dur_ms * (ppq as f64);
                            let ms_per_min = 60.0 * 1000.0;
                            let bpm = ms_per_min / ms_per_beat;

                            beat_count += 1;

                            if beat_count > 4 {
                                beat_count = 1;
                                bar_count += 1;

                                clock_reset_send
                                    .send(true)
                                    .expect("clock_reset_send_failed");
                            }

                            log::info(
                                format!(
                                    "[{:0>3}:{}]\tBPM={:.1}",
                                    bar_count,
                                    beat_count,
                                    bpm,
                                ),
                                now_millis() - clock_start_time,
                            );

                        }

                        midi_clock_send
                            .send((avg_dur_ms * 1000.0) as u64)
                            .expect("midi_clock_send failed");

                        clock_count += 1;
                    } else if message.status == 250 {
                        // Start
                        beat_count = 0;
                        bar_count = 1;
                        clock_count = 0;
                        last_tick = now_millis();
                        clock_start_time = now_millis();
                        log::event("START".to_string(), now_millis() - clock_start_time);
                    } else if message.status == 252 {
                        // Stop
                        clock_start_time = now_millis();
                        log::event("STOP".to_string(), now_millis() - clock_start_time);
                        midi_state_send.send(false).expect("midi_state_send failed");
                        clock_reset_send
                            .send(true)
                            .expect("clock_reset_send failed");
                    }
                }
            }
        }
    });
}
