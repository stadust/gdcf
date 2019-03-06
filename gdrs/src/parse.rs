use crate::error::ApiError;
use gdcf::GDObject;
use gdcf_model::{
    level::{Level, PartialLevel},
    song::NewgroundsSong,
    user::{Creator, User},
};
use gdcf_parse::Parse;

pub fn level(body: &str) -> Result<Vec<GDObject>, ApiError> {
    check_resp!(body);

    let mut sections = body.split('#');

    match sections.next() {
        Some(section) => Ok(vec![Level::parse_iter(section.split(':'))?.into()]),
        None => Err(ApiError::UnexpectedFormat),
    }
}

pub fn levels(body: &str) -> Result<Vec<GDObject>, ApiError> {
    check_resp!(body);

    let mut result = Vec::new();
    let mut sections = body.split('#');

    match sections.next() {
        Some(section) =>
            for fragment in section.split('|') {
                result.push(PartialLevel::parse_str(fragment, ':')?.into());
            },
        None => return Err(ApiError::UnexpectedFormat),
    }

    if let Some(section) = sections.next() {
        // No creators are fine with us
        if !section.is_empty() {
            for fragment in section.split('|') {
                result.push(Creator::parse_unindexed_str(fragment, ':')?.into());
            }
        }
    }

    if let Some(section) = sections.next() {
        // No song fragment is fine with us
        if !section.is_empty() {
            for fragment in section.split("~:~") {
                result.push(NewgroundsSong::parse_str2(fragment, "~|~")?.into());
            }
        }
    }

    Ok(result)
}

pub fn user(body: &str) -> Result<Vec<GDObject>, ApiError> {
    check_resp!(body);

    Ok(vec![User::parse_str(body, ':')?.into()])
}
