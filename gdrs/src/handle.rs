use crate::{error::ApiError, Req};
use gdcf::{
    api::{
        client::Response,
        request::{
            comment::LevelCommentsRequest, user::UserSearchRequest, LevelRequest, LevelsRequest, Request as GdcfRequest, UserRequest,
        },
    },
    Secondary,
};
use gdcf_model::{
    level::{Level, PartialLevel},
    song::NewgroundsSong,
    user::{Creator, SearchedUser, User},
};
use gdcf_parse::Parse;
use log::info;
use gdcf::api::request::comment::ProfileCommentsRequest;

pub trait Handler: GdcfRequest {
    fn endpoint() -> &'static str;
    fn handle(response_body: &str) -> Result<Response<Self::Result>, ApiError>;

    fn to_req(&self) -> Req;
}

impl Handler for LevelRequest {
    fn endpoint() -> &'static str {
        endpoint!("downloadGJLevel22")
    }

    fn handle(response_body: &str) -> Result<Response<Self::Result>, ApiError> {
        check_resp!(response_body);

        let mut sections = response_body.split('#');

        match sections.next() {
            Some(section) => Ok(Response::Exact(Level::parse_iter(section.split(':'))?)),
            None => Err(ApiError::UnexpectedFormat),
        }
    }

    fn to_req(&self) -> Req {
        Req::LevelRequest(self)
    }
}

impl Handler for LevelsRequest {
    fn endpoint() -> &'static str {
        endpoint!("getGJLevels21")
    }

    fn handle(response_body: &str) -> Result<Response<Self::Result>, ApiError> {
        check_resp!(response_body);

        let mut other = Vec::new();
        let mut sections = response_body.split('#');

        let levels: Vec<PartialLevel<u64, u64>> = match sections.next() {
            Some(section) =>
                section
                    .split('|')
                    .map(|fragment| PartialLevel::parse_str(fragment, ':'))
                    .collect::<Result<_, _>>()?,
            None => return Err(ApiError::UnexpectedFormat),
        };

        if let Some(section) = sections.next() {
            // No creators are fine with us
            if !section.is_empty() {
                for fragment in section.split('|') {
                    other.push(Creator::parse_unindexed_str(fragment, ':')?.into());
                }
            }
        }

        if let Some(section) = sections.next() {
            // No song fragment is fine with us
            if !section.is_empty() {
                for fragment in section.split("~:~") {
                    other.push(NewgroundsSong::parse_str2(fragment, "~|~")?.into());
                }
            }
        }

        for level in &levels {
            if other
                .iter()
                .filter_map(|sec| {
                    match sec {
                        Secondary::Creator(c) => Some(c.user_id),
                        _ => None,
                    }
                })
                .find(|&c| c == level.creator)
                .is_none()
            {
                other.push(Secondary::MissingCreator(level.creator))
            }

            if let Some(custom_song) = level.custom_song {
                if other
                    .iter()
                    .filter_map(|sec| {
                        match sec {
                            Secondary::NewgroundsSong(ref n) => Some(n.song_id),
                            _ => None,
                        }
                    })
                    .find(|&n| n == custom_song)
                    .is_none()
                {
                    other.push(Secondary::MissingNewgroundsSong(custom_song))
                }
            }
        }

        Ok(Response::More(levels, other))
    }

    fn to_req(&self) -> Req {
        Req::LevelsRequest(self)
    }
}

impl Handler for UserRequest {
    fn endpoint() -> &'static str {
        endpoint!("getGJUserInfo20")
    }

    fn handle(response_body: &str) -> Result<Response<Self::Result>, ApiError> {
        check_resp!(response_body);

        Ok(Response::Exact(User::parse_str(response_body, ':')?))
    }

    fn to_req(&self) -> Req {
        Req::UserRequest(self)
    }
}

impl Handler for UserSearchRequest {
    fn endpoint() -> &'static str {
        endpoint!("getGJUsers20")
    }

    fn handle(response_body: &str) -> Result<Response<Self::Result>, ApiError> {
        let mut sections = response_body.split('#');

        match sections.next() {
            Some(section) => Ok(Response::Exact(SearchedUser::parse_iter(section.split(':'))?)),
            None => Err(ApiError::UnexpectedFormat),
        }
    }

    fn to_req(&self) -> Req {
        Req::UserSearchRequest(self)
    }
}

impl Handler for LevelCommentsRequest {
    fn endpoint() -> &'static str {
        endpoint!("getGJComments21")
    }

    fn handle(response_body: &str) -> Result<Response<Self::Result>, ApiError> {
        info!(
            "We got a total of {} comments!",
            response_body.split('#').nth(0).unwrap().split('|').count()
        );
        Err(ApiError::NoData)
    }

    fn to_req(&self) -> Req {
        Req::LevelCommentsRequest(self)
    }
}

impl Handler for ProfileCommentsRequest {
    fn endpoint() -> &'static str {
        endpoint!("getGJAccountComments20")
    }

    fn handle(response_body: &str) -> Result<Response<Self::Result>, ApiError> {
        unimplemented!()
    }

    fn to_req(&self) -> Req {
        Req::ProfileCommentsRequest(self)
    }
}
