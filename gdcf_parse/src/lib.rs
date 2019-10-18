//! Crate containing parsers for various Geometry Dash related data
//!
//! This crate is based on work by mgostIH and cos8o
#![feature(trace_macros)]

//trace_macros!(true);

use crate::{error::ValueError, util::SelfZipExt};
use std::collections::{HashMap};

#[macro_use]
extern crate log;

pub mod util;
#[macro_use]
pub mod macros;
pub mod comment;
pub mod convert;
pub mod error;
pub mod level;
pub mod song;
pub mod user;

const INDICES: [&str; 50] = [
    "1", "2", "3", "4", "5", "6", "7", "8", "9", "10", "11", "12", "13", "14", "15", "16", "17", "18", "19", "20", "21", "22", "23", "24",
    "25", "26", "27", "28", "29", "30", "31", "32", "33", "34", "35", "36", "37", "38", "39", "40", "41", "42", "43", "44", "45", "46",
    "47", "48", "49", "50",
];

pub trait Parse<'a>: Sized {
    fn parse<I, F>(iter: I, f: F) -> Result<Self, ValueError<'a>>
    where
        I: Iterator<Item = (&'a str, &'a str)> + Clone,
        F: FnMut(&'a str, &'a str) -> Result<(), ValueError<'a>>;
    fn unparse(self) -> HashMap<&'a str, String>;

    fn parse_iter(iter: impl Iterator<Item = &'a str> + Clone) -> Result<Self, ValueError<'a>> {
        Self::parse(iter.self_zip(), |i, v| {
            Ok(warn!(
                "Unused value '{}' at index '{}'. This can and will lead to incorrect .unparse() results",
                v, i
            ))
        })
    }

    fn parse_unindexed_iter(iter: impl Iterator<Item = &'a str> + Clone) -> Result<Self, ValueError<'a>> {
        // well this is a stupid solution
        Self::parse(INDICES.iter().cloned().zip(iter), |i, v| {
            Ok(warn!(
                "Unused value '{}' at index '{}'. This can and will lead to incorrect .unparse() results",
                v, i
            ))
        })
    }

    fn parse_str(input: &'a str, delimiter: char) -> Result<Self, ValueError> {
        Self::parse_iter(input.split(delimiter))
    }

    fn parse_str2(input: &'a str, delimiter: &'a str) -> Result<Self, ValueError<'a>> {
        Self::parse_iter(input.split(delimiter))
    }

    fn parse_unindexed_str(input: &'a str, delimiter: char) -> Result<Self, ValueError> {
        Self::parse_unindexed_iter(input.split(delimiter))
    }

    fn parse_unindexed_str2(input: &'a str, delimiter: &'a str) -> Result<Self, ValueError<'a>> {
        Self::parse_unindexed_iter(input.split(delimiter))
    }
}

pub struct UnparseSafe<'a, P: Parse<'a>> {
    pub unparsed: HashMap<&'a str, &'a str>,
    pub parsed: P,
}

impl<'a, P: Parse<'a>> Parse<'a> for UnparseSafe<'a, P> {
    fn parse<I, F>(iter: I, _: F) -> Result<Self, ValueError<'a>>
    where
        I: Iterator<Item = (&'a str, &'a str)> + Clone,
        F: FnMut(&'a str, &'a str) -> Result<(), ValueError<'a>>,
    {
        let mut unparsed = HashMap::new();

        let parsed = P::parse(iter, |index, value| {
            unparsed.insert(index, value);
            Ok(())
        })?;

        Ok(UnparseSafe { unparsed, parsed })
    }

    fn unparse(self) -> HashMap<&'a str, String> {
        let mut inner_unparsed = self.parsed.unparse();
        inner_unparsed.extend(self.unparsed.into_iter().map(|(index, value)| (index, value.to_owned())));
        inner_unparsed
    }
}
