use model::GameVersion;
use model::level::Featured;
use model::LevelLength;
use model::LevelRating;
use std::num::ParseIntError;
use std::str::FromStr;
use model::DemonRating;

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

impl ToString for LevelRating {
    fn to_string(&self) -> String {
        match self {
            LevelRating::Auto => "Auto".into(),
            LevelRating::NotAvailable => "NotAvailable".into(),
            LevelRating::Easy => "Easy".into(),
            LevelRating::Normal => "Normal".into(),
            LevelRating::Hard => "Hard".into(),
            LevelRating::Harder => "Harder".into(),
            LevelRating::Insane => "Insane".into(),
            LevelRating::Unknown => "__UNKNOWN_LEVEL_RATING__".into(),
            LevelRating::Demon(demon) => demon.to_string()
        }
    }
}

impl ToString for DemonRating {
    fn to_string(&self) -> String {
        match self {
            DemonRating::Easy => "EasyDemon",
            DemonRating::Medium => "MediumDemon",
            DemonRating::Hard => "HardDemon",
            DemonRating::Insane => "InsaneDemon",
            DemonRating::Extreme => "ExtremeDemon",
            DemonRating::Unknown => "__UNKNOWN_DEMON_RATING__"
        }.to_string()
    }
}