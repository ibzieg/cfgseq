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

use portmidi::PortMidi;
use portmidi::{Direction, MidiMessage};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

// -------------------------------------------------------------------------------------------------

const MIDI_BUFFER_SIZE: usize = 1024;

// -------------------------------------------------------------------------------------------------

pub struct DeviceManager {
    context: PortMidi,
}

impl DeviceManager {
    pub fn new() -> DeviceManager {
        DeviceManager {
            context: PortMidi::new().unwrap(),
        }
    }

    pub fn write_messages(&mut self, device_name: String, messages: Vec<MidiMessage>) {
        match self.context.devices() {
            Ok(devices) => {
                for device_info in devices.into_iter().collect::<Vec<_>>() {
                    if device_info.direction() == Direction::Output
                        && device_info.name() == &device_name
                    {
                        match self.context.device(device_info.id()) {
                            Ok(dev) => match self.context.output_port(dev, 1024) {
                                Ok(mut output_port) => {
                                    for message in &messages {
                                        output_port
                                            .write_message(*message)
                                            .expect("midi write_message failed");
                                    }
                                }
                                Err(e) => {
                                    println!("Failed to open MIDI output port: {}", e);
                                }
                            },
                            Err(e) => {
                                println!("Failed to open MIDI device: {}", e);
                            }
                        }
                    }
                }
            }
            Err(e) => {
                println!("Failed to get MIDI devices: {}", e);
            }
        }
    }
}

// MIDI Devices ------------------------------------------------------------------------------------

pub fn list_midi_devices() {
    let context = portmidi::PortMidi::new().unwrap();

    let devices = context.devices().unwrap().into_iter().collect::<Vec<_>>();

    println!("MIDI Devices:");
    for device in devices {
        println!("\t{:?}", device)
    }
}

// MIDI Listener -----------------------------------------------------------------------------------

pub fn start_midi_listener(
) -> mpsc::Receiver<(portmidi::DeviceInfo, std::vec::Vec<portmidi::MidiEvent>)> {
    let midi_read_wait = Duration::from_micros(500);
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

// MIDI Messages -----------------------------------------------------------------------------------

#[allow(dead_code)]
pub fn is_channel_message(status_bytes: u8) -> bool {
    status_bytes >= 128 && status_bytes <= 239
}

#[allow(dead_code)]
pub fn parse_channel(status_bytes: u8) -> u8 {
    status_bytes & 0b00001111
}

#[allow(dead_code)]
pub fn parse_status(status_bytes: u8) -> u8 {
    status_bytes & 0b011110000
}

pub fn control_change(channel: u8, control: u8, value: u8) -> MidiMessage {
    MidiMessage {
        status: 0xB0 + channel,
        data1: control,
        data2: value,
        data3: 0,
    }
}

pub fn note_on(channel: u8, pitch: u8, velocity: u8) -> MidiMessage {
    MidiMessage {
        status: 0x90 + channel,
        data1: pitch,
        data2: velocity,
        data3: 0,
    }
}

pub fn note_off(channel: u8, pitch: u8, velocity: u8) -> MidiMessage {
    MidiMessage {
        status: 0x80 + channel,
        data1: pitch,
        data2: velocity,
        data3: 0,
    }
}
