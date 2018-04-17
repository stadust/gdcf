use gdcf::model::LevelLength;
use gdcf::model::GameVersion;
use gdcf::model::LevelRating;

use self::util::Join;

use serde::Serializer;
use serde::ser::SerializeMap;

mod request;
mod util;

pub use self::request::BaseRequestRem;
pub use self::request::level::LevelRequestRem;
pub use self::request::level::LevelsRequestRem;
use gdcf::api::request::level::LevelRequestType;
use gdcf::api::request::level::SearchFilters;
use gdcf::api::request::level::SongFilter;
use gdcf::model::DemonRating;

pub(super) fn game_version<S>(version: &GameVersion, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer
{
    serializer.collect_str(&version.to_string())
}

pub(super) fn bool_to_int<S>(value: &bool, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer
{
    serializer.serialize_u8(*value as u8)
}

pub(super) fn length_vec<S>(values: &Vec<LevelLength>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer
{
    if values.is_empty() {
        serializer.serialize_str("-")
    } else {
        serializer.serialize_str(
            &values.into_iter()
                .map(|length| i32::from(*length))
                .join(",")
        )
    }
}

pub(super) fn rating_vec<S>(values: &Vec<LevelRating>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer
{
    if values.is_empty() {
        serializer.serialize_str("-")
    } else {
        serializer.serialize_str(
            &values.into_iter()
                .map(|rating| i32::from(*rating))
                .join(",")
        )
    }
}

pub(super) fn demon_rating<S>(rating: &Option<DemonRating>, serialize: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer
{
    serialize.serialize_i32(i32::from(rating.unwrap()))
}

pub(super) fn req_type<S>(req_type: &LevelRequestType, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer
{
    serializer.serialize_i32(i32::from(*req_type))
}

pub(super) fn search_filters<S>(filters: &SearchFilters, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer
{
    let mut map = serializer.serialize_map(None)?;

    map.serialize_entry("uncompleted", &(filters.uncompleted as u8))?;
    map.serialize_entry("onlyCompleted", &(filters.completed as u8))?;
    map.serialize_entry("featured", &(filters.featured as u8))?;
    map.serialize_entry("original", &(filters.original as u8))?;
    map.serialize_entry("twoPlayer", &(filters.two_player as u8))?;
    map.serialize_entry("coins", &(filters.coins as u8))?;
    map.serialize_entry("epic", &(filters.epic as u8))?;

    match filters.song {
        Some(SongFilter::Main(id)) => map.serialize_entry("song", &id)?,
        Some(SongFilter::Custom(id)) => {
            map.serialize_entry("customSong", &1)?;
            map.serialize_entry("song", &id)?;
        }
        _ => ()
    }

    map.end()
}