use crate::{error::ValueError, Parse};
use gdcf_model::level::data::{
    portal::{self, PortalData, PortalType, Speed},
    LevelInformationSource, LevelMetadata, LevelObject, ObjectData, Stats,
};
#[cfg(feature = "parallel")]
use rayon::{iter::ParallelIterator, str::ParallelString};
use std::time::Duration;

pub struct IterSource<I>(LevelMetadata, I)
where
    I: Iterator<Item = LevelObject>;

#[cfg(feature = "parallel")]
pub struct ParIterSource<I>(LevelMetadata, I)
where
    I: ParallelIterator<Item = LevelObject>;

pub fn parse_lazy<'a>(level_string: &'a str) -> Result<IterSource<impl Iterator<Item = LevelObject> + 'a>, ValueError<'a>> {
    let mut iter = level_string.split(';');

    let metadata = match iter.next() {
        None => return Err(ValueError::NoValue("metadata")),
        Some(s) => LevelMetadata::parse_str(s, ',')?,
    };

    let iter = iter.filter_map(|obj| {
        LevelObject::parse_str(obj, ',')
            .map_err(|err| error!("Ignoring error during parsing of object {} - {}", obj, err))
            .ok()
    });

    Ok(IterSource(metadata, iter))
}

#[cfg(feature = "parallel")]
pub fn parse_lazy_parallel<'a>(
    level_string: &'a str,
) -> Result<ParIterSource<impl ParallelIterator<Item = LevelObject> + 'a>, ValueError<'a>> {
    let (metadata_str, object_str) = match level_string.find(';') {
        Some(idx) => (&level_string[..idx - 1], &level_string[idx + 1..]),
        None => return Err(ValueError::NoValue("metadata")),
    };

    let metadata = LevelMetadata::parse_str(metadata_str, ',')?;

    let iter = object_str.par_split(';').filter_map(|obj| {
        LevelObject::parse_str(obj, ',')
            .map_err(|err| error!("Ignoring error during parsing of object {} - {}", obj, err))
            .ok()
    });

    Ok(ParIterSource(metadata, iter))
}

impl<I> LevelInformationSource for IterSource<I>
where
    I: Iterator<Item = LevelObject>,
{
    fn collect(self) -> Vec<LevelObject> {
        self.1.collect()
    }

    fn stats(self) -> Stats {
        let IterSource(metadata, iter) = self;

        let mut object_count = 0;
        let mut portals = Vec::new();
        let mut furthest_x = 0.0;

        for object in iter {
            object_count += 1;

            if let ObjectData::Portal(PortalData {
                checked: true,
                portal_type: PortalType::Speed(speed),
            }) = object.metadata
            {
                portals.push((object.x, speed))
            }

            furthest_x = f32::max(furthest_x, object.x);
        }

        portals.sort_by(|(x1, _), (x2, _)| x1.partial_cmp(x2).unwrap());

        let duration = Duration::from_secs(portal::get_seconds_from_x_pos(furthest_x, metadata.starting_speed, &portals) as u64);

        Stats { object_count, duration }
    }

    fn metadata(&self) -> LevelMetadata {
        self.0.clone()
    }
}

#[cfg(feature = "parallel")]
impl<I> LevelInformationSource for ParIterSource<I>
where
    I: ParallelIterator<Item = LevelObject>,
{
    fn collect(self) -> Vec<LevelObject> {
        self.1.collect()
    }

    fn stats(self) -> Stats {
        let ParIterSource(metadata, iter) = self;

        let (mut portals, object_count, max_x) = iter
            .fold(
                || (Vec::new(), 0, 0.0),
                |(mut portals, obj_count, max_x), object| {
                    if let ObjectData::Portal(PortalData {
                        checked: true,
                        portal_type: PortalType::Speed(speed),
                    }) = object.metadata
                    {
                        portals.push((object.x, speed))
                    }

                    (portals, obj_count + 1, f32::max(max_x, object.x))
                },
            )
            .reduce(
                || (Vec::with_capacity(32), 0, 0.0),
                |(mut v1, c1, x1), (v2, c2, x2)| {
                    v1.extend(v2);
                    (v1, c1 + c2, f32::max(x1, x2))
                },
            );

        // The parallel parsing fucked up the order already anyway, so we wont have to bother using a stable
        // sort
        portals.sort_unstable_by(|(x1, _), (x2, _)| x1.partial_cmp(x2).unwrap());

        let duration = Duration::from_secs(portal::get_seconds_from_x_pos(max_x, metadata.starting_speed, &portals) as u64);

        Stats { object_count, duration }
    }

    fn metadata(&self) -> LevelMetadata {
        self.0.clone()
    }
}

parser! {
    LevelObject => {
        id(index = 1),
        x(index = 2),
        y(index = 3),
        flipped_y(index = 4),
        flipped_x(index = 5),
        rotation(index = 6),
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
