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
use std::fs;
use std::sync::mpsc::Sender;
use std::thread;
use std::time::Duration;

use notify::event::DataChange::Content;
use notify::event::ModifyKind::Data;
use notify::{RecommendedWatcher, RecursiveMode, Watcher};

use crate::models::Performance;

// Performance File --------------------------------------------------------------------------------

pub fn load_performance_file(file_path: &OsStr) -> Result<Performance, std::io::Error> {
    match fs::read_to_string(file_path) {
        Ok(yaml_text) => match serde_yaml::from_str::<Performance>(&yaml_text) {
            Ok(perf) => Ok(perf),
            Err(e) => Err(std::io::Error::new(std::io::ErrorKind::Other, e)),
        },
        Err(e) => Err(std::io::Error::new(std::io::ErrorKind::Other, e)),
    }
}

pub fn start_file_watcher(file_path: &String, perf_send: Sender<Performance>) {
    let watch_file_path = file_path.to_owned();
    thread::spawn(|| {
        let mut watcher: RecommendedWatcher =
            Watcher::new_immediate(move |res: Result<notify::Event, notify::Error>| match res {
                Ok(event) => {
                    if event.kind == notify::EventKind::Modify(Data(Content)) {
                        match load_performance_file(event.paths[0].as_os_str()) {
                            Ok(perf) => perf_send.send(perf).unwrap(),
                            Err(e) => println!("Error parsing file: {}", e),
                        }
                    }
                }
                Err(e) => println!("watch error: {:?}", e),
            })
            .expect("failed to create watcher");

        watcher
            .watch(watch_file_path, RecursiveMode::Recursive)
            .unwrap();
        loop {
            // Keep the thread running so that we can watch the file indefinitely
            thread::sleep(Duration::from_millis(60_000));
        }
    });
}
