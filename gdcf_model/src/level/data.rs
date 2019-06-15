pub mod ids;
pub mod portal;
pub mod text;
pub mod trigger;

use crate::level::{
    data::{
        portal::{PortalData, Speed},
        text::TextData,
        trigger::ColorTriggerData,
    },
    Level,
};
use flate2::read::GzDecoder;
use std::{io::Read, time::Duration};

#[derive(Debug, PartialEq, Clone, Default, Copy)]
pub struct LevelMetadata {
    pub starting_speed: Speed,
    // ... other fields in the metadata section ...
}

#[derive(Debug, PartialEq, Clone)]
pub struct LevelObject {
    pub id: u16,
    pub x: f32,
    pub y: f32,
    pub flipped_x: bool,
    pub flipped_y: bool,
    pub rotation: f32,
    // ... other fields they all have ...
    pub metadata: ObjectData,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ObjectData {
    None,
    Portal(PortalData),
    Text(TextData),
    ColorTrigger(ColorTriggerData),
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub struct Stats {
    pub duration: Duration,
    pub object_count: u64,
}

impl<S, U> Level<S, U>
where
    U: PartialEq,
    S: PartialEq,
{
    pub fn decompress_data(&self) -> std::io::Result<String> {
        let mut s = String::new();
        let mut d = GzDecoder::new(&self.level_data[..]);

        d.read_to_string(&mut s)?;

        Ok(s)
    }
}

pub trait LevelInformationSource {
    fn collect(self) -> Vec<LevelObject>;

    fn stats(self) -> Stats;

    fn metadata(&self) -> LevelMetadata;
}
