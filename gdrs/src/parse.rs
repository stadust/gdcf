use gdcf::{
    api::response::ProcessedResponse,
    error::{ApiError, ValueError},
    model::{raw::RawObject, Creator, GDObject, Level, NewgroundsSong, PartialLevel},
};
use hyper::Error;
use std::{convert::TryFrom, str::pattern::Pattern};
use gdcf::model::User;

pub fn level(body: &str) -> Result<ProcessedResponse, ApiError<Error>> {
    check_resp!(body);

    let mut sections = body.split("#");

    match sections.next() {
        Some(section) => parse_fragment::<Level<u64, u64>, _>(section, ":").map(ProcessedResponse::One),
        None => Err(ApiError::UnexpectedFormat),
    }
}

pub fn levels(body: &str) -> Result<ProcessedResponse, ApiError<Error>> {
    check_resp!(body);

    let mut result = Vec::new();
    let mut sections = body.split("#");

    match sections.next() {
        Some(section) =>
            for fragment in section.split("|") {
                result.push(parse_fragment::<PartialLevel<u64, u64>, _>(fragment, ":")?);
            },
        None => return Err(ApiError::UnexpectedFormat),
    }

    if let Some(section) = sections.next() {
        // No creators are fine with us
        if !section.is_empty() {
            for fragment in section.split('|') {
                result.push(parse_unindexed_fragment::<Creator, _>(fragment, ':')?);
            }
        }
    }

    if let Some(section) = sections.next() {
        // No song fragment is fine with us
        if !section.is_empty() {
            for fragment in section.split("~:~") {
                result.push(parse_fragment::<NewgroundsSong, _>(fragment, "~|~")?);
            }
        }
    }

    Ok(ProcessedResponse::Many(result))
}

pub fn user(body: &str) -> Result<ProcessedResponse, ApiError<Error>> {
    check_resp!(body);

    Ok(ProcessedResponse::One(parse_fragment::<User, _>(body, ':')?))
}

fn parse_unindexed_fragment<'a, A, P>(fragment: &'a str, seperator: P) -> Result<GDObject, ApiError<Error>>
where
    P: Pattern<'a>,
    A: TryFrom<RawObject, Error = ValueError> + Into<GDObject>,
{
    let mut raw_obj = RawObject::new();

    for (value, idx) in fragment.split(seperator).zip(1..) {
        raw_obj.set(idx, value.into())
    }

    let object: A = TryFrom::try_from(raw_obj)?;

    Ok(object.into())
}

fn parse_fragment<'a, A, P>(fragment: &'a str, seperator: P) -> Result<GDObject, ApiError<Error>>
where
    P: Pattern<'a>,
    A: TryFrom<RawObject, Error = ValueError> + Into<GDObject>,
{
    let mut iter = fragment.split(seperator);
    let mut raw_obj = RawObject::new();

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

    let object: A = TryFrom::try_from(raw_obj)?;

    Ok(object.into())
}
