#![feature(prelude_import)]
#![no_std]
#![deny(
    bare_trait_objects,
    missing_debug_implementations,
    unused_extern_crates,
    patterns_in_fns_without_body,
    stable_features,
    unknown_lints,
    unused_features,
    unused_imports,
    unused_parens
)]

//! The `gdcf` crate is the core of the Geometry Dash Caching Framework.
//! It provides all the core traits required to implement an API Client and
//! a cache which are used by [`Gdcf`].
//!
//! # Geometry Dash Caching Framework
//!
//! The idea behind the Geometry Dash Caching Framework is to provide fast and
//! reliable access to the resources provided by the Geometry Dash servers. It
//! achieves this goal by caching all responses from the servers. When a resource is requested, it
//! is first looked up in the cache. If the cache entry is not yet expired, it is simply returned
//! and the request can be handled nearly instantly without any interaction with the Geometry Dash
//! servers. If the cache entry is existing, but expired, GDCF will make an asynchronous request to
//! the Geometry Dash servers and create a [Future](GdcfFuture) that resolves to the result of that
//! request, while also providing access to the cached value (without the need to poll the Future
//! to completion). The only time you are actually forced to wait for a response from the Geometry
//! Dash servers is when the cache entry for a request isn't existing.
//!
//! Further, GDCF has the ability to "glue together" multiple requests to provide more information
//! about requested objects. It is, for example, possible to issue a [`LevelRequest`]
//! (`downloadGJLevel`) and have GDCF automatically issue a [`LevelsRequest`] (`getGJLevels`) to
//! retrieve the creator and newgrounds song, which aren't provided by the former endpoint.
//!
//! # How to use:
//! This crate only provides the required traits for caches and API clients, and the code that
//! connects them. To use GDCF you first need to either find yourself an existing implementation of
//! those, or write your own.
//!
//! The following example uses the `gdcf_dbcache` crate as its cache implementation (a database
//! cache with sqlite and postgreSQL backend) and the `gdrs` crate as its API client.
//!
//! ```rust
//! // First we need to configure the cache. Here we're using a sqlite in-memory database
//! // whose cache entries expire after 30 minutes.
//! let mut config = DatabaseCacheConfig::sqlite_memory_config();
//! config.invalidate_after(Duration::minutes(30));
//!
//! // Then we can create the actual cache and API wrapper
//! let cache = DatabaseCache::new(config);
//! let client = BoomlingsClient::new();
//!
//! // A database cache needs to go through initialization before it can be used, as it
//! // needs to create all the required tables
//! cache.initialize()?;
//!
//! // Then we can create an instance of the Gdcf struct, which we will use to
//! // actually make all our requests
//! let gdcf = Gdcf::new(client, cache);
//!
//! // And we're good to go! To make a request, we need to initialize one of the
//! // request structs. Here, we're make a requests to retrieve the 6th page of
//! // featured demon levels of any demon difficulty
//! let request = LevelsRequest::default()
//!     .request_type(LevelRequestType::Featured)
//!     .with_rating(LevelRating::Demon(DemonRating::Hard))
//!     .page(5);
//!
//! // To actually issue the request, we call the appropriate method on our Gdcf instance.
//! // The type parameters on these methods determine how much associated information
//! // should be retrieved for the request result. Here we're telling GDCF to also
//! // get us information about the requested levels' custom songs and creators
//! // instead of just their IDs. "paginate_levels" give us a stream over all pages
//! // of results from our request instead of only the page we requested.
//! let stream = gdcf.paginate_levels::<NewgroundsSong, Creator>(request);
//!
//! // Since we have a stream, we can use all our favorite Stream methods from the
//! // futures crate. Here we limit the stream to 50 pages of levels and print
//! // out each level's name, creator, song and song artist.
//! let future = stream
//!     .take(50)
//!     .for_each(|levels| {
//!         for level in levels {
//!             match level.custom_song {
//!                 Some(newgrounds_song) =>
//!                     println!(
//!                         "Retrieved demon level {} by {} using custom song {} by {}",
//!                         level.name, level.creator.name, newgrounds_song.name, newgrounds_song.artist
//!                     ),
//!                 None =>
//!                     println!(
//!                         "Retrieved demon level {} by {} using main song {} by {}",
//!                         level.name,
//!                         level.creator.name,
//!                         level.main_song.unwrap().name,
//!                         level.main_song.unwrap().artist
//!                     ),
//!             }
//!         }
//!
//!         Ok(())
//!     })
//!     .map_err(|error| eprintln!("Something went wrong! {:?}", error));
//!
//! tokio::run(future);
//! ```
#[prelude_import]
use std::prelude::v1::*;
#[macro_use]
extern crate std;
pub extern crate chrono;
extern crate futures;
#[macro_use]
extern crate log;
extern crate failure;
extern crate gdcf_model;

// TODO: it would be nice to be able to differentiate between cache-miss because the data doesn't
// exist and cache-miss because the data simply wasn't requested yet

use api::{
    request::{LevelRequest, LevelsRequest, PaginatableRequest, Request, UserRequest},
    ApiClient,
};
use cache::{Cache, CachedObject};
use error::GdcfError;
use futures::{
    future::{err, ok, Either},
    task, Async, Future, Stream,
};
use gdcf_model::{
    level::{Level, PartialLevel},
    song::{NewgroundsSong, SERVER_SIDED_DATA_INCONSISTENCY_ERROR},
    user::{Creator, User, DELETED},
};
use std::mem;

#[macro_use]
mod macros {

    //pub mod convert;
    //pub mod model;

    // FIXME: move this somewhere more fitting

    // TODO: for levels, get their creator via the getGJProfile endpoint, then we can give PartialLevel
    // a User

    /*
    pub trait ProcessRequest2<A, R, C, T>
    where
        R: Request,
        A: ApiClient + MakeRequest<R>,
        C: Cache,
    {
        fn process_request(&self, request: R) -> GdcfFuture<T, A::Err, C::Err>;
    }

    impl<A, R, C> ProcessRequest2<A, R, C, R::Result> for Gdcf<A, C>
    where
        R: Request,
        A: ApiClient + MakeRequest<R>,
        C: Cache,
    {
    }*/

    // When simply downloading a level, we do not get its song, only the song ID. The song itself is
    // only provided for a LevelsRequest

    // TODO: reintroduce debugging statements

    // In this case, we have the level cached and up-to-date
    // Level uses a main song, we dont need to do anything apart from changing the generic type

    // Level uses a custom song.
    // We cannot do the lookup in the match because then the cache would be locked for the entire match
    // block which would deadlock because of the `process_request` call in it.

    // The custom song is cached, replace the ID with actual song object and change generic type

    // The custom song isn't cached, make a request that's sure to put it into the cache, then perform
    // the exchange

    // Cache lookup failed, create future that resolves to error instantly

    // In this case we have it cached, but not up to date, or not cached at all
    // If we have it cached, we need to update the cached value either with its custom song from the
    // cache, if that exists. If it doesn't, we will end up creating a future that does not contain
    // any cached object.

    // Level itself wasn't cached already

    // We cannot do the lookup in the match because then the cache would be locked for the entire match
    // block which would deadlock because of the `process_request` call in it.

    // Here we must have this logic inside of the future. If we were to lookup the song_id we got from
    // the (potentially) cached object, it might be outdated, leaving us with an up-to-date level
    // object that contains a NewgroundsSong object, which does not represent the song the level
    // uses (because the song was changed between now and the last time the level was cached)

    // Note that we do not need to check if the cache value is out-of-date here, because we only
    // request creators that we put into the cache by the very request whose result we're processing
    // here. I THINK it's impossible to have an outdated creator while not having the level request
    // outdated we well.

    // For very old levels where the players never registered, the accounts got lost somehow.
    // LevelsRequest containing such levels don't contain any creator info about those levels. This
    // again implies that the cache miss, which should be impossible, is such a case.

    // Note that the creator of a level cannot change. We can always use the user ID cached with the
    // level (if existing).

    /* NOTE: here we cannot decide whether the creator isn't
     * cached, or */

    // whether his GD account was deleted. We go with the conversative
    // option and assume it wasn't cached.

    // TODO: impl ProcessRequest<LevelsRequest, Vec<PartialLevel<u64, User>>> for Gdcf

    // TODO: impl ProcessRequest<LevelRequest, Level<u64, User>> for Gdcf

    /*impl<A, C> ProcessRequest<A, C, LevelsRequest, Vec<PartialLevel<u64, User>>> for Gdcf<A, C>
    where
        A: ApiClient,
        C: Cache,
    {
        fn process_request(&self, request: LevelsRequest) -> GdcfFuture<Vec<PartialLevel<u64, User>>, A::Err, C::Err> {
            let raw: GdcfFuture<Vec<PartialLevel<u64, u64>>, _, _> = self.process_request(request);
            let cache = self.cache.clone();

            match raw {
                GdcfFuture {cached: Some(cached), inner: None} => {
                    match self.cache().lookup_user(&cached.creator.into()) {
                        Ok(user) => GdcfFuture::up_to_date(build::partial_level_user(user)),

                        Err(CacheError::CacheMiss) => {
                            //GdcfFuture::absent(self.user)
                            unimplemented!()
                        }

                        Err(err) => GdcfFuture::error(err.into())
                    }
                }

                _ => unreachable!()
            }
        }
    }*/

    // invariant: at least one of the fields is not `None`

    // FIXME:

    macro_rules! collect_one(( $ cache : expr , $ variant : ident ) => {
                             move | response | {
                             let mut result = None ; for obj in response {
                             $ cache . store_object ( & obj ) . map_err (
                             GdcfError :: Cache ) ? ; if let Secondary :: $
                             variant ( level ) = obj { result = Some ( level )
                             } } result . ok_or ( GdcfError :: NoContent ) } }
                             ;);
    macro_rules! collect_many((
                              $ request : expr , $ cache : expr , $ bulk_store
                              : ident , $ variant : ident ) => {
                              move | response | {
                              let mut result = Vec :: new (  ) ; for obj in
                              response {
                              $ cache . store_object ( & obj ) . map_err (
                              GdcfError :: Cache ) ? ; if let Secondary :: $
                              variant ( level ) = obj {
                              result . push ( level ) } } if ! result .
                              is_empty (  ) {
                              $ cache . $ bulk_store ( & $ request , & result
                              ) . map_err ( GdcfError :: Cache ) ? ; Ok (
                              result ) } else { Err ( GdcfError :: NoContent )
                              } } } ;);
    macro_rules! gdcf((
                      $ self : expr , $ request : expr , $ cache_lookup :
                      ident , $ future_closure : expr ) => {
                      {
                      let cache = $ self . cache (  ) ; match cache . $
                      cache_lookup ( & $ request ) {
                      Ok ( cached ) => if cache . is_expired ( & cached ) {
                      info ! (
                      "Cache entry for request {} is expired!" , $ request ) ;
                      GdcfFuture :: outdated ( cached , $ future_closure (  )
                      ) } else {
                      info ! (
                      "Cached entry for request {} is up-to-date!" , $ request
                      ) ; GdcfFuture :: up_to_date ( cached ) } , Err ( error
                      ) => if error . is_cache_miss (  ) {
                      info ! ( "No cache entry for request {}" , $ request ) ;
                      GdcfFuture :: absent ( $ future_closure (  ) ) } else {
                      GdcfFuture :: cache_error ( error ) } , } } } ;);
    macro_rules! setter(( $ name : ident , $ field : ident , $ t : ty ) => {
                        pub fn $ name ( mut self , $ field : $ t ) -> Self {
                        self . $ field = $ field ; self } } ; (
                        $ name : ident , $ t : ty ) => {
                        pub fn $ name ( mut self , arg0 : $ t ) -> Self {
                        self . $ name = arg0 ; self } } ; (
                        $ ( # [ $ attr : meta ] ) * $ name : ident : $ t : ty
                        ) => {
                        $ ( # [ $ attr ] ) * pub fn $ name (
                        mut self , $ name : $ t ) -> Self {
                        self . $ name = $ name ; self } } ; (
                        $ ( # [ $ attr : meta ] ) * $ field : ident [
                        $ name : ident ] : $ t : ty ) => {
                        $ ( # [ $ attr ] ) * pub fn $ name (
                        mut self , $ field : $ t ) -> Self {
                        self . $ field = $ field ; self } });
    macro_rules! const_setter(( $ name : ident , $ field : ident , $ t : ty )
                              => {
                              pub const fn $ name ( mut self , $ field : $ t )
                              -> Self { self . $ field = $ field ; self } } ;
                              ( $ name : ident , $ t : ty ) => {
                              pub const fn $ name ( mut self , arg0 : $ t ) ->
                              Self { self . $ name = arg0 ; self } } ; (
                              $ ( # [ $ attr : meta ] ) * $ name : ident : $ t
                              : ty ) => {
                              $ ( # [ $ attr ] ) * pub const fn $ name (
                              mut self , $ name : $ t ) -> Self {
                              self . $ name = $ name ; self } } ; (
                              $ ( # [ $ attr : meta ] ) * $ field : ident [
                              $ name : ident ] : $ t : ty ) => {
                              $ ( # [ $ attr ] ) * pub const fn $ name (
                              mut self , $ field : $ t ) -> Self {
                              self . $ field = $ field ; self } });
}
pub mod api {
    pub mod client {
        use crate::{
            api::request::{user::UserRequest, LevelRequest, LevelsRequest, Request},
            error::ApiError,
            Secondary,
        };
        use futures::Future;
        pub type ApiFuture<E> = Box<dyn Future<Item = Vec<Secondary>, Error = E> + Send + 'static>;
        pub trait ApiClient: Clone + Sized + Sync + Send + 'static {
            type Err: ApiError;
            fn level(&self, req: LevelRequest) -> ApiFuture<Self::Err>;
            fn levels(&self, req: LevelsRequest) -> ApiFuture<Self::Err>;
            fn user(&self, req: UserRequest) -> ApiFuture<Self::Err>;
        }
        pub enum Response<T> {
            Exact(T),
            More(T, Vec<Secondary>),
        }
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl<T: ::std::fmt::Debug> ::std::fmt::Debug for Response<T> {
            fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                match (&*self,) {
                    (&Response::Exact(ref __self_0),) => {
                        let mut debug_trait_builder = f.debug_tuple("Exact");
                        let _ = debug_trait_builder.field(&&(*__self_0));
                        debug_trait_builder.finish()
                    },
                    (&Response::More(ref __self_0, ref __self_1),) => {
                        let mut debug_trait_builder = f.debug_tuple("More");
                        let _ = debug_trait_builder.field(&&(*__self_0));
                        let _ = debug_trait_builder.field(&&(*__self_1));
                        debug_trait_builder.finish()
                    },
                }
            }
        }
        pub trait MakeRequest<R: Request>: ApiClient {
            fn make(&self, request: &R) -> Box<dyn Future<Item = Response<R::Result>, Error = Self::Err> + Send + 'static>;
        }
    }
    pub mod request {
        //! Module containing structs modelling the requests processable by the
        //! Geometry Dash servers
        //!
        //! Each struct in this module is modelled strictly after the requests made by
        //! the official Geometry Dash client.
        //!
        //! It further does not attempt to provide any (de)serialization for the
        //! request types, as there are simply no sensible defaults. When providing
        //! (de)serialization for requests, take a look at solutions like serde's
        //! remote types.
        //!
        //! Note that all `Hash` impls are to be forward compatible with new fields in
        //! the request. This means, that if an update to the GD API arrives which adds
        //! more fields to a request, those fields are hashed _only_ if they are
        //! different from their default values. This way, the hashes of requests made
        //! before the update will stay the same
        pub use self::{
            level::{LevelRequest, LevelRequestType, LevelsRequest, SearchFilters, SongFilter},
            user::UserRequest,
        };
        use gdcf_model::GameVersion;
        use std::{fmt::Display, hash::Hash};
        pub mod level {
            //! Module containing request definitions for retrieving levels
            use api::request::{BaseRequest, PaginatableRequest, Request, GD_21};
            use gdcf_model::level::{DemonRating, Level, LevelLength, LevelRating, PartialLevel};
            use std::{
                fmt::{Display, Error, Formatter},
                hash::{Hash, Hasher},
            };
            /// Struct modelled after a request to `downloadGJLevel22.php`.
            ///
            /// In the Geometry Dash API, this endpoint is used to download a level from
            /// the servers and retrieve some additional information that isn't provided
            /// with the response to a [`LevelsRequest`]
            #[rustc_copy_clone_marker]
            pub struct LevelRequest {
                /// The base request data
                pub base: BaseRequest,
                /// The ID of the level to download
                ///
                /// ## GD Internals:
                /// This field is called `levelID` in the boomlings API
                pub level_id: u64,
                /// Some weird field the Geometry Dash Client sends along
                ///
                /// ## GD Internals:
                /// This value needs to be converted to an integer for the boomlings API
                pub inc: bool,
                /// Some weird field the Geometry Dash Client sends along
                ///
                /// ## GD Internals:
                /// This field is called `extras` in the boomlings API and needs to be
                /// converted to an integer
                pub extra: bool,
            }
            #[automatically_derived]
            #[allow(unused_qualifications)]
            impl ::std::fmt::Debug for LevelRequest {
                fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                    match *self {
                        LevelRequest {
                            base: ref __self_0_0,
                            level_id: ref __self_0_1,
                            inc: ref __self_0_2,
                            extra: ref __self_0_3,
                        } => {
                            let mut debug_trait_builder = f.debug_struct("LevelRequest");
                            let _ = debug_trait_builder.field("base", &&(*__self_0_0));
                            let _ = debug_trait_builder.field("level_id", &&(*__self_0_1));
                            let _ = debug_trait_builder.field("inc", &&(*__self_0_2));
                            let _ = debug_trait_builder.field("extra", &&(*__self_0_3));
                            debug_trait_builder.finish()
                        },
                    }
                }
            }
            #[automatically_derived]
            #[allow(unused_qualifications)]
            impl ::std::default::Default for LevelRequest {
                #[inline]
                fn default() -> LevelRequest {
                    LevelRequest {
                        base: ::std::default::Default::default(),
                        level_id: ::std::default::Default::default(),
                        inc: ::std::default::Default::default(),
                        extra: ::std::default::Default::default(),
                    }
                }
            }
            #[automatically_derived]
            #[allow(unused_qualifications)]
            impl ::std::clone::Clone for LevelRequest {
                #[inline]
                fn clone(&self) -> LevelRequest {
                    {
                        let _: ::std::clone::AssertParamIsClone<BaseRequest>;
                        let _: ::std::clone::AssertParamIsClone<u64>;
                        let _: ::std::clone::AssertParamIsClone<bool>;
                        let _: ::std::clone::AssertParamIsClone<bool>;
                        *self
                    }
                }
            }
            #[automatically_derived]
            #[allow(unused_qualifications)]
            impl ::std::marker::Copy for LevelRequest {}
            /// Manual `Hash` impl that doesn't hash `base`.
            impl Hash for LevelRequest {
                fn hash<H: Hasher>(&self, state: &mut H) {
                    self.level_id.hash(state);
                    self.inc.hash(state);
                    self.extra.hash(state);
                }
            }
            /// Struct modelled after a request to `getGJLevels21.php`
            ///
            /// In the Geometry Dash API, this endpoint is used to retrieve a list of
            /// levels matching the specified criteria, along with their
            /// [`NewgroundsSong`s](::model::song::NewgroundsSong) and
            /// [`Creator`s](::model::user::Creator).
            pub struct LevelsRequest {
                /// The base request data
                pub base: BaseRequest,
                /// The type of level list to retrieve
                ///
                /// ## GD Internals:
                /// This field is called `type` in the boomlings API and needs to be
                /// converted to an integer
                pub request_type: LevelRequestType,
                /// A search string to filter the levels by
                ///
                /// This value is ignored unless [`LevelsRequest::request_type`] is set to
                /// [`LevelRequestType::Search`] or [`LevelRequestType::User`]
                ///
                /// ## GD Internals:
                /// This field is called `str` in the boomlings API
                pub search_string: String,
                /// A list of level lengths to filter by
                ///
                /// This value is ignored unless [`LevelsRequest::request_type`] is set to
                /// [`LevelRequestType::Search`]
                ///
                /// ## GD Internals:
                /// This field is called `len` in the boomlings API and needs to be
                /// converted to a comma separated list of integers, or a single dash
                /// (`-`) if filtering by level length isn't wanted.
                pub lengths: Vec<LevelLength>,
                /// A list of level ratings to filter by.
                ///
                /// To filter by any demon, add [`LevelRating::Demon`] with any arbitrary
                /// [`DemonRating`] value.
                ///
                /// `ratings` and [`LevelsRequest::demon_rating`] are mutually exlusive.
                ///
                /// This value is ignored unless [`LevelsRequest::request_type`] is set to
                /// [`LevelRequestType::Search`]
                ///
                /// ## GD Internals:
                /// This field is called `diff` in the boomlings API and needs to be
                /// converted to a comma separated list of integers, or a single dash
                /// (`-`) if filtering by level rating isn't wanted.
                pub ratings: Vec<LevelRating>,
                /// Optionally, a single demon rating to filter by. To filter by any demon
                /// rating, use [`LevelsRequest::ratings`]
                ///
                /// `demon_rating` and `ratings` are mutually exlusive.
                ///
                /// This value is ignored unless [`LevelsRequest::request_type`] is set to
                /// [`LevelRequestType::Search`]
                ///
                /// ## GD Internals:
                /// This field is called `demonFilter` in the boomlings API and needs to be
                /// converted to an integer. If filtering by demon rating isn't wanted,
                /// the value has to be omitted from the request.
                pub demon_rating: Option<DemonRating>,
                /// The page of results to retrieve
                pub page: u32,
                /// Some weird value the Geometry Dash client sends along
                pub total: i32,
                /// Search filters to apply.
                ///
                /// This value is ignored unless [`LevelsRequest::request_type`] is set to
                /// [`LevelRequestType::Search`]
                pub search_filters: SearchFilters,
            }
            #[automatically_derived]
            #[allow(unused_qualifications)]
            impl ::std::fmt::Debug for LevelsRequest {
                fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                    match *self {
                        LevelsRequest {
                            base: ref __self_0_0,
                            request_type: ref __self_0_1,
                            search_string: ref __self_0_2,
                            lengths: ref __self_0_3,
                            ratings: ref __self_0_4,
                            demon_rating: ref __self_0_5,
                            page: ref __self_0_6,
                            total: ref __self_0_7,
                            search_filters: ref __self_0_8,
                        } => {
                            let mut debug_trait_builder = f.debug_struct("LevelsRequest");
                            let _ = debug_trait_builder.field("base", &&(*__self_0_0));
                            let _ = debug_trait_builder.field("request_type", &&(*__self_0_1));
                            let _ = debug_trait_builder.field("search_string", &&(*__self_0_2));
                            let _ = debug_trait_builder.field("lengths", &&(*__self_0_3));
                            let _ = debug_trait_builder.field("ratings", &&(*__self_0_4));
                            let _ = debug_trait_builder.field("demon_rating", &&(*__self_0_5));
                            let _ = debug_trait_builder.field("page", &&(*__self_0_6));
                            let _ = debug_trait_builder.field("total", &&(*__self_0_7));
                            let _ = debug_trait_builder.field("search_filters", &&(*__self_0_8));
                            debug_trait_builder.finish()
                        },
                    }
                }
            }
            #[automatically_derived]
            #[allow(unused_qualifications)]
            impl ::std::default::Default for LevelsRequest {
                #[inline]
                fn default() -> LevelsRequest {
                    LevelsRequest {
                        base: ::std::default::Default::default(),
                        request_type: ::std::default::Default::default(),
                        search_string: ::std::default::Default::default(),
                        lengths: ::std::default::Default::default(),
                        ratings: ::std::default::Default::default(),
                        demon_rating: ::std::default::Default::default(),
                        page: ::std::default::Default::default(),
                        total: ::std::default::Default::default(),
                        search_filters: ::std::default::Default::default(),
                    }
                }
            }
            #[automatically_derived]
            #[allow(unused_qualifications)]
            impl ::std::clone::Clone for LevelsRequest {
                #[inline]
                fn clone(&self) -> LevelsRequest {
                    match *self {
                        LevelsRequest {
                            base: ref __self_0_0,
                            request_type: ref __self_0_1,
                            search_string: ref __self_0_2,
                            lengths: ref __self_0_3,
                            ratings: ref __self_0_4,
                            demon_rating: ref __self_0_5,
                            page: ref __self_0_6,
                            total: ref __self_0_7,
                            search_filters: ref __self_0_8,
                        } =>
                            LevelsRequest {
                                base: ::std::clone::Clone::clone(&(*__self_0_0)),
                                request_type: ::std::clone::Clone::clone(&(*__self_0_1)),
                                search_string: ::std::clone::Clone::clone(&(*__self_0_2)),
                                lengths: ::std::clone::Clone::clone(&(*__self_0_3)),
                                ratings: ::std::clone::Clone::clone(&(*__self_0_4)),
                                demon_rating: ::std::clone::Clone::clone(&(*__self_0_5)),
                                page: ::std::clone::Clone::clone(&(*__self_0_6)),
                                total: ::std::clone::Clone::clone(&(*__self_0_7)),
                                search_filters: ::std::clone::Clone::clone(&(*__self_0_8)),
                            },
                    }
                }
            }
            /// Manual Hash impl which doesn't hash the base
            impl Hash for LevelsRequest {
                fn hash<H: Hasher>(&self, state: &mut H) {
                    self.search_filters.hash(state);
                    self.total.hash(state);
                    self.demon_rating.hash(state);
                    self.ratings.hash(state);
                    self.lengths.hash(state);
                    self.search_string.hash(state);
                    self.request_type.hash(state);
                    self.page.hash(state);
                }
            }
            /// Enum representing the various filter states that can be achieved using the
            /// `completed` and `uncompleted` options in the Geometry Dash client
            pub enum CompletionFilter {
                /// No filtering based upon completion
                None,

                /// Filtering based upon a given list of level ids
                List {
                    /// The list of level ids to filter
                    ids: Vec<u64>,
                    /// if `true`, only the levels matching the ids in
                    /// [`ids`](CompletionFilter::List.ids) will be searched, if
                    /// `false`, the levels in [`ids`](CompletionFilter::List.ids) will
                    /// be excluded.
                    include: bool,
                },
            }
            #[automatically_derived]
            #[allow(unused_qualifications)]
            impl ::std::fmt::Debug for CompletionFilter {
                fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                    match (&*self,) {
                        (&CompletionFilter::None,) => {
                            let mut debug_trait_builder = f.debug_tuple("None");
                            debug_trait_builder.finish()
                        },
                        (&CompletionFilter::List {
                            ids: ref __self_0,
                            include: ref __self_1,
                        },) => {
                            let mut debug_trait_builder = f.debug_struct("List");
                            let _ = debug_trait_builder.field("ids", &&(*__self_0));
                            let _ = debug_trait_builder.field("include", &&(*__self_1));
                            debug_trait_builder.finish()
                        },
                    }
                }
            }
            #[automatically_derived]
            #[allow(unused_qualifications)]
            impl ::std::clone::Clone for CompletionFilter {
                #[inline]
                fn clone(&self) -> CompletionFilter {
                    match (&*self,) {
                        (&CompletionFilter::None,) => CompletionFilter::None,
                        (&CompletionFilter::List {
                            ids: ref __self_0,
                            include: ref __self_1,
                        },) =>
                            CompletionFilter::List {
                                ids: ::std::clone::Clone::clone(&(*__self_0)),
                                include: ::std::clone::Clone::clone(&(*__self_1)),
                            },
                    }
                }
            }
            #[automatically_derived]
            #[allow(unused_qualifications)]
            impl ::std::hash::Hash for CompletionFilter {
                fn hash<__H: ::std::hash::Hasher>(&self, state: &mut __H) -> () {
                    match (&*self,) {
                        (&CompletionFilter::List {
                            ids: ref __self_0,
                            include: ref __self_1,
                        },) => {
                            ::std::hash::Hash::hash(&unsafe { ::std::intrinsics::discriminant_value(self) }, state);
                            ::std::hash::Hash::hash(&(*__self_0), state);
                            ::std::hash::Hash::hash(&(*__self_1), state)
                        },
                        _ => ::std::hash::Hash::hash(&unsafe { ::std::intrinsics::discriminant_value(self) }, state),
                    }
                }
            }
            impl Default for CompletionFilter {
                fn default() -> Self {
                    CompletionFilter::None
                }
            }
            impl CompletionFilter {
                /// Constructs a [`CompletionFilter`] that'll restrict the search to the
                /// list of provided ids
                pub const fn completed(completed: Vec<u64>) -> CompletionFilter {
                    CompletionFilter::List {
                        ids: completed,
                        include: true,
                    }
                }

                /// Constructs a [`CompletionFilter`] that'll exclude the list of given ids
                /// from the search
                pub const fn uncompleted(completed: Vec<u64>) -> CompletionFilter {
                    CompletionFilter::List {
                        ids: completed,
                        include: false,
                    }
                }
            }
            /// Struct containing the various search filters provided by the Geometry Dash
            /// client.
            pub struct SearchFilters {
                /// In- or excluding levels that have already been beaten. Since the GDCF
                /// client doesn't really have a notion of "completing" a level, this
                /// can be used to restrict the result a subset of an arbitrary set of
                /// levels, or exclude
                /// an arbitrary set of
                /// levels the result.
                ///
                /// ## GD Internals:
                /// This field abstracts away the `uncompleted`, `onlyCompleted` and
                /// `completedLevels` fields.
                ///
                /// + `uncompleted` is to be set to `1` if we wish to exclude completed
                /// levels from the results (and to `0` otherwise).
                /// + `onlyCompleted` is to be set to `1` if we wish to only search through
                /// completed levels (and to `0` otherwise)
                /// + `completedLevels` is a list of levels ids that have been completed.
                /// If needs to be provided if, and only if, either `uncompleted` or
                /// `onlyCompleted` are set to `1`. The ids are
                /// comma seperated and enclosed by parenthesis.
                pub completion: CompletionFilter,
                /// Only retrieve featured levels
                ///
                /// ## GD Internals:
                /// This value needs to be converted to an integer for the boomlings API
                pub featured: bool,
                /// Only retrieve original (uncopied)  levels
                ///
                /// ## GD Internals:
                /// This value needs to be converted to an integer for the boomlings API
                pub original: bool,
                /// Only retrieve two-player levels
                ///
                /// ## GD Internals:
                /// This field is called `twoPlayer` in the boomlings API and needs to be
                /// converted to an integer
                pub two_player: bool,
                /// Only retrieve levels with coins
                ///
                /// ## GD Internals:
                /// This value needs to be converted to an integer for the boomlings API
                pub coins: bool,
                /// Only retrieve epic levels
                ///
                /// ## GD Internals:
                /// This value needs to be converted to an integer for the boomlings API
                pub epic: bool,
                /// Only retrieve star rated levels
                ///
                /// ## GD Internals:
                /// This field is called `star` in the boomlings API and needs to be
                /// converted to an integer
                pub rated: bool,
                /// Optionally only retrieve levels that match the given `SongFilter`
                ///
                /// ## GD Internals:
                /// This field composes both the `customSong` and `song` fields of the
                /// boomlings API. To filter by main song, set the `song` field to the
                /// id of the main song, and omit the `customSong` field from the
                /// request. To filter
                /// by a newgrounds
                /// song, set `customSong`
                /// to `1` and `song` to the newgrounds ID of the custom song.
                pub song: Option<SongFilter>,
            }
            #[automatically_derived]
            #[allow(unused_qualifications)]
            impl ::std::fmt::Debug for SearchFilters {
                fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                    match *self {
                        SearchFilters {
                            completion: ref __self_0_0,
                            featured: ref __self_0_1,
                            original: ref __self_0_2,
                            two_player: ref __self_0_3,
                            coins: ref __self_0_4,
                            epic: ref __self_0_5,
                            rated: ref __self_0_6,
                            song: ref __self_0_7,
                        } => {
                            let mut debug_trait_builder = f.debug_struct("SearchFilters");
                            let _ = debug_trait_builder.field("completion", &&(*__self_0_0));
                            let _ = debug_trait_builder.field("featured", &&(*__self_0_1));
                            let _ = debug_trait_builder.field("original", &&(*__self_0_2));
                            let _ = debug_trait_builder.field("two_player", &&(*__self_0_3));
                            let _ = debug_trait_builder.field("coins", &&(*__self_0_4));
                            let _ = debug_trait_builder.field("epic", &&(*__self_0_5));
                            let _ = debug_trait_builder.field("rated", &&(*__self_0_6));
                            let _ = debug_trait_builder.field("song", &&(*__self_0_7));
                            debug_trait_builder.finish()
                        },
                    }
                }
            }
            #[automatically_derived]
            #[allow(unused_qualifications)]
            impl ::std::default::Default for SearchFilters {
                #[inline]
                fn default() -> SearchFilters {
                    SearchFilters {
                        completion: ::std::default::Default::default(),
                        featured: ::std::default::Default::default(),
                        original: ::std::default::Default::default(),
                        two_player: ::std::default::Default::default(),
                        coins: ::std::default::Default::default(),
                        epic: ::std::default::Default::default(),
                        rated: ::std::default::Default::default(),
                        song: ::std::default::Default::default(),
                    }
                }
            }
            #[automatically_derived]
            #[allow(unused_qualifications)]
            impl ::std::clone::Clone for SearchFilters {
                #[inline]
                fn clone(&self) -> SearchFilters {
                    match *self {
                        SearchFilters {
                            completion: ref __self_0_0,
                            featured: ref __self_0_1,
                            original: ref __self_0_2,
                            two_player: ref __self_0_3,
                            coins: ref __self_0_4,
                            epic: ref __self_0_5,
                            rated: ref __self_0_6,
                            song: ref __self_0_7,
                        } =>
                            SearchFilters {
                                completion: ::std::clone::Clone::clone(&(*__self_0_0)),
                                featured: ::std::clone::Clone::clone(&(*__self_0_1)),
                                original: ::std::clone::Clone::clone(&(*__self_0_2)),
                                two_player: ::std::clone::Clone::clone(&(*__self_0_3)),
                                coins: ::std::clone::Clone::clone(&(*__self_0_4)),
                                epic: ::std::clone::Clone::clone(&(*__self_0_5)),
                                rated: ::std::clone::Clone::clone(&(*__self_0_6)),
                                song: ::std::clone::Clone::clone(&(*__self_0_7)),
                            },
                    }
                }
            }
            #[automatically_derived]
            #[allow(unused_qualifications)]
            impl ::std::hash::Hash for SearchFilters {
                fn hash<__H: ::std::hash::Hasher>(&self, state: &mut __H) -> () {
                    match *self {
                        SearchFilters {
                            completion: ref __self_0_0,
                            featured: ref __self_0_1,
                            original: ref __self_0_2,
                            two_player: ref __self_0_3,
                            coins: ref __self_0_4,
                            epic: ref __self_0_5,
                            rated: ref __self_0_6,
                            song: ref __self_0_7,
                        } => {
                            ::std::hash::Hash::hash(&(*__self_0_0), state);
                            ::std::hash::Hash::hash(&(*__self_0_1), state);
                            ::std::hash::Hash::hash(&(*__self_0_2), state);
                            ::std::hash::Hash::hash(&(*__self_0_3), state);
                            ::std::hash::Hash::hash(&(*__self_0_4), state);
                            ::std::hash::Hash::hash(&(*__self_0_5), state);
                            ::std::hash::Hash::hash(&(*__self_0_6), state);
                            ::std::hash::Hash::hash(&(*__self_0_7), state)
                        },
                    }
                }
            }
            /// Enum containing the various types of
            /// [`LevelsRequest`] possible
            ///
            /// ## GD Internals:
            /// + Unused values: `8`, `9`, `14`
            /// + The values `15` and `17` are only used in Geometry Dash World and are the
            /// same as `0` ([`LevelRequestType::Search`]) and `6` ([`LevelRequestType::Featured`])
            /// respectively
            #[rustc_copy_clone_marker]
            pub enum LevelRequestType {
                /// A search request.
                ///
                /// Setting this variant will enabled all the available search filters
                ///
                /// ## GD Internals:
                /// This variant is represented by the value `0` in requests
                Search,

                /// Request to retrieve the list of most downloaded levels
                ///
                /// ## GD Internals:
                /// This variant is represented by the value `1` in requests
                MostDownloaded,

                /// Request to retrieve the list of most liked levels
                ///
                /// ## GD Internals:
                /// This variant is represented by the value `2` in requests
                MostLiked,

                /// Request to retrieve the list of treI which I understood more aboutnding levels
                ///
                /// ## GD Internals:
                /// This variant is represented by the value `3` in requests
                Trending,

                /// Request to retrieve the list of most recent levels
                ///
                /// ## GD Internals:
                /// This variant is represented by the value `4` in requests
                Recent,

                /// Retrieve levels by the user whose ID was specified in
                /// [`LevelsRequest::search_string`] (Note that is has to be the
                /// user Id, not the account id)
                ///
                /// ## GD Internals:
                /// This variant is represented by the value `5` in requests
                User,

                /// Request to retrieve the list of featured levels, ordered by their
                /// [featured weight](::model::level::Featured::Featured) weight
                ///
                /// ## GD Internals:
                /// This variant is represented by the value `6` in requests
                Featured,

                /// Request to retrieve a list of levels filtered by some magic criteria
                ///
                /// ## GD Internals:
                /// This variant is represented by the value `7` in requests. According to the GDPS
                /// source, this simply looks for levels that have more than 9999
                /// objects.
                Magic,

                /// Map pack levels. The search string is set to a comma seperated list of
                /// levels, which are the levels contained in the map pack
                ///
                /// ## GD Internals:
                /// This variant is represented by the value `10` in requests
                MapPack,

                /// Request to retrieve the list of levels most recently awarded a rating.
                ///
                /// Using this option you can only receive levels that were awarded a rating in
                /// Geometry Dash 1.9 or later
                ///
                /// ## GD Internals:
                /// This variant is represented by the value `11` in requests
                Awarded,

                /// Unknown how this works
                ///
                /// ## GD Internals:
                /// This variant is represented by the value `12` in requests
                Followed,

                /// Unknown what this is
                ///
                /// ## GD Internals:
                /// This variant is represented by the value `13` in requests
                Friends,

                /// Request to retrieve the levels in the hall of fame
                ///
                /// ## GD Internals:
                /// This variant is represented by the value `16` in requests.
                HallOfFame,
            }
            #[automatically_derived]
            #[allow(unused_qualifications)]
            impl ::std::fmt::Debug for LevelRequestType {
                fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                    match (&*self,) {
                        (&LevelRequestType::Search,) => {
                            let mut debug_trait_builder = f.debug_tuple("Search");
                            debug_trait_builder.finish()
                        },
                        (&LevelRequestType::MostDownloaded,) => {
                            let mut debug_trait_builder = f.debug_tuple("MostDownloaded");
                            debug_trait_builder.finish()
                        },
                        (&LevelRequestType::MostLiked,) => {
                            let mut debug_trait_builder = f.debug_tuple("MostLiked");
                            debug_trait_builder.finish()
                        },
                        (&LevelRequestType::Trending,) => {
                            let mut debug_trait_builder = f.debug_tuple("Trending");
                            debug_trait_builder.finish()
                        },
                        (&LevelRequestType::Recent,) => {
                            let mut debug_trait_builder = f.debug_tuple("Recent");
                            debug_trait_builder.finish()
                        },
                        (&LevelRequestType::User,) => {
                            let mut debug_trait_builder = f.debug_tuple("User");
                            debug_trait_builder.finish()
                        },
                        (&LevelRequestType::Featured,) => {
                            let mut debug_trait_builder = f.debug_tuple("Featured");
                            debug_trait_builder.finish()
                        },
                        (&LevelRequestType::Magic,) => {
                            let mut debug_trait_builder = f.debug_tuple("Magic");
                            debug_trait_builder.finish()
                        },
                        (&LevelRequestType::MapPack,) => {
                            let mut debug_trait_builder = f.debug_tuple("MapPack");
                            debug_trait_builder.finish()
                        },
                        (&LevelRequestType::Awarded,) => {
                            let mut debug_trait_builder = f.debug_tuple("Awarded");
                            debug_trait_builder.finish()
                        },
                        (&LevelRequestType::Followed,) => {
                            let mut debug_trait_builder = f.debug_tuple("Followed");
                            debug_trait_builder.finish()
                        },
                        (&LevelRequestType::Friends,) => {
                            let mut debug_trait_builder = f.debug_tuple("Friends");
                            debug_trait_builder.finish()
                        },
                        (&LevelRequestType::HallOfFame,) => {
                            let mut debug_trait_builder = f.debug_tuple("HallOfFame");
                            debug_trait_builder.finish()
                        },
                    }
                }
            }
            #[automatically_derived]
            #[allow(unused_qualifications)]
            impl ::std::marker::Copy for LevelRequestType {}
            #[automatically_derived]
            #[allow(unused_qualifications)]
            impl ::std::clone::Clone for LevelRequestType {
                #[inline]
                fn clone(&self) -> LevelRequestType {
                    {
                        *self
                    }
                }
            }
            #[automatically_derived]
            #[allow(unused_qualifications)]
            impl ::std::cmp::PartialEq for LevelRequestType {
                #[inline]
                fn eq(&self, other: &LevelRequestType) -> bool {
                    {
                        let __self_vi = unsafe { ::std::intrinsics::discriminant_value(&*self) } as isize;
                        let __arg_1_vi = unsafe { ::std::intrinsics::discriminant_value(&*other) } as isize;
                        if true && __self_vi == __arg_1_vi {
                            match (&*self, &*other) {
                                _ => true,
                            }
                        } else {
                            false
                        }
                    }
                }
            }
            #[automatically_derived]
            #[allow(unused_qualifications)]
            impl ::std::hash::Hash for LevelRequestType {
                fn hash<__H: ::std::hash::Hasher>(&self, state: &mut __H) -> () {
                    match (&*self,) {
                        _ => ::std::hash::Hash::hash(&unsafe { ::std::intrinsics::discriminant_value(self) }, state),
                    }
                }
            }
            #[structural_match]
            #[rustc_copy_clone_marker]
            pub enum SongFilter {
                Main(u8),
                Custom(u64),
            }
            #[automatically_derived]
            #[allow(unused_qualifications)]
            impl ::std::fmt::Debug for SongFilter {
                fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                    match (&*self,) {
                        (&SongFilter::Main(ref __self_0),) => {
                            let mut debug_trait_builder = f.debug_tuple("Main");
                            let _ = debug_trait_builder.field(&&(*__self_0));
                            debug_trait_builder.finish()
                        },
                        (&SongFilter::Custom(ref __self_0),) => {
                            let mut debug_trait_builder = f.debug_tuple("Custom");
                            let _ = debug_trait_builder.field(&&(*__self_0));
                            debug_trait_builder.finish()
                        },
                    }
                }
            }
            #[automatically_derived]
            #[allow(unused_qualifications)]
            impl ::std::marker::Copy for SongFilter {}
            #[automatically_derived]
            #[allow(unused_qualifications)]
            impl ::std::clone::Clone for SongFilter {
                #[inline]
                fn clone(&self) -> SongFilter {
                    {
                        let _: ::std::clone::AssertParamIsClone<u8>;
                        let _: ::std::clone::AssertParamIsClone<u64>;
                        *self
                    }
                }
            }
            #[automatically_derived]
            #[allow(unused_qualifications)]
            impl ::std::hash::Hash for SongFilter {
                fn hash<__H: ::std::hash::Hasher>(&self, state: &mut __H) -> () {
                    match (&*self,) {
                        (&SongFilter::Main(ref __self_0),) => {
                            ::std::hash::Hash::hash(&unsafe { ::std::intrinsics::discriminant_value(self) }, state);
                            ::std::hash::Hash::hash(&(*__self_0), state)
                        },
                        (&SongFilter::Custom(ref __self_0),) => {
                            ::std::hash::Hash::hash(&unsafe { ::std::intrinsics::discriminant_value(self) }, state);
                            ::std::hash::Hash::hash(&(*__self_0), state)
                        },
                    }
                }
            }
            #[automatically_derived]
            #[allow(unused_qualifications)]
            impl ::std::cmp::PartialEq for SongFilter {
                #[inline]
                fn eq(&self, other: &SongFilter) -> bool {
                    {
                        let __self_vi = unsafe { ::std::intrinsics::discriminant_value(&*self) } as isize;
                        let __arg_1_vi = unsafe { ::std::intrinsics::discriminant_value(&*other) } as isize;
                        if true && __self_vi == __arg_1_vi {
                            match (&*self, &*other) {
                                (&SongFilter::Main(ref __self_0), &SongFilter::Main(ref __arg_1_0)) => (*__self_0) == (*__arg_1_0),
                                (&SongFilter::Custom(ref __self_0), &SongFilter::Custom(ref __arg_1_0)) => (*__self_0) == (*__arg_1_0),
                                _ => unsafe { ::std::intrinsics::unreachable() },
                            }
                        } else {
                            false
                        }
                    }
                }

                #[inline]
                fn ne(&self, other: &SongFilter) -> bool {
                    {
                        let __self_vi = unsafe { ::std::intrinsics::discriminant_value(&*self) } as isize;
                        let __arg_1_vi = unsafe { ::std::intrinsics::discriminant_value(&*other) } as isize;
                        if true && __self_vi == __arg_1_vi {
                            match (&*self, &*other) {
                                (&SongFilter::Main(ref __self_0), &SongFilter::Main(ref __arg_1_0)) => (*__self_0) != (*__arg_1_0),
                                (&SongFilter::Custom(ref __self_0), &SongFilter::Custom(ref __arg_1_0)) => (*__self_0) != (*__arg_1_0),
                                _ => unsafe { ::std::intrinsics::unreachable() },
                            }
                        } else {
                            true
                        }
                    }
                }
            }
            #[automatically_derived]
            #[allow(unused_qualifications)]
            impl ::std::cmp::Eq for SongFilter {
                #[inline]
                #[doc(hidden)]
                fn assert_receiver_is_total_eq(&self) -> () {
                    {
                        let _: ::std::cmp::AssertParamIsEq<u8>;
                        let _: ::std::cmp::AssertParamIsEq<u64>;
                    }
                }
            }
            impl SearchFilters {
                pub const fn new() -> SearchFilters {
                    SearchFilters {
                        completion: CompletionFilter::None,
                        featured: false,
                        original: false,
                        two_player: false,
                        coins: false,
                        epic: false,
                        rated: false,
                        song: None,
                    }
                }

                pub const fn rated(mut self) -> SearchFilters {
                    self.rated = true;
                    self
                }

                pub fn only_search(mut self, ids: Vec<u64>) -> SearchFilters {
                    self.completion = CompletionFilter::List { ids, include: true };
                    self
                }

                pub fn exclude(mut self, ids: Vec<u64>) -> SearchFilters {
                    self.completion = CompletionFilter::List { ids, include: false };
                    self
                }

                pub const fn featured(mut self) -> SearchFilters {
                    self.featured = true;
                    self
                }

                pub const fn original(mut self) -> SearchFilters {
                    self.original = true;
                    self
                }

                pub const fn two_player(mut self) -> SearchFilters {
                    self.two_player = true;
                    self
                }

                pub const fn coins(mut self) -> SearchFilters {
                    self.coins = true;
                    self
                }

                pub const fn epic(mut self) -> SearchFilters {
                    self.epic = true;
                    self
                }

                pub const fn main_song(mut self, id: u8) -> SearchFilters {
                    self.song = Some(SongFilter::Main(id));
                    self
                }

                pub const fn custom_song(mut self, id: u64) -> SearchFilters {
                    self.song = Some(SongFilter::Custom(id));
                    self
                }
            }
            impl LevelRequest {
                #[doc = r" Sets the [`BaseRequest`] to be used"]
                #[doc = r""]
                #[doc = r" Allows builder-style creation of requests"]
                pub const fn with_base(mut self, base: BaseRequest) -> Self {
                    self.base = base;
                    self
                }

                #[doc = r" Sets the value of the `inc` field"]
                #[doc = r""]
                #[doc = r" Allows builder-style creation of requests"]
                pub const fn inc(mut self, inc: bool) -> Self {
                    self.inc = inc;
                    self
                }

                #[doc = r" Sets the value of the `extra` field"]
                #[doc = r""]
                #[doc = r" Allows builder-style creation of requests"]
                pub const fn extra(mut self, extra: bool) -> Self {
                    self.extra = extra;
                    self
                }

                /// Constructs a new `LevelRequest` to retrieve the level with the given id
                ///
                /// Uses a default [`BaseRequest`], and sets the
                /// `inc` field to `true` and `extra` to `false`, as are the default
                /// values set the by the Geometry Dash Client
                pub const fn new(level_id: u64) -> LevelRequest {
                    LevelRequest {
                        base: GD_21,
                        level_id,
                        inc: true,
                        extra: false,
                    }
                }
            }
            impl LevelsRequest {
                pub const fn with_base(mut self, base: BaseRequest) -> Self {
                    self.base = base;
                    self
                }

                pub fn filter(mut self, search_filters: SearchFilters) -> Self {
                    self.search_filters = search_filters;
                    self
                }

                pub const fn page(mut self, arg0: u32) -> Self {
                    self.page = arg0;
                    self
                }

                pub const fn total(mut self, arg0: i32) -> Self {
                    self.total = arg0;
                    self
                }

                pub const fn request_type(mut self, arg0: LevelRequestType) -> Self {
                    self.request_type = arg0;
                    self
                }

                pub fn search(mut self, search_string: String) -> Self {
                    self.search_string = search_string;
                    self.request_type = LevelRequestType::Search;
                    self
                }

                pub fn with_id(self, id: u64) -> Self {
                    self.search(id.to_string())
                }

                pub fn with_length(mut self, length: LevelLength) -> Self {
                    self.lengths.push(length);
                    self
                }

                pub fn with_rating(mut self, rating: LevelRating) -> Self {
                    self.ratings.push(rating);
                    self
                }

                pub const fn demon(mut self, demon_rating: DemonRating) -> Self {
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
                        LevelRequestType::MapPack => 10,
                        LevelRequestType::Awarded => 11,
                        LevelRequestType::Followed => 12,
                        LevelRequestType::Friends => 13,
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
                type Result = Level<u64, u64>;
            }
            impl Request for LevelsRequest {
                type Result = Vec<PartialLevel<u64, u64>>;
            }
            impl PaginatableRequest for LevelsRequest {
                fn next(&self) -> Self {
                    self.clone().page(self.page + 1)
                }
            }
            impl Display for LevelRequest {
                fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
                    f.write_fmt(::std::fmt::Arguments::new_v1(
                        &["LevelRequest(", ")"],
                        &match (&self.level_id,) {
                            (arg0,) => [::std::fmt::ArgumentV1::new(arg0, ::std::fmt::Display::fmt)],
                        },
                    ))
                }
            }
            impl Display for LevelsRequest {
                fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
                    match self.request_type {
                        LevelRequestType::Search =>
                            f.write_fmt(::std::fmt::Arguments::new_v1(
                                &["LevelsRequest(Search=", ", page=", ")"],
                                &match (&self.search_string, &self.page) {
                                    (arg0, arg1) =>
                                        [
                                            ::std::fmt::ArgumentV1::new(arg0, ::std::fmt::Display::fmt),
                                            ::std::fmt::ArgumentV1::new(arg1, ::std::fmt::Display::fmt),
                                        ],
                                },
                            )),
                        _ =>
                            f.write_fmt(::std::fmt::Arguments::new_v1(
                                &["LevelsRequest(", ", page=", ")"],
                                &match (&self.request_type, &self.page) {
                                    (arg0, arg1) =>
                                        [
                                            ::std::fmt::ArgumentV1::new(arg0, ::std::fmt::Debug::fmt),
                                            ::std::fmt::ArgumentV1::new(arg1, ::std::fmt::Display::fmt),
                                        ],
                                },
                            )),
                    }
                }
            }
        }
        pub mod user {
            //! Module ontianing request definitions for retrieving users
            use api::request::{BaseRequest, Request, GD_21};
            use gdcf_model::user::User;
            use std::{
                fmt::{Display, Error, Formatter},
                hash::{Hash, Hasher},
            };
            /// Struct modelled after a request to `getGJUserInfo20.php`.
            ///
            /// In the geometry Dash API, this endpoint is used to download player profiles from the
            /// servers by their account IDs
            #[rustc_copy_clone_marker]
            pub struct UserRequest {
                /// The base request data
                pub base: BaseRequest,
                /// The **account ID** (_not_ user ID) of the users whose data to retrieve.
                ///
                /// ## GD Internals:
                /// This field is called `targetAccountID` in the boomlings API
                pub user: u64,
            }
            #[automatically_derived]
            #[allow(unused_qualifications)]
            impl ::std::fmt::Debug for UserRequest {
                fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                    match *self {
                        UserRequest {
                            base: ref __self_0_0,
                            user: ref __self_0_1,
                        } => {
                            let mut debug_trait_builder = f.debug_struct("UserRequest");
                            let _ = debug_trait_builder.field("base", &&(*__self_0_0));
                            let _ = debug_trait_builder.field("user", &&(*__self_0_1));
                            debug_trait_builder.finish()
                        },
                    }
                }
            }
            #[automatically_derived]
            #[allow(unused_qualifications)]
            impl ::std::default::Default for UserRequest {
                #[inline]
                fn default() -> UserRequest {
                    UserRequest {
                        base: ::std::default::Default::default(),
                        user: ::std::default::Default::default(),
                    }
                }
            }
            #[automatically_derived]
            #[allow(unused_qualifications)]
            impl ::std::clone::Clone for UserRequest {
                #[inline]
                fn clone(&self) -> UserRequest {
                    {
                        let _: ::std::clone::AssertParamIsClone<BaseRequest>;
                        let _: ::std::clone::AssertParamIsClone<u64>;
                        *self
                    }
                }
            }
            #[automatically_derived]
            #[allow(unused_qualifications)]
            impl ::std::marker::Copy for UserRequest {}
            impl UserRequest {
                pub const fn with_base(mut self, base: BaseRequest) -> Self {
                    self.base = base;
                    self
                }

                pub const fn new(user_id: u64) -> UserRequest {
                    UserRequest {
                        base: GD_21,
                        user: user_id,
                    }
                }
            }
            impl Hash for UserRequest {
                fn hash<H: Hasher>(&self, state: &mut H) {
                    self.user.hash(state)
                }
            }
            impl Into<UserRequest> for u64 {
                fn into(self) -> UserRequest {
                    UserRequest::new(self)
                }
            }
            impl Display for UserRequest {
                fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
                    f.write_fmt(::std::fmt::Arguments::new_v1(
                        &["UserRequest(", ")"],
                        &match (&self.user,) {
                            (arg0,) => [::std::fmt::ArgumentV1::new(arg0, ::std::fmt::Display::fmt)],
                        },
                    ))
                }
            }
            impl Request for UserRequest {
                type Result = User;
            }
        }
        /// A `BaseRequest` instance that has all its fields set to the
        /// same values a Geometry Dash 2.1 client would use
        pub const GD_21: BaseRequest = BaseRequest::new(
            GameVersion::Version { major: 2, minor: 1 },
            GameVersion::Version { major: 3, minor: 3 },
            "Wmfd2893gb7",
        );
        /// Base data included in every request made
        ///
        /// The fields in this struct are only relevant when making a request to the
        /// `boomlings` servers. When using GDCF with a custom Geometry Dash API, they
        /// can safely be ignored.
        #[structural_match]
        #[rustc_copy_clone_marker]
        pub struct BaseRequest {
            /// The version of the game client we're pretending to be
            ///
            /// ## GD Internals:
            /// This field is called `gameVersion` in the boomlings API and needs to be
            /// converted to a string response
            /// The value of this field doesn't matter, and the request will succeed
            /// regardless of what it's been set to
            pub game_version: GameVersion,
            /// Internal version of the game client we're pretending to be
            ///
            /// ## GD Internals:
            /// This field is called `binaryVersion` in the boomlings API and needs to
            /// be converted to a string
            ///
            /// The value of this field doesn't matter, and the request will succeed
            /// regardless of what it's been set to
            pub binary_version: GameVersion,
            /// The current secret String the server uses to identify valid clients.
            ///
            /// ## GD Internals:
            /// Settings this field to an incorrect value will cause the request to fail
            pub secret: &'static str,
        }
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::std::fmt::Debug for BaseRequest {
            fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                match *self {
                    BaseRequest {
                        game_version: ref __self_0_0,
                        binary_version: ref __self_0_1,
                        secret: ref __self_0_2,
                    } => {
                        let mut debug_trait_builder = f.debug_struct("BaseRequest");
                        let _ = debug_trait_builder.field("game_version", &&(*__self_0_0));
                        let _ = debug_trait_builder.field("binary_version", &&(*__self_0_1));
                        let _ = debug_trait_builder.field("secret", &&(*__self_0_2));
                        debug_trait_builder.finish()
                    },
                }
            }
        }
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::std::clone::Clone for BaseRequest {
            #[inline]
            fn clone(&self) -> BaseRequest {
                {
                    let _: ::std::clone::AssertParamIsClone<GameVersion>;
                    let _: ::std::clone::AssertParamIsClone<GameVersion>;
                    let _: ::std::clone::AssertParamIsClone<&'static str>;
                    *self
                }
            }
        }
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::std::hash::Hash for BaseRequest {
            fn hash<__H: ::std::hash::Hasher>(&self, state: &mut __H) -> () {
                match *self {
                    BaseRequest {
                        game_version: ref __self_0_0,
                        binary_version: ref __self_0_1,
                        secret: ref __self_0_2,
                    } => {
                        ::std::hash::Hash::hash(&(*__self_0_0), state);
                        ::std::hash::Hash::hash(&(*__self_0_1), state);
                        ::std::hash::Hash::hash(&(*__self_0_2), state)
                    },
                }
            }
        }
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::std::marker::Copy for BaseRequest {}
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::std::cmp::PartialEq for BaseRequest {
            #[inline]
            fn eq(&self, other: &BaseRequest) -> bool {
                match *other {
                    BaseRequest {
                        game_version: ref __self_1_0,
                        binary_version: ref __self_1_1,
                        secret: ref __self_1_2,
                    } =>
                        match *self {
                            BaseRequest {
                                game_version: ref __self_0_0,
                                binary_version: ref __self_0_1,
                                secret: ref __self_0_2,
                            } => (*__self_0_0) == (*__self_1_0) && (*__self_0_1) == (*__self_1_1) && (*__self_0_2) == (*__self_1_2),
                        },
                }
            }

            #[inline]
            fn ne(&self, other: &BaseRequest) -> bool {
                match *other {
                    BaseRequest {
                        game_version: ref __self_1_0,
                        binary_version: ref __self_1_1,
                        secret: ref __self_1_2,
                    } =>
                        match *self {
                            BaseRequest {
                                game_version: ref __self_0_0,
                                binary_version: ref __self_0_1,
                                secret: ref __self_0_2,
                            } => (*__self_0_0) != (*__self_1_0) || (*__self_0_1) != (*__self_1_1) || (*__self_0_2) != (*__self_1_2),
                        },
                }
            }
        }
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::std::cmp::Eq for BaseRequest {
            #[inline]
            #[doc(hidden)]
            fn assert_receiver_is_total_eq(&self) -> () {
                {
                    let _: ::std::cmp::AssertParamIsEq<GameVersion>;
                    let _: ::std::cmp::AssertParamIsEq<GameVersion>;
                    let _: ::std::cmp::AssertParamIsEq<&'static str>;
                }
            }
        }
        impl BaseRequest {
            /// Constructs a new `BaseRequest` with the given values.
            pub const fn new(game_version: GameVersion, binary_version: GameVersion, secret: &'static str) -> BaseRequest {
                BaseRequest {
                    game_version,
                    binary_version,
                    secret,
                }
            }
        }
        impl Default for BaseRequest {
            fn default() -> Self {
                GD_21
            }
        }
        /// Trait for types that are meant to be requests whose results can be cached
        /// by GDCF
        ///
        /// A `Request`'s `Hash` result must be forward-compatible with new fields
        /// added to a request. This means that if the GD API adds a new fields to a
        /// requests, making a request without this fields should generate the same
        /// hash value as the same request in
        /// an old version of GDCF without the field in the first place.
        /// This means foremost, that `Hash` impls mustn't hash the `BaseRequest`
        /// they're built upon. If new fields are added in later version of GDCF, they
        /// may only be hashed if they are explicitly set to a value, to ensure the
        /// above-mentioned compatibility
        pub trait Request: Display + Default + Hash + Clone {
            type Result;
        }
        pub trait PaginatableRequest: Request {
            fn next(&self) -> Self;
        }
    }
    pub use self::client::ApiClient;
}
pub mod cache {
    use crate::{
        api::request::{user::UserRequest, LevelRequest, LevelsRequest},
        error::CacheError,
        Secondary,
    };
    use chrono::{DateTime, Duration, NaiveDateTime, Utc};
    use gdcf_model::{
        level::{Level, PartialLevel},
        song::NewgroundsSong,
        user::{Creator, User},
    };
    pub type Lookup<T, E> = Result<CachedObject<T>, E>;
    pub trait CacheConfig {
        fn invalidate_after(&self) -> Duration;
    }
    pub trait Cache: Clone + Send + Sync + 'static {
        type Config: CacheConfig;
        type Err: CacheError;
        fn config(&self) -> &Self::Config;
        fn lookup_partial_levels(&self, req: &LevelsRequest) -> Lookup<Vec<PartialLevel<u64, u64>>, Self::Err>;
        fn store_partial_levels(&mut self, req: &LevelsRequest, levels: &[PartialLevel<u64, u64>]) -> Result<(), Self::Err>;
        fn lookup_level(&self, req: &LevelRequest) -> Lookup<Level<u64, u64>, Self::Err>;
        fn lookup_user(&self, req: &UserRequest) -> Lookup<User, Self::Err>;
        fn lookup_song(&self, newground_id: u64) -> Lookup<NewgroundsSong, Self::Err>;
        fn lookup_creator(&self, user_id: u64) -> Lookup<Creator, Self::Err>;
        /// Stores an arbitrary [`Secondary`] in this [`Cache`]
        fn store_object(&mut self, obj: &Secondary) -> Result<(), Self::Err>;
        fn is_expired<T>(&self, obj: &CachedObject<T>) -> bool {
            let now = Utc::now();
            let then = DateTime::<Utc>::from_utc(obj.last_cached_at(), Utc);
            now - then > self.config().invalidate_after()
        }
    }
    pub struct CachedObject<T> {
        first_cached_at: NaiveDateTime,
        last_cached_at: NaiveDateTime,
        obj: T,
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl<T: ::std::fmt::Debug> ::std::fmt::Debug for CachedObject<T> {
        fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
            match *self {
                CachedObject {
                    first_cached_at: ref __self_0_0,
                    last_cached_at: ref __self_0_1,
                    obj: ref __self_0_2,
                } => {
                    let mut debug_trait_builder = f.debug_struct("CachedObject");
                    let _ = debug_trait_builder.field("first_cached_at", &&(*__self_0_0));
                    let _ = debug_trait_builder.field("last_cached_at", &&(*__self_0_1));
                    let _ = debug_trait_builder.field("obj", &&(*__self_0_2));
                    debug_trait_builder.finish()
                },
            }
        }
    }
    impl<T> CachedObject<T> {
        pub fn new(obj: T, first: NaiveDateTime, last: NaiveDateTime) -> Self {
            CachedObject {
                first_cached_at: first,
                last_cached_at: last,
                obj,
            }
        }

        pub fn last_cached_at(&self) -> NaiveDateTime {
            self.last_cached_at
        }

        pub fn first_cached_at(&self) -> NaiveDateTime {
            self.first_cached_at
        }

        pub fn extract(self) -> T {
            self.obj
        }

        pub fn inner(&self) -> &T {
            &self.obj
        }

        pub(crate) fn map<R, F>(self, f: F) -> CachedObject<R>
        where
            F: FnOnce(T) -> R,
        {
            let CachedObject {
                first_cached_at,
                last_cached_at,
                obj,
            } = self;
            CachedObject {
                first_cached_at,
                last_cached_at,
                obj: f(obj),
            }
        }

        pub(crate) fn try_map<R, F, E>(self, f: F) -> Result<CachedObject<R>, E>
        where
            F: FnOnce(T) -> Result<R, E>,
        {
            let CachedObject {
                first_cached_at,
                last_cached_at,
                obj,
            } = self;
            Ok(CachedObject {
                first_cached_at,
                last_cached_at,
                obj: f(obj)?,
            })
        }
    }
}
pub mod error {
    //! Module containing the various error types used by gdcf
    use failure::Fail;
    pub trait ApiError: Fail {
        fn is_no_result(&self) -> bool;
    }
    pub trait CacheError: Fail {
        fn is_cache_miss(&self) -> bool;
    }
    pub enum GdcfError<A: ApiError, C: CacheError> {
        #[fail(display = "{}", _0)]
        Cache(#[cause] C),

        #[fail(display = "{}", _0)]
        Api(#[cause] A),

        #[fail(display = "Neither cache-lookup, nor API response yielded any result")]
        NoContent,
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl<A: ::std::fmt::Debug + ApiError, C: ::std::fmt::Debug + CacheError> ::std::fmt::Debug for GdcfError<A, C> {
        fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
            match (&*self,) {
                (&GdcfError::Cache(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("Cache");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                },
                (&GdcfError::Api(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("Api");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                },
                (&GdcfError::NoContent,) => {
                    let mut debug_trait_builder = f.debug_tuple("NoContent");
                    debug_trait_builder.finish()
                },
            }
        }
    }
    #[allow(non_upper_case_globals)]
    const _DERIVE_failure_Fail_FOR_GdcfError: () = {
        impl<A: ApiError, C: CacheError> ::failure::Fail for GdcfError<A, C> {
            fn name(&self) -> Option<&str> {
                Some("gdcf::error::GdcfError")
            }

            #[allow(unreachable_code)]
            fn cause(&self) -> ::failure::_core::option::Option<&dyn::failure::Fail> {
                match *self {
                    GdcfError::Cache(ref __binding_0) => return Some(::failure::AsFail::as_fail(__binding_0)),
                    GdcfError::Api(ref __binding_0) => return Some(::failure::AsFail::as_fail(__binding_0)),
                    GdcfError::NoContent => return None,
                }
                None
            }

            #[allow(unreachable_code)]
            fn backtrace(&self) -> ::failure::_core::option::Option<&::failure::Backtrace> {
                match *self {
                    GdcfError::Cache(ref __binding_0) => return None,
                    GdcfError::Api(ref __binding_0) => return None,
                    GdcfError::NoContent => return None,
                }
                None
            }
        }
    };
    #[allow(non_upper_case_globals)]
    const _DERIVE_failure_core_fmt_Display_FOR_GdcfError: () = {
        impl<A: ApiError, C: CacheError> ::failure::_core::fmt::Display for GdcfError<A, C> {
            #[allow(unreachable_code)]
            fn fmt(&self, f: &mut ::failure::_core::fmt::Formatter) -> ::failure::_core::fmt::Result {
                match *self {
                    GdcfError::Cache(ref __binding_0) =>
                        return f.write_fmt(::std::fmt::Arguments::new_v1(
                            &[""],
                            &match (&__binding_0,) {
                                (arg0,) => [::std::fmt::ArgumentV1::new(arg0, ::std::fmt::Display::fmt)],
                            },
                        )),
                    GdcfError::Api(ref __binding_0) =>
                        return f.write_fmt(::std::fmt::Arguments::new_v1(
                            &[""],
                            &match (&__binding_0,) {
                                (arg0,) => [::std::fmt::ArgumentV1::new(arg0, ::std::fmt::Display::fmt)],
                            },
                        )),
                    GdcfError::NoContent =>
                        return f.write_fmt(::std::fmt::Arguments::new_v1(
                            &["Neither cache-lookup, nor API response yielded any result"],
                            &match () {
                                () => [],
                            },
                        )),
                }
                f.write_fmt(::std::fmt::Arguments::new_v1(
                    &["An error has occurred."],
                    &match () {
                        () => [],
                    },
                ))
            }
        }
    };
}
mod exchange {
    use gdcf_model::{
        level::{Level, PartialLevel},
        song::NewgroundsSong,
    };
    pub(crate) fn partial_level_song<User: PartialEq>(
        PartialLevel {
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
            copy_of,
            coin_amount,
            coins_verified,
            stars_requested,
            is_epic,
            index_43,
            object_amount,
            index_46,
            index_47,
            ..
        }: PartialLevel<u64, User>,
        custom_song: Option<NewgroundsSong>,
    ) -> PartialLevel<NewgroundsSong, User> {
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
            copy_of,
            coin_amount,
            coins_verified,
            stars_requested,
            is_epic,
            index_43,
            object_amount,
            index_46,
            index_47,
        }
    }
    pub(crate) fn partial_level_user<Song: PartialEq, User: PartialEq>(
        PartialLevel {
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
            copy_of,
            coin_amount,
            coins_verified,
            stars_requested,
            is_epic,
            index_43,
            object_amount,
            index_46,
            index_47,
            ..
        }: PartialLevel<Song, u64>,
        creator: User,
    ) -> PartialLevel<Song, User> {
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
            copy_of,
            coin_amount,
            coins_verified,
            stars_requested,
            is_epic,
            index_43,
            object_amount,
            index_46,
            index_47,
        }
    }
    pub(crate) fn level_song<User: PartialEq>(
        Level {
            base,
            level_data,
            password,
            time_since_update,
            time_since_upload,
            index_36,
        }: Level<u64, User>,
        song: Option<NewgroundsSong>,
    ) -> Level<NewgroundsSong, User> {
        Level {
            base: partial_level_song(base, song),
            level_data,
            password,
            time_since_update,
            time_since_upload,
            index_36,
        }
    }
    pub(crate) fn level_user<User: PartialEq, Song: PartialEq>(
        Level {
            base,
            level_data,
            password,
            time_since_update,
            time_since_upload,
            index_36,
        }: Level<Song, u64>,
        user: User,
    ) -> Level<Song, User> {
        Level {
            base: partial_level_user(base, user),
            level_data,
            password,
            time_since_upload,
            time_since_update,
            index_36,
        }
    }
}
pub enum Secondary {
    NewgroundsSong(NewgroundsSong),
    PartialLevel(PartialLevel<u64, u64>),
    Level(Level<u64, u64>),
    Creator(Creator),
    User(User),
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl ::std::fmt::Debug for Secondary {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match (&*self,) {
            (&Secondary::NewgroundsSong(ref __self_0),) => {
                let mut debug_trait_builder = f.debug_tuple("NewgroundsSong");
                let _ = debug_trait_builder.field(&&(*__self_0));
                debug_trait_builder.finish()
            },
            (&Secondary::PartialLevel(ref __self_0),) => {
                let mut debug_trait_builder = f.debug_tuple("PartialLevel");
                let _ = debug_trait_builder.field(&&(*__self_0));
                debug_trait_builder.finish()
            },
            (&Secondary::Level(ref __self_0),) => {
                let mut debug_trait_builder = f.debug_tuple("Level");
                let _ = debug_trait_builder.field(&&(*__self_0));
                debug_trait_builder.finish()
            },
            (&Secondary::Creator(ref __self_0),) => {
                let mut debug_trait_builder = f.debug_tuple("Creator");
                let _ = debug_trait_builder.field(&&(*__self_0));
                debug_trait_builder.finish()
            },
            (&Secondary::User(ref __self_0),) => {
                let mut debug_trait_builder = f.debug_tuple("User");
                let _ = debug_trait_builder.field(&&(*__self_0));
                debug_trait_builder.finish()
            },
        }
    }
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl ::std::clone::Clone for Secondary {
    #[inline]
    fn clone(&self) -> Secondary {
        match (&*self,) {
            (&Secondary::NewgroundsSong(ref __self_0),) => Secondary::NewgroundsSong(::std::clone::Clone::clone(&(*__self_0))),
            (&Secondary::PartialLevel(ref __self_0),) => Secondary::PartialLevel(::std::clone::Clone::clone(&(*__self_0))),
            (&Secondary::Level(ref __self_0),) => Secondary::Level(::std::clone::Clone::clone(&(*__self_0))),
            (&Secondary::Creator(ref __self_0),) => Secondary::Creator(::std::clone::Clone::clone(&(*__self_0))),
            (&Secondary::User(ref __self_0),) => Secondary::User(::std::clone::Clone::clone(&(*__self_0))),
        }
    }
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl ::std::cmp::PartialEq for Secondary {
    #[inline]
    fn eq(&self, other: &Secondary) -> bool {
        {
            let __self_vi = unsafe { ::std::intrinsics::discriminant_value(&*self) } as isize;
            let __arg_1_vi = unsafe { ::std::intrinsics::discriminant_value(&*other) } as isize;
            if true && __self_vi == __arg_1_vi {
                match (&*self, &*other) {
                    (&Secondary::NewgroundsSong(ref __self_0), &Secondary::NewgroundsSong(ref __arg_1_0)) => (*__self_0) == (*__arg_1_0),
                    (&Secondary::PartialLevel(ref __self_0), &Secondary::PartialLevel(ref __arg_1_0)) => (*__self_0) == (*__arg_1_0),
                    (&Secondary::Level(ref __self_0), &Secondary::Level(ref __arg_1_0)) => (*__self_0) == (*__arg_1_0),
                    (&Secondary::Creator(ref __self_0), &Secondary::Creator(ref __arg_1_0)) => (*__self_0) == (*__arg_1_0),
                    (&Secondary::User(ref __self_0), &Secondary::User(ref __arg_1_0)) => (*__self_0) == (*__arg_1_0),
                    _ => unsafe { ::std::intrinsics::unreachable() },
                }
            } else {
                false
            }
        }
    }

    #[inline]
    fn ne(&self, other: &Secondary) -> bool {
        {
            let __self_vi = unsafe { ::std::intrinsics::discriminant_value(&*self) } as isize;
            let __arg_1_vi = unsafe { ::std::intrinsics::discriminant_value(&*other) } as isize;
            if true && __self_vi == __arg_1_vi {
                match (&*self, &*other) {
                    (&Secondary::NewgroundsSong(ref __self_0), &Secondary::NewgroundsSong(ref __arg_1_0)) => (*__self_0) != (*__arg_1_0),
                    (&Secondary::PartialLevel(ref __self_0), &Secondary::PartialLevel(ref __arg_1_0)) => (*__self_0) != (*__arg_1_0),
                    (&Secondary::Level(ref __self_0), &Secondary::Level(ref __arg_1_0)) => (*__self_0) != (*__arg_1_0),
                    (&Secondary::Creator(ref __self_0), &Secondary::Creator(ref __arg_1_0)) => (*__self_0) != (*__arg_1_0),
                    (&Secondary::User(ref __self_0), &Secondary::User(ref __arg_1_0)) => (*__self_0) != (*__arg_1_0),
                    _ => unsafe { ::std::intrinsics::unreachable() },
                }
            } else {
                true
            }
        }
    }
}
impl From<NewgroundsSong> for Secondary {
    fn from(song: NewgroundsSong) -> Self {
        Secondary::NewgroundsSong(song)
    }
}
impl From<Creator> for Secondary {
    fn from(creator: Creator) -> Self {
        Secondary::Creator(creator)
    }
}
impl From<PartialLevel<u64, u64>> for Secondary {
    fn from(level: PartialLevel<u64, u64>) -> Self {
        Secondary::PartialLevel(level)
    }
}
impl From<Level<u64, u64>> for Secondary {
    fn from(level: Level<u64, u64>) -> Self {
        Secondary::Level(level)
    }
}
impl From<User> for Secondary {
    fn from(user: User) -> Self {
        Secondary::User(user)
    }
}
impl std::fmt::Display for Secondary {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Secondary::NewgroundsSong(inner) => inner.fmt(f),
            Secondary::PartialLevel(inner) => inner.fmt(f),
            Secondary::Level(inner) => inner.fmt(f),
            Secondary::Creator(inner) => inner.fmt(f),
            Secondary::User(inner) => inner.fmt(f),
        }
    }
}
use crate::error::{ApiError, CacheError};
pub trait ProcessRequest<A: ApiClient, C: Cache, R: Request, T> {
    fn process_request(&self, request: R) -> GdcfFuture<T, A::Err, C::Err>;
    fn paginate(&self, request: R) -> GdcfStream<A, C, R, T, Self>
    where
        R: PaginatableRequest,
        Self: Sized + Clone,
    {
        let next = request.next();
        let current = self.process_request(request);
        GdcfStream {
            next_request: next,
            current_request: current,
            source: self.clone(),
        }
    }
}
impl<A, R, C> ProcessRequest<A, C, R, R::Result> for Gdcf<A, C>
where
    R: Request,
    A: ApiClient + MakeRequest<R>,
    C: Cache,
{
}
use crate::api::client::MakeRequest;
pub struct Gdcf<A, C>
where
    A: ApiClient,
    C: Cache,
{
    client: A,
    cache: C,
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl<A: ::std::fmt::Debug, C: ::std::fmt::Debug> ::std::fmt::Debug for Gdcf<A, C>
where
    A: ApiClient,
    C: Cache,
{
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match *self {
            Gdcf {
                client: ref __self_0_0,
                cache: ref __self_0_1,
            } => {
                let mut debug_trait_builder = f.debug_struct("Gdcf");
                let _ = debug_trait_builder.field("client", &&(*__self_0_0));
                let _ = debug_trait_builder.field("cache", &&(*__self_0_1));
                debug_trait_builder.finish()
            },
        }
    }
}
impl<A, C> Clone for Gdcf<A, C>
where
    A: ApiClient,
    C: Cache,
{
    fn clone(&self) -> Self {
        Gdcf {
            client: self.client.clone(),
            cache: self.cache.clone(),
        }
    }
}
impl<A, C> ProcessRequest<A, C, UserRequest, User> for Gdcf<A, C>
where
    A: ApiClient,
    C: Cache,
{
    fn process_request(&self, request: UserRequest) -> GdcfFuture<User, A::Err, C::Err> {
        {
            let lvl = ::log::Level::Info;
            if lvl <= ::log::STATIC_MAX_LEVEL && lvl <= ::log::max_level() {
                ::log::__private_api_log(
                    ::std::fmt::Arguments::new_v1(
                        &["Processing request "],
                        &match (&request,) {
                            (arg0,) => [::std::fmt::ArgumentV1::new(arg0, ::std::fmt::Display::fmt)],
                        },
                    ),
                    lvl,
                    &("gdcf", "gdcf", "gdcf/src/lib.rs", 283u32),
                );
            }
        };
        {
            let cache = self.cache();
            match cache.lookup_user(&request) {
                Ok(cached) =>
                    if cache.is_expired(&cached) {
                        {
                            let lvl = ::log::Level::Info;
                            if lvl <= ::log::STATIC_MAX_LEVEL && lvl <= ::log::max_level() {
                                ::log::__private_api_log(
                                    ::std::fmt::Arguments::new_v1(
                                        &["Cache entry for request ", " is expired!"],
                                        &match (&request,) {
                                            (arg0,) => [::std::fmt::ArgumentV1::new(arg0, ::std::fmt::Display::fmt)],
                                        },
                                    ),
                                    lvl,
                                    &("gdcf", "gdcf", "gdcf/src/lib.rs", 285u32),
                                );
                            }
                        };
                        GdcfFuture::outdated(
                            cached,
                            (|| {
                                let mut cache = self.cache();
                                self.client.user(request).map_err(GdcfError::Api).and_then(move |response| {
                                    let mut result = None;
                                    for obj in response {
                                        cache.store_object(&obj).map_err(GdcfError::Cache)?;
                                        if let Secondary::User(level) = obj {
                                            result = Some(level)
                                        }
                                    }
                                    result.ok_or(GdcfError::NoContent)
                                })
                            })(),
                        )
                    } else {
                        {
                            let lvl = ::log::Level::Info;
                            if lvl <= ::log::STATIC_MAX_LEVEL && lvl <= ::log::max_level() {
                                ::log::__private_api_log(
                                    ::std::fmt::Arguments::new_v1(
                                        &["Cached entry for request ", " is up-to-date!"],
                                        &match (&request,) {
                                            (arg0,) => [::std::fmt::ArgumentV1::new(arg0, ::std::fmt::Display::fmt)],
                                        },
                                    ),
                                    lvl,
                                    &("gdcf", "gdcf", "gdcf/src/lib.rs", 285u32),
                                );
                            }
                        };
                        GdcfFuture::up_to_date(cached)
                    },
                Err(error) =>
                    if error.is_cache_miss() {
                        {
                            let lvl = ::log::Level::Info;
                            if lvl <= ::log::STATIC_MAX_LEVEL && lvl <= ::log::max_level() {
                                ::log::__private_api_log(
                                    ::std::fmt::Arguments::new_v1(
                                        &["No cache entry for request "],
                                        &match (&request,) {
                                            (arg0,) => [::std::fmt::ArgumentV1::new(arg0, ::std::fmt::Display::fmt)],
                                        },
                                    ),
                                    lvl,
                                    &("gdcf", "gdcf", "gdcf/src/lib.rs", 285u32),
                                );
                            }
                        };
                        GdcfFuture::absent((|| {
                            let mut cache = self.cache();
                            self.client.user(request).map_err(GdcfError::Api).and_then(move |response| {
                                let mut result = None;
                                for obj in response {
                                    cache.store_object(&obj).map_err(GdcfError::Cache)?;
                                    if let Secondary::User(level) = obj {
                                        result = Some(level)
                                    }
                                }
                                result.ok_or(GdcfError::NoContent)
                            })
                        })())
                    } else {
                        GdcfFuture::cache_error(error)
                    },
            }
        }
    }
}
impl<A, C> ProcessRequest<A, C, LevelRequest, Level<u64, u64>> for Gdcf<A, C>
where
    A: ApiClient,
    C: Cache,
{
    fn process_request(&self, request: LevelRequest) -> GdcfFuture<Level<u64, u64>, A::Err, C::Err> {
        {
            let lvl = ::log::Level::Info;
            if lvl <= ::log::STATIC_MAX_LEVEL && lvl <= ::log::max_level() {
                ::log::__private_api_log(
                    ::std::fmt::Arguments::new_v1(
                        &["Processing request ", " with \'u64\' as Song type and \'u64\' as User type"],
                        &match (&request,) {
                            (arg0,) => [::std::fmt::ArgumentV1::new(arg0, ::std::fmt::Display::fmt)],
                        },
                    ),
                    lvl,
                    &("gdcf", "gdcf", "gdcf/src/lib.rs", 303u32),
                );
            }
        };
        {
            let cache = self.cache();
            match cache.lookup_level(&request) {
                Ok(cached) =>
                    if cache.is_expired(&cached) {
                        {
                            let lvl = ::log::Level::Info;
                            if lvl <= ::log::STATIC_MAX_LEVEL && lvl <= ::log::max_level() {
                                ::log::__private_api_log(
                                    ::std::fmt::Arguments::new_v1(
                                        &["Cache entry for request ", " is expired!"],
                                        &match (&request,) {
                                            (arg0,) => [::std::fmt::ArgumentV1::new(arg0, ::std::fmt::Display::fmt)],
                                        },
                                    ),
                                    lvl,
                                    &("gdcf", "gdcf", "gdcf/src/lib.rs", 305u32),
                                );
                            }
                        };
                        GdcfFuture::outdated(
                            cached,
                            (|| {
                                let mut cache = self.cache();
                                self.client.level(request).map_err(GdcfError::Api).and_then(move |response| {
                                    let mut result = None;
                                    for obj in response {
                                        cache.store_object(&obj).map_err(GdcfError::Cache)?;
                                        if let Secondary::Level(level) = obj {
                                            result = Some(level)
                                        }
                                    }
                                    result.ok_or(GdcfError::NoContent)
                                })
                            })(),
                        )
                    } else {
                        {
                            let lvl = ::log::Level::Info;
                            if lvl <= ::log::STATIC_MAX_LEVEL && lvl <= ::log::max_level() {
                                ::log::__private_api_log(
                                    ::std::fmt::Arguments::new_v1(
                                        &["Cached entry for request ", " is up-to-date!"],
                                        &match (&request,) {
                                            (arg0,) => [::std::fmt::ArgumentV1::new(arg0, ::std::fmt::Display::fmt)],
                                        },
                                    ),
                                    lvl,
                                    &("gdcf", "gdcf", "gdcf/src/lib.rs", 305u32),
                                );
                            }
                        };
                        GdcfFuture::up_to_date(cached)
                    },
                Err(error) =>
                    if error.is_cache_miss() {
                        {
                            let lvl = ::log::Level::Info;
                            if lvl <= ::log::STATIC_MAX_LEVEL && lvl <= ::log::max_level() {
                                ::log::__private_api_log(
                                    ::std::fmt::Arguments::new_v1(
                                        &["No cache entry for request "],
                                        &match (&request,) {
                                            (arg0,) => [::std::fmt::ArgumentV1::new(arg0, ::std::fmt::Display::fmt)],
                                        },
                                    ),
                                    lvl,
                                    &("gdcf", "gdcf", "gdcf/src/lib.rs", 305u32),
                                );
                            }
                        };
                        GdcfFuture::absent((|| {
                            let mut cache = self.cache();
                            self.client.level(request).map_err(GdcfError::Api).and_then(move |response| {
                                let mut result = None;
                                for obj in response {
                                    cache.store_object(&obj).map_err(GdcfError::Cache)?;
                                    if let Secondary::Level(level) = obj {
                                        result = Some(level)
                                    }
                                }
                                result.ok_or(GdcfError::NoContent)
                            })
                        })())
                    } else {
                        GdcfFuture::cache_error(error)
                    },
            }
        }
    }
}
impl<A, C> ProcessRequest<A, C, LevelsRequest, Vec<PartialLevel<u64, u64>>> for Gdcf<A, C>
where
    A: ApiClient,
    C: Cache,
{
    fn process_request(&self, request: LevelsRequest) -> GdcfFuture<Vec<PartialLevel<u64, u64>>, A::Err, C::Err> {
        {
            let lvl = ::log::Level::Info;
            if lvl <= ::log::STATIC_MAX_LEVEL && lvl <= ::log::max_level() {
                ::log::__private_api_log(
                    ::std::fmt::Arguments::new_v1(
                        &["Processing request ", " with \'u64\' as Song type and \'u64\' as User type"],
                        &match (&request,) {
                            (arg0,) => [::std::fmt::ArgumentV1::new(arg0, ::std::fmt::Display::fmt)],
                        },
                    ),
                    lvl,
                    &("gdcf", "gdcf", "gdcf/src/lib.rs", 323u32),
                );
            }
        };
        {
            let cache = self.cache();
            match cache.lookup_partial_levels(&request) {
                Ok(cached) =>
                    if cache.is_expired(&cached) {
                        {
                            let lvl = ::log::Level::Info;
                            if lvl <= ::log::STATIC_MAX_LEVEL && lvl <= ::log::max_level() {
                                ::log::__private_api_log(
                                    ::std::fmt::Arguments::new_v1(
                                        &["Cache entry for request ", " is expired!"],
                                        &match (&request,) {
                                            (arg0,) => [::std::fmt::ArgumentV1::new(arg0, ::std::fmt::Display::fmt)],
                                        },
                                    ),
                                    lvl,
                                    &("gdcf", "gdcf", "gdcf/src/lib.rs", 325u32),
                                );
                            }
                        };
                        GdcfFuture::outdated(
                            cached,
                            (|| {
                                let mut cache = self.cache();
                                self.client
                                    .levels(request.clone())
                                    .map_err(GdcfError::Api)
                                    .and_then(move |response| {
                                        let mut result = Vec::new();
                                        for obj in response {
                                            cache.store_object(&obj).map_err(GdcfError::Cache)?;
                                            if let Secondary::PartialLevel(level) = obj {
                                                result.push(level)
                                            }
                                        }
                                        if !result.is_empty() {
                                            cache.store_partial_levels(&request, &result).map_err(GdcfError::Cache)?;
                                            Ok(result)
                                        } else {
                                            Err(GdcfError::NoContent)
                                        }
                                    })
                            })(),
                        )
                    } else {
                        {
                            let lvl = ::log::Level::Info;
                            if lvl <= ::log::STATIC_MAX_LEVEL && lvl <= ::log::max_level() {
                                ::log::__private_api_log(
                                    ::std::fmt::Arguments::new_v1(
                                        &["Cached entry for request ", " is up-to-date!"],
                                        &match (&request,) {
                                            (arg0,) => [::std::fmt::ArgumentV1::new(arg0, ::std::fmt::Display::fmt)],
                                        },
                                    ),
                                    lvl,
                                    &("gdcf", "gdcf", "gdcf/src/lib.rs", 325u32),
                                );
                            }
                        };
                        GdcfFuture::up_to_date(cached)
                    },
                Err(error) =>
                    if error.is_cache_miss() {
                        {
                            let lvl = ::log::Level::Info;
                            if lvl <= ::log::STATIC_MAX_LEVEL && lvl <= ::log::max_level() {
                                ::log::__private_api_log(
                                    ::std::fmt::Arguments::new_v1(
                                        &["No cache entry for request "],
                                        &match (&request,) {
                                            (arg0,) => [::std::fmt::ArgumentV1::new(arg0, ::std::fmt::Display::fmt)],
                                        },
                                    ),
                                    lvl,
                                    &("gdcf", "gdcf", "gdcf/src/lib.rs", 325u32),
                                );
                            }
                        };
                        GdcfFuture::absent((|| {
                            let mut cache = self.cache();
                            self.client
                                .levels(request.clone())
                                .map_err(GdcfError::Api)
                                .and_then(move |response| {
                                    let mut result = Vec::new();
                                    for obj in response {
                                        cache.store_object(&obj).map_err(GdcfError::Cache)?;
                                        if let Secondary::PartialLevel(level) = obj {
                                            result.push(level)
                                        }
                                    }
                                    if !result.is_empty() {
                                        cache.store_partial_levels(&request, &result).map_err(GdcfError::Cache)?;
                                        Ok(result)
                                    } else {
                                        Err(GdcfError::NoContent)
                                    }
                                })
                        })())
                    } else {
                        GdcfFuture::cache_error(error)
                    },
            }
        }
    }
}
impl<A, C, User> ProcessRequest<A, C, LevelRequest, Level<NewgroundsSong, User>> for Gdcf<A, C>
where
    Self: ProcessRequest<A, C, LevelRequest, Level<u64, User>>,
    A: ApiClient,
    C: Cache,
    User: PartialEq + Send + 'static,
{
    fn process_request(&self, request: LevelRequest) -> GdcfFuture<Level<NewgroundsSong, User>, A::Err, C::Err> {
        {
            let lvl = ::log::Level::Info;
            if lvl <= ::log::STATIC_MAX_LEVEL && lvl <= ::log::max_level() {
                ::log::__private_api_log(
                    ::std::fmt::Arguments::new_v1(
                        &[
                            "Processing request ",
                            " with \'NewgroundsSong\' as Song type for arbitrary User type",
                        ],
                        &match (&request,) {
                            (arg0,) => [::std::fmt::ArgumentV1::new(arg0, ::std::fmt::Display::fmt)],
                        },
                    ),
                    lvl,
                    &("gdcf", "gdcf", "gdcf/src/lib.rs", 345u32),
                );
            }
        };
        let raw: GdcfFuture<Level<u64, User>, _, _> = self.process_request(request);
        let gdcf = self.clone();
        match raw {
            GdcfFuture {
                cached: Some(cached),
                inner: None,
            } =>
                match cached.inner().base.custom_song {
                    None => GdcfFuture::up_to_date(cached.map(|inner| exchange::level_song(inner, None))),
                    Some(custom_song_id) => {
                        let lookup = self.cache().lookup_song(custom_song_id);
                        match lookup {
                            Ok(song) => GdcfFuture::up_to_date(cached.map(|inner| exchange::level_song(inner, Some(song.extract())))),
                            Err(ref err) if err.is_cache_miss() => {
                                let cached = cached.extract();
                                {
                                    let lvl = ::log::Level::Warn;
                                    if lvl <= ::log::STATIC_MAX_LEVEL && lvl <= ::log::max_level() {
                                        ::log::__private_api_log(::std::fmt::Arguments::new_v1(&["The level requested was cached, but not its song, performing a request to retrieve it!"],
                                                                                           &match ()
                                                                                                {
                                                                                                ()
                                                                                                =>
                                                                                                [],
                                                                                            }),
                                                             lvl,
                                                             &("gdcf", "gdcf",
                                                               "gdcf/src/lib.rs",
                                                               382u32));
                                    }
                                };
                                GdcfFuture::absent(
                                    self.levels::<u64, u64>(LevelsRequest::default().with_id(cached.base.level_id))
                                        .and_then(move |_| {
                                            let song = gdcf.cache().lookup_song(custom_song_id).map_err(GdcfError::Cache)?;
                                            Ok(exchange::level_song(cached, Some(song.extract())))
                                        }),
                                )
                            },
                            Err(err) => GdcfFuture::cache_error(err),
                        }
                    },
                },
            GdcfFuture { cached, inner: Some(f) } => {
                let cached = match cached {
                    Some(cached) =>
                        match cached.inner().base.custom_song {
                            None => Some(cached.map(|inner| exchange::level_song(inner, None))),
                            Some(custom_song_id) =>
                                match self.cache().lookup_song(custom_song_id) {
                                    Ok(song) => Some(cached.map(|inner| exchange::level_song(inner, Some(song.extract())))),
                                    Err(ref err) if err.is_cache_miss() => None,
                                    Err(err) => return GdcfFuture::cache_error(err),
                                },
                        },
                    None => None,
                };
                GdcfFuture::new(cached,
                                Some(f.and_then(move |level|
                                                    {
                                                        if let Some(song_id) =
                                                               level.base.custom_song
                                                               {
                                                            let lookup =
                                                                gdcf.cache().lookup_song(song_id);
                                                            match lookup {
                                                                Err(ref err)
                                                                if
                                                                err.is_cache_miss()
                                                                => {
                                                                    {
                                                                        let lvl =
                                                                            ::log::Level::Warn;
                                                                        if lvl
                                                                               <=
                                                                               ::log::STATIC_MAX_LEVEL
                                                                               &&
                                                                               lvl
                                                                                   <=
                                                                                   ::log::max_level()
                                                                           {
                                                                            ::log::__private_api_log(::std::fmt::Arguments::new_v1(&["The level the song for the requested level was not cached, performing a request to retrieve it!"],
                                                                                                                                   &match ()
                                                                                                                                        {
                                                                                                                                        ()
                                                                                                                                        =>
                                                                                                                                        [],
                                                                                                                                    }),
                                                                                                     lvl,
                                                                                                     &("gdcf",
                                                                                                       "gdcf",
                                                                                                       "gdcf/src/lib.rs",
                                                                                                       434u32));
                                                                        }
                                                                    };
                                                                    Either::A(gdcf.levels::<u64,
                                                                                            u64>(LevelsRequest::default().with_id(level.base.level_id)).and_then(move
                                                                                                                                                                     |_|
                                                                                                                                                                     {
                                                                                                                                                                         let song =
                                                                                                                                                                             gdcf.cache().lookup_song(song_id).map_err(GdcfError::Cache)?;
                                                                                                                                                                         Ok(exchange::level_song(level,
                                                                                                                                                                                                 Some(song.extract())))
                                                                                                                                                                     }))
                                                                }
                                                                Err(error) =>
                                                                Either::B(err(GdcfError::Cache(error))),
                                                                Ok(song) =>
                                                                Either::B(ok(exchange::level_song(level,
                                                                                                  Some(song.extract())))),
                                                            }
                                                        } else {
                                                            Either::B(ok(exchange::level_song(level,
                                                                                              None)))
                                                        }
                                                    })))
            },
            _ => ::std::rt::begin_panic("internal error: entered unreachable code", &("gdcf/src/lib.rs", 459u32, 18u32)),
        }
    }
}
impl<A, C, User> ProcessRequest<A, C, LevelsRequest, Vec<PartialLevel<NewgroundsSong, User>>> for Gdcf<A, C>
where
    Self: ProcessRequest<A, C, LevelsRequest, Vec<PartialLevel<u64, User>>>,
    A: ApiClient,
    C: Cache,
    User: PartialEq + Send + 'static,
{
    fn process_request(&self, request: LevelsRequest) -> GdcfFuture<Vec<PartialLevel<NewgroundsSong, User>>, A::Err, C::Err> {
        {
            let lvl = ::log::Level::Info;
            if lvl <= ::log::STATIC_MAX_LEVEL && lvl <= ::log::max_level() {
                ::log::__private_api_log(
                    ::std::fmt::Arguments::new_v1(
                        &["Processing request ", " with \'NewgroundsSong\' as Song type"],
                        &match (&request,) {
                            (arg0,) => [::std::fmt::ArgumentV1::new(arg0, ::std::fmt::Display::fmt)],
                        },
                    ),
                    lvl,
                    &("gdcf", "gdcf", "gdcf/src/lib.rs", 472u32),
                );
            }
        };
        let GdcfFuture { cached, inner } = self.process_request(request);
        let cache = self.cache.clone();
        let processor = move |levels: Vec<PartialLevel<u64, User>>| {
            let mut vec = Vec::new();
            for partial_level in levels {
                let built = match partial_level.custom_song {
                    Some(custom_song_id) =>
                        match cache.lookup_song(custom_song_id) {
                            Ok(song) => exchange::partial_level_song(partial_level, Some(song.extract())),
                            Err(err) =>
                                if err.is_cache_miss() {
                                    exchange::partial_level_song(partial_level, Some(SERVER_SIDED_DATA_INCONSISTENCY_ERROR()))
                                } else {
                                    return Err(GdcfError::Cache(err))
                                },
                        },
                    None => exchange::partial_level_song(partial_level, None),
                };
                vec.push(built);
            }
            Ok(vec)
        };
        let cached = match cached {
            Some(cached) =>
                match cached.try_map(processor.clone()) {
                    Ok(cached) => Some(cached),
                    Err(err) => return GdcfFuture::error(err),
                },
            None => None,
        };
        GdcfFuture::new(cached, inner.map(|fut| fut.and_then(processor)))
    }
}
impl<A, C> ProcessRequest<A, C, LevelsRequest, Vec<PartialLevel<u64, Creator>>> for Gdcf<A, C>
where
    A: ApiClient,
    C: Cache,
{
    fn process_request(&self, request: LevelsRequest) -> GdcfFuture<Vec<PartialLevel<u64, Creator>>, A::Err, C::Err> {
        {
            let lvl = ::log::Level::Info;
            if lvl <= ::log::STATIC_MAX_LEVEL && lvl <= ::log::max_level() {
                ::log::__private_api_log(
                    ::std::fmt::Arguments::new_v1(
                        &["Processing request ", " with \'Creator\' as User type"],
                        &match (&request,) {
                            (arg0,) => [::std::fmt::ArgumentV1::new(arg0, ::std::fmt::Display::fmt)],
                        },
                    ),
                    lvl,
                    &("gdcf", "gdcf", "gdcf/src/lib.rs", 522u32),
                );
            }
        };
        let GdcfFuture { cached, inner } = self.process_request(request);
        let cache = self.cache.clone();
        let processor = move |levels: Vec<PartialLevel<u64, u64>>| {
            let mut vec = Vec::new();
            for partial_level in levels {
                vec.push(match cache.lookup_creator(partial_level.creator) {
                    Ok(creator) => exchange::partial_level_user(partial_level, creator.extract()),
                    Err(err) =>
                        if err.is_cache_miss() {
                            exchange::partial_level_user(partial_level, DELETED())
                        } else {
                            return Err(GdcfError::Cache(err))
                        },
                })
            }
            Ok(vec)
        };
        let cached = match cached {
            Some(cached) =>
                match cached.try_map(processor.clone()) {
                    Ok(cached) => Some(cached),
                    Err(err) => return GdcfFuture::error(err),
                },
            None => None,
        };
        GdcfFuture::new(cached, inner.map(|fut| fut.and_then(processor)))
    }
}
impl<A, C> ProcessRequest<A, C, LevelRequest, Level<u64, Creator>> for Gdcf<A, C>
where
    A: ApiClient,
    C: Cache,
{
    fn process_request(&self, request: LevelRequest) -> GdcfFuture<Level<u64, Creator>, A::Err, C::Err> {
        {
            let lvl = ::log::Level::Info;
            if lvl <= ::log::STATIC_MAX_LEVEL && lvl <= ::log::max_level() {
                ::log::__private_api_log(
                    ::std::fmt::Arguments::new_v1(
                        &["Processing request ", " with \'Creator\' as User type"],
                        &match (&request,) {
                            (arg0,) => [::std::fmt::ArgumentV1::new(arg0, ::std::fmt::Display::fmt)],
                        },
                    ),
                    lvl,
                    &("gdcf", "gdcf", "gdcf/src/lib.rs", 572u32),
                );
            }
        };
        let raw: GdcfFuture<Level<u64, u64>, _, _> = self.process_request(request);
        let gdcf = self.clone();
        match raw {
            GdcfFuture {
                cached: Some(cached),
                inner: None,
            } => {
                let lookup = self.cache().lookup_creator(cached.inner().base.creator);
                match lookup {
                    Ok(creator) => {
                        {
                            let lvl = ::log::Level::Info;
                            if lvl <= ::log::STATIC_MAX_LEVEL && lvl <= ::log::max_level() {
                                ::log::__private_api_log(
                                    ::std::fmt::Arguments::new_v1(
                                        &["Level ", " up-to-date, returning cached version!"],
                                        &match (&cached.inner(),) {
                                            (arg0,) => [::std::fmt::ArgumentV1::new(arg0, ::std::fmt::Display::fmt)],
                                        },
                                    ),
                                    lvl,
                                    &("gdcf", "gdcf", "gdcf/src/lib.rs", 588u32),
                                );
                            }
                        };
                        GdcfFuture::up_to_date(cached.map(|inner| exchange::level_user(inner, creator.extract())))
                    },
                    Err(ref err) if err.is_cache_miss() => {
                        {
                            let lvl = ::log::Level::Warn;
                            if lvl <= ::log::STATIC_MAX_LEVEL && lvl <= ::log::max_level() {
                                ::log::__private_api_log(::std::fmt::Arguments::new_v1(&["Level ",
                                                                                         " was up-to-date, but creator is missing from cache. Constructing LevelsRequest to retrieve creator"],
                                                                                       &match (&cached.inner(),)
                                                                                            {
                                                                                            (arg0,)
                                                                                            =>
                                                                                            [::std::fmt::ArgumentV1::new(arg0,
                                                                                                                         ::std::fmt::Display::fmt)],
                                                                                        }),
                                                         lvl,
                                                         &("gdcf", "gdcf",
                                                           "gdcf/src/lib.rs",
                                                           594u32));
                            }
                        };
                        let cached = cached.extract();
                        GdcfFuture::absent(
                            self.levels::<u64, u64>(LevelsRequest::default().with_id(cached.base.level_id))
                                .and_then(move |_| {
                                    let lookup = gdcf.cache().lookup_creator(cached.base.creator);
                                    match lookup {
                                        Ok(creator) => Ok(exchange::level_user(cached, creator.extract())),
                                        Err(ref err) if err.is_cache_miss() => {
                                            let creator = Creator::deleted(cached.base.creator);
                                            gdcf.cache().store_object(&creator.clone().into()).map_err(GdcfError::Cache)?;
                                            Ok(exchange::level_user(cached, creator))
                                        },
                                        Err(err) => Err(GdcfError::Cache(err)),
                                    }
                                }),
                        )
                    },
                    Err(err) => GdcfFuture::cache_error(err),
                }
            },
            GdcfFuture { cached, inner: Some(f) } => {
                let cached = match cached {
                    Some(cached) =>
                        match self.cache().lookup_creator(cached.inner().base.creator) {
                            Ok(creator) => Some(cached.map(|inner| exchange::level_user(inner, creator.extract()))),
                            Err(ref err) if err.is_cache_miss() => None,
                            Err(err) => return GdcfFuture::cache_error(err),
                        },
                    None => None,
                };
                if let Some(ref level) = cached {
                    {
                        let lvl = ::log::Level::Info;
                        if lvl <= ::log::STATIC_MAX_LEVEL && lvl <= ::log::max_level() {
                            ::log::__private_api_log(
                                ::std::fmt::Arguments::new_v1(
                                    &["Cache entry is "],
                                    &match (&level.inner(),) {
                                        (arg0,) => [::std::fmt::ArgumentV1::new(arg0, ::std::fmt::Display::fmt)],
                                    },
                                ),
                                lvl,
                                &("gdcf", "gdcf", "gdcf/src/lib.rs", 644u32),
                            );
                        }
                    };
                } else {
                    {
                        let lvl = ::log::Level::Warn;
                        if lvl <= ::log::STATIC_MAX_LEVEL && lvl <= ::log::max_level() {
                            ::log::__private_api_log(
                                ::std::fmt::Arguments::new_v1(
                                    &["Cache entry for request missing"],
                                    &match () {
                                        () => [],
                                    },
                                ),
                                lvl,
                                &("gdcf", "gdcf", "gdcf/src/lib.rs", 646u32),
                            );
                        }
                    };
                }
                GdcfFuture::new(
                    cached,
                    Some(f.and_then(move |level| {
                        let lookup = gdcf.cache().lookup_creator(level.base.creator);
                        match lookup {
                            Ok(creator) => Either::B(ok(exchange::level_user(level, creator.extract()))),
                            Err(ref err) if err.is_cache_miss() =>
                                Either::A(
                                    gdcf.levels::<u64, u64>(LevelsRequest::default().with_id(level.base.level_id))
                                        .and_then(move |_| {
                                            let lookup = gdcf.cache().lookup_creator(level.base.creator);
                                            match lookup {
                                                Ok(creator) => Ok(exchange::level_user(level, creator.extract())),
                                                Err(ref err) if err.is_cache_miss() => {
                                                    let creator = Creator::deleted(level.base.creator);
                                                    gdcf.cache().store_object(&creator.clone().into()).map_err(GdcfError::Cache)?;
                                                    Ok(exchange::level_user(level, creator))
                                                },
                                                Err(err) => Err(GdcfError::Cache(err)),
                                            }
                                        }),
                                ),
                            Err(error) => Either::B(err(GdcfError::Cache(error))),
                        }
                    })),
                )
            },
            _ => ::std::rt::begin_panic("internal error: entered unreachable code", &("gdcf/src/lib.rs", 685u32, 18u32)),
        }
    }
}
impl<A, C> Gdcf<A, C>
where
    A: ApiClient,
    C: Cache,
{
    pub fn new(client: A, cache: C) -> Gdcf<A, C> {
        Gdcf { client, cache }
    }

    pub fn cache(&self) -> C {
        self.cache.clone()
    }

    pub fn client(&self) -> A {
        self.client.clone()
    }

    /// Processes the given [`LevelRequest`]
    ///
    /// The `User` and `Song` type parameters determine, which sequence of requests should be made
    /// to retrieve the [`Level`]. A plain request to `downloadGJLevel` is equivalent to a call of
    /// `Gdcf::level<u64, u64>`
    ///
    /// `User` can currently be one of the following:
    /// + [`u64`] - The creator is provided as his user ID. Causes no additional requests.
    /// + [`Creator`] - Causes an additional [`LevelsRequest`] to retrieve the creator.
    /// + [`User`] - Causes an additional [`UserRequest`]  to retrieve the creator's profile (Not
    /// Yet Implemented)
    ///
    /// `Song` can currently be one of the following:
    /// + [`u64`] - The custom song is provided only as its newgrounds ID. Causes no additional
    /// requests
    /// + [`NewgroundsSong`] - Causes an additional [`LevelsRequest`] to be made to
    /// retrieve the custom song (only if the level actually uses a custom song though)
    ///
    /// Note that a call of `Gdcf::level<NewgroundsSong, Creator>` will **not** issue the same
    /// `LevelsRequest` twice - GDCF will recognize the cache to be up-to-date when it attempts the
    /// second one and uses the cached value (or at least it will if you set cache-expiry to
    /// anything larger than 0 seconds - but then again why would you use GDCF if you don't use the
    /// cache)
    pub fn level<Song, User>(&self, request: LevelRequest) -> GdcfFuture<Level<Song, User>, A::Err, C::Err>
    where
        Self: ProcessRequest<A, C, LevelRequest, Level<Song, User>>,
        Song: PartialEq,
        User: PartialEq,
    {
        self.process_request(request)
    }

    /// Processes the given [`LevelsRequest`]
    ///
    /// The `User` and `Song` type parameters determine, which sequence of requests should be made
    /// to retrieve the [`Level`].
    ///
    /// `User` can currently be one of the following:
    /// + [`u64`] - The creator are only provided as their user IDs. Causes no additional requests
    /// + [`Creator`] - Causes no additional requests
    /// + [`User`] - Causes up to 10 additional [`UserRequest`]s to retrieve every creator's
    /// profile (Not Yet Implemented)
    ///
    /// `Song` can currently be one of the following:
    /// + [`u64`] - The custom song is provided only as its newgrounds ID. Causes no additional
    /// requests
    /// + [`NewgroundsSong`] - Causes no additional requests.
    pub fn levels<Song, User>(&self, request: LevelsRequest) -> GdcfFuture<Vec<PartialLevel<Song, User>>, A::Err, C::Err>
    where
        Self: ProcessRequest<A, C, LevelsRequest, Vec<PartialLevel<Song, User>>>,
        Song: PartialEq,
        User: PartialEq,
    {
        self.process_request(request)
    }

    /// Generates a stream of pages of levels by incrementing the [`LevelsRequest`]'s `page`
    /// parameter until it hits the first empty page.
    pub fn paginate_levels<Song, User>(
        &self, request: LevelsRequest,
    ) -> impl Stream<Item = Vec<PartialLevel<Song, User>>, Error = GdcfError<A::Err, C::Err>>
    where
        Self: ProcessRequest<A, C, LevelsRequest, Vec<PartialLevel<Song, User>>>,
        Song: PartialEq,
        User: PartialEq,
    {
        self.paginate(request)
    }

    /// Processes the given [`UserRequest`]
    pub fn user(&self, request: UserRequest) -> GdcfFuture<User, A::Err, C::Err> {
        self.process_request(request)
    }
}
#[allow(missing_debug_implementations)]
pub struct GdcfFuture<T, A: ApiError, C: CacheError> {
    pub cached: Option<CachedObject<T>>,
    pub inner: Option<Box<dyn Future<Item = T, Error = GdcfError<A, C>> + Send + 'static>>,
}
impl<T, A: ApiError, C: CacheError> GdcfFuture<T, A, C> {
    fn new<F>(cached: Option<CachedObject<T>>, f: Option<F>) -> Self
    where
        F: Future<Item = T, Error = GdcfError<A, C>> + Send + 'static,
    {
        GdcfFuture {
            cached,
            inner: match f {
                None => None,
                Some(f) => Some(Box::new(f)),
            },
        }
    }

    fn up_to_date(object: CachedObject<T>) -> Self {
        GdcfFuture {
            cached: Some(object),
            inner: None,
        }
    }

    fn outdated<F>(object: CachedObject<T>, f: F) -> Self
    where
        F: Future<Item = T, Error = GdcfError<A, C>> + Send + 'static,
    {
        GdcfFuture::new(Some(object), Some(f))
    }

    fn absent<F>(f: F) -> Self
    where
        F: Future<Item = T, Error = GdcfError<A, C>> + Send + 'static,
    {
        GdcfFuture::new(None, Some(f))
    }

    fn cache_error(error: C) -> Self
    where
        T: Send + 'static,
    {
        GdcfFuture::new(None, Some(err(GdcfError::Cache(error))))
    }

    fn error(error: GdcfError<A, C>) -> Self
    where
        T: Send + 'static,
    {
        GdcfFuture::new(None, Some(err(error)))
    }

    pub fn cached(&self) -> &Option<CachedObject<T>> {
        &self.cached
    }
}
impl<T, A: ApiError, C: CacheError> Future for GdcfFuture<T, A, C> {
    type Error = GdcfError<A, C>;
    type Item = T;

    fn poll(&mut self) -> Result<Async<T>, GdcfError<A, C>> {
        match self.inner {
            Some(ref mut fut) => fut.poll(),
            None => Ok(Async::Ready(self.cached.take().unwrap().extract())),
        }
    }
}
#[allow(missing_debug_implementations)]
pub struct GdcfStream<A, C, R, T, M>
where
    R: PaginatableRequest,
    M: ProcessRequest<A, C, R, T>,
    A: ApiClient,
    C: Cache,
{
    next_request: R,
    current_request: GdcfFuture<T, A::Err, C::Err>,
    source: M,
}
impl<A, C, R, T, M> Stream for GdcfStream<A, C, R, T, M>
where
    R: PaginatableRequest,
    M: ProcessRequest<A, C, R, T>,
    A: ApiClient,
    C: Cache,
{
    type Error = GdcfError<A::Err, C::Err>;
    type Item = T;

    fn poll(&mut self) -> Result<Async<Option<T>>, Self::Error> {
        match self.current_request.poll() {
            Ok(Async::NotReady) => Ok(Async::NotReady),
            Ok(Async::Ready(result)) => {
                task::current().notify();
                let next = self.next_request.next();
                let cur = mem::replace(&mut self.next_request, next);
                self.current_request = self.source.process_request(cur);
                Ok(Async::Ready(Some(result)))
            },
            Err(GdcfError::NoContent) => {
                {
                    let lvl = ::log::Level::Info;
                    if lvl <= ::log::STATIC_MAX_LEVEL && lvl <= ::log::max_level() {
                        ::log::__private_api_log(
                            ::std::fmt::Arguments::new_v1(
                                &["Stream over request ", " terminating due to exhaustion!"],
                                &match (&self.next_request,) {
                                    (arg0,) => [::std::fmt::ArgumentV1::new(arg0, ::std::fmt::Display::fmt)],
                                },
                            ),
                            lvl,
                            &("gdcf", "gdcf", "gdcf/src/lib.rs", 927u32),
                        );
                    }
                };
                Ok(Async::Ready(None))
            },
            Err(GdcfError::Api(ref err)) if err.is_no_result() => {
                {
                    let lvl = ::log::Level::Info;
                    if lvl <= ::log::STATIC_MAX_LEVEL && lvl <= ::log::max_level() {
                        ::log::__private_api_log(
                            ::std::fmt::Arguments::new_v1(
                                &["Stream over request ", " terminating due to exhaustion!"],
                                &match (&self.next_request,) {
                                    (arg0,) => [::std::fmt::ArgumentV1::new(arg0, ::std::fmt::Display::fmt)],
                                },
                            ),
                            lvl,
                            &("gdcf", "gdcf", "gdcf/src/lib.rs", 933u32),
                        );
                    }
                };
                Ok(Async::Ready(None))
            },
            Err(err) => Err(err),
        }
    }
}
