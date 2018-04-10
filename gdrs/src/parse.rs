use gdcf::model::{GDObject, RawObject, Level, FromRawObject};
use gdcf::api::GDError;

use std::str::pattern::Pattern;

pub fn level(body: &str) -> Result<GDObject, GDError> {
    check_resp!(body);

    let mut sections = body.split("#");
    let raw_level = match sections.next() {
        Some(section) => parse_section(section, ":")?,
        None => return Err(GDError::MalformedResponse)
    };

    Ok(Level::from_raw(&raw_level)?.into())
}

fn parse_section<'a, P>(section: &'a str, seperator: P) -> Result<RawObject, GDError>
    where
        P: Pattern<'a>
{
    let mut iter = section.split(seperator);
    let mut raw_obj = RawObject::new();

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
/*
RawObject { values: [NotProvided, Value("39976494"), Value("Violently X"), Value("SVQnUyBGSU5BTExZIE9WRVIuIFtGSVJTVCAyLjEgTFZMXSAiSE9UICYgSEFSRENPUkUgQ09MTEFCIiBXaXRoIENhc3RyaVghIFZpZGVvIGlzIG9uIG15IFlUIERvcmFtaSEgSEFIQUhBLi4uLg=="), Value("<snip>"), Value("2"), Value("3023874"), NotProvided, Value("10"), Value("40"), Value("90750"), NotProvided, Value("0"), Value("21"), Value("8070"), Value("3"), NotProvided, Value("1"), Value("10"), Value("24541"), NotProvided, NotProvided, NotProvided, NotProvided, NotProvided, Value(""), NotProvided, Value("AwcCAAQCBg=="), Value("3 months"), Value("3 months"), Value("36026766"), Value("0"), NotProvided, NotProvided, NotProvided, Value("707798"), Value("222_278_716_90_0_0_366_751_193_202_873_1064_231_0_0_242_359_689_361_220_465_0_137_0_310_256_0_0_0_39_0_0_0_144_0_0_66_123_96_92_0_0_0_0_0_41_0_0_0_0_0_0_0_0_0"), Value("1"), Value("1"), Value("10"), Value("0"), NotProvided, Value("1"), Value("5"), NotProvided, Value("40252"), Value("2225"), Value("104998")] }

*/