use self::{
    metadata::{ObjectMetadata, PortalMetadata},
    portal::{PortalType, Speed},
};
use crate::Parse;
use flate2::read::GzDecoder;
use gdcf::{error::ValueError, model::Level};
use std::{io::Read, time::Duration};

pub mod ids;
pub mod metadata;
pub mod portal;

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

#[derive(Debug, PartialEq, Clone)]
pub struct LevelObject {
    pub id: u16,
    pub x: f32,
    pub y: f32,
    // ... other fields they all have ...
    pub metadata: ObjectMetadata,
}

#[derive(Debug, PartialEq, Clone)]
pub struct LevelMetadata {
    starting_speed: Speed,
    // ... other fields in the metadata section ...
}

impl LevelData {
    pub fn objects(&self) -> impl Iterator<Item = &str> {
        self.0.split(';').skip(1)
    }

    pub fn object_count(&self) -> usize {
        self.objects().count()
    }

    pub fn parsed_objects<'a>(&'a self) -> impl Iterator<Item = LevelObject> + 'a {
        self.objects().filter_map(|obj| {
            LevelObject::parse_str(obj, ',')
                .map_err(|err| error!("Ignoring error during parsing of object {} - {}", obj, err))
                .ok()
        })
    }

    pub fn metadata(&self) -> Result<LevelMetadata, ValueError> {
        match self.0.split(';').nth(0) {
            None => Err(ValueError::NoValue("metadata")),
            Some(s) => LevelMetadata::parse_str(s, ','),
        }
    }

    pub fn starting_speed(&self) -> Result<Speed, ValueError> {
        Ok(self.metadata()?.starting_speed)
    }

    pub fn level_length(&self) -> Result<Duration, ValueError> {
        let mut portals = Vec::new();
        let mut furthest_x = 0.0;

        for object in self.parsed_objects() {
            if let ObjectMetadata::Portal(PortalMetadata {
                checked: true,
                portal_type: PortalType::Speed(speed),
            }) = object.metadata
            {
                portals.push((object.x, speed))
            }

            furthest_x = f32::max(furthest_x, object.x);
        }

        portals.sort_unstable_by(|(x1, _), (x2, _)| x1.partial_cmp(x2).unwrap());

        let seconds = portal::get_seconds_from_x_pos(furthest_x, self.starting_speed()?, &portals);

        Ok(Duration::from_secs(seconds.round() as u64))
    }

    pub fn parse_fully(&self) -> Result<ParsedLevelData, ValueError> {
        let all_objects = self
            .objects()
            .map(|obj| LevelObject::parse_str(obj, ','))
            .collect::<Result<_, _>>()?;

        Ok(ParsedLevelData(self.metadata()?, all_objects))
    }
}

pub struct ParsedLevelData(LevelMetadata, Vec<LevelObject>);

parser! {
    LevelObject => {
        id(index = 1),
        x(index = 2),
        y(index = 3),
        // ... all the other fields ...
        metadata(delegate),
    }
}

fn parse_starting_speed(speed: u8) -> Speed {
    match speed {
        0 => Speed::Slow,
        1 => Speed::Normal,
        2 => Speed::Medium,
        3 => Speed::Fast,
        4 => Speed::VeryFast,
        _ => Speed::Invalid,
    }
}

parser! {
    LevelMetadata => {
        starting_speed(index = kA4, with = parse_starting_speed),
        // ... all the other fields ...
    }
}
