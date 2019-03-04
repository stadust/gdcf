//! Crate containing parsers for various Geometry Dash related data
//!
//! This crate is based on work by mgostIH and cos8o

use crate::util::{SelfZip, SelfZipExt};
use flate2::read::GzDecoder;
use gdcf::{error::ValueError, model::level::Level};
use std::{error::Error, io::Read, str::FromStr};

pub mod util;
#[macro_use]
pub mod macros;
pub mod level;
pub mod song;
pub mod user;

#[derive(Debug)]
pub struct LevelData(String);

pub trait LevelExt {
    fn decompress_data(&self) -> std::io::Result<LevelData>;
}

impl<S: PartialEq, U: PartialEq> LevelExt for Level<S, U> {
    fn decompress_data(&self) -> std::io::Result<LevelData> {
        let mut s = String::new();
        let mut d = GzDecoder::new(&self.level_data[..]);

        d.read_to_string(&mut s)?;

        Ok(LevelData(s))
    }
}

pub struct LevelObject {
    id: u16,
    x: f32,
    y: f32,
    metadata: ObjectMetadata,
}

pub enum ObjectMetadata {
    None,
}

impl LevelObject {
    pub fn id(&self) -> u16 {
        self.id
    }

    pub fn x(&self) -> f32 {
        self.x
    }

    pub fn y(&self) -> f32 {
        self.y
    }
}

impl LevelData {
    pub fn objects(&self) -> impl Iterator<Item = &str> {
        self.0.split(';').skip(1)
    }

    pub fn object_count(&self) -> usize {
        self.objects().count()
    }

    pub fn furthest_object_x(&self) -> f32 {
        self.objects()
            .filter_map(|s| parse_object(s).ok())
            .map(|obj| obj.x())
            .fold(0.0, f32::max)
    }
}

fn parse_object(object: &str) -> Result<LevelObject, Box<dyn Error>> {
    let id = object.split(',').self_zip().find(|(idx, value)| idx == &"1");

    let (mut id, mut x, mut y) = (0, 0.0, 0.0);

    for (idx, value) in object.split(',').self_zip() {
        match idx {
            "1" => id = value.parse()?,
            "2" => x = value.parse()?,
            "3" => y = value.parse()?,
            _ => continue,
        }
    }

    Ok(LevelObject {
        id,
        x,
        y,
        metadata: ObjectMetadata::None,
    })
}

const INDICES: [&str; 50] = [
    "1", "2", "3", "4", "5", "6", "7", "8", "9", "10", "11", "12", "13", "14", "15", "16", "17", "18", "19", "20", "21", "22", "23", "24",
    "25", "26", "27", "28", "29", "30", "31", "32", "33", "34", "35", "36", "37", "38", "39", "40", "41", "42", "43", "44", "45", "46",
    "47", "48", "49", "50",
];

pub trait Parse: Sized {
    fn parse<'a, I, F>(iter: I, f: F) -> Result<Self, ValueError<'a>>
    where
        I: Iterator<Item = (&'a str, &'a str)>,
        F: FnMut(&'a str, &'a str) -> Result<(), ValueError<'a>>;

    fn parse_iter<'a>(iter: impl Iterator<Item = &'a str>) -> Result<Self, ValueError<'a>> {
        Self::parse(iter.self_zip(), |_, _| Ok(()))
    }

    fn parse_unindexed<'a>(iter: impl Iterator<Item = &'a str>) -> Result<Self, ValueError<'a>> {
        // well this is a stupid solution
        Self::parse(INDICES.into_iter().cloned().zip(iter), |_, _| Ok(()))
    }
}

pub fn parse<T>(idx: usize, value: &str) -> Result<Option<T>, ValueError>
where
    T: FromStr,
    T::Err: Error + Send + Sync + 'static,
{
    if value == "" {
        return Ok(None)
    }

    value
        .parse()
        .map(Some)
        .map_err(|error| ValueError::Parse(idx, value, Box::new(error)))
}
