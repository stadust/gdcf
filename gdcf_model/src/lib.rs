pub mod level;
pub mod song;
pub mod user;

#[cfg(feature = "serde_support")]
use serde_derive::{Deserialize, Serialize};

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
