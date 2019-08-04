pub use self::request::{
    comment::{LevelCommentsRequestRem, ProfileCommentsRequestRem},
    level::{LevelRequestRem, LevelsRequestRem},
    user::{UserRequestRem, UserSearchRequestRem},
    BaseRequestRem,
};
use gdcf::api::request::{
    comment::SortMode,
    level::{CompletionFilter, LevelRequestType, SearchFilters, SongFilter},
};
use gdcf_model::{
    level::{DemonRating, LevelLength, LevelRating},
    GameVersion,
};
use gdcf_parse::convert::RobtopInto;
use joinery::Joinable;
use serde::{ser::SerializeMap, Serializer};

mod request;

/// Converts the given [`Vec`] of values convertible into signed integers
/// into a robtop-approved string.
pub fn vec<T: RobtopInto<T, String> + Copy>(list: &[T]) -> String {
    if list.is_empty() {
        String::from("-")
    } else {
        list.iter().map(|v| T::robtop_into_req(*v)).join_with(",").to_string()
    }
}

/// Converts the given `bool` into an `u8`, returning `0` if `value ==
/// false` and `1`otherwise.
///
/// This can be seen as the inverse to [`bool`](::convert::to::bool)
pub const fn bool(value: bool) -> u8 {
    value as u8
}

pub fn level_list(ids: &[u64]) -> String {
    let mut ids = ids.iter().join_with(",").to_string();
    ids.push(')');
    ids.insert(0, '(');
    ids
}

pub(super) fn game_version<S>(version: &GameVersion, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&version.robtop_into())
}

pub(super) fn bool_to_int<S>(value: &bool, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_u8(*value as u8)
}

pub(super) fn length_vec<S>(values: &[LevelLength], serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&vec(values))
}

pub(super) fn rating_vec<S>(values: &[LevelRating], serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&vec(values))
}

pub(super) fn demon_rating<S>(rating: &Option<DemonRating>, serialize: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serialize.serialize_str(&rating.unwrap().robtop_into_req())
}

pub(super) fn req_type<S>(req_type: &LevelRequestType, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_i32(i32::from(*req_type))
}

pub(super) fn sort_mode<S>(sort_mode: &SortMode, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match sort_mode {
        SortMode::Liked => serializer.serialize_i32(1),
        SortMode::Recent => serializer.serialize_i32(0),
    }
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
            map.serialize_entry("completedLevels", &level_list(ids))?;
            if include {
                map.serialize_entry("uncompleted", &0)?;
                map.serialize_entry("onlyCompleted", &1)?;
            } else {
                map.serialize_entry("uncompleted", &1)?;
                map.serialize_entry("onlyCompleted", &0)?;
            }
        },
    }

    map.serialize_entry("featured", &bool(filters.featured))?;
    map.serialize_entry("original", &bool(filters.original))?;
    map.serialize_entry("twoPlayer", &bool(filters.two_player))?;
    map.serialize_entry("coins", &bool(filters.coins))?;
    map.serialize_entry("epic", &bool(filters.epic))?;
    map.serialize_entry("star", &bool(filters.rated))?;

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
