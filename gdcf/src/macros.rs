macro_rules! lookup {
    ($self: expr, $lookup: ident, $req: expr) => {{

        let cached = cache.$lookup(&$req)?;

        (cached, cache.is_expired(cached))
    }};
}

macro_rules! gdcf {
    ($api: ident, $req_type: tt, $lookup: ident) => {
        fn $api(&self, req: $req_type) -> Result<<$req_type as Request>::Result, CacheError<C::Err>> {
            debug!("Received request {}, initiating cache lookup!", req);

            let cache = self.cache();

            match cache.$lookup(&req) {
                Ok(cached) => {
                    if cache.is_expired(&cached) {
                        self.refresh(req)
                    }

                    Ok(cached.extract())
                }

                Err(CacheError::CacheMiss) => {
                    self.refresh(req);

                    Err(CacheError::CacheMiss)
                }

                Err(err) => Err(err)
            }
        }
    }
}

macro_rules! store {
    ($cache: expr, $store: ident, $raw_obj: expr) => {
        {
            FromRawObject::from_raw(&$raw_obj).map(|constructed|{
                info!("Caching {}", constructed);
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

macro_rules! lock {
    (@$mutex: expr) => {&*$mutex.lock().unwrap()};
    ($mutex: expr) => {$mutex.lock().unwrap()};
}
