use crate::{
    convert::{RobtopFrom, RobtopInto},
    error::ValueError,
};
use base64::{DecodeError, URL_SAFE};
use gdcf_model::user::Color;
use std::num::ParseIntError;

#[derive(Debug, Clone)]
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

/// Performs URL-safe base64 decoding on the given [`str`] and tries to
/// build a UTF8 String from the resulting bytes.
///
/// # Errors
/// If the given string isn't valid URL-safe base64, a [`DecodeError`] is
/// returned
pub fn b64_decode_string(encoded: &str) -> Result<String, DecodeError> {
    base64::decode_config(encoded, URL_SAFE).map(|bytes| String::from_utf8_lossy(&bytes[..]).to_string())
}

/// Performs robtop's XOR en-/decryption routine on `encrypted` using `key`
///
/// Note that although both `encrypted` and `key` are `str`s, the decryption
/// is done directly on the bytes, and the result of each byte-wise XOR
/// operation is casted to `char`, meaning this function only works for
/// ASCII strings.
pub fn xor_decrypt(encrypted: &str, key: &str) -> String {
    encrypted
        .bytes()
        .zip(key.bytes().cycle())
        .map(|(enc_byte, key_byte)| (enc_byte ^ key_byte) as char)
        .collect()
}

pub fn parse<'a, T>(idx: &'a str, value: &'a str) -> Result<Option<T>, ValueError<'a>>
where
    T: RobtopFrom<T, &'a str>,
{
    if value == "" {
        return Ok(None)
    }

    T::robtop_from(value)
        .map(Some)
        .map_err(|error| ValueError::Parse(idx, value, error))
}

// FIXME: this is just fucking horrible
pub(crate) fn unparse<T>(value: T) -> String
where
    T: RobtopInto<T, String>,
{
    value.robtop_into()
}

pub(crate) fn can_omit<T>(value: &T) -> bool
where
    T: RobtopInto<T, String>,
{
    value.can_omit()
}
