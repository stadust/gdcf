macro_rules! lookup {
    ($self: expr, $lookup: ident, $req: expr) => {{
        debug!("Initiating cache lookup for {}", $req);

        let cache = $self.cache();
        let cached = cache.$lookup(&$req);
        let expired = cached.as_ref().map_or(true, |co| cache.is_expired(co));

        (cached, expired)
    }};
}

#[cfg(not(ensure_cache_integrity))]
macro_rules! retrieve_one {
    ($api: ident, $req_type: tt, $lookup: ident) => {
        pub fn $api(&self, req: $req_type) -> Option<<$req_type as Request>::Result> {
            debug!("Received request {}", req);

            let (cached, expired) = lookup!(self, $lookup, req);

            if expired {
                info!("Cache entry for {} is either expired or non existant, refreshing!", req);

                let future = req.make(&*self.client());

                self.client().spawn(store_one(self.sender.clone(), future));
            }

            cached.map(|co| co.extract())
        }
    }
}


#[cfg(ensure_cache_integrity)]
macro_rules! retrieve_one {
    ($api: ident, $req_type: tt, $lookup: ident) => {
        pub fn $api(&self, req: $req_type) -> Option<<$req_type as Request>::Result> {
            debug!("Received request {}", req);

            let (cached, expired) = lookup!(self, $lookup, req);

            if expired {
                info!("Cache entry for {} is either expired or non existant, refreshing!", req);

                let cache = Arc::clone(&self.cache);
                let client = Arc::clone(&self.client);
                let sender = self.sender.clone();

                let future = req.make(&*self.client())
                    .and_then(move |raw_object| {
                        let integrity_req = concat_idents!($api, _integrity)(&*cache.lock().unwrap(), &raw_object)?;

                        if let Some(req) = integrity_req {
                            info!("Integrity for result is not given, making integrity request {}", req)

                            store_many(sender.clone(), req.make(&*client.lock().unwrap()))
                                .map(|_| sender.send(raw_object).unwrap());
                        } else {
                            sender.send(raw_object).unwrap()
                        }

                        Ok(())
                    })
                    .map_err(|e| error!("Unexpected error while retrieving integrity data for cache: {:?}", e));

                self.client().spawn(future);
            }

            cached.map(|co| co.extract())
        }
    }
}

macro_rules! retrieve_many {
    ($api: ident, $req_type: tt, $lookup: ident) => {
        pub fn $api(&self, req: $req_type) -> Option<<$req_type as Request>::Result> {
            debug!("Received request {}", req);

            let (cached, expired) = lookup!(self, $lookup, req);

            if expired {
                info!("Cache entry for {} is either expired or non existant, refreshing!", req);

                let future = req.make(&*self.client());
                self.client().spawn(store_many(self.sender.clone(), future));
            }

            cached.map(|co| co.extract())
        }
    }
}

macro_rules! store {
    ($cache: expr, $store: ident, $raw_obj: expr) => {
        {
            FromRawObject::from_raw(&$raw_obj).map(|constructed|{
                debug!("Caching {}!", constructed);
                $cache.$store(constructed)
            })
        }
    };
}