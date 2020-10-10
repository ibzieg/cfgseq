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

#[derive(Debug, Serialize, Deserialize)]
pub struct Controller {
    pub device: String,
    pub channel: u8,
    pub ppq: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Track {
    pub instrument: String,
    pub follow: Option<String>,
    pub play: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Scene {
    pub name: String,
    pub master: String,
    pub tracks: Vec<Track>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SequenceStep {
    pub pitch: Option<Vec<u8>>,
    pub velocity: Option<u8>,
    pub data: Option<Vec<u8>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Sequence {
    pub name: String,
    pub steps: Vec<Option<SequenceStep>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ModDevice {
    pub device: String,
    pub channel: u32,
    pub control: u8,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Instrument {
    pub name: String,
    pub device: String,
    pub channel: u32,
    pub data: Option<Vec<ModDevice>>,
    pub sequences: Vec<Sequence>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Performance {
    pub controller: Controller,
    pub playlist: Vec<String>,
    pub scenes: Vec<Scene>,
    pub instruments: Vec<Instrument>,
}
