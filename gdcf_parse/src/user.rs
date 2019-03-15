use crate::{
    convert::{TwitchConverter, TwitterConverter, YoutubeConverter},
    error::ValueError,
    Parse,
};
use gdcf_model::user::{Creator, User};

pub fn youtube(value: &str) -> Option<String> {
    if value.is_empty() {
        None
    } else {
        Some(format!("https://www.youtube.com/channel/{}", value))
    }
}

pub fn twitter(value: &str) -> Option<String> {
    if value.is_empty() {
        None
    } else {
        Some(format!("https://www.twitter.com/{}", value))
    }
}

pub fn twitch(value: &str) -> Option<String> {
    if value.is_empty() {
        None
    } else {
        Some(format!("https://www.twitch.tv/{}", value))
    }
}

parser! {
    User => {
        name(index = 1),
        user_id(index = 2),
        stars(index = 3),
        demons(index = 4),
        creator_points(index = 8),
        index_10(index = 10),
        index_11(index = 11),
        secret_coins(index = 13),
        account_id(index = 16),
        user_coins(index = 17),
        index_18(index = 18),
        index_19(index = 19),
        youtube_url(index = 20, parse_infallible = YoutubeConverter, default),
        cube_index(index = 21),
        ship_index(index = 22),
        ball_index(index = 23),
        ufo_index(index = 24),
        wave_index(index = 25),
        robot_index(index = 26),
        has_glow(index = 28),
        index_29(index = 29),
        global_rank(index = 30),
        index_31(index = 31),
        spider_index(index = 43),
        twitter_url(index = 44, parse_infallible = TwitterConverter, default),
        twitch_url(index = 45, parse_infallible = TwitchConverter, default),
        diamonds(index = 46),
        death_effect_index(index = 48),
        index_49(index = 49),
        index_50(index = 50),
    }
}

parser! {
    Creator => {
        user_id(index = 1),
        name(index = 2),
        account_id(index = 3),
    }
}
