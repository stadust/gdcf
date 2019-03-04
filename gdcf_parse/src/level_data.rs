use self::{
    metadata::{ObjectMetadata, PortalMetadata},
    portal::{PortalType, Speed},
};
use crate::{util::SelfZipExt, Parse};
use flate2::read::GzDecoder;
use gdcf::{error::ValueError, model::Level};
use std::{error::Error, io::Read, time::Duration};

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

#[derive(Debug)]
pub struct LevelObject {
    pub id: u16,
    pub x: f32,
    pub y: f32,
    // ... other fields they all have ...
    pub metadata: ObjectMetadata,
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
            LevelObject::parse_iter(obj.split(','))
                .map_err(|err| println!("{} - {}", obj, err))
                .ok()
        })
    }

    pub fn starting_speed(&self) -> Speed {
        match self.0.split(';').nth(0) {
            Some(segment) =>
                match segment.split(',').self_zip().find(|(key, _)| key == &"kA4") {
                    Some((_, value)) =>
                        match value.parse() {
                            Ok(0) => Speed::Slow,
                            Ok(1) => Speed::Normal,
                            Ok(2) => Speed::Medium,
                            Ok(3) => Speed::Fast,
                            Ok(4) => Speed::VeryFast,
                            _ => Speed::Invalid,
                        },
                    _ => Speed::Invalid,
                },
            _ => Speed::Invalid,
        }
    }

    pub fn level_length(&self) -> Duration {
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

        let seconds = portal::get_seconds_from_x_pos(furthest_x, self.starting_speed(), &portals);

        Duration::from_secs(seconds.round() as u64)
    }
}

parser! {
    LevelObject => {
        id(index = 1),
        x(index = 2),
        y(index = 3),
        metadata(delegate),
    }
}
