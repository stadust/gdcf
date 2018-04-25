macro_rules! lookup {
    ($self: expr, $lookup: ident, $req: expr) => {{
        debug!("Initiating cache lookup for {}", $req);

        let cache = $self.cache();
        let cached = cache.$lookup(&$req);
        let expired = cached.as_ref().map_or(true, |co| cache.is_expired(co));

        (cached, expired)
    }};
}

macro_rules! gdcf {
    ($api: ident, $req_type: tt, $lookup: ident) => {
        fn $api(&self, req: $req_type) -> Option<<$req_type as Request>::Result> {
            debug!("Received request {}", req);

            let (cached, expired) = lookup!(self, $lookup, req);

            if expired {
                self.refresh(req);
            }

            cached.map(|co| co.extract())
        }
    }
}

macro_rules! store {
    ($cache: expr, $store: ident, $raw_obj: expr) => {
        {
            FromRawObject::from_raw(&$raw_obj).map(|constructed|{
                debug!("Caching {}", constructed);
                $cache.$store(constructed)
            })
        }
    };
}

macro_rules! setter {
    ($name: ident, $field: ident, $t: ty) => {
        pub fn $name(mut self, arg0: $t) -> Self {
            self.$field = arg0;
            self
        }
    };

    ($name: ident, $t: ty) => {
        pub fn $name(mut self, arg0: $t) -> Self {
            self.$name = arg0;
            self
        }
    }
}
