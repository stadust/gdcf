use crate::level::SemiLevel;
use derive_more::Display;
use gdcf::{
    api::request::{LevelRequest, LevelsRequest, UserRequest},
    cache::{CreatorKey, Key, NewgroundsSongKey},
};
use gdcf_model::level::PartialLevel;
use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};

pub(crate) trait DatabaseKey: Key {
    fn database_key(&self) -> i64;
}

impl DatabaseKey for CreatorKey {
    fn database_key(&self) -> i64 {
        self.0 as i64
    }
}

#[derive(Debug, Display)]
pub(crate) struct SemiLevelKey(pub u64);

impl Key for SemiLevelKey {
    type Result = SemiLevel;
}

impl DatabaseKey for SemiLevelKey {
    fn database_key(&self) -> i64 {
        self.0 as i64
    }
}

#[derive(Debug, Display)]
pub struct PartialLevelKey(pub u64);

impl Key for PartialLevelKey {
    type Result = PartialLevel<Option<u64>, u64>;
}

impl DatabaseKey for PartialLevelKey {
    fn database_key(&self) -> i64 {
        self.0 as i64
    }
}

impl DatabaseKey for LevelRequest {
    fn database_key(&self) -> i64 {
        self.level_id as i64
    }
}

impl DatabaseKey for NewgroundsSongKey {
    fn database_key(&self) -> i64 {
        self.0 as i64
    }
}

impl DatabaseKey for UserRequest {
    fn database_key(&self) -> i64 {
        self.user as i64
    }
}

impl DatabaseKey for LevelsRequest {
    fn database_key(&self) -> i64 {
        let mut state = DefaultHasher::new();

        self.search_filters.hash(&mut state);
        self.total.hash(&mut state);
        self.demon_rating.hash(&mut state);
        self.ratings.hash(&mut state);
        self.lengths.hash(&mut state);
        self.search_string.hash(&mut state);
        self.request_type.hash(&mut state);
        self.page.hash(&mut state);

        state.finish() as i64
    }
}
