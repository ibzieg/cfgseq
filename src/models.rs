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
use serde::{Deserialize, Serialize};

use crate::config::{DEFAULT_MIDI_CHANNEL, DEFAULT_PARTS_PER_QUARTER};
use serde::de::Unexpected::Seq;


// Controller --------------------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize)]
pub struct Controller {
    pub device: String,
    pub channel: u8,
    pub ppq: Option<u64>,
}

impl Clone for Controller {
    fn clone(&self) -> Controller {
        Controller {
            device: self.device.to_owned(),
            channel: self.channel.to_owned(),
            ppq: self.ppq.to_owned(),
        }
    }
}

impl Controller {
    pub fn new() -> Controller {
        Controller {
            device: String::new(),
            channel: DEFAULT_MIDI_CHANNEL,
            ppq: Some(DEFAULT_PARTS_PER_QUARTER),
        }
    }
}

// Track -------------------------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize)]
pub struct Track {
    pub instrument: String,
    pub follow: Option<String>,
    pub play: Vec<String>,
}

impl Clone for Track {
    fn clone(&self) -> Track {
        Track {
            instrument: self.instrument.to_owned(),
            follow: self.follow.to_owned(),
            play: self.play.to_vec(),
        }
    }
}

// Scene -------------------------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize)]
pub struct Scene {
    pub name: String,
    pub master: Option<String>,
    pub tracks: Vec<Track>,
}

impl Clone for Scene {
    fn clone(&self) -> Scene {
        Scene {
            name: self.name.to_owned(),
            master: self.master.to_owned(),
            tracks: self.tracks.to_vec(),
        }
    }
}

// SequenceStep ------------------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize)]
pub struct SequenceStep {
    pub pitch: Option<Vec<u8>>,
    pub velocity: Option<u8>,
    pub data: Option<Vec<u8>>,
}

impl Clone for SequenceStep {
    fn clone(&self) -> SequenceStep {
        SequenceStep {
            pitch: self.pitch.to_owned(),
            velocity: self.velocity.to_owned(),
            data: match &self.data {
                Some(d) => Some(d.to_vec()),
                None => None,
            },
        }
    }
}

// Sequence ----------------------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize)]
pub struct Sequence {
    pub name: String,
    pub steps: Vec<Option<SequenceStep>>,
}

impl Sequence {
    pub fn new() -> Sequence {
        Sequence {
            name: String::new(),
            steps: Vec::new(),
        }
    }
}

impl Clone for Sequence {
    fn clone(&self) -> Sequence {
        Sequence {
            name: self.name.to_string(),
            steps: self.steps.to_vec(),
        }
    }
}

// ModDevice ---------------------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize)]
pub struct ModDevice {
    pub device: String,
    pub channel: u32,
    pub control: u8,
}

impl Clone for ModDevice {
    fn clone(&self) -> ModDevice {
        ModDevice {
            device: self.device.to_string(),
            channel: self.channel.to_owned(),
            control: self.control.to_owned(),
        }
    }
}

// Instrument --------------------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize)]
pub struct Instrument {
    pub name: String,
    pub device: String,
    pub channel: u8,
    pub data: Option<Vec<ModDevice>>,
    pub sequences: Vec<Sequence>,
}

impl Instrument {
    pub fn new() -> Instrument {
        Instrument {
            name: String::new(),
            device: String::new(),
            channel: DEFAULT_MIDI_CHANNEL,
            data: None,
            sequences: Vec::new(),
        }
    }

    pub fn find_sequence(&self, name: &String) -> Option<&Sequence> {
        let mut result: Option<&Sequence> = None;
        for sequence in &self.sequences {
            if &sequence.name == name {
                result = Some(sequence);
            }
        }
        result
    }
}

impl Clone for Instrument {
    fn clone(&self) -> Instrument {
        Instrument {
            name: self.name.to_string(),
            device: self.name.to_string(),
            channel: self.channel.to_owned(),
            data: match &self.data {
                Some(d) => Some(d.to_vec()),
                None => None,
            },
            sequences: self.sequences.to_vec(),
        }
    }
}

// Performance -------------------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize)]
pub struct Performance {
    pub controller: Controller,
    pub playlist: Vec<String>,
    pub scenes: Vec<Scene>,
    pub instruments: Vec<Instrument>,
}

impl Performance {
    pub fn new() -> Performance {
        Performance {
            controller: Controller::new(),
            playlist: Vec::new(),
            scenes: Vec::new(),
            instruments: Vec::new(),
        }
    }

    pub fn find_scene(&self, name: &String) -> Option<&Scene> {
        let mut result: Option<&Scene> = None;
        for scene in &self.scenes {
            if &scene.name == name {
                result = Some(scene);
            }
        }
        result
    }

    pub fn find_instrument(&self, name: &String) -> Option<&Instrument> {
        let mut result: Option<&Instrument> = None;
        for instrument in &self.instruments {
            if &instrument.name == name {
                result = Some(instrument);
            }
        }
        result
    }
}
