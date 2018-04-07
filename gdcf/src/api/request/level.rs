use api::request::BaseRequest;
use api::request::Request;

pub struct LevelRequest {
    base: BaseRequest,

    pub lid: u64,
    pub inc: bool,
    pub extra: bool,
}

impl LevelRequest {
    pub fn new(lid: u64) -> LevelRequest {
        LevelRequest {
            base: BaseRequest::default(),
            lid,
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


impl Request for LevelRequest {
    fn form_data(&self) -> Vec<(&str, String)> {
        let mut data = self.base.form_data();

        data.push(("levelID", self.lid.to_string()));
        data.push(("inc", String::from(if self.extra { "1" } else { "0" })));
        data.push(("extras", String::from(if self.extra { "1" } else { "0" })));

        data
    }
}