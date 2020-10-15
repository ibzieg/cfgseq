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
use std::collections::HashMap;
use std::ffi::OsStr;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;

use crate::context::Context;
use crate::midi::DeviceManager;
use crate::models::{Controller, Performance};
use crate::performance_file::{load_performance_file, start_file_watcher};
use crate::sequence_player::SequencePlayer;

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
        let mut scene_index = 0;
        let mut clock_count = 0;
        let mut bar_count = 0;

        let mut players: HashMap<String, SequencePlayer> = HashMap::new();
        let mut device_manager = DeviceManager::new();

        loop {
            let perf_update = perf_update_recv.try_recv();
            if perf_update.is_ok() {
                perf = perf_update.unwrap();
                ctrl_updated.send(perf.controller.clone()).unwrap();
            }
            let reset_msg = clock_reset_recv.try_recv();
            if reset_msg.is_ok() {
                // Advance the players
                if clock_count == 0 {
                    scene_index = 0;
                    bar_count = 0;
                } else {
                    bar_count += 1;
                }
                clock_count = 0;

                let scene_name = &perf.playlist[scene_index % perf.playlist.len()];

                match perf.find_scene(scene_name) {
                    Some(scene) => {
                        for track in &scene.tracks {
                            // TODO: If this track is a follower, do nothing
                            match perf.find_instrument(&track.instrument) {
                                Some(inst) => {
                                    // TODO: If the inst is the Master, and bar_count > track.play.len(), then advance the scene
                                    let seq_name =
                                        track.play[bar_count % track.play.len()].to_string();
                                    players.insert(
                                        inst.name.to_string(),
                                        SequencePlayer::new(inst.clone(), seq_name),
                                    );
                                }
                                None => println!("No instrument called '{}'", &track.instrument),
                            }
                        }
                    }
                    None => println!("No scene called '{}'", scene_name),
                }
            }
            let clock_msg = mult_clock_recv.try_recv();
            if clock_msg.is_ok() {
                clock_count += 1;
                let scene_name = &perf.playlist[scene_index % perf.playlist.len()];
                match perf.find_scene(scene_name) {
                    Some(scene) => {
                        for track in &scene.tracks {
                            match players.get_mut(&track.instrument) {
                                Some(player) => {
                                    player.clock(&mut device_manager);
                                }
                                None => {}
                            }
                        }
                    }
                    None => println!("No scene called '{}'", scene_name),
                }
            }
        }
    });

    ctrl
}
