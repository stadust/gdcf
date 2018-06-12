use gdcf::model::GameVersion;
use gdcf::model::LevelLength;
use gdcf::model::LevelRating;

use serde::ser::SerializeMap;
use serde::Serializer;

mod request;
mod util;

pub use self::request::level::LevelRequestRem;
pub use self::request::level::LevelsRequestRem;
pub use self::request::BaseRequestRem;
use gdcf::api::request::level::LevelRequestType;
use gdcf::api::request::level::SearchFilters;
use gdcf::api::request::level::SongFilter;
use gdcf::model::DemonRating;
use gdcf::convert::{self, from};

pub(super) fn game_version<S>(version: &GameVersion, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.collect_str(&version.to_string())
}

pub(super) fn bool_to_int<S>(value: &bool, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_u8(*value as u8)
}

pub(super) fn length_vec<S>(values: &Vec<LevelLength>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&convert::from::vec(values))
}

pub(super) fn rating_vec<S>(values: &Vec<LevelRating>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&convert::from::vec(values))
}

pub(super) fn demon_rating<S>(rating: &Option<DemonRating>, serialize: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serialize.serialize_i32(rating.unwrap().into())
}

pub(super) fn req_type<S>(req_type: &LevelRequestType, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_i32(i32::from(*req_type))
}

pub(super) fn search_filters<S>(filters: &SearchFilters, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut map = serializer.serialize_map(None)?;

    map.serialize_entry("uncompleted", &from::bool(filters.uncompleted))?;
    map.serialize_entry("onlyCompleted", &from::bool(filters.completed))?;
    map.serialize_entry("featured", &from::bool(filters.featured))?;
    map.serialize_entry("original", &from::bool(filters.original))?;
    map.serialize_entry("twoPlayer", &from::bool(filters.two_player))?;
    map.serialize_entry("coins", &from::bool(filters.coins))?;
    map.serialize_entry("epic", &from::bool(filters.epic))?;
    map.serialize_entry("star", &from::bool(filters.rated))?;

    match filters.song {
        Some(SongFilter::Main(id)) => map.serialize_entry("song", &id)?,
        Some(SongFilter::Custom(id)) => {
            map.serialize_entry("customSong", &1)?;
            map.serialize_entry("song", &id)?;
        }
        _ => (),
    }

    map.end()
}
