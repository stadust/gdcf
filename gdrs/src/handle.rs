use crate::{error::ApiError, Req};
use gdcf::{
    api::{
        client::Response,
        request::{
            comment::{LevelCommentsRequest, ProfileCommentsRequest},
            user::UserSearchRequest,
            LevelRequest, LevelsRequest, Request as GdcfRequest, UserRequest,
        },
    },
    Secondary,
};
use gdcf_model::{
    comment::{CommentUser, LevelComment, ProfileComment},
    level::{Level, PartialLevel},
    song::NewgroundsSong,
    user::{Creator, SearchedUser, User},
};
use gdcf_parse::Parse;
use log::{info, trace, warn};

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

        let levels: Vec<PartialLevel<Option<u64>, u64>> = match sections.next() {
            Some(section) =>
                section
                    .split('|')
                    .map(|fragment| PartialLevel::parse_str(fragment, ':'))
                    .collect::<Result<_, _>>()?,
            None => return Err(ApiError::UnexpectedFormat),
        };

        info!("Found {} levels", levels.len());

        if let Some(section) = sections.next() {
            // No creators are fine with us
            if !section.is_empty() {
                for fragment in section.split('|') {
                    other.push(Creator::parse_unindexed_str(fragment, ':')?.into());
                }
            }
        }

        let creator_count = other.len();

        info!("Found {} creators", creator_count);

        if let Some(section) = sections.next() {
            // No song fragment is fine with us
            if !section.is_empty() {
                for fragment in section.split("~:~") {
                    other.push(NewgroundsSong::parse_str2(fragment, "~|~")?.into());
                }
            }
        }

        info!("Found {} songs", other.len() - creator_count);

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
                warn!("Creator of level {} missing in response data!", level);

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
                    warn!("Song of level {} missing in response data!", level);

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
        check_resp!(response_body);

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
        check_resp!(response_body);

        let mut sections = response_body.split('#');

        match sections.next() {
            Some(section) => {
                let mut comments = Vec::new();

                for object in section.split('|') {
                    let mut parts = object.split(':');

                    if let (Some(raw_comment), Some(raw_user)) = (parts.next(), parts.next()) {
                        trace!("Processing comment {} by user {}", raw_comment, raw_user);

                        let comment = LevelComment::parse_str(raw_comment, '~')?;

                        // This is the dummy placeholder object used by robtop when the player has been deleted
                        let user = if raw_user == "1~~9~~10~~11~~14~~15~~16~" {
                            None
                        } else {
                            Some(CommentUser::parse_str(raw_user, '~')?)
                        };

                        comments.push(LevelComment {
                            user,
                            content: comment.content,
                            user_id: comment.user_id,
                            likes: comment.likes,
                            comment_id: comment.comment_id,
                            is_flagged_spam: comment.is_flagged_spam,
                            time_since_post: comment.time_since_post,
                            progress: comment.progress,
                            is_elder_mod: comment.is_elder_mod,
                            special_color: comment.special_color,
                        })
                    } else {
                        return Err(ApiError::UnexpectedFormat)
                    }
                }

                info!("We got a total of {} comments!", comments.len());

                Ok(Response::Exact(comments))
            },
            None => Err(ApiError::UnexpectedFormat),
        }
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
        check_resp!(response_body);

        let mut sections = response_body.split('#');

        match sections.next() {
            Some(section) => {
                let mut comments = Vec::new();

                for object in section.split('|') {
                    comments.push(ProfileComment::parse_str(object, '~')?)
                }

                info!("We got a total of {} comments!", comments.len());

                Ok(Response::Exact(comments))
            },
            None => Err(ApiError::UnexpectedFormat),
        }
    }

    fn to_req(&self) -> Req {
        Req::ProfileCommentsRequest(self)
    }
}
