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

// Project -----------------------------------------------------------------------------------------

pub const PROJECT_NAME: &'static str = env!("CARGO_PKG_NAME");
pub const VERSION: &'static str = env!("CARGO_PKG_VERSION");

// Context  ----------------------------------------------------------------------------------------

pub const DEFAULT_MIDI_CHANNEL: i32 = 0;
