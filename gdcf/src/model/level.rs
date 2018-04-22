use std::convert::From;
use std::str::FromStr;
use std::num::ParseIntError;
use std;

use model::{GameVersion, ValueError, RawObject, FromRawObject};
use model::song::MainSong;
use model::de;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
#[cfg_attr(ser, derive(Serialize))]
pub enum LevelLength {
    Tiny,
    Short,
    Medium,
    Long,
    ExtraLong,
    Unknown,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
#[cfg_attr(ser, derive(Serialize))]
pub enum LevelRating {
    Auto,
    Demon(DemonRating),
    NotAvailable,
    Easy,
    Normal,
    Hard,
    Harder,
    Insane,
    Unknown,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
#[cfg_attr(ser, derive(Serialize))]
pub enum DemonRating {
    Easy,
    Medium,
    Hard,
    Insane,
    Extreme,
    Unknown,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
#[cfg_attr(ser, derive(Serialize))]
pub enum Featured {
    NotFeatured,
    Unfeatured,
    Featured(u32),
}

#[derive(Debug, FromRawObject)]
pub struct PartialLevel {
    #[raw_data(index = 1)]
    lid: u64,

    #[raw_data(index = 2)]
    name: String,

    #[raw_data(index = 3, deserialize_with="de::into_option", default)]
    description: Option<String>,

    // Index 4 not provided for partial levels

    #[raw_data(index = 5)]
    version: u32,

    #[raw_data(index = 6)]
    creator_id: u64,

    #[raw_data(index = 8, deserialize_with = "de::int_to_bool")]
    has_difficulty_rating: bool,

    #[raw_data(custom = "de::level_rating")]
    difficulty: LevelRating,

    #[raw_data(index = 10)]
    downloads: u32,

    #[raw_data(custom = "de::main_song")]
    main_song: Option<&'static MainSong>,

    /// The gd version the request was uploaded/last updated in. Index: 13
    #[raw_data(index = 13)]
    gd_version: GameVersion,

    #[raw_data(index = 14)]
    likes: u32,

    #[raw_data(index = 15)]
    length: LevelLength,

    #[raw_data(index = 17, deserialize_with = "de::int_to_bool", default = false)]
    is_demon: bool,

    #[raw_data(index = 18)]
    stars: u8,

    /// 0 if the request isn't featured, otherwise an arbitrary value that indicates the ranking on the featured list.
    #[raw_data(index = 19, deserialize_with = "de::featured")]
    featured: Featured,

    #[raw_data(index = 25, deserialize_with = "de::int_to_bool", default = false)]
    is_auto: bool,

    // Index 27 is not provided for partial levels
    // Index 28 is not provided for partial levels
    // Index 29 is not provided for partial levels

    #[raw_data(index = 30, deserialize_with = "de::default_to_none")]
    copy_of: Option<u64>,

    #[raw_data(index = 35, deserialize_with = "de::default_to_none")]
    custom_song_id: Option<u64>,

    // Index 36 is not provided for partial levels

    #[raw_data(index = 37)]
    coin_amount: u8,

    // Index 38 has unknown usage
    #[raw_data(index = 38)]
    index_38: String,

    #[raw_data(index = 39, deserialize_with = "de::default_to_none")]
    stars_requested: Option<u8>,

    #[raw_data(index = 42, deserialize_with = "de::int_to_bool")]
    is_epic: bool,

    // Index 43 has unknown usage
    #[raw_data(index = 43)]
    index_43: String,

    #[raw_data(index = 45)]
    object_amount: u32,

    // Index 46 has unknown usage
    //#[raw_data(index = 46)]
    //index_46: String,

    // Index 47 has unknown usage
    //#[raw_data(index = 47)]
    //index_47: String,
}

#[derive(Debug, FromRawObject)]
pub struct Level {
    #[raw_data(flatten)]
    base: PartialLevel,

    /// GZip compressed level data. Index: 4
    #[raw_data(index = 4)]
    level_data: String,

    /// The request's password (encrypted).  Index: 27
    #[raw_data(index = 27)]
    password: String,

    /// Index: 28
    #[raw_data(index = 28)]
    time_since_upload: String,

    /// Index: 29
    #[raw_data(index = 29)]
    time_since_update: String,

    // Index 36 has unknown usage
    //#[raw_data(index = 36)]
    //index_36: String,
}

impl FromStr for GameVersion {
    type Err = ParseIntError;

    fn from_str(s: &str) -> Result<Self, <Self as FromStr>::Err> {
        let version: i32 = s.parse()?;

        if version == 10 {
            Ok(GameVersion::Unknown)
        } else {
            Ok(GameVersion::Version { major: (version / 10) as u8, minor: (version % 10) as u8 })
        }
    }
}

impl ToString for GameVersion {
    fn to_string(&self) -> String {
        match *self {
            GameVersion::Unknown => String::from("10"),
            GameVersion::Version { minor, major } => (minor + 10 * major).to_string()
        }
    }
}

impl FromStr for LevelLength {
    type Err = ParseIntError;

    fn from_str(s: &str) -> Result<Self, <Self as FromStr>::Err> {
        let length = s.parse()?;

        Ok(
            match length {
                0 => LevelLength::Tiny,
                1 => LevelLength::Short,
                2 => LevelLength::Medium,
                3 => LevelLength::Long,
                4 => LevelLength::ExtraLong,
                _ => LevelLength::Unknown
            }
        )
    }
}

impl From<LevelLength> for i32 {
    fn from(length: LevelLength) -> Self {
        match length {
            LevelLength::Tiny => 0,
            LevelLength::Short => 1,
            LevelLength::Medium => 2,
            LevelLength::Long => 3,
            LevelLength::ExtraLong => 4,
            LevelLength::Unknown => std::i32::MAX
        }
    }
}

impl From<i32> for LevelRating {
    fn from(value: i32) -> Self {
        match value {
            0 => LevelRating::NotAvailable,
            10 => LevelRating::Easy,
            20 => LevelRating::Normal,
            30 => LevelRating::Hard,
            40 => LevelRating::Harder,
            50 => LevelRating::Insane,
            _ => LevelRating::Unknown
        }
    }
}

impl From<LevelRating> for i32 {
    fn from(rating: LevelRating) -> Self {
        match rating {
            LevelRating::Auto => -3,
            LevelRating::Demon(_) => -2,
            LevelRating::NotAvailable => -1,
            LevelRating::Easy => 1,
            LevelRating::Normal => 2,
            LevelRating::Hard => 3,
            LevelRating::Harder => 4,
            LevelRating::Insane => 5,
            LevelRating::Unknown => std::i32::MAX
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
            _ => DemonRating::Unknown
        }
    }
}

impl From<DemonRating> for i32 {
    fn from(rating: DemonRating) -> i32 {
        match rating {
            DemonRating::Easy => 1,
            DemonRating::Medium => 2,
            DemonRating::Hard => 3,
            DemonRating::Insane => 4,
            DemonRating::Extreme => 5,
            DemonRating::Unknown => std::i32::MAX
        }
    }
}