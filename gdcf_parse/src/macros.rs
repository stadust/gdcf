macro_rules! __match_arm_expr {
    // Custom parser function
    (@ $_: expr, $field_name: ident, $value: expr, index = $idx: expr, parse = $external: ident) => {{
        use $crate::convert::RobtopFrom;
        $field_name = match $external::robtop_from($value) {
            Err(err) => return Err(ValueError::Parse(stringify!($idx), $value, err)),
            Ok(v) => Some(v),
        }
    }};

    (@ $_: expr, $field_name: ident, $value: expr, index = $idx: expr, parse = $external: ident, $($__:tt)*) => {{
        __match_arm_expr!(@ $_, $field_name, $value, index = $idx, parse = $external)
    }};

    // Custom parser that cannot fail
    (@ $_: expr, $field_name: ident, $value: expr, index = $idx: expr, parse_infallible = $external: ident) => {{
        use $crate::convert::RobtopFromInfallible;
        $field_name = Some($external::robtop_from_infallible($value))
    }};

    (@ $_: expr, $field_name: ident, $value: expr, index = $idx: expr, parse_infallible = $external: ident, $($__:tt)*) => {{
        __match_arm_expr!(@ $_, $field_name, $value, index = $idx, parse_infallible = $external)
    }};

    // no parsing at all
    (@ $_: expr, $field_name: ident, $value: expr, index = $idx: expr, noparse) => {{
        $field_name = Some($value)
    }};

    (@ $_: expr, $field_name: ident, $value: expr, index = $idx: expr, noparse, $($__:tt)*) => {{
        __match_arm_expr!(@ $_, $field_name, $value, index = $idx, noparse)
    }};

    // Built-in parsing
    (@ $_: expr, $field_name: ident, $value: expr, index = $idx: expr) => {{
        $field_name = parse(stringify!($idx), $value)?
    }};

    (@ $_: expr, $field_name: ident, $value: expr, index = $idx: expr, $($__:tt)*) => {{
        __match_arm_expr!(@ $_, $field_name, $value, index = $idx)
    }};

    // Custom parser function, but delegate original value upward
    (@ $f: expr, $field_name: ident, $value: expr, ^index = $idx: expr, parse = $external: ident) => {{
        $f($idx, $value)?;
        __match_arm_expr!(@ $f, $field_name, $value, index = $idx, parse = $external)
    }};

    (@ $f: expr, $field_name: ident, $value: expr, ^index = $idx: expr, parse = $external: ident, $($__:tt)*) => {{
        __match_arm_expr!(@ $f, $field_name, $value, ^index = $idx, parse = $external)
    }};

    // Custom parser that cannot fail, but delegate the value upward
    (@ $f: expr, $field_name: ident, $value: expr, ^index = $idx: expr, parse_infallible = $external: ident) => {{
        $f(stringify!($idx), $value)?;
        __match_arm_expr!(@ $f, $field_name, $value, index = $idx, parse_infallible = $external)
    }};

    (@ $f: expr, $field_name: ident, $value: expr, ^index = $idx: expr, parse_infallible = $external: ident, $($also_tokens:tt)*) => {{
        __match_arm_expr!(@ $f, $field_name, $value, ^index = $idx, parse_infallible = $external)
    }};

    // No parsing, but delegate the value upward
    (@ $f: expr, $field_name: ident, $value: expr, ^index = $idx: expr, noparse) => {{
        $f(stringify!($idx), $value)?;
        __match_arm_expr!(@ $f, $field_name, $value, index = $idx, noparse)
    }};

    (@ $f: expr, $field_name: ident, $value: expr, ^index = $idx: expr, noparse, $($also_tokens:tt)*) => {{
        $f(stringify!($idx), $value)?;
        __match_arm_expr!(@ $f, $field_name, $value, ^index = $idx, noparse)
    }};

    // Build-in parsing, but delegate the value upward
    (@ $f: expr, $field_name: ident, $value: expr, ^index = $idx: expr, $($also_tokens:tt)*) => {{
        $f(stringify!($idx), $value)?;
        __match_arm_expr!(@ $f, $field_name, $value, index = $idx)
    }};

    (@ $f: expr, $field_name: ident, $value: expr, ^index = $idx: expr) => {{
        __match_arm_expr!(@ $f, $field_name, $value, ^index = $idx)
    }};
}

macro_rules! __index {
    ($(^)?index = $idx: expr) => {
        stringify!($idx)
    };

    ($(^)?index = $idx: expr, $($also_tokens:tt)*) => {
        stringify!($idx)
    };
}

macro_rules! __unwrap {
    ($field_name: ident($(^)?index = $idx: expr $(,noparse)?)) => {
        $field_name.ok_or(ValueError::NoValue(stringify!($idx)))?
    };

    ($field_name: ident($(^)?index = $idx: expr, optional)) => {
        $field_name
    };

    ($field_name: ident($(^)?index = $idx: expr, default)) => {
        $field_name.unwrap_or_default()
    };

    ($field_name: ident($(^)?index = $idx: expr $(,noparse)?, default = $default_func: path)) => {
        $field_name.unwrap_or_self($default_func)
    };

    ($field_name: ident($(^)?index = $idx: expr $(,noparse)?, parse_infallible = $_: ty $(, $($crap:tt)*)?)) => {
        __unwrap!($field_name(index = $idx $(, $($crap)*)?))
    };

    ($field_name: ident($(^)?index = $idx: expr $(,noparse)?, parse = $_: ty $(, $($crap:tt)*)?)) => {
        __unwrap!($field_name(index = $idx $(, $($crap)*)?))
    };
}

macro_rules! parser {
    ($struct_name: ty => {$($tokens:tt)*}$(, $($tokens2:tt)*)?) => {
        parser!(@ $struct_name [] [] [] [$($tokens)*] [$($($tokens2)*)?]);
    };

    (@ $struct_name: ty [$($crap:tt)*] [] [$($crap3:tt)*] [$field_name: ident(custom $($data: tt)*), $($tokens:tt)*] [$($rest:tt)*]) => {
        parser!(@ $struct_name [$($crap)*] [] [$($crap3)*, $field_name(custom $($data)*)] [$($tokens)*] [$($rest)*]);
    };

    (@ $struct_name: ty [$($crap:tt)*] [] [$($crap3:tt)*] [$field_name: ident(delegate), $($tokens:tt)*] [$($rest:tt)*]) => {
        parser!(@@ $struct_name, $field_name [$($crap)*] [] [$($crap3)*] [$($tokens)*] [$($rest)*]);
    };

    (@ $struct_name: ty [$($crap:tt)*] [] [$($crap3:tt)*] [$field_name: ident($($data: tt)*), $($tokens:tt)*] [$($rest:tt)*]) => {
        parser!(@ $struct_name [$($crap)*, $field_name($($data)*)] [] [$($crap3)*] [$($tokens)*] [$($rest)*]);
    };

    (@ $struct_name: ty [$($crap:tt)*] [$($crap2:tt)*] [$($crap3:tt)*] [] [$field_name: ident($($data: tt)*), $($rest:tt)*]) => {
        parser!(@ $struct_name [$($crap)*] [$($crap2)*, $field_name($($data)*)] [$($crap3)*] [] [$($rest)*]);
    };

    (@ $struct_name: ty
        [$(, $field_name: ident($($tokens:tt)*))*]
        [$(, $helper_field: ident($($tokens2:tt)*))*]
        [$(, $custom_field: ident(custom = $func: path, depends_on = [$($field: expr),*]))*]
        [] []
    ) => {
        impl Parse for $struct_name {
            #[inline]
            fn parse<'a, I, F>(iter: I, mut f: F) -> Result<Self, ValueError<'a>>
            where
                I: Iterator<Item = (&'a str, &'a str)> + Clone,
                F: FnMut(&'a str, &'a str) -> Result<(), ValueError<'a>>
            {
                use $crate::util::parse;

                trace!("Parsing {}", stringify!($struct_name));

                $(
                    let mut $field_name = None;
                )*

                $(
                    let mut $helper_field = None;
                )*

                for (idx, value) in iter.into_iter() {
                    match idx {
                        $(
                            __index!($($tokens)*) => __match_arm_expr!(@ f, $field_name, value, $($tokens)*),
                        )*
                        $(
                            __index!($($tokens2)*) => __match_arm_expr!(@ f, $helper_field, value, $($tokens2)*),
                        )*
                        _ => f(idx, value)?
                    }
                }

                $(
                    let $field_name = __unwrap!($field_name($($tokens)*));
                )*

                $(
                    let $helper_field = __unwrap!($helper_field($($tokens2)*));
                )*

                Ok(Self {
                    $(
                        $field_name,
                    )*
                    $(
                        $custom_field: $func($($field,)*),
                    )*
                })
            }

            fn unparse(self) -> std::collections::HashMap<String, String> {
                //use crate::convert::RobtopConvert;

                let Self {
                    $(
                        $field_name,
                    )*
                    $(
                        $custom_field,
                    )*
                } = self;

                let mut map = std::collections::HashMap::new();

                /*$(
                    map.insert(__index!($($tokens)*).to_string(), RobtopConvert::<_, String, str>::robtop_into($field_name));
                )**/

                map
            }
        }
    };

    (@@ $struct_name: ty, $delegated: ident [$($crap:tt)*] [] [$($crap3:tt)*] [$field_name: ident(custom $($data: tt)*), $($tokens:tt)*] [$($rest:tt)*]) => {
        parser!(@@ $struct_name, $delegated [$($crap)*] [] [$($crap3)*, $field_name(custom $($data)*)] [$($tokens)*] [$($rest)*]);
    };

    (@@ $struct_name: ty, $delegated: ident [$($crap:tt)*] [] [$($crap3:tt)*] [$field_name: ident($($data: tt)*), $($tokens:tt)*] [$($rest:tt)*]) => {
        parser!(@@ $struct_name, $delegated [$($crap)*, $field_name($($data)*)] [] [$($crap3)*] [$($tokens)*] [$($rest)*]);
    };

    (@@ $struct_name: ty, $delegated: ident [$($crap:tt)*] [$($crap2:tt)*] [$($crap3:tt)*] [] [$field_name: ident($($data: tt)*), $($rest:tt)*]) => {
        parser!(@@ $struct_name, $delegated [$($crap)*] [$($crap2)*, $field_name($($data)*)] [$($crap3)*] [] [$($rest)*]);
    };

    (@@ $struct_name: ty, $delegated: ident
        [$(, $field_name: ident($($tokens:tt)*))*]
        [$(, $helper_field: ident($($tokens2:tt)*))*]
        [$(, $custom_field: ident(custom = $func: path, depends_on = [$($field: expr),*]))*]
        [] []
    ) => {
        impl Parse for $struct_name {
            #[inline]
            fn parse<'a, I, F>(iter: I, mut f: F) -> Result<Self, ValueError<'a>>
            where
                I: Iterator<Item = (&'a str, &'a str)> + Clone,
                F: FnMut(&'a str, &'a str) -> Result<(), ValueError<'a>>
            {
                use $crate::util::parse;

                trace!("Parsing {}", stringify!($struct_name));

                $(
                    let mut $field_name = None;
                )*

                $(
                    let mut $helper_field = None;
                )*

                let closure = |idx: &'a str, value: &'a str| -> Result<(), ValueError<'a>> {
                    match idx {
                        $(
                            __index!($($tokens)*) => __match_arm_expr!(@ f, $field_name, value, $($tokens)*),
                        )*
                        $(
                            __index!($($tokens2)*) => __match_arm_expr!(@ f, $helper_field, value, $($tokens2)*),
                        )*
                        _ => f(idx, value)?
                    }

                    Ok(())
                };

                let $delegated = Parse::parse(iter, closure)?;

                $(
                    let $field_name = __unwrap!($field_name($($tokens)*));
                )*

                $(
                    let $helper_field = __unwrap!($helper_field($($tokens2)*));
                )*

                Ok(Self {
                    $delegated,
                    $(
                        $field_name,
                    )*
                    $(
                        $custom_field: $func($($field,)*),
                    )*
                })
            }
        }
    };
}
