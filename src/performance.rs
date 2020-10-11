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
use std::ffi::OsStr;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;
use std::collections::HashMap;

use portmidi::MidiMessage;

use crate::context::Context;
use crate::models::{Performance, Controller};
use crate::performance_file::{load_performance_file, start_file_watcher};
use crate::sequence_player::{SequencePlayer, Instrument, Sequence};


pub fn start_performance(
    context: &Context,
    clock_reset_recv: Receiver<bool>,
    mult_clock_recv: Receiver<u64>,
    ctrl_updated: Sender<Controller>,
) -> Controller {
    let mut perf: Performance = Performance::new();
    match load_performance_file(OsStr::new(&context.performance.to_owned())) {
        Ok(initial_perf) => perf = initial_perf,
        Err(e) => {
            println!("Failed to load file: {}", e);
        }
    }

    let ctrl: Controller = perf.controller.clone();

    let (perf_update_send, perf_update_recv): (Sender<Performance>, Receiver<Performance>) =
        channel();

    start_file_watcher(&context.performance.to_owned(), perf_update_send);

    thread::spawn(move || {
        let device_index: i32 = 4;

        let mut scene_index = 0;
        let mut clock_count = 0;
        let mut bar_count = 0;

        let players: HashMap<String, SequencePlayer> = HashMap::new();

        // initialize the PortMidi context.
        let context = portmidi::PortMidi::new().unwrap();

        let mut out_port = context
            .device(device_index)
            .and_then(|dev| context.output_port(dev, 1024))
            .unwrap();



        loop {
            let perf_update = perf_update_recv.try_recv();
            if perf_update.is_ok() {
                perf = perf_update.unwrap();
                ctrl_updated.send(perf.controller.clone()).unwrap();
            }
            let reset_msg = clock_reset_recv.try_recv();
            if reset_msg.is_ok() {
                // Advance the players
                println!("reset");
                if clock_count == 0 {
                    scene_index = 0;
                    bar_count = 0;
                } else {
                    bar_count += 1;
                }
                clock_count = 0;

                let scene_name = &perf.playlist[scene_index % perf.playlist.len()];

                let found_scene = perf.find_scene(scene_name);
                if found_scene.is_some() {
                    let scene = found_scene.unwrap();
                    for track in &scene.tracks {
                        let found_inst = perf.find_instrument(&track.instrument);
                        if found_inst.is_some() {
                            let inst = found_inst.unwrap();
                            // If the inst is the Master, and bar_count > track.play.len(), then advance the scene
                            let seq_name = &track.play[bar_count % track.play.len()];
                            let found_seq = inst.find_sequence(seq_name);
                            if found_seq.is_some() {
                                let seq = found_seq.unwrap();
                                println!("{}:{}", &track.instrument, seq.name);
                                // Set the player here, if it's not a follower
                            }
                        }
                    }
                }


            }
            let clock_msg = mult_clock_recv.try_recv();
            if clock_msg.is_ok() {
                clock_count += 1;

                let mut messages: Vec<MidiMessage> = Vec::new();
                // messages.append(player1.clock().as_mut());
                // If any tracks are follows of this one, then reset their Player


                for message in messages {
                    out_port
                        .write_message(message)
                        .expect("midi write_message failed");
                }
            }
        }
    });

    ctrl
}

