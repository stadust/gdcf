use std::str::FromStr;
use std::num::ParseIntError;
use std::convert::From;
use model::Value;
use serde::Serializer;

#[derive(Debug)]
pub enum GameVersion {
    Unknown,
    Version { minor: u8, major: u8 },
}

#[derive(Debug)]
pub enum LevelLength {
    Tiny,
    Short,
    Medium,
    Long,
    ExtraLong,
}

#[derive(Debug)]
pub enum LevelRating {
    Auto,
    Demon(DemonRating),
    NotAvailable,
    Easy,
    Normal,
    Hard,
    Harder,
    Insane,
}

#[derive(Debug)]
pub enum DemonRating {
    Easy,
    Medium,
    Hard,
    Insane,
    Extreme,
}

#[derive(Debug)]
pub struct PartialLevel {
    /// The request's id. Index: 1
    lid: u64,
    /// The request's name. Index: 2
    name: String,
    /// The request's description. Index: 3
    description: String,

    // Index 4 not provided for partial levels

    /// The levels' version. Index: 5
    version: u32,

    // The request creator's userid. index: 6
    creator_id: u64,

    // Index 7 is unused

    /// Index 8
    has_difficulty_rating: bool,

    /// Index 9
    difficulty: LevelRating,

    // The amount of downloads the request received. Index: 10
    downloads: u32,

    // Index 11 is unused

    /// The main song id. Index: 12
    main_song_id: Option<u32>,

    /// The gd version the request was uploaded/last updated in. Index: 13
    gd_version: GameVersion,

    // The amount of likes the level has received. Index: 14
    likes: u32,

    /// The length of the request. Index: 15
    length: LevelLength,

    // Index 16 is unused

    /// Index: 17
    is_demon: bool,

    /// Index: 18
    stars: u8,

    /// 0 if the request isn't featured, otherwise an arbitrary value that indicates the ranking on the featured list.
    /// Index: 19
    featured_weight: u32,

    // Index 20 is unused
    // Index 21 is unused
    // Index 22 is unused
    // Index 23 is unused
    // Index 24 is unused

    /// Index: 25
    is_auto: bool,

    // Index 26 is unused
    // Index 27 is not provided for partial levels
    // Index 28 is not provided for partial levels
    // Index 29 is not provided for partial levels

    /// The id of the request this one is a copy of. Index: 30
    copy_of: Option<u64>,

    // Index 32 is unused
    // Index 33 is unused
    // Index 34 is unused

    // Index: 35
    custom_song_id: Option<u64>,

    // Index 36 is not provided for partial levels

    /// Index: 37
    coin_amount: u8,

    // Index 38 has unknown usage

    /// Index 39
    stars_requested: Option<u8>,

    // Index 40 is unused
    // Index 41 is unused

    /// Index: 42
    is_epic: bool,

    // Index 43 has unknown usage
    // Index 44 is unused

    /// Index: 45
    object_amount: u32,

    // Index 46 has unknown usage
    // Index 47 has unknown usage
}

#[derive(Debug)]
pub struct Level {
    base: PartialLevel,

    /// GZip compressed level data. Index: 4
    level_data: String,

    /// The request's password.  Index: 27
    password: String,

    /// Index: 28
    time_since_upload: String,

    /// Index: 29
    time_since_update: String,

    // Index 36 has unknown usage
}

impl ToString for GameVersion {
    fn to_string(&self) -> String {
        match *self {
            GameVersion::Unknown => String::from("10"),
            GameVersion::Version { minor, major } => (minor + 10 * major).to_string()
        }
    }
}

impl LevelRating {
    pub fn value(&self) -> i32 {
        match *self {
            LevelRating::Auto => -3,
            LevelRating::Demon(_) => -2,
            LevelRating::NotAvailable => -1,
            LevelRating::Easy => 1,
            LevelRating::Normal => 2,
            LevelRating::Hard => 3,
            LevelRating::Harder => 4,
            LevelRating::Insane => 5
        }
    }

    pub fn from_i32(value: i32, is_demon: bool) -> LevelRating {
        if is_demon {
            LevelRating::Demon(DemonRating::from(value))
        } else {
            match value {
                0 => LevelRating::NotAvailable,
                10 => LevelRating::Easy,
                20 => LevelRating::Normal,
                30 => LevelRating::Hard,
                40 => LevelRating::Harder,
                50 => LevelRating::Insane,  // TODO: auto
                _ => panic!("Invalid enum value for LevelRating: {}", value)
            }
        }
    }
}

impl DemonRating {
    pub fn values(&self) -> i32 {
        match *self {
            DemonRating::Easy => 1,
            DemonRating::Medium => 2,
            DemonRating::Hard => 3,
            DemonRating::Insane => 4,
            DemonRating::Extreme => 5
        }
    }
}

impl From<i32> for DemonRating {
    fn from(value: i32) -> DemonRating {
        match value {
            10 => DemonRating::Easy,
            20 => DemonRating::Medium,
            30 => DemonRating::Hard,
            40 => DemonRating::Insane,
            50 => DemonRating::Extreme,
            _ => panic!("Invalid enum value for DemonRating: {}", value)
        }
    }
}
