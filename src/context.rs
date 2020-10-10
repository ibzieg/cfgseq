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
use crate::config::DEFAULT_MIDI_CHANNEL;

pub struct Context {
    pub midi_channel: u8,
    pub midi_output: String,
    pub performance: String,
    pub debug: bool,
}

impl Context {
    pub fn new() -> Context {
        Context {
            midi_channel: DEFAULT_MIDI_CHANNEL,
            midi_output: String::new(),
            performance: String::new(),
            debug: false,
        }
    }
}
