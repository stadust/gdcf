use error::ValueError;
pub use self::level::{DemonRating, Level, LevelLength, LevelRating, PartialLevel};
pub use self::song::{MainSong, NewgroundsSong};
use std::error::Error;
use std::iter;
use std::marker::Sized;
use std::str::FromStr;

mod de;
pub mod level;
pub mod song;

#[derive(Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub enum GameVersion {
    Unknown,
    Version { minor: u8, major: u8 },
}

#[derive(Debug, Eq, PartialEq)]
pub enum ObjectType {
    PartialLevel,
    Level,
    NewgroundsSong,
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub enum Value {
    /// The value at the specified index is not provided by the endpoint the object was retrieved from
    NotProvided,
    /// The specified index has the given string value
    Value(String),
}

#[derive(Debug)]
pub struct RawObject {
    values: Vec<Value>,
    pub object_type: ObjectType,
}

pub trait FromRawObject: Sized {
    fn from_raw(obj: &RawObject) -> Result<Self, ValueError>;
}

impl RawObject {
    pub fn new(object_type: ObjectType) -> RawObject {
        RawObject {
            values: Vec::new(),
            object_type,
        }
    }

    pub fn get<T>(&self, idx: usize) -> Result<T, ValueError>
        where
            T: FromStr + 'static,
            <T as FromStr>::Err: Error,
    {
        match self.values.get(idx) {
            None => Err(ValueError::IndexOutOfBounds(idx)),
            Some(value) => match *value {
                Value::Value(ref string) => match string.parse() {
                    Ok(parsed) => Ok(parsed),
                    Err(err) => Err(ValueError::Parse(idx, string.clone(), box err)),
                },
                Value::NotProvided => Err(ValueError::NoValue(idx)),
            },
        }
    }

    pub fn get_with<T, E, F>(&self, idx: usize, f: F) -> Result<T, ValueError>
        where
            E: Error + 'static,
            F: Fn(&String) -> Result<T, E>,
    {
        match self.values.get(idx) {
            None => Err(ValueError::IndexOutOfBounds(idx)),
            Some(value) => match *value {
                Value::Value(ref string) => {
                    f(string).map_err(|err| ValueError::Parse(idx, string.clone(), box err))
                }
                Value::NotProvided => Err(ValueError::NoValue(idx)),
            },
        }
    }

    pub fn get_with_or<T, E, F>(&self, idx: usize, f: F, default: T) -> Result<T, ValueError>
        where
            E: Error + 'static,
            F: Fn(&String) -> Result<T, E>,
    {
        match self.values.get(idx) {
            None => Ok(default),
            Some(value) => match *value {
                Value::Value(ref string) => {
                    f(string).map_err(|err| ValueError::Parse(idx, string.clone(), box err))
                }
                Value::NotProvided => Ok(default),
            },
        }
    }

    pub fn get_with_or_default<T, E, F>(&self, idx: usize, f: F) -> Result<T, ValueError>
        where
            T: Default,
            E: Error + 'static,
            F: Fn(&String) -> Result<T, E>,
    {
        match self.values.get(idx) {
            None => Ok(Default::default()),
            Some(value) => match *value {
                Value::Value(ref string) => {
                    f(string).map_err(|err| ValueError::Parse(idx, string.clone(), box err))
                }
                Value::NotProvided => Ok(Default::default()),
            },
        }
    }

    pub fn get_or<T>(&self, idx: usize, default: T) -> Result<T, ValueError>
        where
            T: FromStr + 'static,
            <T as FromStr>::Err: Error,
    {
        match self.values.get(idx) {
            None => Ok(default),
            Some(value) => match *value {
                Value::Value(ref string) => match string.parse() {
                    Ok(parsed) => Ok(parsed),
                    Err(err) => Err(ValueError::Parse(idx, string.clone(), box err)),
                },
                Value::NotProvided => Ok(default),
            },
        }
    }

    pub fn get_or_default<T>(&self, idx: usize) -> Result<T, ValueError>
        where
            T: FromStr + Default + 'static,
            <T as FromStr>::Err: Error,
    {
        self.get_or(idx, Default::default())
    }

    pub fn set(&mut self, idx: usize, string: String) {
        let len = self.values.len();
        let value = if string == "" {
            Value::NotProvided
        } else {
            Value::Value(string)
        };

        if idx >= len {
            if idx > len {
                self.values
                    .extend(iter::repeat(Value::NotProvided).take(idx - len))
            }
            self.values.push(value)
        } else {
            self.values[idx] = value
        }
    }
}
