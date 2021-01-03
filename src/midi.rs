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

use spin_sleep;

// -------------------------------------------------------------------------------------------------

const MIDI_BUFFER_SIZE: usize = 1024;
const VERBOSE_DEBUG: bool = true;

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
                                        if VERBOSE_DEBUG { println!("{:?}", message); }
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
    let midi_read_wait = Duration::from_micros(50);
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
            spin_sleep::sleep(midi_read_wait);
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

pub fn program_change(channel: u8, program: u8) -> MidiMessage {
    MidiMessage {
        status: 0xC0 + channel,
        data1: program,
        data2: 0,
        data3: 0,
    }
}

// MIDI Data ---------------------------------------------------------------------------------------

pub fn parse_midi_note(symbol: &String) -> u8 {
    match symbol.parse::<u8>() {
        Ok(n) => n,
        Err(_) => parse_midi_note_symbol(symbol)
    }
}

pub fn parse_midi_note_symbol(symbol: &String) -> u8 {
    let mut note_index: u8 = 0;
    let mut octave_index: u8 = 0;

    symbol.chars().nth(0).map(|n| {
        symbol.chars().nth(1).map(|s| {
            let mut octave = String::from("0");
            let mut sharp = String::from("");

            if s == '#' {
                sharp = String::from("#");
                let octave_char = symbol.chars().nth(2);
                if octave_char.is_some() {
                    octave = octave_char.unwrap().to_string();
                }
            } else {
                octave = s.to_string();
            }

            let note = [n.to_string(), sharp.to_string()].join("");
            match note.as_str() {
                "C" => { note_index = 0; },
                "C#" => { note_index = 1; },
                "D" => { note_index = 2; },
                "D#" => { note_index = 3; },
                "E" => { note_index = 4; },
                "F" => { note_index = 5; },
                "F#" => { note_index = 6; },
                "G" => { note_index = 7; },
                "G#" => { note_index = 8; },
                "A" => { note_index = 9; },
                "A#" => { note_index = 10; },
                "B" => { note_index = 11; },
                _ => { note_index = 0; },
            }

            match octave.as_str() {
                "0" => { octave_index = 0; }
                "1" => { octave_index = 1; }
                "2" => { octave_index = 2; }
                "3" => { octave_index = 3; }
                "4" => { octave_index = 4; }
                "5" => { octave_index = 5; }
                "6" => { octave_index = 6; }
                "7" => { octave_index = 7; }
                "8" => { octave_index = 8; }
                "9" => { octave_index = 9; }
                _ => { octave_index = 0; }
            }
        });
    });

    note_index + octave_index * 12
}

// Tests -------------------------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use crate::midi::{parse_midi_note_symbol, parse_midi_note};

    #[test]
    fn test_parse_midi_note_symbol() {
        assert_eq!(parse_midi_note_symbol(&"A#2".to_string()), 34);
        assert_eq!(parse_midi_note_symbol(&"D5".to_string()), 62);
        assert_eq!(parse_midi_note_symbol(&"C5".to_string()), 60);
        assert_eq!(parse_midi_note_symbol(&"E5".to_string()), 64);
        assert_eq!(parse_midi_note_symbol(&"G#4".to_string()), 56);
        assert_eq!(parse_midi_note_symbol(&"x".to_string()), 0);
        assert_eq!(parse_midi_note_symbol(&"x#zw".to_string()), 0);
    }

    #[test]
    fn test_parse_midi_note() {
        assert_eq!(parse_midi_note(&"A#2".to_string()), 34);
        assert_eq!(parse_midi_note(&"D5".to_string()), 62);
        assert_eq!(parse_midi_note(&"C5".to_string()), 60);
        assert_eq!(parse_midi_note(&"E5".to_string()), 64);
        assert_eq!(parse_midi_note(&"G#4".to_string()), 56);
        assert_eq!(parse_midi_note(&"x".to_string()), 0);
        assert_eq!(parse_midi_note(&"x#zw".to_string()), 0);

        assert_eq!(parse_midi_note(&"34".to_string()), 34);
        assert_eq!(parse_midi_note(&"62".to_string()), 62);
        assert_eq!(parse_midi_note(&"60".to_string()), 60);
        assert_eq!(parse_midi_note(&"64".to_string()), 64);
        assert_eq!(parse_midi_note(&"56".to_string()), 56);
        assert_eq!(parse_midi_note(&"0".to_string()), 0);
        assert_eq!(parse_midi_note(&"111".to_string()), 111);

    }
}

