macro_rules! __parsing {
    (@ $value: expr, index = $idx: expr, parse = $func: path) => {
        match $func($value) {
            Err(err) => return Err(ValueError::Parse($idx, $value, err)),
            Ok(v) => v,
        }
    };
    (@ $value: expr, index = $idx: expr, parse = $func: path, $($also_tokens:tt)*) => {
        match $func($value) {
            Err(err) => return Err(ValueError::Parse($idx, $value, err)),
            Ok(v) => v,
        }
    };

    (@ $value: expr, index = $idx: expr, with = $func: path) => {
        $func(parse($idx, $value)?)
    };

    (@ $value: expr, index = $idx: expr, with = $func: path, $($also_tokens:tt)*) => {
        $func(parse($idx, $value)?)
    };

    (@ $value: expr, index = $idx: expr, parse_infallible = $func: path) => {
        $func($value)
    };

    (@ $value: expr, index = $idx: expr, parse_infallible = $func: path, $($also_tokens:tt)*) => {
        $func($value)
    };

    (@ $value: expr, index = $idx: expr, $($also_tokens:tt)*) => {
        parse($idx, $value)?
    };

    (@ $value: expr, index = $idx: expr) => {
        parse($idx, $value)?
    };
}

macro_rules! __index {
    (index = $idx: expr) => {
        stringify!($idx)
    };

    (index = $idx: expr, $($also_tokens:tt)*) => {
        stringify!($idx)
    };
}

macro_rules! __unwrap {
    ($field_name: ident(index = $idx: expr)) => {
        $field_name.ok_or(ValueError::NoValue($idx))?
    };

    ($field_name: ident(index = $idx: expr, default)) => {
        $field_name.unwrap_or_default()
    };

    ($field_name: ident(index = $idx: expr, default = $default_func: path)) => {
        $field_name.unwrap_or_self($default_func)
    };

    // FIXME: this
    ($field_name: ident(index = $idx: expr, $thing: ident = $a: expr, $($crap:tt)*)) => {
        __unwrap!($field_name(index = $idx, $($crap)*))
    };

    ($field_name: ident(index = $idx: expr, $($crap:tt)*)) => {
        $field_name.ok_or(ValueError::NoValue($idx))?
    }
}

macro_rules! parser {
    ($struct_name: ty => {$($tokens:tt)*}, $($tokens2:tt)*) => {
        parser!(@ $struct_name [] [] [] [$($tokens)*] [$($tokens2)*]);
    };

    /*(@@ $iter: ident, ($delegate:path, $df: ident) [$($crap:tt)*] $field_name: ident($($data: tt)*), $($tokens:tt)*) => {
        parser!(@@ iter, ($delegate, $df) [$field_name($($data)*), $($crap)*] $($tokens)*)
    };

    (@@ $iter: ident, ($delegate: path, $delegate_field: ident) [$($field_name: ident($($tokens:tt)*),)*]) => {{
        $(
            let mut $field_name = None;
        )*

        let closure = |idx: &'a str, value: &'a str| -> Result<(), ValueError<'a>> {
            match idx {
                $(
                    __index!($($tokens)*) => $field_name = Some(__parsing!(@ value, $($tokens)*)),
                )*
            }
        }

        Ok(Self {
            $delegate_field: $delegate($iter, closure),
            $(
                $field_name: __unwrap!($field_name($($tokens)*)),
            )*
        })
    }};

    (@ $iter: ident [$($fn: ident($($ts:tt)*)),*] $field_name: ident(delegate = $delegation: path), $($tokens:tt)*) => {
        parser!(@@ $iter, ($delegation, $field_name) [$($fn($($ts)*),)*] $($tokens)*)
    };*/
    (@ $struct_name: ty [$($crap:tt)*] [] [$($crap3:tt)*] [$field_name: ident(custom $($data: tt)*), $($tokens:tt)*] [$($rest:tt)*]) => {
        parser!(@ $struct_name [$($crap)*] [] [$($crap3)*, $field_name(custom $($data)*)] [$($tokens)*] [$($rest)*]);
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
            fn parse<'a, I, F>(iter: SelfZip<I>, mut f: F) -> Result<Self, ValueError<'a>>
            where
                I: Iterator<Item = &'a str>,
                F: FnMut(&'a str, &'a str) -> Result<(), ValueError<'a>>
            {
                $(
                    let mut $field_name = None;
                )*

                $(
                    let mut $helper_field = None;
                )*

                for (idx, value) in iter {
                    match idx {
                        $(
                            __index!($($tokens)*) => $field_name = Some(__parsing!(@ value, $($tokens)*)),
                        )*
                        $(
                            __index!($($tokens2)*) => $helper_field = Some(__parsing!(@ value, $($tokens2)*)),
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
        }
    };

    /*(@ $iter: ident [$($crap:tt)*] $field_name: ident($($data: tt)*), $($tokens:tt)*) => {
        parser!(@ $iter [$field_name($($data)*), $($crap)*] $($tokens)*)
    };

    (@ $iter: ident [$($field_name: ident($($tokens:tt)*),)*]) => {{
        $(
            let mut $field_name = None;
        )*

        for (idx, value) in $iter {
            match idx {
                $(
                    __index!($($tokens)*) => $field_name = Some(__parsing!(@ value, $($tokens)*)),
                )*
            }
        }

        Ok(Self {
            $(
                $field_name: __unwrap!($field_name($($tokens)*)),
            )*
        })
    }}*/
}
