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

macro_rules! const_setter {
    ($name: ident, $field: ident, $t: ty) => {
        pub const fn $name(mut self, $field: $t) -> Self {
            self.$field = $field;
            self
        }
    };

    ($name: ident, $t: ty) => {
        pub const fn $name(mut self, arg0: $t) -> Self {
            self.$name = arg0;
            self
        }
    };

    ($(#[$attr:meta])* $name: ident: $t: ty) => {
        $(#[$attr])*
        pub const fn $name(mut self, $name: $t) -> Self {
            self.$name = $name;
            self
        }
    };

    ($(#[$attr:meta])* $field:ident[$name: ident]: $t: ty) => {
        $(#[$attr])*
        pub const fn $name(mut self, $field: $t) -> Self {
            self.$field = $field;
            self
        }
    }
}

macro_rules! query_upgrade {
    ($cache: expr, $cache_request: expr, $refresh_request: expr) => {{
        use crate::cache::CacheEntryMeta;

        match $cache.lookup(&$cache_request)? {
            CacheEntry::Missing => Ok(UpgradeQuery::One(Some($refresh_request), None)),
            CacheEntry::DeducedAbsent => Err(UpgradeError::UpgradeFailed),
            CacheEntry::MarkedAbsent(meta) =>
                if meta.is_expired() {
                    Ok(UpgradeQuery::One(Some($refresh_request), None))
                } else {
                    Err(UpgradeError::UpgradeFailed)
                },
            CacheEntry::Cached(user, meta) =>
                if meta.is_expired() {
                    Ok(UpgradeQuery::One(Some($refresh_request), Some(user)))
                } else {
                    Ok(UpgradeQuery::One(None, Some(user)))
                },
        }
    }};
}
