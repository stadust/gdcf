use gdcf::{
    api::response::ProcessedResponse,
    error::ApiError,
    model::{Creator, Level, NewgroundsSong, PartialLevel, User},
};
use gdcf_parse::Parse;
use hyper::Error;

pub fn level(body: &str) -> Result<ProcessedResponse, ApiError<Error>> {
    check_resp!(body);

    let mut sections = body.split('#');

    match sections.next() {
        Some(section) => Ok(ProcessedResponse::One(Level::parse_iter(section.split(':'))?.into())),
        None => Err(ApiError::UnexpectedFormat),
    }
}

pub fn levels(body: &str) -> Result<ProcessedResponse, ApiError<Error>> {
    check_resp!(body);

    let mut result = Vec::new();
    let mut sections = body.split('#');

    match sections.next() {
        Some(section) =>
            for fragment in section.split('|') {
                result.push(PartialLevel::parse_iter(fragment.split(':'))?.into());
            },
        None => return Err(ApiError::UnexpectedFormat),
    }

    if let Some(section) = sections.next() {
        // No creators are fine with us
        if !section.is_empty() {
            for fragment in section.split('|') {
                result.push(Creator::parse_unindexed(fragment.split(':'))?.into());
            }
        }
    }

    if let Some(section) = sections.next() {
        // No song fragment is fine with us
        if !section.is_empty() {
            for fragment in section.split("~:~") {
                result.push(NewgroundsSong::parse_iter(fragment.split("~|~"))?.into());
            }
        }
    }

    Ok(ProcessedResponse::Many(result))
}

pub fn user(body: &str) -> Result<ProcessedResponse, ApiError<Error>> {
    check_resp!(body);

    Ok(ProcessedResponse::One(User::parse_iter(body.split(':'))?.into()))
}
