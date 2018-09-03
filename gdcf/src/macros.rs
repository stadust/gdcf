macro_rules! gdcf_one {
    ($api_call: ident, $cache_lookup: ident, $request_type: ty, $result_type: ty, $enum_variant: ident) => {
        pub fn $api_call(&self, request: $request_type) -> impl Future<Item = $result_type, Error=GdcfError<A::Err, C::Err>> + Send + 'static {
            let cache = self.cache();

            fn generate_future<A: ApiClient, C: Cache>(gdcf: &Gdcf<A, C>, request: $request_type) -> impl Future<Item=$result_type, Error=GdcfError<A::Err, C::Err>> {
                let cache = gdcf.cache.clone();

                gdcf.client().$api_call(request)
                    .map_err(GdcfError::Api)
                    .and_then(move |response| {
                        let mut result = None;
                        let mut cache = cache.lock().unwrap();

                        for obj in response {
                            cache.store_object(&obj)?;

                            if let GDObject::$enum_variant(inner) = obj {
                                result = Some(inner);
                            }
                        }

                        result.ok_or(GdcfError::NoContent)
                    })
            };


            let gdcf_future = match cache.$cache_lookup(&request) {
                Ok(cached) =>
                    if cache.is_expired(&cached) {
                        info!("Cache entry for request {} is expired!", request);

                        GdcfFuture::outdated(cached.extract(), generate_future(self, request))
                    } else {
                        GdcfFuture::up_to_date(cached.extract())
                    },

                Err(CacheError::CacheMiss) => {
                    info!("No cache entry for request {}", request);

                    GdcfFuture::absent(generate_future(self, request))
                },

                Err(err) => return Either::B(result(Err(GdcfError::Cache(err)))),
            };

            Either::A(gdcf_future)
        }
     }
}

macro_rules! gdcf_many {
    ($api_call: ident, $cache_lookup: ident, $cache_store: ident, $request_type: ty, $result_type: ty, $enum_variant: ident) => {
        pub fn $api_call(&self, request: $request_type) -> impl Future<Item = Vec<$result_type>, Error=GdcfError<A::Err, C::Err>> + Send + 'static {
            let cache = self.cache();

            fn generate_future<A: ApiClient, C: Cache>(gdcf: &Gdcf<A, C>, request: $request_type) -> impl Future<Item=Vec<$result_type>, Error=GdcfError<A::Err, C::Err>> {
                let cache = gdcf.cache.clone();

                gdcf.client().$api_call(request.clone())
                    .map_err(GdcfError::Api)
                    .and_then(move |response| {
                        let mut result = Vec::new();
                        let mut cache = cache.lock().unwrap();

                        for obj in response {
                            match obj {
                                GDObject::$enum_variant(object) => result.push(object),
                                _ => cache.store_object(&obj)?
                            }
                        }

                        if result.is_empty() {
                            Err(GdcfError::NoContent)
                        } else {
                            cache.$cache_store(&request, &result)?;

                            Ok(result)
                        }
                    })
            };


            let gdcf_future = match cache.$cache_lookup(&request) {
                Ok(cached) =>
                    if cache.is_expired(&cached) {
                        info!("Cache entry for request {} is expired!", request);

                        GdcfFuture::outdated(cached.extract(), generate_future(self, request))
                    } else {
                        GdcfFuture::up_to_date(cached.extract())
                    },

                Err(CacheError::CacheMiss) => {
                    info!("No cache entry for request {}", request);

                    GdcfFuture::absent(generate_future(self, request))
                },

                Err(err) => return Either::B(result(Err(GdcfError::Cache(err)))),
            };

            Either::A(gdcf_future)
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
