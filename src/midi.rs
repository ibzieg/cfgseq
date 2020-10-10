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

use std::sync::mpsc;
use std::thread;
use std::time::Duration;

const MIDI_BUFFER_SIZE: usize = 1024;

pub fn list_midi_devices() {
    let context = portmidi::PortMidi::new().unwrap();

    let devices = context.devices().unwrap().into_iter().collect::<Vec<_>>();

    println!("MIDI Devices:");
    for device in devices {
        println!("\t{:?}", device)
    }
}

pub fn start_midi_channel(
) -> mpsc::Receiver<(portmidi::DeviceInfo, std::vec::Vec<portmidi::MidiEvent>)> {
    let midi_read_wait = Duration::from_millis(10);
    let context = portmidi::PortMidi::new().unwrap();
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let in_ports = context
            .devices()
            .unwrap()
            .into_iter()
            .filter_map(|dev| context.input_port(dev, MIDI_BUFFER_SIZE).ok())
            .collect::<Vec<_>>();
        loop {
            for port in &in_ports {
                if let Ok(Some(events)) = port.read_n(MIDI_BUFFER_SIZE) {
                    tx.send((port.device(), events)).unwrap();
                }
            }
            thread::sleep(midi_read_wait);
        }
    });
    rx
}

pub fn is_channel_message(status_bytes: u8) -> bool {
    status_bytes >= 128 && status_bytes <= 239
}

pub fn parse_channel(status_bytes: u8) -> u8 {
    status_bytes & 0b00001111
}

pub fn parse_status(status_bytes: u8) -> u8 {
    status_bytes & 0b011110000
}
