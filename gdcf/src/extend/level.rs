use super::Extendable;
use crate::{
    api::request::{LevelRequest, LevelsRequest, Request, SearchFilters, UserRequest},
    cache::{Cache, Lookup},
    extend::Upgrade,
};
use gdcf_model::{
    level::{Level, PartialLevel},
    song::NewgroundsSong,
    user::{Creator, SearchedUser, User},
};

// FIXME: this impl isn't usable from Gdcf yet yet
impl<C: Cache, Song: PartialEq, User: PartialEq> Upgrade<C, Level<Song, User>> for PartialLevel<Song, User> {
    type From = Self;
    type Request = LevelRequest;
    type Upgrade = Level<Option<u64>, u64>;

    fn upgrade_request(from: &Self::From) -> Option<Self::Request> {
        Some(from.level_id.into())
    }

    fn default_upgrade() -> Option<Self::Upgrade> {
        None
    }

    fn lookup_upgrade(from: &Self::From, cache: &C, request_result: Level<Option<u64>, u64>) -> Result<Self::Upgrade, <C as Cache>::Err> {
        Ok(request_result)
    }

    fn upgrade(self, upgrade: Self::Upgrade) -> Level<Song, User> {
        let Level {
            level_data,
            password,
            time_since_update,
            time_since_upload,
            index_36,
            ..
        } = upgrade;

        Level {
            base: self,
            level_data,
            password,
            time_since_update,
            time_since_upload,
            index_36,
        }
    }

    fn current(&self) -> &Self::From {
        self
    }
}

impl<C: Cache> Upgrade<C, Level<Option<NewgroundsSong>, u64>> for Level<Option<u64>, u64>
where
    C: Lookup<NewgroundsSong>,
{
    type From = Option<u64>;
    type Request = LevelsRequest;
    type Upgrade = Option<NewgroundsSong>;

    fn upgrade_request(from: &Self::From) -> Option<Self::Request> {
        match from {
            Some(song_id) => Some(LevelsRequest::default().filter(SearchFilters::default().custom_song(*song_id))),
            None => None,
        }
    }

    fn default_upgrade() -> Option<Self::Upgrade> {
        Some(None)
    }

    fn lookup_upgrade(from: &Self::From, cache: &C, _: <LevelsRequest as Request>::Result) -> Result<Self::Upgrade, <C as Cache>::Err> {
        Ok(match from {
            Some(song_id) => cache.lookup(*song_id)?.into(),
            None => None,
        })
    }

    fn upgrade(self, upgrade: Self::Upgrade) -> Level<Option<NewgroundsSong>, u64> {
        change_level_song(self, upgrade)
    }

    fn current(&self) -> &Self::From {
        &self.base.custom_song
    }
}

impl<C: Cache + Lookup<NewgroundsSong>> Upgrade<C, PartialLevel<Option<NewgroundsSong>, u64>> for PartialLevel<Option<u64>, u64> {
    type From = Option<u64>;
    type Request = LevelsRequest;
    type Upgrade = Option<NewgroundsSong>;

    fn upgrade_request(from: &Self::From) -> Option<Self::Request> {
        match from {
            Some(song_id) => Some(LevelsRequest::default().filter(SearchFilters::default().custom_song(*song_id))),
            None => None,
        }
    }

    fn default_upgrade() -> Option<Self::Upgrade> {
        Some(None)
    }

    fn lookup_upgrade(from: &Self::From, cache: &C, _: <LevelsRequest as Request>::Result) -> Result<Self::Upgrade, <C as Cache>::Err> {
        Ok(match from {
            Some(song_id) => cache.lookup(*song_id)?.into(),
            None => None,
        })
    }

    fn upgrade(self, upgrade: Self::Upgrade) -> PartialLevel<Option<NewgroundsSong>, u64> {
        change_partial_level_song(self, upgrade)
    }

    fn current(&self) -> &Self::From {
        &self.custom_song
    }
}

impl<C: Cache> Extendable<C, PartialLevel<Option<NewgroundsSong>, u64>> for PartialLevel<Option<u64>, u64>
where
    C: Lookup<NewgroundsSong>,
{
    type Extension = Option<NewgroundsSong>;
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

    fn extend(self, custom_song: Option<NewgroundsSong>) -> PartialLevel<Option<NewgroundsSong>, u64> {
        change_partial_level_song(self, custom_song)
    }

    fn change_extension(
        current: PartialLevel<Option<NewgroundsSong>, u64>,
        new_extension: Option<NewgroundsSong>,
    ) -> PartialLevel<Option<NewgroundsSong>, u64> {
        change_partial_level_song(current, new_extension)
    }
}

impl<C: Cache, Song: PartialEq> Extendable<C, Level<Song, Option<Creator>>> for Level<Song, u64>
where
    C: Lookup<Creator>,
{
    type Extension = Option<Creator>;
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

    fn extend(self, addon: Option<Creator>) -> Level<Song, Option<Creator>> {
        change_level_user(self, addon)
    }

    fn change_extension(current: Level<Song, Option<Creator>>, new_extension: Option<Creator>) -> Level<Song, Option<Creator>> {
        change_level_user(current, new_extension)
    }
}

impl<C: Cache, Song: PartialEq> Extendable<C, PartialLevel<Song, Option<Creator>>> for PartialLevel<Song, u64>
where
    C: Lookup<Creator>,
{
    type Extension = Option<Creator>;
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

    fn extend(self, creator: Option<Creator>) -> PartialLevel<Song, Option<Creator>> {
        change_partial_level_user(self, creator)
    }

    fn change_extension(
        current: PartialLevel<Song, Option<Creator>>,
        new_extension: Option<Creator>,
    ) -> PartialLevel<Song, Option<Creator>> {
        change_partial_level_user(current, new_extension)
    }
}

// FIXME: this impl needs to go from Option<Creator>, not u64 since we need the account id, not user
// id
impl<C: Cache, Song: PartialEq> Extendable<C, PartialLevel<Song, Option<User>>> for PartialLevel<Song, u64> {
    type Extension = Option<User>;
    type Request = UserRequest;

    fn lookup_extension(&self, cache: &C, request_result: User) -> Result<Option<User>, <C as Cache>::Err> {
        Ok(Some(request_result))
    }

    fn on_extension_absent() -> Option<Option<User>> {
        Some(None)
    }

    fn extension_request(&self) -> Self::Request {
        self.creator.into()
    }

    fn extend(self, addon: Option<User>) -> PartialLevel<Song, Option<User>> {
        change_partial_level_user(self, addon)
    }

    fn change_extension(current: PartialLevel<Song, Option<User>>, new_extension: Option<User>) -> PartialLevel<Song, Option<User>> {
        change_partial_level_user(current, new_extension)
    }
}

impl<C: Cache, Song: PartialEq> Extendable<C, Level<Song, Option<User>>> for Level<Song, u64> {
    type Extension = Option<User>;
    type Request = UserRequest;

    fn lookup_extension(&self, cache: &C, request_result: User) -> Result<Option<User>, <C as Cache>::Err> {
        Ok(Some(request_result))
    }

    fn on_extension_absent() -> Option<Option<User>> {
        Some(None)
    }

    fn extension_request(&self) -> Self::Request {
        self.base.creator.into()
    }

    fn extend(self, addon: Option<User>) -> Level<Song, Option<User>> {
        change_level_user(self, addon)
    }

    fn change_extension(current: Level<Song, Option<User>>, new_extension: Option<User>) -> Level<Song, Option<User>> {
        change_level_user(current, new_extension)
    }
}

fn change_partial_level_song<OldSong: PartialEq, NewSong: PartialEq, User: PartialEq>(
    partial_level: PartialLevel<OldSong, User>,
    new_song: NewSong,
) -> PartialLevel<NewSong, User> {
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
    } = partial_level;

    PartialLevel {
        custom_song: new_song,

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

fn change_partial_level_user<OldUser: PartialEq, NewUser: PartialEq, Song: PartialEq>(
    partial_level: PartialLevel<Song, OldUser>,
    new_user: NewUser,
) -> PartialLevel<Song, NewUser> {
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
    } = partial_level;

    PartialLevel {
        creator: new_user,

        level_id,
        name,
        description,
        version,
        custom_song,
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

fn change_level_user<OldUser: PartialEq, NewUser: PartialEq, Song: PartialEq>(
    level: Level<Song, OldUser>,
    new_user: NewUser,
) -> Level<Song, NewUser> {
    let Level {
        base,
        level_data,
        password,
        time_since_update,
        time_since_upload,
        index_36,
    } = level;

    Level {
        base: change_partial_level_user(base, new_user),
        level_data,
        password,
        time_since_update,
        time_since_upload,
        index_36,
    }
}

fn change_level_song<OldSong: PartialEq, NewSong: PartialEq, User: PartialEq>(
    level: Level<OldSong, User>,
    new_song: NewSong,
) -> Level<NewSong, User> {
    let Level {
        base,
        level_data,
        password,
        time_since_update,
        time_since_upload,
        index_36,
    } = level;

    Level {
        base: change_partial_level_song(base, new_song),
        level_data,
        password,
        time_since_update,
        time_since_upload,
        index_36,
    }
}
