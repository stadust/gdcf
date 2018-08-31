//! Module containing various utility functions for converting Robtop
//! datatypes/formats
//!
//! This module also contains all the [`From`] and [`Into`] impls for the
//! models. This module mainly exists to have them all in a centralized
//! location, so that they can easily be looked at for reference.

pub mod int;
pub mod str;

/// Module containing various functions converting from a robtop format to
/// something saner.
///
/// These are conversations that couldn't be implemented as [`Into`] or
/// [`From`] impls, either because its impossible (e.g. decoding a string), or
/// because of Rust's orphan rules.
pub mod to {
    use base64::{self, DecodeError, URL_SAFE};
    use model::level::Password;
    use percent_encoding::percent_decode;
    use std::str::Utf8Error;

    /// Takes a percent-encoded URL and decodes it
    ///
    /// # Errors
    /// If the decoded data cannot be put together as UTF8, an [`Utf8Error`] is
    /// returned
    pub fn decoded_url(encoded: &str) -> Result<String, Utf8Error> {
        let utf8_cow = percent_decode(encoded.as_bytes()).decode_utf8()?;

        Ok(utf8_cow.to_string())
    }

    /// Performs URL-safe base64 decoding on the given [`str`] and returns the
    /// decoded bytes
    ///
    /// # Errors
    /// If the given string isn't valid URL-safe base64, an [`DecodeError`] is
    /// returned
    pub fn b64_decoded_bytes(encoded: &str) -> Result<Vec<u8>, DecodeError> {
        base64::decode_config(encoded, URL_SAFE)
    }

    /// Performs URL-safe base64 decoding on the given [`str`] and tries to
    /// build a UTF8 String from the resulting bytes.
    ///
    /// # Errors
    /// If the given string isn't valid URL-safe base64, a [`DecodeError`] is
    /// returned
    ///
    /// # Panics
    /// Panics if the decoded data isn't valid UTF8. Use [`b64_decoded_bytes`]
    /// if you aren't sure that the output is valid UTF8.
    pub fn b64_decoded_string(encoded: &str) -> Result<String, DecodeError> {
        b64_decoded_bytes(encoded).map(|bytes| String::from_utf8_lossy(&bytes[..]).to_string())
    }

    /// Converts the given `u8` into a `bool` by returning `true` if `value !=
    /// 0`, and `false` otherwise
    ///
    /// This can be seen as the inverse to [`bool`](::convert::from::bool)
    pub fn bool(value: u8) -> bool {
        value != 0
    }

    /// Attempts to parse the given `str` into a [`Password`]
    ///
    /// # Errors
    /// If the given string isn't `"0"` and also isn't valid URL-safe base64, a
    /// [`DecodeError`] is returned
    pub fn level_password(encrypted: &str) -> Result<Password, DecodeError> {
        match encrypted.as_ref() {
            "0" => Ok(Password::NoCopy),
            pass => {
                let decoded = b64_decoded_string(pass)?;
                let mut decrypted = xor_decrypted(&decoded, "26364");

                if decrypted.len() == 1 {
                    Ok(Password::FreeCopy)
                } else {
                    decrypted.remove(0);
                    Ok(Password::PasswordCopy(decrypted))
                }
            },
        }
    }

    /// Performs robtop's XOR en-/decryption routine on `encrypted` using `key`
    ///
    /// Note that although both `encrypted` and `key` are `str`s, the decryption
    /// is done directly on the bytes, and the result of each byte-wise XOR
    /// operation is casted to `char`, meaning this function only works for
    /// ASCII strings.
    pub fn xor_decrypted(encrypted: &str, key: &str) -> String {
        encrypted
            .bytes()
            .zip(key.bytes().cycle())
            .map(|(enc_byte, key_byte)| (enc_byte ^ key_byte) as char)
            .collect()
    }
}

/// Module containing various functions converting to a robtop format from
/// something saner.
///
/// These conversions may be useful when making requests to the boomlings API.
///
/// These are conversations that couldn't be implemented as `Into` or `From`
/// impls, either because its impossible (e.g. decoding a string), or because
/// of Rust's orphan rules.
pub mod from {
    use joinery::Joinable;

    /// Converts the given [`Vec`] of values convertible into signed integers
    /// into a robtop-approved string.
    pub fn vec<T: Into<i32> + Copy>(list: &Vec<T>) -> String {
        if list.is_empty() {
            String::from("-")
        } else {
            list.into_iter().map(|v| T::into(*v)).join_with(",").to_string()
        }
    }

    /// Converts the given `bool` into an `u8`, returning `0` if `value ==
    /// false` and `1`otherwise.
    ///
    /// This can be seen as the inverse to [`bool`](::convert::to::bool)
    pub fn bool(value: bool) -> u8 {
        value as u8
    }

    pub fn level_list(ids: &Vec<u64>) -> String {
        let mut ids = ids.iter().join_with(",").to_string();
        ids.push(')');
        ids.insert(0, '(');
        ids
    }
}
