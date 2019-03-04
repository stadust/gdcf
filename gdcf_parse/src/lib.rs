/*#![feature(trace_macros)]

trace_macros!(true);*/

use crate::{
    error::ValueError,
    util::{SelfZip, SelfZipExt},
};
use flate2::read::GzDecoder;
use gdcf::model::level::Level;
use std::{error::Error, io::Read, num::ParseFloatError, str::FromStr};

pub mod error;
pub mod util;
#[macro_use]
pub mod macros;
pub mod level;

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

pub trait Parse: Sized {
    fn parse<'a, I, F>(iter: SelfZip<I>, f: F) -> Result<Self, ValueError<'a>>
    where
        I: Iterator<Item = &'a str>,
        F: FnMut(&'a str, &'a str) -> Result<(), ValueError<'a>>;

    fn parse_iter<'a, I>(iter: SelfZip<I>) -> Result<Self, ValueError<'a>>
    where
        I: Iterator<Item = &'a str>,
    {
        Self::parse(iter, |_, _| Ok(()))
    }
}

pub fn parse<'a, T>(idx: usize, value: &'a str) -> Result<T, ValueError<'a>>
where
    T: FromStr,
    T::Err: Error + 'static,
{
    if value == "" {
        return Err(ValueError::NoValue(idx))
    }

    value.parse().map_err(|error| ValueError::Parse(idx, value, Box::new(error)))
}
