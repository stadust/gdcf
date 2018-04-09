use api::request::BaseRequest;
use api::request::ser;

#[derive(Serialize, Debug)]
pub struct LevelRequest {
    #[serde(flatten)]
    base: BaseRequest,

    #[cfg_attr(feature="robtop-names", serde(rename = "levelID"))]
    pub level_id: u64,

    #[cfg_attr(feature="robtop-types", serde(serialize_with = "ser::bool_to_int"))]
    pub inc: bool,

    #[cfg_attr(feature="robtop-types", serde(serialize_with = "ser::bool_to_int"))]
    #[cfg_attr(feature="robtop-names", serde(rename = "extras"))]
    pub extra: bool,
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
