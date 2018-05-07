//! Module containing various utility functions for converting Robtop datatypes/formats
//!
//! This module also contains all the `From` and `Into` impls for the models. This module mainly
//! exists to have them all in a centralized location, so that they can easily be looked at
//! for reference.


pub mod str;
pub mod int;

/// Module containing various functions converting from a robtop format to something saner.
///
/// These are conversations that couldn't be implemented as `Into` or `From` impls, either
/// because its impossible (e.g. decoding a string), or because of Rust's orphan rules.
pub mod to {
    use percent_encoding::percent_decode;
    use std::str::Utf8Error;

    pub fn decoded_url(encoded: &str) -> Result<String, Utf8Error> {
        let utf8_cow = percent_decode(encoded.as_bytes()).decode_utf8()?;

        Ok(utf8_cow.to_string())
    }

    pub fn bool(value: u8) -> bool {
        value != 0
    }
}

/// Module containing various functions converting to a robtop format from something saner.
///
/// These conversions may be useful when making requests to the boomlings API.
///
/// These are conversations that couldn't be implemented as `Into` or `From` impls, either
/// because its impossible (e.g. decoding a string), or because of Rust's orphan rules.
pub mod from {
    use ext::Join;

    pub fn vec<T: Into<i32> + Copy>(list: &Vec<T>) -> String {
        if list.is_empty() {
            String::from("-")
        } else {
            list.into_iter()
                .map(|v| T::into(*v))
                .join(",")
        }
    }

    pub fn bool(value: bool) -> u8 {
        value as u8
    }
}
