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
use docopt::Docopt;
use serde::Deserialize;

use crate::config::{PROJECT_NAME, VERSION};
use crate::context::Context;
use crate::midi::list_midi_devices;
use crate::sequencer::play_sequencer;

mod config;
mod context;
mod midi;
mod midi_controller;
mod sequencer;
mod sequence_player;

// Options -----------------------------------------------------------------------------------------

const USAGE: &'static str = "
CFG SEQ

Usage:
  specsynth list-devices
  specsynth [--midi-device=<device_name>] [--midi-channel=<channel_index>] [--debug]
  specsynth (-h | --help)

Options:
  -h --help                        Show this screen.
  -d --debug                       Enable debug features
  --midi-device=<device_name>      MIDI input device name.
  --midi-channel=<channel_index>   MIDI input channel index.
";

#[derive(Debug, Deserialize)]
struct Args {
    flag_debug: bool,
    flag_midi_device: Vec<String>,
    flag_midi_channel: Option<i32>,
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
        play_sequencer(&context_from_args(&args));
    }
}

fn context_from_args(args: &Args) -> Context {
    let mut context: Context = Context::new();

    context.debug = args.flag_debug;
    if context.debug {
        println!("{:?}", args);
    }

    if args.flag_midi_device.len() > 0 {
        context.midi_output = args.flag_midi_device[0].to_owned();
    }
    if args.flag_midi_channel.is_some() {
        context.midi_channel = args.flag_midi_channel.unwrap();
    }

    context
}
