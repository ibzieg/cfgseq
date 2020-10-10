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

use std::{thread, time};

// use std::f32::consts::PI;
// use std::sync::mpsc;

use crate::context::Context;
use crate::midi_controller::start_midi_controller;

pub fn play_sequencer(context: &Context) {
    // let (factors_send, factors_recv): (mpsc::Sender<AudioFactors>, mpsc::Receiver<AudioFactors>) =
    //     mpsc::channel();

    start_midi_controller(context);

    let done = false;
    let mut minute_count = 0;
    while !done {
        let ten_millis = time::Duration::from_millis(60_000);
        thread::sleep(ten_millis);

        minute_count += 1;
        println!("Playing for {} minutes.", minute_count);
    }
}
