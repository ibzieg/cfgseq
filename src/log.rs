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
use ansi_term::{Color, Style};
use chrono::Duration;

pub fn format_duration(dur: Duration) -> String {
    let mut t = dur.num_milliseconds();

    let hr = t / (1000 * 60 * 60);
    t = t - (hr * 1000 * 60 * 60);

    let min = t / (1000 * 60);
    t = t - (min * 1000 * 60);

    let sec = t / 1000;

    let ms = (t - (sec * 1000)) % 1000;

    format!("{:0>2}:{:0>2}:{:0>2}.{:0<3.3}", hr, min, sec, ms,)
}

pub fn print_timestamp(timestamp: u128) {
    let dur = Duration::milliseconds(timestamp as i64);

    print!(
        "{}",
        Style::new()
            .bold()
            .paint(format!("[{}]\t", format_duration(dur)))
    );
}

pub fn info(text: String, timestamp: u128) {
    print_timestamp(timestamp);
    println!("{}", text);
}

pub fn event(text: String, timestamp: u128) {
    info(Color::Purple.paint(text).to_string(), timestamp);
}

pub fn success(text: String, timestamp: u128) {
    info(Color::Green.paint(text).to_string(), timestamp);
}
