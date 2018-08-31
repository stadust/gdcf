//! Module containing various conversion to and from [`String`] for GDCF models

use model::{level::Featured, DemonRating, GameVersion, LevelLength, LevelRating};
use std::{num::ParseIntError, str::FromStr};

impl FromStr for GameVersion {
    type Err = ParseIntError;

    fn from_str(s: &str) -> Result<GameVersion, ParseIntError> {
        s.parse().map(u8::into)
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
            LevelRating::Demon(demon) => demon.to_string(),
        }
    }
}

impl From<String> for LevelRating {
    fn from(s: String) -> Self {
        match s.as_ref() {
            "Auto" => LevelRating::Auto,
            "NotAvailable" => LevelRating::NotAvailable,
            "Easy" => LevelRating::Easy,
            "Hard" => LevelRating::Hard,
            "Normal" => LevelRating::Normal,
            "Harder" => LevelRating::Harder,
            "Insane" => LevelRating::Harder,
            "__UNKNOWN_LEVEL_RATING__" => LevelRating::Unknown,
            _ => LevelRating::Demon(DemonRating::from(s)),
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
            DemonRating::Unknown => "__UNKNOWN_DEMON_RATING__",
        }.to_string()
    }
}

impl From<String> for DemonRating {
    fn from(s: String) -> Self {
        match s.as_ref() {
            "EasyDemon" => DemonRating::Easy,
            "MediumDemon" => DemonRating::Medium,
            "HardDemon" => DemonRating::Hard,
            "InsaneDemon" => DemonRating::Insane,
            "ExtremeDemon" => DemonRating::Extreme,
            _ => DemonRating::Unknown,
        }
    }
}

impl ToString for LevelLength {
    fn to_string(&self) -> String {
        match self {
            LevelLength::Tiny => "Tiny",
            LevelLength::Medium => "Medium",
            LevelLength::Short => "Short",
            LevelLength::Long => "Long",
            LevelLength::ExtraLong => "ExtraLong",
            LevelLength::Unknown => "__UNKNOWN_LEVEL_LENGTH__",
        }.to_string()
    }
}

impl From<String> for LevelLength {
    fn from(s: String) -> Self {
        match s.as_ref() {
            "Tiny" => LevelLength::Tiny,
            "Medium" => LevelLength::Medium,
            "Short" => LevelLength::Short,
            "Long" => LevelLength::Long,
            "ExtraLong" => LevelLength::ExtraLong,
            _ => LevelLength::Unknown,
        }
    }
}
