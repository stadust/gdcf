//! Module containing all models related to users and their profiles

use error::ValueError;
use model::{de, raw::RawObject};
use std::{
    convert::TryFrom,
    fmt::{Display, Error, Formatter},
};

lazy_static! {
    pub static ref DELETED: Creator = Creator {
        user_id: 0,
        name: String::new(),
        account_id: None
    };
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
#[derive(FromRawObject, Debug, Eq, PartialEq, Clone)]
pub struct Creator {
    /// The [`Creator`]'s unique user ID
    #[raw_data(index = 1)]
    pub user_id: u64,

    /// The [`Creator`]'s name
    #[raw_data(index = 2)]
    pub name: String,

    /// The [`Creator`]'s unique account ID
    #[raw_data(index = 3, deserialize_with = "de::default_to_none")]
    pub account_id: Option<u64>,
}

impl Display for Creator {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "Creator({})", self.name)
    }
}

/// Struct representing a Geometry Dash User
///
/// ## GD Internals:
/// The Geometry Dash servers provide user data in a `getGJProfile` response
///
/// ### Unused Indices
/// The following indices aren't used by the Geometry Dash servers: `5`, `6`, `7`, `9`, `12`, `14`,
/// `15`, `27`, `32`, `33`, `34`, `35`, `36`, `37`, `38`, `39`, `40`, `41`, `42`, `47`
#[derive(Debug, FromRawObject, Eq, PartialEq, Clone)]
#[cfg_attr(feature = "deser", derive(Serialize, Deserialize))]
pub struct User {
    /// The [`User`]'s name
    ///
    /// ## GD Internals:
    /// This value is provided at index `1`.
    #[raw_data(index = 1)]
    pub name: String,

    /// The [`User`]'s unique user ID
    ///
    /// ## GD Internals:
    /// This value is provided at index `2`
    #[raw_data(index = 2)]
    pub user_id: u64,

    /// The amount of stars this [`User`] has collected.
    ///
    /// ## GD Internals:
    /// This value is provided at index `3`
    #[raw_data(index = 3)]
    pub stars: u32,

    /// The demons of stars this [`User`] has beaten.
    ///
    /// ## GD Internals:
    /// This value is provided at index `4`
    #[raw_data(index = 4)]
    pub demons: u16,

    /// The amount of creator points this [`User`] was awarded.
    ///
    /// ## GD Internals:
    /// This value is provided at index `8`
    #[raw_data(index = 8)]
    pub creator_points: u16,

    // TODO: figure this value out
    ///
    /// ## GD Internals:
    /// This value is provided at index `10`
    #[raw_data(index = 10)]
    pub index_10: String,

    // TODO: figure this value out
    ///
    /// ## GD Internals:
    /// This value is provided at index `10`
    #[raw_data(index = 11)]
    pub index_11: String,

    /// The amount of secret coins this [`User`] has collected.
    ///
    /// ## GD Internals:
    /// This value is provided at index `13`
    #[raw_data(index = 13)]
    pub secret_coins: u8,

    /// The [`User`]'s unique account ID
    ///
    /// ## GD Internals:
    /// This value is provided at index `16`
    #[raw_data(index = 16)]
    pub account_id: u64,

    /// The amount of user coins this [`User`] has collected.
    ///
    /// ## GD Internals:
    /// This value is provided at index `17`
    #[raw_data(index = 17)]
    pub user_coins: u16,

    // TODO: figure this value out
    ///
    /// ## GD Internals:
    /// This value is provided at index `18`
    #[raw_data(index = 18)]
    pub index_18: String,

    // TODO: figure this value out
    ///
    /// ## GD Internals:
    /// This value is provided at index `19`
    #[raw_data(index = 19)]
    pub index_19: String,

    /// The link to the [`User`]'s [YouTube](https://youtube.com) channel, if provided
    ///
    /// ## GD Internals
    /// This value is provided at index `20`. The value provided is only the `username` section of an `https://www.youtube.com/user/{username}` URL
    #[raw_data(index = 20, default, deserialize_with = "de::youtube")]
    pub youtube_url: Option<String>,

    /// The 1-based index of the cube this [`User`] currently uses. Indexing of icons starts at the
    /// top left corner and then goes left-to-right and top-to-bottom
    ///
    /// ## GD Internals:
    /// This value is provied at index `21`
    #[raw_data(index = 21)]
    pub cube_index: u16,

    /// The 1-based index of the ship this [`User`] currently uses. Indexing of icons starts at the
    /// top left corner and then goes left-to-right and top-to-bottom
    ///
    /// ## GD Internals:
    /// This value is provied at index `22`
    #[raw_data(index = 22)]
    pub ship_index: u8,

    /// The 1-based index of the ball this [`User`] currently uses. Indexing of icons starts at the
    /// top left corner and then goes left-to-right and top-to-bottom
    ///
    /// ## GD Internals:
    /// This value is provied at index `23`
    #[raw_data(index = 23)]
    pub ball_index: u8,

    /// The 1-based index of the UFO this [`User`] currently uses. Indexing of icons starts at the
    /// top left corner and then goes left-to-right and top-to-bottom
    ///
    /// ## GD Internals:
    /// This value is provied at index `24`
    #[raw_data(index = 24)]
    pub ufo_index: u8,

    /// The 1-based index of the wave this [`User`] currently uses. Indexing of icons starts at the
    /// top left corner and then goes left-to-right and top-to-bottom
    ///
    /// ## GD Internals:
    /// This value is provied at index `25`
    #[raw_data(index = 25)]
    pub wave_index: String,

    /// The 1-based index of the robot this [`User`] currently uses. Indexing of icons starts at the
    /// top left corner and then goes left-to-right and top-to-bottom
    ///
    /// ## GD Internals:
    /// This value is provied at index `26`
    #[raw_data(index = 26)]
    pub robot_index: u8,

    /// Values indicating whether this [`User`] has glow activated or not.
    ///
    /// ## GD Internals:
    /// This value is provied at index `27`, as an integer
    #[raw_data(index = 28, deserialize_with = "de::int_to_bool")]
    pub has_glow: bool,

    // TODO: figure this value out
    ///
    /// ## GD Internals:
    /// This value is provided at index `29`
    #[raw_data(index = 29)]
    pub index_29: String,

    /// This [`User`]'s global rank. [`None`] if he is banned or not ranked.
    ///
    /// ## GD Internals:
    /// This value is provided at index `30`. For unranked/banned users it's `0`
    #[raw_data(index = 30, deserialize_with = "de::default_to_none")]
    pub global_rank: Option<u32>,

    // TODO: figure this value out
    ///
    /// ## GD Internals:
    /// This value is provided at index `31`
    #[raw_data(index = 31)]
    pub index_31: String,

    /// The 1-based index of the spider this [`User`] currently uses. Indexing of icons starts at
    /// the top left corner and then goes left-to-right and top-to-bottom
    ///
    /// ## GD Internals:
    /// This value is provied at index `43`
    #[raw_data(index = 43)]
    pub spider_index: u8,

    /// The link to the [`User`]'s [Twitter](https://twitter.com) account, if provided
    ///
    /// ## GD Internals
    /// This value is provided at index `44`. The value provided is only the `username` section of an `https://www.twitter.com/{username}` URL
    #[raw_data(index = 44, default, deserialize_with = "de::twitter")]
    pub twitter_url: Option<String>,

    /// The link to the [`User`]'s [Twitch](https://twitch.tv) channel, if provided
    ///
    /// ## GD Internals
    /// This value is provided at index `45`. The value provided is only the `username` section of an `https://twitch.tv/{username}` URL
    #[raw_data(index = 45, default, deserialize_with = "de::twitch")]
    pub twitch_url: Option<String>,

    /// The amount of diamonds this [`User`] has collected.
    ///
    /// ## GD Internals:
    /// This value is provided at index `46`
    #[raw_data(index = 46)]
    pub diamonds: u16,

    /// The 1-based index of the death-effect this [`User`] currently uses. Indexing of icons
    /// starts at the top left corner and then goes left-to-right and top-to-bottom
    ///
    /// ## GD Internals:
    /// This value is provied at index `48`
    #[raw_data(index = 48)]
    pub death_effect_index: u8,

    // TODO: figure this value out
    ///
    /// ## GD Internals:
    /// This value is provided at index `49`
    #[raw_data(index = 49)]
    pub index_49: String,

    // TODO: figure this value out
    ///
    /// ## GD Internals:
    /// This value is provided at index `50`
    #[raw_data(index = 50)]
    pub index_50: String,
}


impl Display for User {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "User({}, {})", self.user_id, self.name)
    }
}