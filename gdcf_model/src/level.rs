//! Module containing all models related to Geometry Dash levels

pub mod data;

use crate::{song::MainSong, GameVersion};
use std::fmt::{Display, Error, Formatter};

#[cfg(feature = "serde_support")]
use serde::Deserializer;
#[cfg(feature = "serde_support")]
use serde_derive::{Deserialize, Serialize};

/// Enum representing the possible level lengths known to GDCF
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde_support", derive(Serialize, Deserialize))]
pub enum LevelLength {
    /// Enum variant that's used by the [`From<i32>`](From) impl for when an
    /// unrecognized value is passed
    Unknown,

    /// Tiny
    ///
    /// ## GD Internals:
    /// This variant is represented by the value `0` in both requests and
    /// responses
    Tiny,

    /// Short
    ///
    /// ## GD Internals:
    /// This variant is represented by the value `1` in both requests and
    /// responses
    Short,

    /// Medium
    ///
    /// ## GD Internals:
    /// This variant is represented by the value `2` in both requests and
    /// responses
    Medium,

    /// Long
    ///
    /// ## GD Internals:
    /// This variant is represented by the value `3` in both requests and
    /// responses
    Long,

    /// Extra Long, sometime referred to as `XL`
    ///
    /// ## GD Internals:
    /// This variant is represented by the value `4` in both requests and
    /// responses
    ExtraLong,
}

/// Enum representing the possible level ratings
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde_support", derive(Serialize, Deserialize))]
pub enum LevelRating {
    /// Enum variant that's used by the [`From<i32>`](From) impl for when an
    /// unrecognized value is passed
    Unknown,

    /// Not Available, sometimes referred to as `N/A` or `NA`
    ///
    /// ## GD Internals:
    /// This variant is represented by the value `-1` in requests and by the
    /// value `0` in responses
    NotAvailable,

    /// Auto rating
    ///
    /// ## GD Internals:
    /// This variant is represented by the value `-3` in requests, and not
    /// included in responses.
    Auto,

    /// Easy rating
    ///
    /// ## GD Internals:
    /// This variant is represented by the value `1` in requests and by the
    /// value `10` in responses
    Easy,

    /// Normal rating
    ///
    /// ## GD Internals:
    /// This variant is represented by the value `2` in requests and by the
    /// value `20` in responses
    Normal,

    /// Hard rating
    ///
    /// ## GD Internals:
    /// This variant is represented by the value `3` in requests and by the
    /// value `30` in responses
    Hard,

    /// Harder rating
    ///
    /// ## GD Internals:
    /// This variant is represented by the value `4` in requests and by the
    /// value `40` in responses
    Harder,

    /// Insane rating
    ///
    /// ## GD Internals:
    /// This variant is represented by the value `5` in requests and by the
    /// value `50` in responses
    Insane,

    /// Demon rating.
    ///
    /// ## GD Internals:
    /// This variant is represented by the value `-2` in requests. In
    /// responses, you will have to first check the provided level is a
    /// demon and then interpret the provided
    /// `rating` value as a [`DemonRating`]
    Demon(DemonRating),
}

/// Enum representing the possible demon difficulties
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde_support", derive(Serialize, Deserialize))]
pub enum DemonRating {
    /// Enum variant that's used by the [`From<i32>`](From) impl for when an
    /// unrecognized value is passed
    Unknown,

    /// Easy demon
    ///
    /// ## GD Internals:
    /// This variant is represented by the value `1` in requests and by the
    /// value `10` in responses
    Easy,

    /// Medium demon
    ///
    /// ## GD Internals:
    /// This variant is represented by the value `2` in requests and by the
    /// value `20` in responses
    Medium,

    /// Hard demon
    ///
    /// ## GD Internals:
    /// This variant is represented by the value `3` in requests and by the
    /// value `30` in responses
    Hard,

    /// Insane demon
    ///
    /// ## GD Internals:
    /// This variant is represented by the value `4` in requests and by the
    /// value `40` in responses
    Insane,

    /// Extreme demon
    ///
    /// ## GD Internals:
    /// This variant is represented by the value `5` in requests and by the
    /// value `50` in responses
    Extreme,
}

/// Enum representing a levels featured state
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde_support", derive(Serialize, Deserialize))]
pub enum Featured {
    /// The level isn't featured, and has never been featured before
    NotFeatured,

    /// The level isn't featured, but used to be (it either got unrated, or
    /// unfeatured, like Sonic Wave)
    Unfeatured,

    /// The level is featured, and has the contained value as its featured
    /// weight.
    ///
    /// The featured weight determines how high on the featured pages the level
    /// appear, where a higher value means a higher position.
    Featured(u32),
}

/// Enum representing a level's copy status
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde_support", derive(Serialize, Deserialize))]
pub enum Password {
    /// The level isn't copyable (which I see the irony of, considering we
    /// literally have a copy of it in the GDCF database. shush.)
    NoCopy,

    /// The level is free to copy
    FreeCopy,

    /// The level requires the specified password to copy
    PasswordCopy(String),
}

// TODO: figure out a way to make the raw_type annotation not take the type by string.

/// Struct representing partial levels. These are returned to
/// [`LevelsRequest`](::api::request::level::LevelsRequest)s and only
/// contain metadata
/// on the level.
///
///
/// ## GD Internals:
/// The Geometry Dash servers provide lists of partial levels via the
/// `getGJLevels` endpoint.
///
/// ### Unmapped values:
/// + Index `8`: Index 8 is a boolean value indicating whether the level has a
/// difficulty rating that isn't N/A. This is equivalent to checking if
/// [`PartialLevel::difficulty`] is unequal to
/// [`LevelRating::NotAvailable`]
/// + Index `17`: Index 17 is a boolean value indicating whether
/// the level is a demon level. This is equivalent to checking if
/// [`PartialLevel::difficulty`] is the [`LevelRating::Demon`] variant.
/// + Index `25`: Index 25 is a boolean value indicating
/// whether the level is an auto level. This is equivalent to checking if
/// [`PartialLevel::difficulty`] is equal to
/// [`LevelRating::Auto`].
///
/// ### Unprovided values:
/// These values are not provided for by the `getGJLevels` endpoint and are
/// thus only modelled in the [`Level`] struct: `4`, `27`,
/// `28`, `29`, `36`
///
/// ### Unused indices:
/// The following indices aren't used by the Geometry Dash servers: `11`, `16`,
/// `17`, `20`, `21`, `22`, `23`, `24`, `26`, `31`, `32`, `33`, `34`, `40`,
/// `41`, `44`
#[cfg_attr(feature = "serde_support", derive(Serialize, Deserialize))]
#[derive(Debug, Eq, PartialEq, Clone)]
pub struct PartialLevel<Song, User>
where
    Song: PartialEq,
    User: PartialEq,
{
    /// The [`Level`]'s unique level id
    ///
    /// ## GD Internals:
    /// This value is provided at index `1`.
    pub level_id: u64,

    /// The [`Level`]'s name
    ///
    /// ## GD Internals:
    /// This value is provided at index `2`.
    pub name: String,

    /// The [`Level`]'s description. Is [`None`] if the creator didn't put any
    /// description.
    ///
    /// ## GD Internals:
    /// This value is provided at index `3` and encoded using urlsafe base 64.
    pub description: Option<String>,

    /// The [`PartialLevel`]'s version. The version get incremented every time
    /// the level is updated, and the initial version is always version 1.
    ///
    /// ## GD Internals:
    /// This value is provided at index `5`.
    pub version: u32,

    /// The ID of the [`Level`]'s creator
    ///
    /// ## GD Internals:
    /// This value is provided at index `6`.
    pub creator: User,

    /// The difficulty of this [`PartialLevel`]
    ///
    /// ## GD Internals:
    /// This value is a construct from the value at the indices `9`, `17` and
    /// `25`, whereas index 9 is an integer representation of either the
    /// [`LevelRating`] or the [`DemonRating`]
    /// struct, depending on the value of index 17.
    ///
    /// If index 25 is set to true, the level is an auto level and the value at
    /// index 9 is some nonsense, in which case it is ignored.
    pub difficulty: LevelRating,

    /// The amount of downloads
    ///
    /// ## GD Internals:
    /// This value is provided at index `10`
    pub downloads: u32,

    /// The [`MainSong`] the level uses, if any.
    ///
    /// ## GD Internals:
    /// This value is provided at index `12`. Interpretation is additionally
    /// dependant on the value at index `35` (the custom song id), as
    /// without that information, a value of `0` for
    /// this field could either mean the level uses `Stereo Madness` or no
    /// main song.
    #[cfg_attr(feature = "serde_support", serde(deserialize_with = "deserialize_main_song"))]
    pub main_song: Option<&'static MainSong>,

    /// The gd version the request was uploaded/last updated in.
    ///
    /// ## GD Internals:
    /// This value is provided at index `13`
    pub gd_version: GameVersion,

    /// The amount of likes this [`PartialLevel`] has received
    ///
    /// ## GD Internals:
    /// This value is provided at index `14`
    pub likes: i32,

    /// The length of this [`PartialLevel`]
    ///
    /// ## GD Internals:
    /// This value is provided as an integer representation of the
    /// [`LevelLength`] struct at index `15`
    pub length: LevelLength,

    /// The amount of stars completion of this [`PartialLevel`] awards
    ///
    /// ## GD Internals:
    /// This value is provided at index `18`
    pub stars: u8,

    /// This [`PartialLevel`]s featured state
    ///
    /// ## GD Internals:
    /// This value is provided at index `19`
    pub featured: Featured,

    /// The ID of the level this [`PartialLevel`] is a copy of, or [`None`], if
    /// this [`PartialLevel`] isn't a copy.
    ///
    /// ## GD Internals:
    /// This value is provided at index `30`
    pub copy_of: Option<u64>,

    /// The id of the newgrounds song this [`PartialLevel`] uses, or [`None`]
    /// if it useds a main song.
    ///
    /// ## GD Internals:
    /// This value is provided at index `35`, and a value of `0` means, that no
    /// custom song is used.
    pub custom_song: Option<Song>,

    /// The amount of coints in this [`PartialLevel`]
    ///
    /// ## GD Internals:
    /// This value is provided at index `37`
    pub coin_amount: u8,

    /// Value indicating whether the user coins (if present) in this
    /// [`PartialLevel`] are verified
    ///
    /// ## GD Internals:
    /// This value is provided at index `38`, as an integer
    pub coins_verified: bool,

    /// The amount of stars the level creator has requested when uploading this
    /// [`PartialLevel`], or [`None`] if no stars were requested.
    ///
    /// ## GD Internals:
    /// This value is provided at index `39`, and a value of `0` means no stars
    /// were requested
    pub stars_requested: Option<u8>,

    /// Value indicating whether this [`PartialLevel`] is epic
    ///
    /// ## GD Internals:
    /// This value is provided at index `42`, as an integer
    pub is_epic: bool,

    // TODO: figure this value out
    /// According to the GDPS source its a value called `starDemonDiff`. It
    /// seems to correlate to the level's difficulty.
    ///
    /// ## GD Internals:
    /// This value is provided at index `43` and seems to be an integer
    pub index_43: String,

    /// The amount of objects in this [`PartialLevel`]
    ///
    /// ## GD Internals:
    /// This value is provided at index `45`, although only for levels uploaded
    /// in version 2.1 or later. For all older levels this is always `0`
    pub object_amount: u32,

    /// According to the GDPS source this is always `1`, although that is
    /// evidently wrong
    ///
    /// ## GD Internals:
    /// This value is provided at index `46` and seems to be an integer
    pub index_46: String,

    /// According to the GDPS source, this is always `2`, although that is
    /// evidently wrong
    ///
    /// ## GD Internals:
    /// This value is provided at index `47` and seems to be an integer
    pub index_47: String,
}

/// Struct representing full levels, extending [`PartialLevel`] with the fields
/// only retrieved when fully downloading a level.
///
/// ## GD Internals:
/// The Geometry Dash servers provide full information about a level via the
/// `downloadGJLevel` endpoint
///
/// ### Unused indices:
/// The following indices aren't used by the Geometry Dash servers: `11`, `16`,
/// `17`, `20`, `21`, `22`, `23`, `24`, `26`, `31`, `32`, `33`, `34`, `40`,
/// `41`, `44`
#[derive(Debug, Eq, PartialEq, Clone)]
#[cfg_attr(feature = "serde_support", derive(Serialize))]
pub struct Level<Song, User>
where
    Song: PartialEq,
    User: PartialEq,
{
    /// The [`PartialLevel`] this [`Level`] instance supplements
    pub base: PartialLevel<Song, User>,

    /// The raw level data. Note that GDCF performs the base64 decoding, though
    /// not the `DEFLATE` decompression, since he compressed version of
    /// the level data requires the least amount of space.
    ///
    /// Use the ['gdcf_parse`] crate if you want to process this data.
    ///
    /// ## GD Internals:
    /// This value is provided at index `4`, and is urlsafe base64 encoded and
    /// `DEFLATE` compressed
    #[cfg_attr(feature = "serialize_level_data", serde(serialize_with = "base64_encode"))]
    #[cfg_attr(all(feature = "serde_support", not(feature = "serialize_level_data")), serde(skip_serializing))]
    pub level_data: Vec<u8>,

    /// The level's password
    ///
    /// ## GD Internals:
    /// This value is provided at index `27`, and is to be interpreted as
    /// follows: + If the provided value is `"0"`, then the level isn't
    /// copyable + Otherwise the value is base64 encoded and "encrypted"
    /// using robtop's XOR routine using key `26364`. If the "decrypted"
    /// value is `"1"`, the level is free to
    /// copy. Otherwise the decrypted value is the level password.
    pub password: Password,

    /// The time passed since the `Level` was uploaded
    ///
    /// ## GD Internals:
    /// This value is provided at index `28`
    pub time_since_upload: String,

    /// The time passed since this [`Level`] was last updated
    ///
    /// ## GD Internals:
    /// This value is provided at index `29`
    pub time_since_update: String,

    /// According to the GDPS source, this is a value called `extraString`
    pub index_36: String,
}

impl<Song, User> Display for PartialLevel<Song, User>
where
    Song: PartialEq,
    User: PartialEq,
{
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "PartialLevel({}, {})", self.level_id, self.name)
    }
}

impl<Song, User> Display for Level<Song, User>
where
    Song: PartialEq,
    User: PartialEq,
{
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "Level({}, {})", self.base.level_id, self.base.name)
    }
}

#[cfg(feature = "serde_support")]
fn deserialize_main_song<'de, D>(deserializer: D) -> Result<Option<&'static MainSong>, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::Deserialize as _;

    Ok(Option::<u8>::deserialize(deserializer)?.map(From::from))
}

#[cfg(feature = "serialize_level_data")]
use base64::{encode_config, URL_SAFE};
#[cfg(feature = "serialize_level_data")]
use serde::Serializer;

#[cfg(feature = "serialize_level_data")]
fn base64_encode<S>(level_data: &[u8], serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.collect_str(&encode_config(level_data, URL_SAFE))
}

impl From<i32> for Featured {
    fn from(value: i32) -> Self {
        match value {
            -1 => Featured::Unfeatured,
            0 => Featured::NotFeatured,
            _ => Featured::Featured(value as u32),
        }
    }
}

impl Into<i32> for Featured {
    fn into(self) -> i32 {
        match self {
            Featured::Unfeatured => -1,
            Featured::NotFeatured => 0,
            Featured::Featured(value) => value as i32,
        }
    }
}

impl From<i32> for LevelLength {
    fn from(length: i32) -> Self {
        match length {
            0 => LevelLength::Tiny,
            1 => LevelLength::Short,
            2 => LevelLength::Medium,
            3 => LevelLength::Long,
            4 => LevelLength::ExtraLong,
            _ => LevelLength::Unknown,
        }
    }
}

impl Into<i32> for LevelLength {
    fn into(self) -> i32 {
        match self {
            LevelLength::Tiny => 0,
            LevelLength::Short => 1,
            LevelLength::Medium => 2,
            LevelLength::Long => 3,
            LevelLength::ExtraLong => 4,
            LevelLength::Unknown => std::i32::MAX,
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
            _ => LevelRating::Unknown,
        }
    }
}

impl Into<i32> for LevelRating {
    fn into(self) -> i32 {
        match self {
            LevelRating::Auto => -3,
            LevelRating::Demon(_) => -2,
            LevelRating::NotAvailable => -1,
            LevelRating::Easy => 1,
            LevelRating::Normal => 2,
            LevelRating::Hard => 3,
            LevelRating::Harder => 4,
            LevelRating::Insane => 5,
            LevelRating::Unknown => std::i32::MAX,
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
            _ => DemonRating::Unknown,
        }
    }
}

impl Into<i32> for DemonRating {
    fn into(self) -> i32 {
        match self {
            DemonRating::Easy => 1,
            DemonRating::Medium => 2,
            DemonRating::Hard => 3,
            DemonRating::Insane => 4,
            DemonRating::Extreme => 5,
            DemonRating::Unknown => std::i32::MAX,
        }
    }
}

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
        }
        .to_string()
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
        }
        .to_string()
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
