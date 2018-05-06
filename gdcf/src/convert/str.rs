use model::GameVersion;
use model::level::Featured;
use model::LevelLength;
use model::LevelRating;
use std::num::ParseIntError;
use std::str::FromStr;

impl FromStr for GameVersion {
    type Err = ParseIntError;

    fn from_str(s: &str) -> Result<GameVersion, ParseIntError> {
        let version: i32 = s.parse()?;

        if version == 10 {
            Ok(GameVersion::Unknown)
        } else {
            Ok(GameVersion::Version {
                major: (version / 10) as u8,
                minor: (version % 10) as u8,
            })
        }
    }
}

impl ToString for GameVersion {
    fn to_string(&self) -> String {
        match self {
            GameVersion::Unknown => String::from("10"),
            GameVersion::Version { minor, major } => (minor + 10 * major).to_string(),
        }
    }
}

from_str!(LevelLength);
from_str!(Featured);