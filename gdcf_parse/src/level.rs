use crate::{
    error::ValueError,
    parse,
    util::{default_to_none, int_to_bool, process_description, process_difficulty, process_song, SelfZip},
    Parse,
};
use gdcf::model::PartialLevel;

parser! {
    PartialLevel<u64, u64> => {
        level_id(index = 1),
        name(index = 2),
        description(index = 3, with2 = process_description, default),
        version(index = 5),
        creator(index = 6),
        difficulty(custom = process_difficulty, depends_on = [rating, is_auto, is_demon]),
        downloads(index = 10),
        main_song(custom = process_song, depends_on = [main_song_id, &custom_song]),
        gd_version(index = 13),
        likes(index = 14),
        length(index = 15),
        stars(index = 18),
        featured(index = 19),
        copy_of(index = 30, with = default_to_none),
        custom_song(index = 35, with = default_to_none),
        coin_amount(index = 37),
        coins_verified(index = 38, with = int_to_bool),
        stars_requested(index = 39, with = default_to_none),
        is_epic(index = 42, with = int_to_bool),
        index_43(index = 43),
        object_amount(index = 45),
        index_46(index = 46),
        index_47(index = 47),
    },
    main_song_id(index = 12, default),
    rating(index = 9),
    is_demon(index = 17, with = int_to_bool),
    is_auto(index = 25, with = int_to_bool),
}
