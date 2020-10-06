extern crate portmidi;

use portmidi::{OutputPort, MidiMessage};

use crate::config::{TICKS_PER_MEASURE};

const CHANNEL: u8 = 13;

pub struct Sequence {
    steps: Vec<u8>,
}

impl Sequence {
    pub fn new(count: usize, pitch: u8) -> Sequence {
        Sequence {
            steps: vec![pitch; count],
        }
    }
}

pub struct SequencePlayer {
    sequence: Sequence,
    step_index: usize,
    clock_count: usize,
}

impl SequencePlayer {
    pub fn new(seq: Sequence) -> SequencePlayer {
        SequencePlayer {
            sequence: seq,
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
        let total_steps = self.sequence.steps.len() * 2;
        let ticks_per_step = TICKS_PER_MEASURE as usize / total_steps;

        let mut messages: Vec<MidiMessage> = Vec::new();

        if self.clock_count % ticks_per_step == 0 && self.step_index < total_steps {
            let pitch: u8;
            let velocity: u8 = 100;
            if self.step_index % 2 == 0 {
                pitch = self.sequence.steps[self.step_index / 2];
                // println!("{} {} note-on", self.step_index, pitch);
                messages.push(note_on(pitch, velocity));
            } else {
                pitch = self.sequence.steps[(self.step_index-1) / 2];
                // println!("{} {} note-off", self.step_index, pitch);
                messages.push(note_off(pitch, 0));
            }
            self.step_index += 1;
        }

        self.clock_count += 1;

        messages
    }
}

pub fn note_on(pitch: u8, velocity: u8) -> MidiMessage {
    MidiMessage {
        status: 0x90 + CHANNEL,
        data1: pitch,
        data2: velocity,
        data3: 0,
    }
}

pub fn note_off(pitch: u8, velocity: u8) -> MidiMessage {
    MidiMessage {
        status: 0x80 + CHANNEL,
        data1: pitch,
        data2: velocity,
        data3: 0,
    }
}