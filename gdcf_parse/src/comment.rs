use crate::{Parse, ValueError};
use gdcf_model::comment::{CommentUser, LevelComment, ProfileComment};

parser! {
    ProfileComment => {
        index_2(index = 2),
        index_4(index = 4),
        index_6(index = 6),
        index_9(index = 9),
    }
}
parser! {
    LevelComment => {
        user(custom = dummy[]),
        index_2(index = 2),
        index_3(index = 3),
        index_4(index = 4),
        index_6(index = 6),
        index_7(index = 7),
        index_9(index = 9),
        index_10(index = 10),
    }
}
parser! {
    CommentUser => {
        index_1(index = 1),
        index_9(index = 9),
        index_10(index = 10),
        index_11(index = 11),
        index_14(index = 14),
        index_15(index = 15),
        index_16(index = 16),
    }
}

fn dummy() {}
