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

// use std::sync::mpsc;
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::context::Context;

use crate::midi::{is_channel_message, parse_channel, parse_status, start_midi_channel};

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

pub fn start_midi_controller(context: &Context) {
    let midi_recv = start_midi_channel();

    let debug = context.debug;
    let midi_output = context.midi_output.to_string();
    let midi_channel = context.midi_channel;

    let mut clock_count = 0;
    let mut beat_count = 0;
    let mut last_clock_time = now_millis();
    let mut bpm_history: [f64; 4] = [0.0; 4];

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

                        clock_count += 1;
                        let ppq = 24;
                        let sync = 2;

                        if clock_count % ppq == 0 {
                            beat_count += 1;
                            let now = now_millis();
                            let elapsed = now - last_clock_time;
                            last_clock_time = now;

                            let ms_per_beat = elapsed as f64;
                            let ms_per_min = 60.0 * 1000.0;
                            let bpm = ms_per_min / ms_per_beat;

                            bpm_history[beat_count % bpm_history.len()] = bpm;
                        }

                        if clock_count % (ppq as usize / sync) == 0 {
                            println!(" BPM={:.1}", average(&bpm_history));
                        }
                    } else if message.status == 250 {
                        // Start
                    } else if message.status == 252 {
                        // Stop
                    }
                }
            }
        }
    });
}
