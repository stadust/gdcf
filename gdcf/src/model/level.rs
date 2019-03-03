//! Module containing all models related to Geometry Dash levels

use convert;
use error::ValueError;
use flate2::read::GzDecoder;
use model::{de, raw::RawObject, GameVersion, MainSong};
#[cfg(feature = "deser")]
use serde::{Deserialize, Serialize, Serializer};
use std::{
    convert::TryFrom,
    fmt::{Display, Error, Formatter},
    io::Read,
    num::ParseFloatError,
};

/// Enum representing the possible level lengths known to GDCF
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "deser", derive(Serialize, Deserialize))]
pub enum LevelLength {
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

    /// Enum variant that's used by the [`From<i32>`](From) impl for when an
    /// unrecognized value is passed
    Unknown,
}

/// Enum representing the possible level ratings
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "deser", derive(Serialize, Deserialize))]
pub enum LevelRating {
    /// Auto rating
    ///
    /// ## GD Internals:
    /// This variant is represented by the value `-3` in requests, and not
    /// included in responses.
    Auto,

    /// Demon rating.
    ///
    /// ## GD Internals:
    /// This variant is represented by the value `-2` in requests. In
    /// responses, you will have to first check the provided level is a
    /// demon and then interpret the provided
    /// `rating` value as a [`DemonRating`]
    Demon(DemonRating),

    /// Not Available, sometimes referred to as `N/A` or `NA`
    ///
    /// ## GD Internals:
    /// This variant is represented by the value `-1` in requests and by the
    /// value `0` in responses
    NotAvailable,

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

    /// Enum variant that's used by the [`From<i32>`](From) impl for when an
    /// unrecognized value is passed
    Unknown,
}

/// Enum representing the possible demon difficulties
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "deser", derive(Serialize, Deserialize))]
pub enum DemonRating {
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

    /// Enum variant that's used by the [`From<i32>`](From) impl for when an
    /// unrecognized value is passed
    Unknown,
}

/// Enum representing a levels featured state
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "deser", derive(Serialize, Deserialize))]
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
#[cfg_attr(feature = "deser", derive(Serialize, Deserialize))]
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
#[cfg_attr(feature = "deser", derive(Serialize))] // TODO: a Deserialize impl will have to be custom-written due to the &'static MainSong reference
#[derive(Debug, FromRawObject, Eq, PartialEq, Clone)]
pub struct PartialLevel<#[raw_type("u64")] Song, #[raw_type("u64")] User>
// FIXME: the raw_type attribute breaks serde for some reason. Find an alternative
where
    Song: PartialEq,
    User: PartialEq,
{
    /// The [`Level`]'s unique level id
    ///
    /// ## GD Internals:
    /// This value is provided at index `1`.
    #[raw_data(index = 1)]
    pub level_id: u64,

    /// The [`Level`]'s name
    ///
    /// ## GD Internals:
    /// This value is provided at index `2`.
    #[raw_data(index = 2)]
    pub name: String,

    /// The [`Level`]'s description. Is [`None`] if the creator didn't put any
    /// description.
    ///
    /// ## GD Internals:
    /// This value is provided at index `3` and encoded using urlsafe base 64.
    #[raw_data(index = 3, deserialize_with = "de::description", default)]
    pub description: Option<String>,

    /// The [`PartialLevel`]'s version. The version get incremented every time
    /// the level is updated, and the initial version is always version 1.
    ///
    /// ## GD Internals:
    /// This value is provided at index `5`.
    #[raw_data(index = 5)]
    pub version: u32,

    /// The ID of the [`Level`]'s creator
    ///
    /// ## GD Internals:
    /// This value is provided at index `6`.
    #[raw_data(index = 6)]
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
    #[raw_data(custom = "de::level_rating")]
    pub difficulty: LevelRating,

    /// The amount of downloads
    ///
    /// ## GD Internals:
    /// This value is provided at index `10`
    #[raw_data(index = 10)]
    pub downloads: u32,

    /// The [`MainSong`] the level uses, if any.
    ///
    /// ## GD Internals:
    /// This value is provided at index `12`. Interpretation is additionally
    /// dependant on the value at index `35` (the custom song id), as
    /// without that information, a value of `0` for
    /// this field could either mean the level uses `Stereo Madness` or no
    /// main song.
    #[raw_data(custom = "de::main_song")]
    pub main_song: Option<&'static MainSong>,

    /// The gd version the request was uploaded/last updated in.
    ///
    /// ## GD Internals:
    /// This value is provided at index `13`
    #[raw_data(index = 13)]
    pub gd_version: GameVersion,

    /// The amount of likes this [`PartialLevel`] has received
    ///
    /// ## GD Internals:
    /// This value is provided at index `14`
    #[raw_data(index = 14)]
    pub likes: i32,

    /// The length of this [`PartialLevel`]
    ///
    /// ## GD Internals:
    /// This value is provided as an integer representation of the
    /// [`LevelLength`] struct at index `15`
    #[raw_data(index = 15)]
    pub length: LevelLength,

    /// The amount of stars completion of this [`PartialLevel`] awards
    ///
    /// ## GD Internals:
    /// This value is provided at index `18`
    #[raw_data(index = 18)]
    pub stars: u8,

    /// This [`PartialLevel`]s featured state
    ///
    /// ## GD Internals:
    /// This value is provided at index `19`
    #[raw_data(index = 19)]
    pub featured: Featured,

    /// The ID of the level this [`PartialLevel`] is a copy of, or [`None`], if
    /// this [`PartialLevel`] isn't a copy.
    ///
    /// ## GD Internals:
    /// This value is provided at index `30`
    #[raw_data(index = 30, deserialize_with = "de::default_to_none")]
    pub copy_of: Option<u64>,

    /// The id of the newgrounds song this [`PartialLevel`] uses, or [`None`]
    /// if it useds a main song.
    ///
    /// ## GD Internals:
    /// This value is provided at index `35`, and a value of `0` means, that no
    /// custom song is used.
    #[raw_data(index = 35, deserialize_with = "de::default_to_none")]
    pub custom_song: Option<Song>,

    /// The amount of coints in this [`PartialLevel`]
    ///
    /// ## GD Internals:
    /// This value is provided at index `37`
    #[raw_data(index = 37)]
    pub coin_amount: u8,

    /// Value indicating whether the user coins (if present) in this
    /// [`PartialLevel`] are verified
    ///
    /// ## GD Internals:
    /// This value is provided at index `38`, as an integer
    #[raw_data(index = 38, deserialize_with = "de::int_to_bool")]
    pub coins_verified: bool,

    /// The amount of stars the level creator has requested when uploading this
    /// [`PartialLevel`], or [`None`] if no stars were requested.
    ///
    /// ## GD Internals:
    /// This value is provided at index `39`, and a value of `0` means no stars
    /// were requested
    #[raw_data(index = 39, deserialize_with = "de::default_to_none")]
    pub stars_requested: Option<u8>,

    /// Value indicating whether this [`PartialLevel`] is epic
    ///
    /// ## GD Internals:
    /// This value is provided at index `42`, as an integer
    #[raw_data(index = 42, deserialize_with = "de::int_to_bool")]
    pub is_epic: bool,

    // TODO: figure this value out
    /// According to the GDPS source its a value called `starDemonDiff`. It
    /// seems to correlate to the level's difficulty.
    ///
    /// ## GD Internals:
    /// This value is provided at index `43` and seems to be an integer
    #[raw_data(index = 43)]
    pub index_43: String,

    /// The amount of objects in this [`PartialLevel`]
    ///
    /// ## GD Internals:
    /// This value is provided at index `45`, although only for levels uploaded
    /// in version 2.1 or later. For all older levels this is always `0`
    #[raw_data(index = 45)]
    pub object_amount: u32,

    /// According to the GDPS source this is always `1`, although that is
    /// evidently wrong
    ///
    /// ## GD Internals:
    /// This value is provided at index `46` and seems to be an integer
    #[raw_data(index = 46, default)]
    pub index_46: String,

    /// According to the GDPS source, this is always `2`, although that is
    /// evidently wrong
    ///
    /// ## GD Internals:
    /// This value is provided at index `47` and seems to be an integer
    #[raw_data(index = 47, default)]
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
#[derive(Debug, FromRawObject, Eq, PartialEq, Clone)]
#[cfg_attr(feature = "deser", derive(Serialize))]
pub struct Level<#[raw_type("u64")] Song, #[raw_type("u64")] User>
where
    Song: PartialEq,
    User: PartialEq,
{
    /// The [`PartialLevel`] this [`Level`] instance supplements
    #[raw_data(flatten)]
    pub base: PartialLevel<Song, User>,

    /// The raw level data. Note that GDCF performs the base64 decoding, though
    /// not the `DEFLATE` decompression, since he compressed version of
    /// the level data requires the least amount of space.
    ///
    /// ## GD Internals:
    /// This value is provided at index `4`, and is urlsafe base64 encoded and
    /// `DEFLATE` compressed
    #[raw_data(index = 4, deserialize_with = "convert::to::b64_decoded_bytes")]
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
    #[raw_data(index = 27, deserialize_with = "convert::to::level_password")]
    pub password: Password,

    /// The time passed since the `Level` was uploaded
    ///
    /// ## GD Internals:
    /// This value is provided at index `28`
    #[raw_data(index = 28)]
    pub time_since_upload: String,

    /// The time passed since this [`Level`] was last updated
    ///
    /// ## GD Internals:
    /// This value is provided at index `29`
    #[raw_data(index = 29)]
    pub time_since_update: String,

    /// According to the GDPS source, this is a value called `extraString`
    #[raw_data(index = 36, default)]
    pub index_36: String,
}

// The following code has been contributed by cos8o! Thank you!

#[derive(Debug)]
pub struct LevelData(String);

impl<S: PartialEq, U: PartialEq> Level<S, U> {
    pub fn decompress_data(&self) -> std::io::Result<LevelData> {
        let mut s = String::new();
        let mut d = GzDecoder::new(&self.level_data[..]);

        d.read_to_string(&mut s)?;

        Ok(LevelData(s))
    }
}

impl LevelData {
    pub fn objects(&self) -> Vec<&str> {
        self.0.split(';').skip(1).collect()
    }

    pub fn object_count(&self) -> usize {
        self.objects().len()
    }

    pub fn furthest_object_x(&self) -> f32 {
        self.objects().iter().filter_map(|&s| object_x(s).ok()).fold(0.0, f32::max)
    }
}

fn object_x(object: &str) -> Result<f32, ParseFloatError> {
    let mut iter = object.split(',');

    match iter.position(|v| v == "2") {
        Some(idx) if idx % 2 == 0 =>
            match iter.next() {
                Some(v) => v.parse(),
                None => Ok(0.0),
            },
        _ => Ok(0.0),
    }
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
