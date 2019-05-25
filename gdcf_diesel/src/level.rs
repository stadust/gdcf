use crate::{meta::Entry, wrap::Wrapped, Cache};
use diesel::{backend::Backend, deserialize::FromSqlRow, ExpressionMethods, Queryable, RunQueryDsl};
use gdcf::cache::{CacheEntry, Lookup, Store};
use gdcf_model::level::{Level, Password};
use log::debug;
use std::fmt::Display;

// Daily reminder that diesel is a piece of shit

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

impl<'a> diesel::Insertable<level::table> for &'a Level<u64, u64> {
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

impl<'a> diesel::query_builder::AsChangeset for Wrapped<&'a Level<u64, u64>> {
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

lookup_simply!(SemiLevel, level, level_meta, level_id);

impl Lookup<Level<u64, u64>> for Cache {
    fn lookup(&self, key: u64) -> Result<CacheEntry<Level<u64, u64>, Self>, Self::Err> {
        let CacheEntry { object: semi, metadata }: CacheEntry<SemiLevel, _> = self.lookup(key)?;
        let partial = self.lookup(semi.level_id)?.into_inner();

        Ok(CacheEntry {
            object: Level {
                base: partial,
                level_data: semi.level_data,
                password: semi.level_password,
                time_since_upload: semi.time_since_upload,
                time_since_update: semi.time_since_update,
                index_36: semi.index_36,
            },
            metadata,
        })
    }
}

impl Store<Level<u64, u64>> for Cache {
    fn store(&mut self, obj: &Level<u64, u64>, key: u64) -> Result<Self::CacheEntryMeta, Self::Err> {
        self.store(&obj.base, obj.base.level_id)?;

        debug!("Storing {}", obj);

        let entry = Entry::new(key);

        update_entry!(self, entry, level_meta::table, level_meta::level_id);
        upsert!(self, obj, level::table, level::level_id);

        Ok(entry)
    }
}
