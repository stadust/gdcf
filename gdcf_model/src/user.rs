//! Module containing all models related to users and their profiles

use std::fmt::{Display, Error, Formatter};

#[cfg(feature = "serde_support")]
use serde_derive::{Deserialize, Serialize};

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

    // TODO: figure this value out
    ///
    /// ## GD Internals:
    /// This value is provided at index `10`
    pub index_10: String,

    // TODO: figure this value out
    ///
    /// ## GD Internals:
    /// This value is provided at index `10`
    pub index_11: String,

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

    // TODO: figure this value out
    ///
    /// ## GD Internals:
    /// This value is provided at index `49`
    pub index_49: String,

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
