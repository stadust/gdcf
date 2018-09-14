pub use self::request::{
    level::{LevelRequestRem, LevelsRequestRem},
    user::UserRequestRem,
    BaseRequestRem,
};
use gdcf::{
    api::request::level::{CompletionFilter, LevelRequestType, SearchFilters, SongFilter},
    convert::{self, from},
    model::{DemonRating, GameVersion, LevelLength, LevelRating},
};
use serde::{ser::SerializeMap, Serializer};

mod request;

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

    match filters.completion {
        CompletionFilter::None => {
            map.serialize_entry("uncompleted", &0)?;
            map.serialize_entry("onlyCompleted", &0)?;
        },
        CompletionFilter::List { ref ids, include } => {
            map.serialize_entry("completedLevels", &convert::from::level_list(ids))?;
            if include {
                map.serialize_entry("uncompleted", &0)?;
                map.serialize_entry("onlyCompleted", &1)?;
            } else {
                map.serialize_entry("uncompleted", &1)?;
                map.serialize_entry("onlyCompleted", &0)?;
            }
        },
    }

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
        },
        _ => (),
    }

    map.end()
}
