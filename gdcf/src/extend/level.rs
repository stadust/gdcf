use super::Extendable;
use crate::{
    api::request::{LevelRequest, LevelsRequest},
    cache::{Cache, Lookup},
};
use gdcf_model::{
    level::{Level, PartialLevel},
    song::NewgroundsSong,
    user::Creator,
};

// FIXME: this impl isn't usable from Gdcf yet yet
impl<C: Cache, Song: PartialEq, User: PartialEq> Extendable<C, Level<Song, User>, Level<Option<u64>, u64>> for PartialLevel<Song, User> {
    type Request = LevelRequest;

    fn lookup_extension(&self, cache: &C, request_result: Level<Option<u64>, u64>) -> Result<Level<Option<u64>, u64>, <C as Cache>::Err> {
        Ok(request_result)
    }

    fn on_extension_absent() -> Option<Level<Option<u64>, u64>> {
        None
    }

    fn extension_request(&self) -> Self::Request {
        self.level_id.into()
    }

    fn combine(self, addon: Level<Option<u64>, u64>) -> Level<Song, User> {
        let Level {
            level_data,
            password,
            time_since_update,
            time_since_upload,
            index_36,
            ..
        } = addon;

        Level {
            base: self,
            level_data,
            password,
            time_since_update,
            time_since_upload,
            index_36,
        }
    }
}

impl<C: Cache> Extendable<C, Level<Option<NewgroundsSong>, u64>, Option<NewgroundsSong>> for Level<Option<u64>, u64>
where
    C: Lookup<NewgroundsSong>,
{
    type Request = LevelsRequest;

    fn lookup_extension(
        &self,
        cache: &C,
        request_result: Vec<PartialLevel<Option<u64>, u64>>,
    ) -> Result<Option<NewgroundsSong>, <C as Cache>::Err> {
        Ok(match self.base.custom_song {
            Some(song_id) => cache.lookup(song_id)?.into(),
            None => None,
        })
    }

    fn on_extension_absent() -> Option<Option<NewgroundsSong>> {
        Some(None)
    }

    fn extension_request(&self) -> Self::Request {
        LevelsRequest::default().with_id(self.base.level_id)
    }

    fn combine(self, addon: Option<NewgroundsSong>) -> Level<Option<NewgroundsSong>, u64> {
        let Level {
            base,
            level_data,
            password,
            time_since_update,
            time_since_upload,
            index_36,
        } = self;

        Level {
            base: Extendable::<C, _, _>::combine(base, addon),
            level_data,
            password,
            time_since_update,
            time_since_upload,
            index_36,
        }
    }
}

impl<C: Cache> Extendable<C, PartialLevel<Option<NewgroundsSong>, u64>, Option<NewgroundsSong>> for PartialLevel<Option<u64>, u64>
where
    C: Lookup<NewgroundsSong>,
{
    type Request = LevelsRequest;

    fn lookup_extension(&self, cache: &C, _: Vec<PartialLevel<Option<u64>, u64>>) -> Result<Option<NewgroundsSong>, <C as Cache>::Err> {
        Ok(match self.custom_song {
            Some(song_id) => cache.lookup(song_id)?.into(),
            None => None,
        })
    }

    fn on_extension_absent() -> Option<Option<NewgroundsSong>> {
        Some(None)
    }

    fn extension_request(&self) -> Self::Request {
        LevelsRequest::default().with_id(self.level_id)
    }

    fn combine(self, custom_song: Option<NewgroundsSong>) -> PartialLevel<Option<NewgroundsSong>, u64> {
        let PartialLevel {
            level_id,
            name,
            description,
            version,
            difficulty,
            downloads,
            main_song,
            gd_version,
            likes,
            length,
            stars,
            featured,
            index_31,
            copy_of,
            coin_amount,
            coins_verified,
            stars_requested,
            index_40,
            is_epic,
            index_43,
            object_amount,
            index_46,
            index_47,
            creator,
            ..
        } = self;

        PartialLevel {
            custom_song,

            level_id,
            name,
            description,
            version,
            creator,
            difficulty,
            downloads,
            main_song,
            gd_version,
            likes,
            length,
            stars,
            featured,
            index_31,
            copy_of,
            coin_amount,
            coins_verified,
            stars_requested,
            index_40,
            is_epic,
            index_43,
            object_amount,
            index_46,
            index_47,
        }
    }
}

impl<C: Cache, Song: PartialEq> Extendable<C, Level<Song, Option<Creator>>, Option<Creator>> for Level<Song, u64>
where
    C: Lookup<Creator>,
{
    type Request = LevelsRequest;

    fn lookup_extension(
        &self,
        cache: &C,
        request_result: Vec<PartialLevel<Option<u64>, u64>>,
    ) -> Result<Option<Creator>, <C as Cache>::Err> {
        Ok(cache.lookup(self.base.creator)?.into())
    }

    fn on_extension_absent() -> Option<Option<Creator>> {
        Some(None)
    }

    fn extension_request(&self) -> Self::Request {
        LevelsRequest::default().with_id(self.base.level_id)
    }

    fn combine(self, addon: Option<Creator>) -> Level<Song, Option<Creator>> {
        let Level {
            base,
            level_data,
            password,
            time_since_update,
            time_since_upload,
            index_36,
        } = self;

        Level {
            base: Extendable::<C, _, _>::combine(base, addon),
            level_data,
            password,
            time_since_update,
            time_since_upload,
            index_36,
        }
    }
}

impl<C: Cache, Song: PartialEq> Extendable<C, PartialLevel<Song, Option<Creator>>, Option<Creator>> for PartialLevel<Song, u64>
where
    C: Lookup<Creator>,
{
    type Request = LevelsRequest;

    fn lookup_extension(&self, cache: &C, _: Vec<PartialLevel<Option<u64>, u64>>) -> Result<Option<Creator>, <C as Cache>::Err> {
        Ok(cache.lookup(self.creator)?.into())
    }

    fn on_extension_absent() -> Option<Option<Creator>> {
        Some(None)
    }

    fn extension_request(&self) -> Self::Request {
        LevelsRequest::default().with_id(self.level_id)
    }

    fn combine(self, creator: Option<Creator>) -> PartialLevel<Song, Option<Creator>> {
        let PartialLevel {
            level_id,
            name,
            description,
            version,
            difficulty,
            downloads,
            main_song,
            gd_version,
            likes,
            length,
            stars,
            featured,
            index_31,
            copy_of,
            coin_amount,
            coins_verified,
            stars_requested,
            index_40,
            is_epic,
            index_43,
            object_amount,
            index_46,
            index_47,
            custom_song,
            ..
        } = self;

        PartialLevel {
            custom_song,

            level_id,
            name,
            description,
            version,
            creator,
            difficulty,
            downloads,
            main_song,
            gd_version,
            likes,
            length,
            stars,
            featured,
            index_31,
            copy_of,
            coin_amount,
            coins_verified,
            stars_requested,
            index_40,
            is_epic,
            index_43,
            object_amount,
            index_46,
            index_47,
        }
    }
}
