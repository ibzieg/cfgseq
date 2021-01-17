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

use std::collections::HashMap;

use portmidi::MidiMessage;

use crate::config::{DEFAULT_VELOCITY, TICKS_PER_MEASURE};
use crate::midi;
use crate::midi::{DeviceManager, parse_midi_note};
use crate::models::{Instrument};


const DEFAULT_PLAY_RATE: u8 = 4; // Quarter note

// ActiveNoteMap

pub type ActiveNoteMap = HashMap<u8, usize>;

// impl Clone for ActiveNoteMap {
//     fn clone(&self) -> ActiveNoteMap {
//         let mut result = ActiveNoteMap::new();
//         for (note, ticks) in self {
//             result.insert(note, ticks);
//         }
//         result
//     }
// }

// Player Clock Result -----------------------------------------------------------------------------

pub struct PlayerClockResult {
    pub note_was_played:  bool,
    pub sequence_ended: bool,
}

impl PlayerClockResult {
    pub fn new() -> PlayerClockResult {
        PlayerClockResult {
            note_was_played: false,
            sequence_ended: false
        }
    }
}

// Sequence Player ---------------------------------------------------------------------------------

pub struct SequencePlayer {
    pub instrument: Instrument,
    pub seq_name: String,
    step_index: usize,
    clock_count: usize,
    pub bar_count: usize,
    note_on_map: ActiveNoteMap,
}

impl SequencePlayer {
    pub fn new(inst: Instrument, seq_name: String) -> SequencePlayer {
        SequencePlayer {
            instrument: inst,
            seq_name: seq_name,
            step_index: 0,
            clock_count: 0,
            bar_count: 0,
            note_on_map: ActiveNoteMap::new(),
        }
    }

    #[allow(dead_code)]
    pub fn reset(&mut self) {
        self.step_index = 0;
        self.clock_count = 0;
        self.bar_count = 0;
    }

    pub fn next_bar(&mut self, device_manager: &mut DeviceManager) {
        self.step_index = 0;
        self.clock_count = 0;
        self.bar_count += 1;
        self.note_off_all(device_manager);
    }

    pub fn note_off_all(&mut self, device_manager: &mut DeviceManager) {
        let mut messages: Vec<MidiMessage> = Vec::new();
        for (note, _) in &self.note_on_map {
            messages.push(midi::note_off(self.instrument.channel - 1, *note, 0));
        }
        if messages.len() > 0 {
            device_manager.write_messages(self.instrument.device.to_string(), messages);
        }
        self.note_on_map.clear();
    }

    pub fn clock(&mut self, device_manager: &mut DeviceManager) -> PlayerClockResult {
        let mut result = PlayerClockResult::new();

        let mut messages: Vec<MidiMessage> = Vec::new();

        let seq_name = self.seq_name.to_owned();
        let instrument = &self.instrument;
        let note_on_map = &mut self.note_on_map;
        let inst_channel = instrument.channel - 1;

        // Clear any Note-Off events first
        let mut note_off_messages: Vec<MidiMessage> = Vec::new();
        let note_off_map = note_on_map.clone();
        for (note, ticks) in note_off_map {
            if ticks > 0 {
                if ticks == 1 {
                    // Execute Note-Off
                    note_off_messages.push(midi::note_off(self.instrument.channel - 1, note, 0));
                }
                // Decrement
                note_on_map.insert(note, ticks - 1);
            }
        }
        if note_off_messages.len() > 0 {
            device_manager.write_messages(self.instrument.device.to_string(), note_off_messages);
        }


        for sequence in instrument.sequences.iter().filter(|s| s.name == seq_name) {
            let total_steps = sequence.steps.len();

            let rate = sequence.rate.unwrap_or(DEFAULT_PLAY_RATE);

            let ticks_per_step = TICKS_PER_MEASURE as usize / rate as usize;

            // Execute current step
            if self.clock_count % ticks_per_step == 0 && self.step_index < total_steps {
                let maybe_step = &sequence.steps[self.step_index];
                maybe_step.as_ref().map(|step| {
                    // Program Change
                    step.program.as_ref().map(|p| {
                        messages.push(midi::program_change(
                            inst_channel,
                            *p
                        ));
                    });

                    // Note-On
                    let mut velocity = DEFAULT_VELOCITY;
                    step.velocity.as_ref().map(|value| velocity = parse_midi_note(&value));

                    step.pitch.as_ref().map(|notes| {
                        let duration_rate = step.duration.unwrap_or(rate);
                        let duration_ticks = TICKS_PER_MEASURE as usize / duration_rate as usize;
                        for note in notes {
                            let p = parse_midi_note(note);
                            messages.push(midi::note_on(
                                inst_channel,
                                p,
                                velocity,
                            ));
                            result.note_was_played = true;
                            note_on_map.insert(p, duration_ticks);
                        }
                    });

                    // Control Changes
                    step.data.as_ref().map(|values| {
                        for i in 0..values.len() {
                            let value = values[i];

                            instrument.data.as_ref().map(|mod_devices| {
                                if i < mod_devices.len() {
                                    let device = &mod_devices[i];
                                    let message = midi::control_change(
                                        device.channel.to_owned() - 1,
                                        device.control,
                                        value,
                                    );
                                    device_manager.write_messages(
                                        device.device.to_string(),
                                        vec![message],
                                    );
                                }
                            });
                        }
                    });
                });

                self.step_index += 1;
            }

            if self.clock_count >= (ticks_per_step * total_steps) - 1 {
                result.sequence_ended = true;
            }
        }

        self.clock_count += 1;


        if messages.len() > 0 {
            device_manager.write_messages(self.instrument.device.to_string(), messages);
        }

        result
    }
}
