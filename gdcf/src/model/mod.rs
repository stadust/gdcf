//! Module containing all the GDCF models
//!
//! The GDCF models do not attempt to exactly represent the responses received
//! with the [`ApiClient`](`::api::client::ApiClient`) but rather provide a
//! level of abstraction
//! that makes it easy to
//! work with the provided
//! data.
//!
//! Note that the purpose of some values sent by the servers is unknown. These
//! values of provided as [`String`]s and named after the index they appeared
//! in the server data.

pub use self::{
    level::{DemonRating, Featured, Level, LevelLength, LevelRating, PartialLevel, Password},
    song::{MainSong, NewgroundsSong},
    user::{Creator, User},
};
use std::fmt::{self, Display, Formatter};

mod de;
pub mod level;
pub mod raw;
pub mod song;
pub mod user;

/// Enum modelling the version of a Geometry Dash client
#[derive(Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq, Hash)]
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

#[derive(Debug, Clone, PartialEq)]
pub enum GDObject {
    NewgroundsSong(NewgroundsSong),
    PartialLevel(PartialLevel<u64, u64>),
    Level(Level<u64, u64>),
    Creator(Creator),
    User(User),
}

impl From<NewgroundsSong> for GDObject {
    fn from(song: NewgroundsSong) -> Self {
        GDObject::NewgroundsSong(song)
    }
}

impl From<Creator> for GDObject {
    fn from(creator: Creator) -> Self {
        GDObject::Creator(creator)
    }
}

impl From<PartialLevel<u64, u64>> for GDObject {
    fn from(level: PartialLevel<u64, u64>) -> Self {
        GDObject::PartialLevel(level)
    }
}

impl From<Level<u64, u64>> for GDObject {
    fn from(level: Level<u64, u64>) -> Self {
        GDObject::Level(level)
    }
}

impl From<User> for GDObject {
    fn from(user: User) -> Self {
        GDObject::User(user)
    }
}

impl Display for GDObject {
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        match self {
            GDObject::NewgroundsSong(inner) => inner.fmt(f),
            GDObject::PartialLevel(inner) => inner.fmt(f),
            GDObject::Level(inner) => inner.fmt(f),
            GDObject::Creator(inner) => inner.fmt(f),
            GDObject::User(inner) => inner.fmt(f),
        }
    }
}
