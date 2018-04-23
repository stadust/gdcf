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
        pub fn $api(&self, req: $req_type) -> Option<<$req_type as Request>::Result> {
            debug!("Received request {}", req);

            let (cached, expired) = lookup!(self, $lookup, req);

            if expired {
                info!("Cache entry for {} is either expired or non existant, refreshing!", req);

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