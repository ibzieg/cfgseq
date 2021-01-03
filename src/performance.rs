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
use std::time::{Duration};

use crate::context::Context;
use crate::midi::DeviceManager;
use crate::models::{Controller, Performance};
use crate::performance_file::{load_performance_file, start_file_watcher};
use crate::sequence_player::SequencePlayer;
use crate::config::{TICKS_PER_MEASURE};
use crate::log;

pub fn start_performance(
    context: &Context,
    clock_reset_recv: Receiver<bool>,
    mult_clock_recv: Receiver<u64>,
    ctrl_updated: Sender<Controller>,
) -> Controller {
    let mut perf =
        load_performance_file(OsStr::new(&context.performance.to_owned()))
            .expect("Failed to load file");

    let ctrl: Controller = perf.controller.clone();

    let (perf_update_send, perf_update_recv): (Sender<Performance>, Receiver<Performance>) =
        channel();

    start_file_watcher(&context.performance.to_owned(), perf_update_send);

    thread::spawn(move || {
        let mut perf_ctrl = PerformanceController::new(perf);

        let wait_dur = Duration::from_micros(1);

        loop {
            let perf_update = perf_update_recv.try_recv();
            if perf_update.is_ok() {
                perf = perf_update.unwrap();
                ctrl_updated.send(perf.controller.clone()).unwrap();
                perf_ctrl.update_def(perf);
            }
            let reset_msg = clock_reset_recv.try_recv();
            if reset_msg.is_ok() {
                perf_ctrl.reset();
            }
            let clock_msg = mult_clock_recv.try_recv();
            if clock_msg.is_ok() {
                // let now = SystemTime::now();
                perf_ctrl.clock();
                // println!("elapsed={}us", now.elapsed().unwrap().as_micros());
            }
            thread::sleep(wait_dur);
        }
    });

    ctrl
}

// PerformanceController ---------------------------------------------------------------------------

struct PerformanceController {
    scene_index: usize,
    clock_count: usize,
    bar_count: usize,
    players: HashMap<String, SequencePlayer>,
    perf: Performance,
    device_manager: DeviceManager
}

impl PerformanceController {
    pub fn new(perf: Performance) -> PerformanceController {
        let mut perf_ctrl = PerformanceController {
            scene_index: 0,
            clock_count: 0,
            bar_count: 0,
            players: HashMap::new(),
            perf,
            device_manager: DeviceManager::new()
        };
        perf_ctrl.reset();
        perf_ctrl
    }

    pub fn update_def(&mut self, def: Performance) {
        self.perf = def;

        let playlist_index = self.scene_index % self.perf.playlist.len();
        let scene_name = &self.perf.playlist[playlist_index].to_string();

        let bar_count = self.bar_count.to_owned();

        for scene in self.perf.scenes.iter().filter(|s| &s.name == scene_name) {
            for track in scene.tracks.iter() {
                for inst in self.perf.instruments.iter().filter(|i| &i.name == &track.instrument) {
                    match self.players.get_mut(&track.instrument) {
                        Some(player) => {
                            player.instrument = inst.clone();
                            if track.follow.is_none() {
                                player.seq_name =
                                    track.play[bar_count % track.play.len()].to_string();
                            } else {
                                // Followers use their own bar count instead
                                player.seq_name =
                                    track.play[player.bar_count % track.play.len()].to_string();
                            }
                        },
                        None => {
                            let seq_name =
                                track.play[self.bar_count % track.play.len()].to_string();
                            self.players.insert(
                                inst.name.to_string(),
                                SequencePlayer::new(inst.clone(), seq_name)
                            );
                        }
                    }
                }
            }
        }
    }

    pub fn reset(&mut self) {
        self.clock_count = 0;
        self.scene_index = 0;
        self.bar_count = 0;
        self.init_scene();
    }

    pub fn init_scene(&mut self) {
        let playlist_index = self.scene_index % self.perf.playlist.len();
        let scene_name = &self.perf.playlist[playlist_index].to_string();

        for scene in self.perf.scenes.iter().filter(|s| &s.name == scene_name) {
            for track in &scene.tracks {
                for inst in self.perf.instruments.iter().filter(|i| &i.name == &track.instrument) {
                    let seq_name =
                        track.play[self.bar_count % track.play.len()].to_string();
                    self.players.insert(
                        inst.name.to_string(),
                        SequencePlayer::new(inst.clone(), seq_name)
                    );
                }
            }
        }
    }

    pub fn next_bar(&mut self) {
        self.bar_count += 1;

        let playlist_index = self.scene_index % self.perf.playlist.len();
        let scene_name = &self.perf.playlist[playlist_index].to_string();
        let mut next_scene = false;

        for scene in self.perf.scenes.iter().filter(|s| &s.name == scene_name) {
            let first_track = &scene.tracks[0];
            if self.bar_count >= first_track.play.len() {
                next_scene = true;
            }
        }

        if next_scene {
            self.scene_index += 1;
            self.bar_count = 0;
            self.init_scene();
        }

        let playlist_index = self.scene_index % self.perf.playlist.len();
        let scene_name = &self.perf.playlist[playlist_index].to_string();


        let bar_count = self.bar_count.to_owned();

        // Advance non-follower sequence players
        for scene in self.perf.scenes.iter().filter(|s| &s.name == scene_name) {
            for track in scene.tracks.iter().filter(|t| t.follow.is_none()) {
                self.players.get_mut(&track.instrument).map(|player| {
                    player.reset();
                    player.seq_name =
                        track.play[bar_count  % track.play.len()].to_string();
                });
            }
        }

        if next_scene {
            log::event(format!("SCENE {}/{} \"{}\" ", self.scene_index, self.perf.playlist.len(), scene_name), 0);
        }
    }

    pub fn clock(&mut self) {

        if self.clock_count % TICKS_PER_MEASURE as usize == 0 {
            // println!("ticker per bar = {}", self.clock_count);
            self.clock_count = 0;
            self.next_bar();
        }

        let playlist_index = self.scene_index % self.perf.playlist.len();
        let scene_name = &self.perf.playlist[playlist_index].to_string();

        let mut device_manager = &mut self.device_manager;

        for scene in self.perf.scenes.iter().filter(|s| &s.name == scene_name) {
            let mut note_played: HashMap<String, bool> = HashMap::new();
            // First clock all the non-followers
            for track in scene.tracks.iter().filter(|t| t.follow.is_none()) {
                self.players.get_mut(&track.instrument).map(|player| {
                    note_played.insert(track.instrument.to_string(), player.clock(&mut device_manager));
                });
            }
            // Then clock all the followers
            for track in scene.tracks.iter().filter(|t| t.follow.is_some()) {
                self.players.get_mut(&track.instrument).map(|player| {
                    let follow_name = track.follow.as_ref().unwrap_or(&String::from("")).to_string();
                    let note_was_played = note_played.get(&follow_name).unwrap_or(&false);
                    if *note_was_played {
                        player.next_bar(&mut device_manager);
                        player.seq_name =
                            track.play[player.bar_count  % track.play.len()].to_string();
                    }
                    player.clock(&mut device_manager);
                });
            }
        }

        self.clock_count += 1;
    }
}
