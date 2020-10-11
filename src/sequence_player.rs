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

use crate::models::{Instrument, Sequence};
use crate::config::{TICKS_PER_MEASURE, DEFAULT_VELOCITY};
use crate::midi;

// Sequence Player ---------------------------------------------------------------------------------

pub struct SequencePlayer {
    pub instrument: Instrument,
    pub seq_name: String,
    step_index: usize,
    clock_count: usize,
}

impl SequencePlayer {
    pub fn new(inst: Instrument, seq_name: String) -> SequencePlayer {
        SequencePlayer {
            instrument: inst,
            seq_name: seq_name,
            step_index: 0,
            clock_count: 0,
        }
    }

    pub fn reset(&mut self) {
        self.step_index = 0;
        self.clock_count = 0;
    }

    pub fn clock(&mut self) -> Vec<MidiMessage> {
        // double the step length, so that we can note-off on odd steps
        let mut messages: Vec<MidiMessage> = Vec::new();

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
                                Some(ps) => for p in ps {
                                    messages.push(midi::note_on(self.instrument.channel, *p, velocity));
                                },
                                None => {},
                            }
                        }
                    } else {
                        let maybe_step = &sequence.steps[(self.step_index - 1) / 2];
                        if maybe_step.is_some() {
                            let step = maybe_step.as_ref().unwrap().clone();
                            let velocity = 0;
                            match &step.pitch {
                                Some(ps) => for p in ps {
                                    messages.push(midi::note_on(self.instrument.channel, *p, velocity));
                                },
                                None => {},
                            }
                        }
                    }
                    self.step_index += 1;
                }
            },
            None => {},
        }

        self.clock_count += 1;

        messages
    }
}
