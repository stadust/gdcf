use crate::{error::ValueError, Parse};
use gdcf_model::level::data::{portal::Speed, LevelMetadata, LevelObject, ParsedIterator};

#[derive(Debug)]
pub struct LevelData(String);

pub fn parse_iter<'a>(level_string: &'a str) -> Result<ParsedIterator<impl Iterator<Item = LevelObject> + 'a>, ValueError<'a>> {
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

    Ok(ParsedIterator(metadata, iter))
}

pub fn metadata(level_string: &str) -> Result<LevelMetadata, ValueError> {
    match level_string.split(';').nth(0) {
        None => Err(ValueError::NoValue("metadata")),
        Some(s) => LevelMetadata::parse_str(s, ','),
    }
}

pub fn objects<'a>(level_string: &'a str) -> impl Iterator<Item = LevelObject> + 'a {
    level_string.split(';').skip(1).filter_map(|obj| {
        LevelObject::parse_str(obj, ',')
            .map_err(|err| error!("Ignoring error during parsing of object {} - {}", obj, err))
            .ok()
    })
}

/*
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

        portals.sort_by(|(x1, _), (x2, _)| x1.partial_cmp(x2).unwrap());

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
*/
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
