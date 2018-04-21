use gdcf::model::RawObject;
use gdcf::api::GDError;

use std::str::pattern::Pattern;
use gdcf::model::ObjectType;

pub fn level(body: &str) -> Result<RawObject, GDError> {
    check_resp!(body);

    let mut sections = body.split("#");

    match sections.next() {
        Some(section) => parse_fragment(ObjectType::Level, section, ":"),
        None => Err(GDError::MalformedResponse)
    }
}

pub fn levels(body: &str) -> Result<Vec<RawObject>, GDError> {
    check_resp!(body);

    let mut result = Vec::new();
    let mut sections = body.split("#");

    match sections.next() {
        Some(section) => {
            for fragment in section.split("|") {
                result.push(parse_fragment(ObjectType::PartialLevel, fragment, ":")?);
            }
        }
        None => return Err(GDError::MalformedResponse)
    }

    sections.next(); // ignore the creator section

    match sections.next() {
        Some(section) => {
            for fragment in section.split("~:~") {
                result.push(parse_fragment(ObjectType::NewgroundsSong, fragment, "~|~")?);
            }
        }
        None => return Err(GDError::MalformedResponse)
    }

    Ok(result)
}

fn parse_fragment<'a, P>(obj_type: ObjectType, fragment: &'a str, seperator: P) -> Result<RawObject, GDError>
    where
        P: Pattern<'a>
{
    println!("{}", fragment);

    let mut iter = fragment.split(seperator);
    let mut raw_obj = RawObject::new(obj_type);

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