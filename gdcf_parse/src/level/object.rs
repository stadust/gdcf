use crate::{
    error::ValueError,
    util::{int_to_bool, parse},
    Parse,
};
use gdcf_model::level::data::{
    ids,
    portal::{PortalData, PortalType},
    ObjectData,
};

impl Parse for ObjectData {
    fn parse<'a, I, F>(iter: I, mut f: F) -> Result<Self, ValueError<'a>>
    where
        I: Iterator<Item = (&'a str, &'a str)> + Clone,
        F: FnMut(&'a str, &'a str) -> Result<(), ValueError<'a>>,
    {
        let id = match iter.clone().find(|(idx, _)| idx == &"1") {
            Some((idx, id)) => parse(idx, id)?.ok_or(ValueError::NoValue("1"))?,
            None => return Err(ValueError::NoValue("1")),
        };

        match id {
            ids::SLOW_PORTAL | ids::NORMAL_PORTAL | ids::MEDIUM_PORTAL | ids::FAST_PORTAL | ids::VERY_FAST_PORTAL =>
                Ok(ObjectData::Portal(PortalData::parse(iter, f)?)),
            // .. all the other types of metadata, which might have proper parsers ...
            _ => {
                // We aren't delegating further, so we gotta drive the iterator to completion
                for (idx, value) in iter {
                    f(idx, value)?
                }

                Ok(ObjectData::None)
            },
        }
    }
}

parser! {
    PortalData => {
        checked(index = 13, with = int_to_bool),
        portal_type(custom = PortalType::from_id, depends_on = [id]),
    },
    id(^index = 1),
}
