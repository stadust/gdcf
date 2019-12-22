use crate::{
    key::{DatabaseKey, PartialLevelKey, SemiLevelKey},
    meta::Entry,
    wrap::Wrapped,
    Cache,
};
use diesel::{backend::Backend, deserialize::FromSqlRow, ExpressionMethods, Queryable, RunQueryDsl};
use gdcf::{
    api::request::LevelRequest,
    cache::{CacheEntry, Lookup, Store},
};
use gdcf_model::level::{Level, Password};
use log::{debug, warn};
use std::fmt::Display;

#[derive(Debug, Clone)]
pub(crate) struct SemiLevel {
    level_id: u64,
    level_data: Vec<u8>,
    level_password: Password,
    time_since_upload: String,
    time_since_update: String,
    index_36: String,
}

diesel_stuff! {
    level (level_id, SemiLevel) {
        (level_id, level_id, u64),
        (level_data, level_data, Vec<u8>),
        (level_password, level_password, Password),
        (time_since_upload, time_since_upload, String),
        (time_since_update, time_since_update, String),
        (index_36, index_36, String)
    }
}

impl<'a> diesel::Insertable<level::table> for &'a Level<Option<u64>, u64> {
    type Values = <Values<'a> as diesel::Insertable<level::table>>::Values;

    fn values(self) -> Self::Values {
        use level::columns::*;

        (
            level_id.eq(self.base.level_id as i64),
            level_data.eq(&self.level_data[..]),
            level_password.eq(match self.password {
                Password::NoCopy => None,
                Password::FreeCopy => Some("1"),
                Password::PasswordCopy(ref password) => Some(password.as_ref()),
            }),
            time_since_upload.eq(&self.time_since_upload[..]),
            time_since_update.eq(&self.time_since_update[..]),
            index_36.eq(&self.index_36[..]),
        )
            .values()
    }
}

impl<'a> diesel::query_builder::AsChangeset for Wrapped<&'a Level<Option<u64>, u64>> {
    type Changeset = <Values<'a> as diesel::query_builder::AsChangeset>::Changeset;
    type Target = level::table;

    fn as_changeset(self) -> Self::Changeset {
        let Wrapped(lvl) = self;

        use level::columns::*;

        (
            level_id.eq(lvl.base.level_id as i64),
            level_data.eq(&lvl.level_data[..]),
            level_password.eq(match lvl.password {
                Password::NoCopy => None,
                Password::FreeCopy => Some("1"),
                Password::PasswordCopy(ref password) => Some(password.as_ref()),
            }),
            time_since_upload.eq(&lvl.time_since_upload[..]),
            time_since_update.eq(&lvl.time_since_update[..]),
            index_36.eq(&lvl.index_36[..]),
        )
            .as_changeset()
    }
}

impl Display for SemiLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "rest data of level {}", self.level_id)
    }
}

meta_table!(level_meta, level_id);

lookup_simply!(SemiLevelKey, level, level_meta, level_id);

impl Lookup<LevelRequest> for Cache {
    fn lookup(&self, key: &LevelRequest) -> Result<CacheEntry<Level<Option<u64>, u64>, Entry>, Self::Err> {
        match self.lookup(&SemiLevelKey(key.level_id))? {
            CacheEntry::Cached(semi_level, meta) => {
                let semi_level: SemiLevel = semi_level;

                match self.lookup(&PartialLevelKey(semi_level.level_id))? {
                    CacheEntry::Cached(partial, _) =>
                        Ok(CacheEntry::Cached(
                            Level {
                                base: partial,
                                level_data: semi_level.level_data,
                                password: semi_level.level_password,
                                time_since_upload: semi_level.time_since_upload,
                                time_since_update: semi_level.time_since_update,
                                index_36: semi_level.index_36,
                            },
                            meta,
                        )),
                    CacheEntry::MarkedAbsent(meta) => Ok(CacheEntry::MarkedAbsent(meta)),
                    CacheEntry::Missing => Ok(CacheEntry::Missing),
                }
            },
            CacheEntry::MarkedAbsent(meta) => Ok(CacheEntry::MarkedAbsent(meta)),
            CacheEntry::Missing => Ok(CacheEntry::Missing),
        }
    }
}

impl Store<LevelRequest> for Cache {
    fn store(&mut self, obj: &Level<Option<u64>, u64>, key: &LevelRequest) -> Result<Self::CacheEntryMeta, Self::Err> {
        self.store(&obj.base, &PartialLevelKey(obj.base.level_id))?;

        debug!("Storing {} under key {}", obj, key);

        let entry = Entry::new(key.database_key());

        update_entry!(self, entry, level_meta::table, level_meta::level_id);
        upsert!(self, obj, level::table, level::level_id);

        Ok(entry)
    }

    fn mark_absent(&mut self, key: &LevelRequest) -> Result<Entry, Self::Err> {
        warn!("Marking Level with key {} as absent!", key.database_key());

        let entry = Entry::absent(key.database_key());
        update_entry!(self, entry, level_meta::table, level_meta::level_id);
        Ok(entry)
    }
}
