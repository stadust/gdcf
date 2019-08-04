macro_rules! __match_arm_expr {
    // Custom parser function
    (! $field_name: ident, $value: expr, index = $idx: expr, parse = $external: ident) => {{
        $field_name = match $external::robtop_from($value) {
            Err(err) => return Err(ValueError::Parse(stringify!($idx), $value, err)),
            Ok(v) => Some(v),
        }
    }};

    // Custom parser that cannot fail
    (! $field_name: ident, $value: expr, index = $idx: expr, parse_infallible = $external: ident) => {{
        $field_name = Some($external::robtop_from_infallible($value))
    }};

    // Built-in parsing
    (! $field_name: ident, $value: expr, index = $idx: expr) => {{
        $field_name = parse(stringify!($idx), $value)?
    }};

    (@  $_: expr, // Closure to propagate upward values to, irrelevant here
        $field_name: ident, // Name of the field we're currently processing
        $value: expr,       // The value to be parsed
        index = $idx: expr, // The index at which we find this data in the response

        noparse  // No parsing should happen

        // Things for ignore for this macro
        $(, extract = $___: path[$($____:tt)*])? // 'extract' only matters during unparse generation
        $(, default)?  // 'default' only matters when __unwrap!
        $(, default_with = $__ :path)? // 'default_with' only matters when __unwrap!
        $(, optional)?  // 'optional' only matters for __unwrap!
        $(, optional_non_default)?  // 'optional_non_default' only matters for __unwrap!
    ) => {
        $field_name = Some($value)
    };

    (@  $_: expr, // Closure to propagate upward values to, irrelevant here
        $field_name: ident, // Name of the field we're currently processing
        $value: expr,       // The value to be parsed
        index = $idx: expr, // The index at which we find this data in the response

        ignore // The value should be ignored during parsing

        // Things for ignore for this macro
        $(, extract = $___: path[$($____:tt)*])? // 'extract' only matters during unparse generation
        $(, default)?  // 'default' only matters when __unwrap!
        $(, default_with = $__ :path)? // 'default_with' only matters when __unwrap!
        $(, optional)?  // 'optional' only matters for __unwrap!
        $(, optional_non_default)?  // 'optional_non_default' only matters for __unwrap!
    ) => {{}};

    (@  $_: expr, // Closure to propagate upward values to, irrelevant here
        $field_name: ident, // Name of the field we're currently processing
        $value: expr,       // The value to be parsed
        index = $idx: expr // The index at which we find this data in the response

        $(, parse_infallible = $p: ident)?  // External, infallible parser
        $(, parse = $p2: ident)?  // External parser

        // Things for ignore for this macro
        $(, extract = $___: path[$($____:tt)*])? // 'extract' only matters during unparse generation
        $(, default)?  // 'default' only matters when __unwrap!
        $(, default_with = $__ :path)? // 'default_with' only matters when __unwrap!
        $(, optional)?  // 'optional' only matters for __unwrap!
        $(, optional_non_default)?  // 'optional_non_default' only matters for __unwrap!
    ) => {
        __match_arm_expr!(! $field_name, $value, index = $idx $(, parse_infallible = $p)? $(, parse = $p2)?)
    };

    // The same as above, but it was indicated that the value should be propagated upward
    (@  $f: expr,
        $field_name: ident, // Name of the field we're currently processing
        $value: expr,
        ^index = $idx: expr // The index at which we find this data in the response
        $(, $($rest:tt)*)?  // We'll deal with the rest above
    ) => {{
        $f(stringify!($idx), $value)?;
        __match_arm_expr!(@ $f, $field_name, $value, index = $idx $(, $($rest)*)?)
    }}
}

macro_rules! __into_expr {
    (@ $map: expr, $value: expr, index = $idx: expr, parse = $external: ident, optional $(, $($__:tt)*)?) => {{
        if RobtopInto::<$external, _>::can_omit(&$value) {
            $map.insert(stringify!($idx), RobtopInto::<$external, _>::robtop_into($value));
        }
    }};

    (@ $map: expr, $value: expr, index = $idx: expr, parse = $external: ident, optional_non_default $(, $($__:tt)*)?) => {{
        if let Some(value) = $value {
            if !RobtopInto::<$external, _>::can_omit(&value) {
                $map.insert(stringify!($idx), RobtopInto::<$external, _>::robtop_into(value));
            }
        }
    }};

    // Custom parser function
    (@ $map: expr, $value: expr, index = $idx: expr, parse = $external: ident $(, $($__:tt)*)?) => {{
        $map.insert(stringify!($idx), RobtopInto::<$external, _>::robtop_into($value))
    }};

    // Custom parser that cannot fail
    (@ $map: expr, $value: expr, index = $idx: expr, parse_infallible = $external: ident $(, $($__:tt)*)?) => {{
        __into_expr!(@ $map, $value, index = $idx, parse = $external)
    }};

    // Built-in parsing
    (@ $map: expr, $value: expr, index = $idx: expr $(, $($__:tt)*)?) => {{
        let value = crate::util::unparse($value);
        if !crate::util::can_omit(&value) {
            $map.insert(stringify!($idx), value);
        }
    }};

    (@ $map: expr, $value: expr, index = $idx: expr $(, $($__:tt)*)?) => {{
        $map.insert(stringify!($idx), crate::util::unparse($value))
    }};

    // Unparsing of helper variables
    (! $map: expr, index = $idx: expr $(, parse = $_: ident)? $(, ignore)? $(, noparse)? $(, parse_infallible = $t: ident)?, extract = $extractor: path[$($arg: expr),*] $(, $($__:tt)*)?) => {{
        $map.insert(stringify!($idx), $extractor($($arg,)*))
    }};

    (! $map: expr, ^index = $idx: expr $(, $($__:tt)*)?) => {{
        /* we're ignoring a value because it gets propagated upward: */
    }};

    (! $($t:tt)*) => {{
        compile_error!("Please specific an extractor via `extract = <...>` for helper variables")
    }};

    (@ $($t:tt)*) => {
        /* we're ignoring a value because it gets propagated upward: */
    }
}

macro_rules! __index {
    ($(^)?index = $idx: expr $(, $($__:tt)*)?) => {
        stringify!($idx)
    };
}

macro_rules! __unwrap {
    ($field_name: ident($(^)?index = $idx: expr)) => {
        $field_name.ok_or(ValueError::NoValue(stringify!($idx)))?
    };

    ($field_name: ident($(^)?index = $idx: expr, optional)) => {
        $field_name.unwrap_or_default()
    };

    ($field_name: ident($(^)?index = $idx: expr, optional_non_default)) => {
        $field_name
    };

    ($field_name: ident($(^)?index = $idx: expr, default)) => {
        $field_name.unwrap_or_default()
    };

    ($field_name: ident($(^)?index = $idx: expr, ignore $(, $($t:tt)*)?)) => {{
        // do nothing on parse
    }};

    ($field_name: ident($(^)?index = $idx: expr, default = $default_func: path)) => {
        $field_name.unwrap_or_self($default_func)
    };

    ($field_name: ident($(^)?index = $idx: expr, parse_infallible = $_: ty $(, $($crap:tt)*)?)) => {
        __unwrap!($field_name(index = $idx $(, $($crap)*)?))
    };

    ($field_name: ident($(^)?index = $idx: expr, parse = $_: ty $(, $($crap:tt)*)?)) => {
        __unwrap!($field_name(index = $idx $(, $($crap)*)?))
    };

    ($field_name: ident($(^)?index = $idx: expr, noparse $(, $($crap:tt)*)?)) => {
        __unwrap!($field_name(index = $idx $(, $($crap)*)?))
    };

    ($field_name: ident($(^)?index = $idx: expr, extract = $_: path[$($field: expr),*] $(, $($crap:tt)*)?)) => {
        __unwrap!($field_name(index = $idx $(, $($crap)*)?))
    };
}

macro_rules! __declare {
    ($field_name: ident, index = $idx: expr, ignore $(, $($t:tt)*)?) => {{}};
    ($field_name: ident, $($t:tt)*) => {
        let mut $field_name = None;
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
        [$(, $custom_field: ident(custom = $func: path[$($field: expr),*]))*]
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
                use $crate::convert::RobtopFromInfallible;
                use $crate::convert::RobtopFrom;

                trace!("Parsing {}", stringify!($struct_name));

                $(
                    __declare!($field_name, $($tokens)*);
                )*

                $(
                    __declare!($helper_field, $($tokens2)*);
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

                trace!("Finished parsing {}", stringify!($struct_name));

                Ok(Self {
                    $(
                        $field_name,
                    )*
                    $(
                        $custom_field: $func($($field,)*),
                    )*
                })
            }

            fn unparse(self) -> std::collections::HashMap<&'static str, String> {
                use crate::convert::RobtopInto;

                let Self {
                    $(
                        $field_name,
                    )*
                    $(
                        $custom_field,
                    )*
                } = self;

                let mut map = std::collections::HashMap::new();

                $(
                    __into_expr!(! map, $($tokens2)*);
                )*

                $(
                    __into_expr!(@ map, $field_name, $($tokens)*);
                )*

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
                use $crate::convert::RobtopFromInfallible;
                use $crate::convert::RobtopFrom;

                trace!("Parsing {}", stringify!($struct_name));

                $(
                    __declare!($field_name, $($tokens)*);
                )*

                $(
                    __declare!($helper_field, $($tokens2)*);
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

                trace!("Finished parsing {}", stringify!($struct_name));

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

            fn unparse(self) -> std::collections::HashMap<&'static str, String> {
                use crate::convert::RobtopInto;

                let Self {
                    $(
                        $field_name,
                    )*
                    $(
                        $custom_field,
                    )*
                    $delegated
                } = self;

                let mut map = $delegated.unparse();//std::collections::HashMap::new();

                $(
                    __into_expr!(! map, $($tokens2)*);
                )*

                $(
                    __into_expr!(@ map, $field_name, $($tokens)*);
                )*

                map
            }
        }
    };
}
