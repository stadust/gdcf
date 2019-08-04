use crate::{
    convert::{Base64Converter, RGBColor, TwoBool},
    Parse, ValueError,
};
use gdcf_model::comment::{CommentUser, LevelComment, ProfileComment};

parser! {
    ProfileComment => {
        content(index = 2, parse_infallible = Base64Converter, default),
        likes(index = 4),
        comment_id(index = 6),
        time_since_post(index = 9),
    }
}

parser! {
    LevelComment => {
        user(custom = dummy[]),
        content(index = 2, parse_infallible = Base64Converter, default),
        user_id(index = 3),
        likes(index = 4),
        comment_id(index = 6),
        is_flagged_spam(index = 7),
        time_since_post(index = 9),
        progress(index = 10, default),
        is_elder_mod(index = 11, parse = TwoBool, optional),
        special_color(index = 12, parse = RGBColor, optional_non_default),
    }
}

parser! {
    CommentUser => {
        name(index = 1),
        icon_index(index = 9),
        primary_color(index = 10),
        secondary_color(index = 11),
        icon_type(index = 14),
        has_glow(index = 15, parse = TwoBool),
        account_id(index = 16),
    }
}

fn dummy() {}
