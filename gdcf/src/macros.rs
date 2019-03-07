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
