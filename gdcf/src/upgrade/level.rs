use crate::{
    api::request::{LevelRequest, LevelRequestType, LevelsRequest, Request, SearchFilters, UserRequest},
    cache::{Cache, CacheEntry, CreatorKey, Lookup, NewgroundsSongKey},
    upgrade::{Upgradable, UpgradeError, UpgradeQuery},
};
use gdcf_model::{
    level::{Level, PartialLevel},
    song::NewgroundsSong,
    user::{Creator, User},
};

impl<Song, User> Upgradable<Level<Song, User>> for PartialLevel<Song, User> {
    type From = PartialLevel<Option<u64>, u64>;
    type LookupKey = LevelRequest;
    type Request = LevelRequest;
    type Upgrade = Level<Option<u64>, u64>;

    fn query_upgrade<C: Cache + Lookup<Self::LookupKey>>(
        &self,
        cache: &C,
        ignored_cached: bool,
    ) -> Result<UpgradeQuery<Self::Request, Self::Upgrade>, UpgradeError<C::Err>> {
        query_upgrade!(
            cache,
            LevelRequest::new(self.level_id),
            LevelRequest::new(self.level_id),
            ignored_cached
        )
    }

    fn process_query_result<C: Cache + Lookup<Self::LookupKey>>(
        &self,
        _cache: &C,
        resolved_query: UpgradeQuery<CacheEntry<Level<Option<u64>, u64>, C::CacheEntryMeta>, Self::Upgrade>,
    ) -> Result<UpgradeQuery<(), Self::Upgrade>, UpgradeError<C::Err>> {
        match resolved_query.one() {
            (None, Some(user)) => Ok(UpgradeQuery::One(None, Some(user))),
            (Some(CacheEntry::Cached(user, _)), _) => Ok(UpgradeQuery::One(None, Some(user))),
            _ => Err(UpgradeError::UpgradeFailed),
        }
    }

    fn upgrade<State>(self, upgrade: UpgradeQuery<State, Self::Upgrade>) -> (Level<Song, User>, UpgradeQuery<State, Self::From>) {
        let upgrade = upgrade.one().1.unwrap();

        let (partial_level, song) = change_partial_level_song(self, ());
        let (partial_level, user) = change_partial_level_user(partial_level, ());

        let (level, song_id) = change_level_song(upgrade, song);
        let (level, creator_id) = change_level_user(level, user);

        let partial_level = change_partial_level_user(partial_level, creator_id).0;
        let partial_level = change_partial_level_song(partial_level, song_id).0;

        (level, UpgradeQuery::One(None, Some(partial_level)))
    }

    fn downgrade<State>(
        upgraded: Level<Song, User>,
        downgrade: UpgradeQuery<State, Self::From>,
    ) -> (Self, UpgradeQuery<State, Self::Upgrade>) {
        let downgrade = downgrade.one().1.unwrap();

        let (level, song) = change_level_song(upgraded, ());
        let (level, creator) = change_level_user(level, ());

        let (partial_level, song_id) = change_partial_level_song(downgrade, song);
        let (partial_level, creator_id) = change_partial_level_user(partial_level, creator);

        let level = change_level_user(level, creator_id).0;
        let level = change_level_song(level, song_id).0;

        (partial_level, UpgradeQuery::One(None, Some(level)))
    }
}

impl<User> Upgradable<Level<Option<NewgroundsSong>, User>> for Level<Option<u64>, User> {
    type From = Option<u64>;
    type LookupKey = NewgroundsSongKey;
    type Request = LevelsRequest;
    type Upgrade = Option<NewgroundsSong>;

    fn query_upgrade<C: Cache + Lookup<Self::LookupKey>>(
        &self,
        cache: &C,
        ignored_cached: bool,
    ) -> Result<UpgradeQuery<Self::Request, Self::Upgrade>, UpgradeError<C::Err>> {
        match self.base.custom_song {
            Some(song_id) =>
                query_upgrade_option!(
                    cache,
                    NewgroundsSongKey(song_id),
                    LevelsRequest::default()
                        .filter(SearchFilters::default().custom_song(song_id))
                        .request_type(LevelRequestType::MostLiked),
                    ignored_cached
                ),
            None => Ok(UpgradeQuery::One(None, Some(None))),
        }
    }

    fn process_query_result<C: Cache + Lookup<Self::LookupKey>>(
        &self,
        cache: &C,
        resolved_query: UpgradeQuery<CacheEntry<<Self::Request as Request>::Result, C::CacheEntryMeta>, Self::Upgrade>,
    ) -> Result<UpgradeQuery<(), Self::Upgrade>, UpgradeError<C::Err>> {
        match resolved_query.one() {
            (None, Some(newgrounds_song)) => Ok(UpgradeQuery::One(None, Some(newgrounds_song))),
            (Some(_), _) =>
                match cache.lookup(&NewgroundsSongKey(self.base.custom_song.unwrap()))? {
                    CacheEntry::Cached(song, _) => Ok(UpgradeQuery::One(None, Some(Some(song)))),
                    _ => Ok(UpgradeQuery::One(None, Some(None))),
                },
            _ => Err(UpgradeError::UpgradeFailed),
        }
    }

    fn upgrade<State>(
        self,
        upgrade: UpgradeQuery<State, Self::Upgrade>,
    ) -> (Level<Option<NewgroundsSong>, User>, UpgradeQuery<State, Self::From>) {
        let (level, old_song) = change_level_song(self, upgrade.one().1.unwrap());

        (level, UpgradeQuery::One(None, Some(old_song)))
    }

    fn downgrade<State>(
        upgraded: Level<Option<NewgroundsSong>, User>,
        downgrade: UpgradeQuery<State, Self::From>,
    ) -> (Self, UpgradeQuery<State, Self::Upgrade>) {
        let (level, new_song) = change_level_song(upgraded, downgrade.one().1.unwrap());

        (level, UpgradeQuery::One(None, Some(new_song)))
    }
}

impl<User> Upgradable<PartialLevel<Option<NewgroundsSong>, User>> for PartialLevel<Option<u64>, User> {
    type From = Option<u64>;
    type LookupKey = NewgroundsSongKey;
    type Request = LevelsRequest;
    type Upgrade = Option<NewgroundsSong>;

    fn query_upgrade<C: Cache + Lookup<Self::LookupKey>>(
        &self,
        cache: &C,
        ignored_cached: bool,
    ) -> Result<UpgradeQuery<Self::Request, Self::Upgrade>, UpgradeError<C::Err>> {
        match self.custom_song {
            Some(song_id) =>
                query_upgrade_option!(
                    cache,
                    NewgroundsSongKey(song_id),
                    LevelsRequest::default()
                        .filter(SearchFilters::default().custom_song(song_id))
                        .request_type(LevelRequestType::MostLiked),
                    ignored_cached
                ),
            None => Ok(UpgradeQuery::One(None, Some(None))),
        }
    }

    fn process_query_result<C: Cache + Lookup<Self::LookupKey>>(
        &self,
        cache: &C,
        resolved_query: UpgradeQuery<CacheEntry<<Self::Request as Request>::Result, C::CacheEntryMeta>, Self::Upgrade>,
    ) -> Result<UpgradeQuery<(), Self::Upgrade>, UpgradeError<C::Err>> {
        match resolved_query.one() {
            (None, Some(newgrounds_song)) => Ok(UpgradeQuery::One(None, Some(newgrounds_song))),
            (Some(_), _) =>
                match cache.lookup(&NewgroundsSongKey(self.custom_song.unwrap()))? {
                    CacheEntry::Cached(song, _) => Ok(UpgradeQuery::One(None, Some(Some(song)))),
                    _ => Ok(UpgradeQuery::One(None, Some(None))),
                },
            _ => Err(UpgradeError::UpgradeFailed),
        }
    }

    fn upgrade<State>(
        self,
        upgrade: UpgradeQuery<State, Self::Upgrade>,
    ) -> (PartialLevel<Option<NewgroundsSong>, User>, UpgradeQuery<State, Self::From>) {
        let (level, old_song) = change_partial_level_song(self, upgrade.one().1.unwrap());

        (level, UpgradeQuery::One(None, Some(old_song)))
    }

    fn downgrade<State>(
        upgraded: PartialLevel<Option<NewgroundsSong>, User>,
        downgrade: UpgradeQuery<State, Self::From>,
    ) -> (Self, UpgradeQuery<State, Self::Upgrade>) {
        let (level, new_song) = change_partial_level_song(upgraded, downgrade.one().1.unwrap());

        (level, UpgradeQuery::One(None, Some(new_song)))
    }
}

impl<Song> Upgradable<Level<Song, Option<Creator>>> for Level<Song, u64> {
    type From = u64;
    type LookupKey = CreatorKey;
    type Request = LevelsRequest;
    type Upgrade = Option<Creator>;

    fn query_upgrade<C: Cache + Lookup<Self::LookupKey>>(
        &self,
        cache: &C,
        ignored_cached: bool,
    ) -> Result<UpgradeQuery<Self::Request, Self::Upgrade>, UpgradeError<C::Err>> {
        query_upgrade_option!(
            cache,
            CreatorKey(self.base.creator),
            LevelsRequest::default()
                .search(self.base.creator.to_string())
                .request_type(LevelRequestType::User),
            ignored_cached
        )
    }

    fn process_query_result<C: Cache + Lookup<Self::LookupKey>>(
        &self,
        cache: &C,
        resolved_query: UpgradeQuery<CacheEntry<<Self::Request as Request>::Result, C::CacheEntryMeta>, Self::Upgrade>,
    ) -> Result<UpgradeQuery<(), Self::Upgrade>, UpgradeError<C::Err>> {
        match resolved_query.one() {
            (None, Some(creator)) => Ok(UpgradeQuery::One(None, Some(creator))),
            (Some(_), _) =>
                match cache.lookup(&CreatorKey(self.base.creator))? {
                    CacheEntry::Cached(creator, _) => Ok(UpgradeQuery::One(None, Some(Some(creator)))),
                    _ => Ok(UpgradeQuery::One(None, Some(None))),
                },
            _ => Err(UpgradeError::UpgradeFailed),
        }
    }

    fn upgrade<State>(
        self,
        upgrade: UpgradeQuery<State, Self::Upgrade>,
    ) -> (Level<Song, Option<Creator>>, UpgradeQuery<State, Self::From>) {
        let (level, old_creator) = change_level_user(self, upgrade.one().1.unwrap());

        (level, UpgradeQuery::One(None, Some(old_creator)))
    }

    fn downgrade<State>(
        upgraded: Level<Song, Option<Creator>>,
        downgrade: UpgradeQuery<State, Self::From>,
    ) -> (Self, UpgradeQuery<State, Self::Upgrade>) {
        let (level, new_creator) = change_level_user(upgraded, downgrade.one().1.unwrap());

        (level, UpgradeQuery::One(None, Some(new_creator)))
    }
}

impl<Song> Upgradable<PartialLevel<Song, Option<Creator>>> for PartialLevel<Song, u64> {
    type From = u64;
    type LookupKey = CreatorKey;
    type Request = LevelsRequest;
    type Upgrade = Option<Creator>;

    fn query_upgrade<C: Cache + Lookup<Self::LookupKey>>(
        &self,
        cache: &C,
        ignored_cached: bool,
    ) -> Result<UpgradeQuery<Self::Request, Self::Upgrade>, UpgradeError<C::Err>> {
        query_upgrade_option!(
            cache,
            CreatorKey(self.creator),
            LevelsRequest::default()
                .search(self.creator.to_string())
                .request_type(LevelRequestType::User),
            ignored_cached
        )
    }

    fn process_query_result<C: Cache + Lookup<Self::LookupKey>>(
        &self,
        cache: &C,
        resolved_query: UpgradeQuery<CacheEntry<<Self::Request as Request>::Result, C::CacheEntryMeta>, Self::Upgrade>,
    ) -> Result<UpgradeQuery<(), Self::Upgrade>, UpgradeError<C::Err>> {
        match resolved_query.one() {
            (None, Some(creator)) => Ok(UpgradeQuery::One(None, Some(creator))),
            (Some(_), _) =>
                match cache.lookup(&CreatorKey(self.creator))? {
                    CacheEntry::Cached(creator, _) => Ok(UpgradeQuery::One(None, Some(Some(creator)))),
                    _ => Ok(UpgradeQuery::One(None, Some(None))),
                },
            _ => Err(UpgradeError::UpgradeFailed),
        }
    }

    fn upgrade<State>(
        self,
        upgrade: UpgradeQuery<State, Self::Upgrade>,
    ) -> (PartialLevel<Song, Option<Creator>>, UpgradeQuery<State, Self::From>) {
        let (level, old_creator) = change_partial_level_user(self, upgrade.one().1.unwrap());

        (level, UpgradeQuery::One(None, Some(old_creator)))
    }

    fn downgrade<State>(
        upgraded: PartialLevel<Song, Option<Creator>>,
        downgrade: UpgradeQuery<State, Self::From>,
    ) -> (Self, UpgradeQuery<State, Self::Upgrade>) {
        let (level, new_creator) = change_partial_level_user(upgraded, downgrade.one().1.unwrap());

        (level, UpgradeQuery::One(None, Some(new_creator)))
    }
}

impl<Song> Upgradable<Level<Song, Option<User>>> for Level<Song, Option<Creator>> {
    type From = Option<Creator>;
    type LookupKey = UserRequest;
    type Request = UserRequest;
    type Upgrade = Option<User>;

    fn query_upgrade<C: Cache + Lookup<Self::LookupKey>>(
        &self,
        cache: &C,
        ignored_cached: bool,
    ) -> Result<UpgradeQuery<Self::Request, Self::Upgrade>, UpgradeError<C::Err>> {
        match self.base.creator.as_ref().and_then(|creator| creator.account_id) {
            Some(account_id) => query_upgrade_option!(cache, UserRequest::new(account_id), UserRequest::new(account_id), ignored_cached),
            None => Ok(UpgradeQuery::One(None, Some(None))),
        }
    }

    fn process_query_result<C: Cache + Lookup<Self::LookupKey>>(
        &self,
        _cache: &C,
        resolved_query: UpgradeQuery<CacheEntry<User, C::CacheEntryMeta>, Self::Upgrade>,
    ) -> Result<UpgradeQuery<(), Self::Upgrade>, UpgradeError<C::Err>> {
        match resolved_query.one() {
            (None, Some(user)) => Ok(UpgradeQuery::One(None, Some(user))),
            (Some(CacheEntry::Cached(user, _)), _) => Ok(UpgradeQuery::One(None, Some(Some(user)))),
            (Some(_), _) => Ok(UpgradeQuery::One(None, Some(None))),
            _ => Err(UpgradeError::UpgradeFailed),
        }
    }

    fn upgrade<State>(self, upgrade: UpgradeQuery<State, Self::Upgrade>) -> (Level<Song, Option<User>>, UpgradeQuery<State, Self::From>) {
        let (level, creator) = change_level_user(self, upgrade.one().1.unwrap());

        (level, UpgradeQuery::One(None, Some(creator)))
    }

    fn downgrade<State>(
        upgraded: Level<Song, Option<User>>,
        downgrade: UpgradeQuery<State, Self::From>,
    ) -> (Self, UpgradeQuery<State, Self::Upgrade>) {
        let (level, user) = change_level_user(upgraded, downgrade.one().1.unwrap());

        (level, UpgradeQuery::One(None, Some(user)))
    }
}

impl<Song> Upgradable<PartialLevel<Song, Option<User>>> for PartialLevel<Song, Option<Creator>> {
    type From = Option<Creator>;
    type LookupKey = UserRequest;
    type Request = UserRequest;
    type Upgrade = Option<User>;

    fn query_upgrade<C: Cache + Lookup<Self::LookupKey>>(
        &self,
        cache: &C,
        ignored_cached: bool,
    ) -> Result<UpgradeQuery<Self::Request, Self::Upgrade>, UpgradeError<C::Err>> {
        match self.creator.as_ref().and_then(|creator| creator.account_id) {
            Some(account_id) => query_upgrade_option!(cache, UserRequest::new(account_id), UserRequest::new(account_id), ignored_cached),
            None => Ok(UpgradeQuery::One(None, Some(None))),
        }
    }

    fn process_query_result<C: Cache + Lookup<Self::LookupKey>>(
        &self,
        _cache: &C,
        resolved_query: UpgradeQuery<CacheEntry<User, C::CacheEntryMeta>, Self::Upgrade>,
    ) -> Result<UpgradeQuery<(), Self::Upgrade>, UpgradeError<C::Err>> {
        match resolved_query.one() {
            (None, Some(user)) => Ok(UpgradeQuery::One(None, Some(user))),
            (Some(CacheEntry::Cached(user, _)), _) => Ok(UpgradeQuery::One(None, Some(Some(user)))),
            (Some(_), _) => Ok(UpgradeQuery::One(None, Some(None))),
            _ => Err(UpgradeError::UpgradeFailed),
        }
    }

    fn upgrade<State>(
        self,
        upgrade: UpgradeQuery<State, Self::Upgrade>,
    ) -> (PartialLevel<Song, Option<User>>, UpgradeQuery<State, Self::From>) {
        let (level, creator) = change_partial_level_user(self, upgrade.one().1.unwrap());

        (level, UpgradeQuery::One(None, Some(creator)))
    }

    fn downgrade<State>(
        upgraded: PartialLevel<Song, Option<User>>,
        downgrade: UpgradeQuery<State, Self::From>,
    ) -> (Self, UpgradeQuery<State, Self::Upgrade>) {
        let (level, user) = change_partial_level_user(upgraded, downgrade.one().1.unwrap());

        (level, UpgradeQuery::One(None, Some(user)))
    }
}

fn change_partial_level_song<OldSong, NewSong, User>(
    partial_level: PartialLevel<OldSong, User>,
    new_song: NewSong,
) -> (PartialLevel<NewSong, User>, OldSong) {
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
        custom_song,
    } = partial_level;

    (
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
        },
        custom_song,
    )
}

fn change_partial_level_user<OldUser, NewUser, Song>(
    partial_level: PartialLevel<Song, OldUser>,
    new_user: NewUser,
) -> (PartialLevel<Song, NewUser>, OldUser) {
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
        creator,
    } = partial_level;

    (
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
        },
        creator,
    )
}

fn change_level_user<OldUser, NewUser, Song>(level: Level<Song, OldUser>, new_user: NewUser) -> (Level<Song, NewUser>, OldUser) {
    let Level {
        base,
        level_data,
        password,
        time_since_update,
        time_since_upload,
        index_36,
    } = level;

    let (new_base, old_user) = change_partial_level_user(base, new_user);

    (
        Level {
            base: new_base,
            level_data,
            password,
            time_since_update,
            time_since_upload,
            index_36,
        },
        old_user,
    )
}

fn change_level_song<OldSong, NewSong, User>(level: Level<OldSong, User>, new_song: NewSong) -> (Level<NewSong, User>, OldSong) {
    let Level {
        base,
        level_data,
        password,
        time_since_update,
        time_since_upload,
        index_36,
    } = level;

    let (new_base, old_song) = change_partial_level_song(base, new_song);

    (
        Level {
            base: new_base,
            level_data,
            password,
            time_since_update,
            time_since_upload,
            index_36,
        },
        old_song,
    )
}
