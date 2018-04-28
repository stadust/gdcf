use gdcf::model::RawObject;
use gdcf::model::ObjectType;

use gdcf::error::ApiError;

use std::str::pattern::Pattern;
use gdcf::api::response::ProcessedResponse;
use hyper::Error;

pub fn level(body: &str) -> Result<ProcessedResponse, ApiError<Error>> {
    check_resp!(body);

    let mut sections = body.split("#");

    match sections.next() {
        Some(section) => parse_fragment(ObjectType::Level, section, ":").map(ProcessedResponse::One),
        None => Err(ApiError::UnexpectedFormat),
    }
}

pub fn levels(body: &str) -> Result<ProcessedResponse, ApiError<Error>> {
    check_resp!(body);

    let mut result = Vec::new();
    let mut sections = body.split("#");

    match sections.next() {
        Some(section) => {
            for fragment in section.split("|") {
                result.push(parse_fragment(ObjectType::PartialLevel, fragment, ":")?);
            }
        }
        None => return Err(ApiError::UnexpectedFormat),
    }

    sections.next(); // ignore the creator section (for now)

    match sections.next() {
        Some(section) => {
            for fragment in section.split("~:~") {
                result.push(parse_fragment(ObjectType::NewgroundsSong, fragment, "~|~")?);
            }
        }
        None => return Err(ApiError::UnexpectedFormat),
    }

    Ok(ProcessedResponse::Many(result))
}

fn parse_fragment<'a, P>(
    obj_type: ObjectType,
    fragment: &'a str,
    seperator: P,
) -> Result<RawObject, ApiError<Error>>
where
    P: Pattern<'a>,
{
    let mut iter = fragment.split(seperator);
    let mut raw_obj = RawObject::new(obj_type);

    while let Some(idx) = iter.next() {
        let idx = match idx.parse() {
            Err(_) => return Err(ApiError::UnexpectedFormat),
            Ok(idx) => idx,
        };

        let value = match iter.next() {
            Some(value) => value,
            None => return Err(ApiError::UnexpectedFormat),
        };

        raw_obj.set(idx, value.into());
    }

    Ok(raw_obj)
}
