use std::str::FromStr;

use gdcf::model::{
    song::{MainSong, MAIN_SONGS, UNKNOWN},
    LevelRating,
};

pub struct SelfZip<I> {
    iter: I,
}

impl<I: Iterator> Iterator for SelfZip<I> {
    type Item = (I::Item, I::Item);

    fn next(&mut self) -> Option<Self::Item> {
        match (self.iter.next(), self.iter.next()) {
            (Some(a), Some(b)) => Some((a, b)),
            _ => None,
        }
    }
}

pub trait SelfZipExt: Iterator {
    fn self_zip(self) -> SelfZip<Self>
    where
        Self: Sized,
    {
        SelfZip { iter: self }
    }
}

impl<I> SelfZipExt for I where I: Iterator {}

pub fn process_difficulty(rating: i32, is_auto: bool, is_demon: bool) -> LevelRating {
    if is_demon {
        LevelRating::Demon(rating.into())
    } else if is_auto {
        LevelRating::Auto
    } else {
        rating.into()
    }
}

pub fn process_song(main_song: usize, custom_song: &Option<u64>) -> Option<&'static MainSong> {
    if custom_song.is_none() {
        Some(MAIN_SONGS.get(main_song).unwrap_or(&UNKNOWN))
    } else {
        None
    }
}

pub fn process_description(value: &str) -> Option<String> {
    // I have decided that level descriptions are so broken that we simply ignore it if they fail to
    // parase
    gdcf::convert::to::b64_decoded_string(value).ok()
}

pub fn default_to_none<T>(value: T) -> Option<T>
where
    T: FromStr + Default + PartialEq,
{
    if value == Default::default() {
        None
    } else {
        Some(value)
    }
}

pub fn int_to_bool(value: u8) -> bool {
    gdcf::convert::to::bool(value)
}
