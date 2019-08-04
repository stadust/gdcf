#![deny(missing_copy_implementations)]
#![deny(missing_debug_implementations)]

pub mod comment;
pub mod level;
pub mod song;
pub mod user;

#[cfg(feature = "serde_support")]
use serde_derive::{Deserialize, Serialize};
use std::{
    fmt::{Display, Formatter},
    num::ParseIntError,
    str::FromStr,
};

/// Enum modelling the version of a Geometry Dash client
#[derive(Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde_support", derive(Serialize, Deserialize))]
pub enum GameVersion {
    /// Variant representing an unknown version. This variant is only used for
    /// levels that were uploaded before the game started tracking the
    /// version. This variant's string
    /// representation is `"10"`
    Unknown,

    /// Variant representing a the version represented by the given minor/major
    /// values in the form `major.minor`
    Version { minor: u8, major: u8 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameMode {
    Cube,
    Ship,
    Ball,
    Ufo,
    Wave,
    Robot,
    Spider,
    Unknown(u8),
}

impl From<u8> for GameVersion {
    fn from(version: u8) -> Self {
        if version == 10 {
            GameVersion::Unknown
        } else {
            GameVersion::Version {
                major: (version / 10) as u8,
                minor: (version % 10) as u8,
            }
        }
    }
}

impl Into<u8> for GameVersion {
    fn into(self) -> u8 {
        match self {
            GameVersion::Unknown => 10,
            GameVersion::Version { minor, major } => major * 10 + minor,
        }
    }
}

impl Display for GameVersion {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            GameVersion::Unknown => write!(f, "Unknown"),
            GameVersion::Version { minor, major } => write!(f, "{}.{}", major, minor),
        }
    }
}

impl From<u8> for GameMode {
    fn from(i: u8) -> Self {
        match i {
            0 => GameMode::Cube,
            1 => GameMode::Ship,
            2 => GameMode::Ball,
            3 => GameMode::Ufo,
            4 => GameMode::Wave,
            5 => GameMode::Robot,
            6 => GameMode::Spider,
            i => GameMode::Unknown(i),
        }
    }
}

impl Into<u8> for GameMode {
    fn into(self) -> u8 {
        match self {
            GameMode::Cube => 0,
            GameMode::Ship => 1,
            GameMode::Ball => 2,
            GameMode::Ufo => 3,
            GameMode::Wave => 4,
            GameMode::Robot => 5,
            GameMode::Spider => 6,
            GameMode::Unknown(idx) => idx,
        }
    }
}
