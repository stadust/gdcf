use error::ValueError;
use model::{LevelRating, RawObject};
use model::level::Featured;
use model::song::{MAIN_SONGS, MainSong, UNKNOWN};
use percent_encoding::percent_decode;
use std::num::ParseIntError;
use std::str::FromStr;

pub(super) fn level_rating(raw_obj: &RawObject) -> Result<LevelRating, ValueError> {
    let is_demon = raw_obj.get_with_or(17, int_to_bool, false)?;
    let is_auto = raw_obj.get_with_or(25, int_to_bool, false)?;
    let rating: i32 = raw_obj.get(9)?;

    if is_demon {
        Ok(LevelRating::Demon(rating.into()))
    } else if is_auto {
        Ok(LevelRating::Auto)
    } else {
        Ok(rating.into())
    }
}

pub(super) fn main_song(raw_obj: &RawObject) -> Result<Option<&'static MainSong>, ValueError> {
    if raw_obj.get::<u64>(35)? == 0 {
        Ok(Some(
            MAIN_SONGS
                .get(raw_obj.get::<usize>(12)?)
                .unwrap_or(&UNKNOWN),
        ))
    } else {
        Ok(None)
    }
}

#[cfg_attr(feature = "cargo-clippy", allow(ptr_arg))]
pub(super) fn default_to_none<T>(value: &String) -> Result<Option<T>, <T as FromStr>::Err>
    where
        T: FromStr + Default + PartialEq,
{
    let value: T = value.parse()?;

    if value == Default::default() {
        Ok(None)
    } else {
        Ok(Some(value))
    }
}

#[cfg_attr(feature = "cargo-clippy", allow(ptr_arg))]
pub(super) fn int_to_bool(value: &String) -> Result<bool, ParseIntError> {
    Ok(value.parse::<u8>()? != 0)
}

#[cfg_attr(feature = "cargo-clippy", allow(ptr_arg))]
pub(super) fn into_option(value: &String) -> Result<Option<String>, !> {
    Ok(Some(value.clone()))
}

#[cfg_attr(feature = "cargo-clippy", allow(ptr_arg))]
pub(super) fn featured(value: &String) -> Result<Featured, ParseIntError> {
    let value: i32 = value.parse()?;

    Ok(match value {
        -1 => Featured::Unfeatured,
        0 => Featured::NotFeatured,
        _ => Featured::Featured(value as u32),
    })
}

#[cfg_attr(feature = "cargo-clippy", allow(ptr_arg))]
pub(super) fn url(value: &String) -> Result<String, !> {
    Ok(percent_decode(value.as_bytes()).decode_utf8().unwrap().to_string())
}