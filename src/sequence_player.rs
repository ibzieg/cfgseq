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

use portmidi::MidiMessage;

use crate::config::{DEFAULT_VELOCITY, TICKS_PER_MEASURE};
use crate::midi;
use crate::midi::DeviceManager;
use crate::models::Instrument;

// Sequence Player ---------------------------------------------------------------------------------

pub struct SequencePlayer {
    pub instrument: Instrument,
    pub seq_name: String,
    step_index: usize,
    clock_count: usize,
    pub bar_count: usize,
    note_on_list: Vec<u8>,
}

impl SequencePlayer {
    pub fn new(inst: Instrument, seq_name: String) -> SequencePlayer {
        SequencePlayer {
            instrument: inst,
            seq_name: seq_name,
            step_index: 0,
            clock_count: 0,
            bar_count: 0,
            note_on_list: Vec::new(),
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
        for note in &self.note_on_list {
            messages.push(midi::note_off(
                self.instrument.channel - 1,
                *note,
                0,
            ));
        }
        if messages.len() > 0 {
            device_manager.write_messages(self.instrument.device.to_string(), messages);
        }
        self.note_on_list.clear();
    }

    pub fn clock(&mut self, device_manager: &mut DeviceManager) -> bool {
        // double the step length, so that we can note-off on odd steps
        let mut messages: Vec<MidiMessage> = Vec::new();

        let mut note_on_was_triggered = false;

        match self.instrument.find_sequence(&self.seq_name) {
            Some(sequence) => {
                let total_steps = sequence.steps.len() * 2;
                let ticks_per_step = TICKS_PER_MEASURE as usize / total_steps;

                if self.clock_count % ticks_per_step == 0 && self.step_index < total_steps {
                    if self.step_index % 2 == 0 {
                        let maybe_step = &sequence.steps[self.step_index / 2];
                        if maybe_step.is_some() {
                            let step = maybe_step.as_ref().unwrap().clone();
                            let mut velocity = DEFAULT_VELOCITY;
                            match step.velocity {
                                Some(v) => velocity = v,
                                _ => {}
                            }
                            match &step.pitch {
                                Some(ps) => {
                                    for p in ps {
                                        messages.push(midi::note_on(
                                            self.instrument.channel - 1,
                                            *p,
                                            velocity,
                                        ));
                                        note_on_was_triggered = true;
                                        self.note_on_list.push(*p);
                                    }
                                }
                                None => {}
                            }
                            match &step.data {
                                Some(values) => {
                                    for i in 0..values.len() {
                                        let value = values[i];

                                        match &self.instrument.data {
                                            Some(mod_devices) => {
                                                if i < mod_devices.len() {
                                                    let device = &mod_devices[i];
                                                    let message = midi::control_change(
                                                        device.channel - 1,
                                                        device.control,
                                                        value,
                                                    );
                                                    device_manager.write_messages(
                                                        device.device.to_string(),
                                                        vec![message],
                                                    );
                                                }
                                            }
                                            None => {}
                                        }
                                    }
                                }
                                None => {}
                            }
                        }
                    } else {
                        self.note_off_all(device_manager);
                    }
                    self.step_index += 1;
                }
            }
            None => {}
        }

        self.clock_count += 1;

        if messages.len() > 0 {
            device_manager.write_messages(self.instrument.device.to_string(), messages);
        }

        note_on_was_triggered
    }
}
