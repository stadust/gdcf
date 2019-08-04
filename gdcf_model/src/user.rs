//! Module containing all models related to users and their profiles

use std::fmt::{Display, Error, Formatter};

use crate::GameMode;
#[cfg(feature = "serde_support")]
use serde_derive::{Deserialize, Serialize};

/// Enum representing the different types of moderator a user can be
#[derive(Debug, PartialEq, Eq, Hash, Copy, Clone)]
#[cfg_attr(feature = "serde_support", derive(Serialize, Deserialize))]
pub enum ModLevel {
    /// User isn't a moderator
    None,

    /// User is a normal moderator
    Normal,

    /// User is an elder moderator
    Elder,

    /// Unknown or invalid value. This variant will be constructed if robtop ever adds more
    /// moderator levels and will hold the internal game value associated with the new moderator
    /// level
    Unknown(u8),
}

impl Default for ModLevel {
    fn default() -> ModLevel {
        ModLevel::None
    }
}

// Enum representing an in-game icon color
#[derive(Debug, PartialEq, Eq, Hash, Copy, Clone)]
#[cfg_attr(feature = "serde_support", derive(Serialize, Deserialize))]
pub enum Color {
    /// A color whose index was known to GDCF which could be converted to RGB values
    Known(u8, u8, u8),

    /// The index of some unknown colors. This variant will be constructed if robtop ever adds more
    /// colors and while GDCF hasn't updated yet
    Unknown(u8),
}

/// Struct representing a [`Level`](::model::level::Level)'s creator.
///
/// ## GD Internals:
/// These minimal representations of a [`User`] are provided by the Geometry Dash servers in a
/// `getGJLevels` response.
///
/// ### Indexing:
/// These objects aren't indexed in the response. The indexes used here are based on the order in
/// which the fields appear in the response
#[derive(Debug, Eq, PartialEq, Clone)]
#[cfg_attr(feature = "serde_support", derive(Serialize, Deserialize))]
pub struct Creator {
    /// The [`Creator`]'s unique user ID
    pub user_id: u64,

    /// The [`Creator`]'s name
    pub name: String,

    /// The [`Creator`]'s unique account ID
    pub account_id: Option<u64>,
}

impl Display for Creator {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "Creator({})", self.name)
    }
}

impl Creator {
    pub fn deleted(id: u64) -> Creator {
        Creator {
            user_id: id,
            name: "<DELETED>".to_string(),
            account_id: None,
        }
    }
}

/// Struct representing a Geometry Dash User
///
/// ## GD Internals:
/// The Geometry Dash servers provide user data in a `getGJUserInfo` response
///
/// ### Unused Indices
/// The following indices aren't used by the Geometry Dash servers: `5`, `6`, `7`, `9`, `12`, `14`,
/// `15`, `27`, `32`, `33`, `34`, `35`, `36`, `37`, `38`, `39`, `40`, `41`, `42`, `47`
#[derive(Debug, Eq, PartialEq, Clone)]
#[cfg_attr(feature = "serde_support", derive(Serialize, Deserialize))]
pub struct User {
    /// The [`User`]'s name
    ///
    /// ## GD Internals:
    /// This value is provided at index `1`.
    pub name: String,

    /// The [`User`]'s unique user ID
    ///
    /// ## GD Internals:
    /// This value is provided at index `2`
    pub user_id: u64,

    /// The amount of stars this [`User`] has collected.
    ///
    /// ## GD Internals:
    /// This value is provided at index `3`
    pub stars: u32,

    /// The demons of stars this [`User`] has beaten.
    ///
    /// ## GD Internals:
    /// This value is provided at index `4`
    pub demons: u16,

    /// The amount of creator points this [`User`] was awarded.
    ///
    /// ## GD Internals:
    /// This value is provided at index `8`
    pub creator_points: u16,

    /// This [`User`]'s primary color
    ///
    /// ## GD Internals:
    /// This value is provided at index `10`. The game internally assigned each color some really
    /// obscure ID that doesn't correspond to the index in the game's color selector at all, which
    /// makes it pretty useless. GDCF thus translates all in-game colors into their RGB
    /// representation.
    pub primary_color: Color,

    /// This [`User`]'s secondary color
    ///
    /// ## GD Internals:
    /// This value is provided at index `11`. Same things as above apply
    pub secondary_color: Color,

    /// The amount of secret coins this [`User`] has collected.
    ///
    /// ## GD Internals:
    /// This value is provided at index `13`
    pub secret_coins: u8,

    /// The [`User`]'s unique account ID
    ///
    /// ## GD Internals:
    /// This value is provided at index `16`
    pub account_id: u64,

    /// The amount of user coins this [`User`] has collected.
    ///
    /// ## GD Internals:
    /// This value is provided at index `17`
    pub user_coins: u16,

    // TODO: figure this value out
    ///
    /// ## GD Internals:
    /// This value is provided at index `18`
    pub index_18: String,

    // TODO: figure this value out
    ///
    /// ## GD Internals:
    /// This value is provided at index `19`
    pub index_19: String,

    /// The link to the [`User`]'s [YouTube](https://youtube.com) channel, if provided
    ///
    /// ## GD Internals
    /// This value is provided at index `20`. The value provided is only the `username` section of an `https://www.youtube.com/user/{username}` URL
    pub youtube_url: Option<String>,

    /// The 1-based index of the cube this [`User`] currently uses. Indexing of icons starts at the
    /// top left corner and then goes left-to-right and top-to-bottom
    ///
    /// ## GD Internals:
    /// This value is provied at index `21`
    pub cube_index: u16,

    /// The 1-based index of the ship this [`User`] currently uses. Indexing of icons starts at the
    /// top left corner and then goes left-to-right and top-to-bottom
    ///
    /// ## GD Internals:
    /// This value is provied at index `22`
    pub ship_index: u8,

    /// The 1-based index of the ball this [`User`] currently uses. Indexing of icons starts at the
    /// top left corner and then goes left-to-right and top-to-bottom
    ///
    /// ## GD Internals:
    /// This value is provied at index `23`
    pub ball_index: u8,

    /// The 1-based index of the UFO this [`User`] currently uses. Indexing of icons starts at the
    /// top left corner and then goes left-to-right and top-to-bottom
    ///
    /// ## GD Internals:
    /// This value is provied at index `24`
    pub ufo_index: u8,

    /// The 1-based index of the wave this [`User`] currently uses. Indexing of icons starts at the
    /// top left corner and then goes left-to-right and top-to-bottom
    ///
    /// ## GD Internals:
    /// This value is provied at index `25`
    pub wave_index: u8,

    /// The 1-based index of the robot this [`User`] currently uses. Indexing of icons starts at the
    /// top left corner and then goes left-to-right and top-to-bottom
    ///
    /// ## GD Internals:
    /// This value is provied at index `26`
    pub robot_index: u8,

    /// Values indicating whether this [`User`] has glow activated or not.
    ///
    /// ## GD Internals:
    /// This value is provied at index `27`, as an integer
    pub has_glow: bool,

    // TODO: figure this value out
    ///
    /// ## GD Internals:
    /// This value is provided at index `29`
    pub index_29: String,

    /// This [`User`]'s global rank. [`None`] if he is banned or not ranked.
    ///
    /// ## GD Internals:
    /// This value is provided at index `30`. For unranked/banned users it's `0`
    pub global_rank: Option<u32>,

    // TODO: figure this value out
    ///
    /// ## GD Internals:
    /// This value is provided at index `31`
    pub index_31: String,

    /// The 1-based index of the spider this [`User`] currently uses. Indexing of icons starts at
    /// the top left corner and then goes left-to-right and top-to-bottom
    ///
    /// ## GD Internals:
    /// This value is provied at index `43`
    pub spider_index: u8,

    /// The link to the [`User`]'s [Twitter](https://twitter.com) account, if provided
    ///
    /// ## GD Internals
    /// This value is provided at index `44`. The value provided is only the `username` section of an `https://www.twitter.com/{username}` URL
    pub twitter_url: Option<String>,

    /// The link to the [`User`]'s [Twitch](https://twitch.tv) channel, if provided
    ///
    /// ## GD Internals
    /// This value is provided at index `45`. The value provided is only the `username` section of an `https://twitch.tv/{username}` URL
    pub twitch_url: Option<String>,

    /// The amount of diamonds this [`User`] has collected.
    ///
    /// ## GD Internals:
    /// This value is provided at index `46`
    pub diamonds: u16,

    /// The 1-based index of the death-effect this [`User`] currently uses. Indexing of icons
    /// starts at the top left corner and then goes left-to-right and top-to-bottom
    ///
    /// ## GD Internals:
    /// This value is provied at index `48`
    pub death_effect_index: u8,

    /// The level of moderator this [`User`] is
    ///
    /// ## GD Internals:
    /// This value is provided at index `49`
    pub mod_level: ModLevel,

    // TODO: figure this value out
    ///
    /// ## GD Internals:
    /// This value is provided at index `50`
    pub index_50: String,
}

impl Display for User {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "User({}, {})", self.user_id, self.name)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchedUser {
    /// This [`SearchedUser`]'s name
    ///
    /// ## GD Internals:
    /// This value is provided at index `1`
    pub name: String,

    /// The [`SearchedUser`]'s unique user ID
    ///
    /// ## GD Internals:
    /// This value is provided at index `2`
    pub user_id: u64,

    /// This [`SearchedUser`]'s stars
    ///
    /// ## GD Internals:
    /// This value is provided at index `3`
    pub stars: u32,

    /// This [`SearchedUser`]'s beaten demons
    ///
    /// ## GD Internals:
    /// This value is provided at index `4`
    pub demons: u16,

    // TODO: figure this value out
    ///
    /// ## GD Internals:
    /// This value is provided at index `6`
    pub index_6: Option<String>,

    /// This [`SearchedUsers`] creator points
    ///
    /// ## GD Internals:
    /// This value is provided at index `8`
    pub creator_points: u16,

    /// The index of the icon being displayed.
    ///
    /// ## GD Internals:
    /// This value is provided at index `9`
    pub icon_index: u16,

    /// This [`SearchedUser`]'s primary color
    ///
    /// ## GD Internals:
    /// This value is provided at index `10`. The game internally assigned each color some really
    /// obscure ID that doesn't correspond to the index in the game's color selector at all, which
    /// makes it pretty useless. GDCF thus translates all in-game colors into their RGB
    /// representation.
    pub primary_color: Color,

    /// This [`SearchedUser`]'s secondary color
    ///
    /// ## GD Internals:
    /// This value is provided at index `11`. Same things as above apply
    pub secondary_color: Color,

    /// The amount of secret coins this [`SearchedUser`] has collected.
    ///
    /// ## GD Internals:
    /// This value is provided at index `13`
    pub secret_coins: u8,

    /// The type of icon being displayed
    ///
    /// ## GD Internals:
    /// This value is provided at index `14`
    pub icon_type: GameMode,

    /// Values indicating whether this [`SearchedUser`] has glow activated or not.
    ///
    /// ## GD Internals:
    /// This value is provided at index `15`
    pub has_glow: bool,

    /// The [`SearchedUser`]'s unique account ID
    ///
    /// ## GD Internals:
    /// This value is provided at index `16`
    pub account_id: u64,

    /// The amount of user coins this [`SearchedUser`] has collected.
    ///
    /// ## GD Internals:
    /// This value is provided at index `17`
    pub user_coins: u16,
}

impl Display for SearchedUser {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "SearchedUser({}, {})", self.user_id, self.name)
    }
}

impl Into<u8> for ModLevel {
    fn into(self) -> u8 {
        match self {
            ModLevel::None => 0,
            ModLevel::Normal => 1,
            ModLevel::Elder => 2,
            ModLevel::Unknown(inner) => inner,
        }
    }
}

impl From<u8> for ModLevel {
    fn from(i: u8) -> Self {
        match i {
            0 => ModLevel::None,
            1 => ModLevel::Normal,
            2 => ModLevel::Elder,
            i => ModLevel::Unknown(i),
        }
    }
}

impl From<u8> for Color {
    fn from(idx: u8) -> Self {
        // This match expression is listing the colors in order of the in-game selection menu!
        match idx {
            0 => Color::Known(125, 255, 0),
            1 => Color::Known(0, 255, 0),
            2 => Color::Known(0, 255, 125),
            3 => Color::Known(0, 255, 255),
            16 => Color::Known(0, 200, 255),
            4 => Color::Known(0, 125, 255),
            5 => Color::Known(0, 0, 255),
            6 => Color::Known(125, 0, 255),
            13 => Color::Known(185, 0, 255),
            7 => Color::Known(255, 0, 255),
            8 => Color::Known(255, 0, 125),
            9 => Color::Known(255, 0, 0),
            29 => Color::Known(255, 75, 0),
            10 => Color::Known(255, 125, 0),
            14 => Color::Known(255, 185, 0),
            11 => Color::Known(255, 255, 0),
            12 => Color::Known(255, 255, 255),
            17 => Color::Known(175, 175, 175),
            18 => Color::Known(80, 80, 80),
            15 => Color::Known(0, 0, 0),
            27 => Color::Known(125, 125, 0),
            32 => Color::Known(100, 150, 0),
            28 => Color::Known(75, 175, 0),
            38 => Color::Known(0, 150, 0),
            20 => Color::Known(0, 175, 75),
            33 => Color::Known(0, 150, 100),
            21 => Color::Known(0, 125, 125),
            34 => Color::Known(0, 100, 150),
            22 => Color::Known(0, 75, 175),
            39 => Color::Known(0, 0, 150),
            23 => Color::Known(75, 0, 175),
            35 => Color::Known(100, 0, 150),
            24 => Color::Known(125, 0, 125),
            36 => Color::Known(150, 0, 100),
            25 => Color::Known(175, 0, 75),
            37 => Color::Known(150, 0, 0),
            30 => Color::Known(150, 50, 0),
            26 => Color::Known(175, 75, 0),
            31 => Color::Known(150, 100, 0),
            19 => Color::Known(255, 255, 125),
            40 => Color::Known(125, 255, 175),
            41 => Color::Known(125, 125, 255),
            idx => Color::Unknown(idx),
        }
    }
}
