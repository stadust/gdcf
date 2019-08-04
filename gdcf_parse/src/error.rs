#[derive(Debug)]
pub enum ValueError<'a> {
    NoValue(&'a str),
    Parse(&'a str, &'a str, String),
}

impl std::error::Error for ValueError<'_> {}

impl std::fmt::Display for ValueError<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ValueError::NoValue(idx) => write!(f, "No value provided at index {}", idx),
            ValueError::Parse(idx, value, cause) => write!(f, "The value '{}' at index {} could not be parsed: {}", value, idx, cause),
        }
    }
}
