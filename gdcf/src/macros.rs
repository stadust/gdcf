macro_rules! __gdcf_inner__ {
    ($func: ident, $req: ty, $result: ty, $lookup: ident, $fut: ident) => {
        pub fn $func(&self, req: $req) -> GdcfFuture<$result, A::Err, C::Err> {
            let cache = lock!(self.cache);

            match cache.$lookup(&req) {
                Ok(cached) => {
                    if cache.is_expired(&cached) {
                        info!("Cache entry for request {} is expired!", req);

                        GdcfFuture::outdated(cached.extract(), self.clone().$fut(req))
                    } else {
                        info!("Cache entry for request {} is up-to-date, making no request", req);

                        GdcfFuture::up_to_date(cached.extract())
                    }
                }

                Err(CacheError::CacheMiss) => {
                    info!("No cache entry for request {}", req);

                    GdcfFuture::absent(self.clone().$fut(req))
                }

                Err(err) => panic!("Error accessing cache! {:?}", err)
            }
        }
    }
}

macro_rules! gdcf_one {
    ($func: ident, $req: ty, $result: ident, $lookup: ident, $fut: ident) => {
        __gdcf_inner__!($func, $req, $result, $lookup, $fut);

        fn $fut(self, req: $req) -> impl Future<Item=$result, Error=GdcfError<A::Err, C::Err>> + Send + 'static {
            let cache = self.cache.clone();
            let future = lock!(self.client).$func(&req);

            future.map_err(GdcfError::Api)
                .and_then(move |response| self.integrity(response))
                .and_then(move |response| {
                    let mut chosen = None;
                    let mut cache = lock!(cache);

                    for obj in response {
                        cache.store_object(&obj)?;

                        if let GDObject::$result(inner) = obj {
                            chosen = Some(inner);
                        }
                    }

                    // TODO: a proper error variant would be nice here!
                    Ok(chosen.unwrap())
                })
        }
    }
}

macro_rules! gdcf_many {
    ($func: ident, $req: ty, $result: ident, $lookup: ident, $bulk_store: ident, $fut: ident) => {
        __gdcf_inner__!($func, $req, Vec<$result>, $lookup, $fut);

        fn $fut(self, req: $req) -> impl Future<Item=Vec<$result>, Error=GdcfError<A::Err, C::Err>> + Send + 'static {
            let cache = self.cache.clone();
            let future = lock!(self.client).$func(&req);

            future.map_err(GdcfError::Api)
                .and_then(move |response| self.integrity(response))
                .and_then(move |response| {
                    let mut chosen = Vec::new();
                    let mut cache = lock!(cache);

                    for obj in response {
                        match obj {
                            GDObject::$result(inner) => chosen.push(inner),
                            _ => cache.store_object(&obj)?
                        }
                    }

                    cache.$bulk_store(&req, &chosen)?;
                    Ok(chosen)
                })
        }
    }
}

macro_rules! setter {
    ($name: ident, $field: ident, $t: ty) => {
        pub fn $name(mut self, $field: $t) -> Self {
            self.$field = $field;
            self
        }
    };

    ($name: ident, $t: ty) => {
        pub fn $name(mut self, arg0: $t) -> Self {
            self.$name = arg0;
            self
        }
    };

    ($(#[$attr:meta])* $name: ident: $t: ty) => {
        $(#[$attr])*
        pub fn $name(mut self, $name: $t) -> Self {
            self.$name = $name;
            self
        }
    };

    ($(#[$attr:meta])* $field:ident[$name: ident]: $t: ty) => {
        $(#[$attr])*
        pub fn $name(mut self, $field: $t) -> Self {
            self.$field = $field;
            self
        }
    }
}

macro_rules! lock {
    (@$mutex: expr) => {&*$mutex.lock().unwrap()};
    (!$mutex: expr) => {&mut *$mutex.lock().unwrap()};
    ($mutex: expr) => {$mutex.lock().unwrap()};
}

macro_rules! into_gdo {
    ($tp: ident) => {
        impl From<$tp> for GDObject {
            fn from(level: $tp) -> Self {
                GDObject::$tp(level)
            }
        }
    };
}

macro_rules! from_str {
    ($target: ident) => {
        impl FromStr for $target {
            type Err = ParseIntError;

            fn from_str(s: &str) -> Result<$target, ParseIntError> {
                let value: i32 = s.parse()?;

                Ok($target::from(value))
            }
        }
    };
}