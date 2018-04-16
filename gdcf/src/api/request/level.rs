use api::request::BaseRequest;
use model::LevelRating;
use model::LevelLength;
use serde::{Serializer, Serialize};
use model::DemonRating;

#[derive(Debug, Default)]
pub struct LevelRequest {
    pub base: BaseRequest,
    pub level_id: u64,
    pub inc: bool,
    pub extra: bool,
}

#[derive(Debug, Default)]
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

#[derive(Debug, Default)]
pub struct SearchFilters {
    pub uncompleted: bool,
    pub completed: bool,
    pub featured: bool,
    pub original: bool,
    pub two_player: bool,
    pub coins: bool,
    pub epic: bool,
    pub song: Option<SongFilter>,
}

#[derive(Debug, Copy, Clone)]
pub enum LevelRequestType {
    Featured
}

#[derive(Debug)]
pub enum SongFilter {
    Main(u8),
    Custom(u64),
}

impl SearchFilters {
    pub fn new() -> SearchFilters {
        SearchFilters::default()
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

    pub fn with_base(mut self, base: BaseRequest) -> LevelRequest {
        self.base = base;
        self
    }

    pub fn with_inc(mut self, inc: bool) -> LevelRequest {
        self.inc = inc;
        self
    }

    pub fn with_extra(mut self, extra: bool) -> LevelRequest {
        self.extra = extra;
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
            LevelRequestType::Featured => 6
        }
    }
}