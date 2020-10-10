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
use std::thread;
use std::time::Duration;

use docopt::Docopt;
use serde::Deserialize;

use crate::config::{PROJECT_NAME, VERSION};
use crate::context::Context;
use crate::midi::list_midi_devices;
use crate::controller::start_controller;

mod config;
mod context;
mod midi;
mod controller;
mod models;
mod sequence_player;
mod performance_file;

// Options -----------------------------------------------------------------------------------------

const USAGE: &'static str = "
CFG SEQ

Usage:
  cfgseq list-devices
  cfgseq [--midi-device=<device_name>] [--midi-channel=<channel_index>] [--performance=<perf_file>] [--debug]
  cfgseq (-h | --help)

Options:
  -h --help                        Show this screen.
  -d --debug                       Enable debug features
  --midi-device=<device_name>      MIDI input device name.
  --midi-channel=<channel_index>   MIDI input channel index.
  --performance=<perf_file>        Performance definition file.
";

#[derive(Debug, Deserialize)]
struct Args {
    flag_debug: bool,
    flag_midi_device: Vec<String>,
    flag_midi_channel: Option<u8>,
    flag_performance: Vec<String>,
    cmd_list_devices: bool,
}

// Main --------------------------------------------------------------------------------------------

fn main() {
    println!("{} {}\n", PROJECT_NAME, VERSION);
    run();
}

fn run() {
    let args: Args = Docopt::new(USAGE)
        .and_then(|d| d.deserialize())
        .unwrap_or_else(|e| e.exit());

    if args.cmd_list_devices {
        list_midi_devices();
    } else {
        start(&context_from_args(&args));
    }
}

fn context_from_args(args: &Args) -> Context {
    let mut context: Context = Context::new();

    context.debug = args.flag_debug;
    if context.debug {
        println!("{:?}", args);
    }

    if args.flag_performance.len() > 0 {
        context.performance = args.flag_performance[0].to_owned();
    }

    if args.flag_midi_device.len() > 0 {
        context.midi_output = args.flag_midi_device[0].to_owned();
    }
    if args.flag_midi_channel.is_some() {
        context.midi_channel = args.flag_midi_channel.unwrap();
    }

    context
}

pub fn start(context: &Context) {
    start_controller(context);

    loop {
        thread::sleep(Duration::from_millis(60_000));
    }
}
