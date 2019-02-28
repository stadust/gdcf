use base64::DecodeError;
use convert;
use error::ValueError;
use model::{
    raw::RawObject,
    song::{MainSong, MAIN_SONGS, UNKNOWN},
    LevelRating,
};
use std::{num::ParseIntError, str::FromStr};

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
        Ok(Some(MAIN_SONGS.get(raw_obj.get::<usize>(12)?).unwrap_or(&UNKNOWN)))
    } else {
        Ok(None)
    }
}

pub(super) fn description(value: &str) -> Result<Option<String>, !> {
    // I have decided that level descriptions are so broken that we simply ignore it if they fail to
    // parase
    Ok(convert::to::b64_decoded_string(value).ok())
}

pub(super) fn default_to_none<T>(value: &str) -> Result<Option<T>, <T as FromStr>::Err>
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

pub(super) fn int_to_bool(value: &str) -> Result<bool, ParseIntError> {
    Ok(convert::to::bool(value.parse()?))
}

pub(super) fn into_option(value: &str) -> Result<Option<String>, !> {
    Ok(Some(value.to_string()))
}

pub(super) fn youtube(value: &str) -> Result<Option<String>, !> {
    Ok(Some(format!("https://www.youtube.com/channel/{}", value)))
}

pub(super) fn twitter(value: &str) -> Result<Option<String>, !> {
    Ok(Some(format!("https://www.twitter.com/{}", value)))
}

pub(super) fn twitch(value: &str) -> Result<Option<String>, !> {
    Ok(Some(format!("https://www.twitch.tv/{}", value)))
}
