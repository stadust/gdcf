use gdcf::model::{RawObject, Level, FromRawObject};
use gdcf::api::GDError;

use std::str::pattern::Pattern;
use gdcf::model::ObjectType;

pub fn level(body: &str) -> Result<RawObject, GDError> {
    check_resp!(body);

    let mut sections = body.split("#");

    match sections.next() {
        Some(section) => parse_section(section, ":"),
        None => Err(GDError::MalformedResponse)
    }
}

fn parse_section<'a, P>(section: &'a str, seperator: P) -> Result<RawObject, GDError>
    where
        P: Pattern<'a>
{
    let mut iter = section.split(seperator);
    let mut raw_obj = RawObject::new(ObjectType::Level);

    while let Some(idx) = iter.next() {
        let idx = match idx.parse() {
            Err(_) => return Err(GDError::MalformedResponse),
            Ok(idx) => idx
        };

        let value = match iter.next() {
            Some(value) => value,
            None => return Err(GDError::MalformedResponse)
        };

        raw_obj.set(idx, value.into());
    }

    Ok(raw_obj)
}