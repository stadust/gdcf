pub mod level;
pub mod song;

use std::str::FromStr;

pub use self::level::{LevelRating, LevelLength, DemonRating, Level, PartialLevel};
use std::error::Error;
use std::iter;
use std::marker::Sized;

#[derive(Debug)]
pub enum GameVersion {
    Unknown,
    Version { minor: u8, major: u8 },
}

pub enum GDObject {
    Level(Level)
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub enum Value {
    /// The value at the specified index is not provided by the endpoint the object was retrieved from
    NotProvided,
    /// The specified index has the given string value
    Value(String),
}

#[derive(Debug)]
pub enum ValueError
{
    IndexOutOfBounds(usize),
    NoValue(usize),
    Parse(usize, Box<Error>),
}

#[derive(Debug)]
pub struct RawObject {
    values: Vec<Value>
}

pub trait FromRawObject: Sized {
    fn from_raw(obj: &RawObject) -> Result<Self, ValueError>;
}

impl RawObject {
    pub fn new() -> RawObject {
        RawObject {
            values: Vec::new()
        }
    }

    pub fn get<T: 'static>(&self, idx: usize) -> Result<T, ValueError>
        where
            T: FromStr,
            <T as FromStr>::Err: Error
    {
        match self.values.get(idx) {
            None => Err(ValueError::IndexOutOfBounds(idx)),
            Some(value) => {
                match *value {
                    Value::Value(ref string) => match string.parse() {
                        Ok(parsed) => Ok(parsed),
                        Err(err) => Err(ValueError::Parse(idx, box err))
                    },
                    Value::NotProvided => Err(ValueError::NoValue(idx))
                }
            }
        }
    }

    pub fn get_with<T, E: 'static, F>(&self, idx: usize, f: F) -> Result<T, ValueError>
        where
            E: Error,
            F: Fn(&String) -> Result<T, E>
    {
        match self.values.get(idx) {
            None => Err(ValueError::IndexOutOfBounds(idx)),
            Some(value) => {
                match *value {
                    Value::Value(ref string) => f(string).map_err(|err| ValueError::Parse(idx, box err)),
                    Value::NotProvided => Err(ValueError::NoValue(idx))
                }
            }
        }
    }

    pub fn get_with_or<T, E: 'static, F>(&self, idx: usize, f: F, default: T) -> Result<T, ValueError>
        where
            E: Error,
            F: Fn(&String) -> Result<T, E>
    {
        match self.values.get(idx) {
            None => Ok(default),
            Some(value) => {
                match *value {
                    Value::Value(ref string) => f(string).map_err(|err| ValueError::Parse(idx, box err)),
                    Value::NotProvided => Ok(default)
                }
            }
        }
    }

    pub fn get_with_or_default<T, E: 'static, F>(&self, idx: usize, f: F) -> Result<T, ValueError>
        where
            T: Default,
            E: Error,
            F: Fn(&String) -> Result<T, E>
    {
        match self.values.get(idx) {
            None => Ok(Default::default()),
            Some(value) => {
                match *value {
                    Value::Value(ref string) => f(string).map_err(|err| ValueError::Parse(idx, box err)),
                    Value::NotProvided => Ok(Default::default())
                }
            }
        }
    }

    pub fn get_or<T: 'static>(&self, idx: usize, default: T) -> Result<T, ValueError>
        where
            T: FromStr,
            <T as FromStr>::Err: Error
    {
        match self.values.get(idx) {
            None => Ok(default),
            Some(value) => {
                match *value {
                    Value::Value(ref string) => match string.parse() {
                        Ok(parsed) => Ok(parsed),
                        Err(err) => Err(ValueError::Parse(idx, box err))
                    },
                    Value::NotProvided => Ok(default)
                }
            }
        }
    }

    pub fn get_or_default<T: 'static>(&self, idx: usize) -> Result<T, ValueError>
        where
            T: FromStr + Default,
            <T as FromStr>::Err: Error
    {
        self.get_or(idx, Default::default())
    }

    pub fn set(&mut self, idx: usize, string: String) {
        let len = self.values.len();
        let value = if string == "" { Value::NotProvided } else { Value::Value(string) };

        if idx >= len {
            if idx > len {
                self.values.extend(iter::repeat(Value::NotProvided).take(idx - len))
            }
            self.values.push(value)
        } else {
            self.values[idx] = value
        }
    }
}

