//! Module containing request definitions for retrieving levels

use crate::api::request::{BaseRequest, PaginatableRequest, Request, GD_21};
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
#[derive(Debug, Default, Clone, Copy)]
pub struct LevelRequest {
    /// Whether this [`LevelRequest`] request forces a cache refresh. This is not a HTTP
    /// request field!
    pub force_refresh: bool,

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
/// [`NewgroundsSong`s](::model::song::NewgroundsSong) and [`Creator`s](::model::user::Creator).
#[derive(Debug, Default, Clone)]
pub struct LevelsRequest {
    /// Whether this [`LevelsRequest`] request forces a cache refresh. This is not a HTTP
    /// request field!
    pub force_refresh: bool,

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
    /// To filter by any demon, add [`LevelRating::Demon`] with any arbitrary [`DemonRating`] value.
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
#[derive(Debug, Clone, Hash)]
pub enum CompletionFilter {
    /// No filtering based upon completion
    None,

    /// Filtering based upon a given list of level ids
    List {
        /// The list of level ids to filter
        ids: Vec<u64>,

        /// if `true`, only the levels matching the ids in [`ids`](CompletionFilter::List.ids) will
        /// be searched, if `false`, the levels in [`ids`](CompletionFilter::List.ids) will
        /// be excluded.
        include: bool,
    },
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
#[derive(Debug, Default, Clone, Hash)]
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

/// Enum containing the various types of
/// [`LevelsRequest`] possible
///
/// ## GD Internals:
/// + Unused values: `8`, `9`, `14`
/// + The values `15` and `17` are only used in Geometry Dash World and are the
/// same as `0` ([`LevelRequestType::Search`]) and `6` ([`LevelRequestType::Featured`]) respectively
#[derive(Debug, Copy, Clone, PartialEq, Hash)]
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

    /// Retrieve levels by the user whose ID was specified in [`LevelsRequest::search_string`]
    /// (Note that is has to be the user Id, not the account id)
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
    /// This variant is represented by the value `7` in requests. According to the GDPS source,
    /// this simply looks for levels that have more than 9999 objects.
    Magic,

    /// Map pack levels. The search string is set to a comma seperated list of
    /// levels, which are the levels contained in the map pack
    ///
    /// ## GD Internals:
    /// This variant is represented by the value `10` in requests
    MapPack,

    /// Request to retrieve the list of levels most recently awarded a rating.
    ///
    /// Using this option you can only receive levels that were awarded a rating in Geometry Dash
    /// 1.9 or later
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

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum SongFilter {
    Main(u8),
    Custom(u64),
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
    const_setter! {
        /// Sets the [`BaseRequest`] to be used
        ///
        /// Allows builder-style creation of requests
        base[with_base]: BaseRequest
    }

    const_setter! {
        /// Sets the value of the `inc` field
        ///
        /// Allows builder-style creation of requests
        inc: bool
    }

    const_setter! {
        /// Sets the value of the `extra` field
        ///
        /// Allows builder-style creation of requests
        extra: bool
    }

    pub const fn force_refresh(mut self) -> Self {
        self.force_refresh = true;
        self
    }

    /// Constructs a new `LevelRequest` to retrieve the level with the given id
    ///
    /// Uses a default [`BaseRequest`], and sets the
    /// `inc` field to `true` and `extra` to `false`, as are the default
    /// values set the by the Geometry Dash Client
    pub const fn new(level_id: u64) -> LevelRequest {
        LevelRequest {
            force_refresh: false,
            base: GD_21,
            level_id,
            inc: true,
            extra: false,
        }
    }
}

impl LevelsRequest {
    const_setter!(with_base, base, BaseRequest);

    // idk why this one can't be const
    setter!(filter, search_filters, SearchFilters);

    const_setter!(page, u32);

    const_setter!(total, i32);

    const_setter!(request_type, LevelRequestType);

    pub const fn force_refresh(mut self) -> Self {
        self.force_refresh = true;
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
    type Result = Level<Option<u64>, u64>;

    fn key(&self) -> u64 {
        self.level_id
    }

    fn forces_refresh(&self) -> bool {
        self.force_refresh
    }

    fn set_force_refresh(&mut self, force_refresh: bool) {
        self.force_refresh = force_refresh
    }
}

impl Request for LevelsRequest {
    type Result = Vec<PartialLevel<Option<u64>, u64>>;

    fn forces_refresh(&self) -> bool {
        self.force_refresh
    }

    fn set_force_refresh(&mut self, force_refresh: bool) {
        self.force_refresh = force_refresh
    }
}

impl PaginatableRequest for LevelsRequest {
    fn next(&mut self) {
        self.page += 1;
    }
}

impl Display for LevelRequest {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "LevelRequest({})", self.level_id)
    }
}

impl Display for LevelsRequest {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        match self.request_type {
            LevelRequestType::Search => write!(f, "LevelsRequest(Search={}, page={})", self.search_string, self.page),
            _ => write!(f, "LevelsRequest({:?}, page={})", self.request_type, self.page),
        }
    }
}
