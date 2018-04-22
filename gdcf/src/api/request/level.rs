#[cfg(ser)]
use serde::{Serialize, Serializer};

use api::request::BaseRequest;
use api::request::Request;
use model::DemonRating;
use model::Level;
use model::LevelLength;
use model::LevelRating;
use model::PartialLevel;

#[derive(Debug, Default)]
pub struct LevelRequest {
    pub base: BaseRequest,
    pub level_id: u64,
    pub inc: bool,
    pub extra: bool,
}

#[derive(Debug, Default, Clone, Hash)]
pub struct LevelsRequest {
    pub base: BaseRequest,
    pub request_type: LevelRequestType,

    pub search_string: String,

    pub lengths: Vec<LevelLength>,
    pub ratings: Vec<LevelRating>,
    pub demon_rating: Option<DemonRating>,

    pub page: u32,
    pub total: i32,

    pub search_filters: SearchFilters,
}

#[derive(Debug, Default, Copy, Clone, Hash)]
pub struct SearchFilters {
    pub uncompleted: bool,
    pub completed: bool,
    pub featured: bool,
    pub original: bool,
    pub two_player: bool,
    pub coins: bool,
    pub epic: bool,
    pub rated: bool,
    pub song: Option<SongFilter>,
}

#[derive(Debug, Copy, Clone, PartialOrd, PartialEq, Hash)]
pub enum LevelRequestType {
    Search,
    MostDownloaded,
    MostLiked,
    Trending,
    Recent,
    User,
    Featured,
    Magic,
    Unknown8,
    Awarded,
    Followed,
    Friend,
    Unknown12,
    Unknown13,
    Unknown14,
    Unknown15,
    HallOfFame,
}

#[derive(Debug, Copy, Clone, Hash)]
pub enum SongFilter {
    Main(u8),
    Custom(u64),
}

impl SearchFilters {
    pub fn new() -> SearchFilters {
        SearchFilters::default()
    }

    pub fn rated(mut self) -> SearchFilters {
        self.rated = true;
        self
    }

    pub fn uncompleted(mut self) -> SearchFilters {
        self.uncompleted = true;
        self
    }

    pub fn completed(mut self) -> SearchFilters {
        self.completed = true;
        self
    }

    pub fn featured(mut self) -> SearchFilters {
        self.featured = true;
        self
    }

    pub fn original(mut self) -> SearchFilters {
        self.original = true;
        self
    }

    pub fn two_player(mut self) -> SearchFilters {
        self.two_player = true;
        self
    }

    pub fn coins(mut self) -> SearchFilters {
        self.coins = true;
        self
    }

    pub fn epic(mut self) -> SearchFilters {
        self.epic = true;
        self
    }

    pub fn main_song(mut self, id: u8) -> SearchFilters {
        self.song = Some(SongFilter::Main(id));
        self
    }

    pub fn custom_song(mut self, id: u64) -> SearchFilters {
        self.song = Some(SongFilter::Custom(id));
        self
    }
}

impl LevelRequest {
    pub fn new(level_id: u64) -> LevelRequest {
        LevelRequest {
            base: BaseRequest::default(),
            level_id,
            inc: true,
            extra: false,
        }
    }

    setter!(with_base, base, BaseRequest);
    setter!(inc, bool);
    setter!(extra, bool);
}

impl LevelsRequest {
    setter!(with_base, base, BaseRequest);

    setter!(filter, search_filters, SearchFilters);
    setter!(page, u32);
    setter!(total, i32);
    setter!(request_type, LevelRequestType);

    pub fn search(mut self, search_string: String) -> Self {
        self.search_string = search_string;
        self.request_type = LevelRequestType::Search;
        self
    }

    pub fn with_length(mut self, length: LevelLength) -> Self {
        self.lengths.push(length);
        self
    }

    pub fn with_rating(mut self, rating: LevelRating) -> Self {
        self.ratings.push(rating);
        self
    }

    pub fn demon(mut self, demon_rating: DemonRating) -> Self {
        self.demon_rating = Some(demon_rating);
        self
    }
}

impl Default for LevelRequestType {
    fn default() -> LevelRequestType {
        LevelRequestType::Featured
    }
}

impl From<LevelRequestType> for i32 {
    fn from(req_type: LevelRequestType) -> Self {
        match req_type {
            LevelRequestType::Search => 0,
            LevelRequestType::MostDownloaded => 1,
            LevelRequestType::MostLiked => 2,
            LevelRequestType::Trending => 3,
            LevelRequestType::Recent => 4,
            LevelRequestType::User => 5,
            LevelRequestType::Featured => 6,
            LevelRequestType::Magic => 7,
            LevelRequestType::Unknown8 => 8,
            LevelRequestType::Awarded => 9,
            LevelRequestType::Followed => 10,
            LevelRequestType::Friend => 11,
            LevelRequestType::Unknown12 => 12,
            LevelRequestType::Unknown13 => 13,
            LevelRequestType::Unknown14 => 14,
            LevelRequestType::Unknown15 => 15,
            LevelRequestType::HallOfFame => 16,
        }
    }
}

impl From<u64> for LevelRequest {
    fn from(lid: u64) -> Self {
        LevelRequest::new(lid)
    }
}

impl Request for LevelRequest {
    type Result = Level;
}

impl Request for LevelsRequest {
    type Result = Vec<PartialLevel>;
}
