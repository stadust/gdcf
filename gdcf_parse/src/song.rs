use crate::{
    error::ValueError,
    util::{decode_url, into_option},
    Parse,
};
use gdcf_model::song::NewgroundsSong;

parser! {
    NewgroundsSong => {
        song_id(index = 1),
        name(index = 2),
        index_3(index = 3, default),
        artist(index = 4, default),
        filesize(index = 5),
        index_6(index = 6, with = into_option, default),
        index_7(index = 7, with = into_option, default),
        index_8(index = 8),
        link(index = 10, parse = decode_url),
    }
}
