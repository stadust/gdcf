use model::level::Featured;
use model::LevelLength;
use model::LevelRating;
use std;
use model::DemonRating;

impl From<i32> for Featured {
    fn from(value: i32) -> Self {
        match value {
            -1 => Featured::Unfeatured,
            0 => Featured::NotFeatured,
            _ => Featured::Featured(value as u32),
        }
    }
}

impl Into<i32> for Featured {
    fn into(self) -> i32 {
        match self {
            Featured::Unfeatured => -1,
            Featured::NotFeatured => 0,
            Featured::Featured(value) => value as i32
        }
    }
}

impl From<i32> for LevelLength {
    fn from(length: i32) -> Self {
        match length {
            0 => LevelLength::Tiny,
            1 => LevelLength::Short,
            2 => LevelLength::Medium,
            3 => LevelLength::Long,
            4 => LevelLength::ExtraLong,
            _ => LevelLength::Unknown,
        }
    }
}

impl Into<i32> for LevelLength {
    fn into(self) -> i32 {
        match self {
            LevelLength::Tiny => 0,
            LevelLength::Short => 1,
            LevelLength::Medium => 2,
            LevelLength::Long => 3,
            LevelLength::ExtraLong => 4,
            LevelLength::Unknown => std::i32::MAX,
        }
    }
}

impl From<i32> for LevelRating {
    fn from(value: i32) -> Self {
        match value {
            0 => LevelRating::NotAvailable,
            10 => LevelRating::Easy,
            20 => LevelRating::Normal,
            30 => LevelRating::Hard,
            40 => LevelRating::Harder,
            50 => LevelRating::Insane,
            _ => LevelRating::Unknown,
        }
    }
}

impl Into<i32> for LevelRating {
    fn into(self) -> i32 {
        match self {
            LevelRating::Auto => -3,
            LevelRating::Demon(_) => -2,
            LevelRating::NotAvailable => -1,
            LevelRating::Easy => 1,
            LevelRating::Normal => 2,
            LevelRating::Hard => 3,
            LevelRating::Harder => 4,
            LevelRating::Insane => 5,
            LevelRating::Unknown => std::i32::MAX,
        }
    }
}

impl From<i32> for DemonRating {
    fn from(value: i32) -> DemonRating {
        match value {
            10 => DemonRating::Easy,
            20 => DemonRating::Medium,
            30 => DemonRating::Hard,
            40 => DemonRating::Insane,
            50 => DemonRating::Extreme,
            _ => DemonRating::Unknown,
        }
    }
}

impl Into<i32> for DemonRating {
    fn into(self) -> i32 {
        match self {
            DemonRating::Easy => 1,
            DemonRating::Medium => 2,
            DemonRating::Hard => 3,
            DemonRating::Insane => 4,
            DemonRating::Extreme => 5,
            DemonRating::Unknown => std::i32::MAX,
        }
    }
}