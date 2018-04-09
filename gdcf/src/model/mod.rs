pub mod level;

use std::str::FromStr;

pub use self::level::{LevelRating, LevelLength, DemonRating, Level, PartialLevel};
use std::error::Error;
use std::iter;

pub enum GDObject {
    Level(Level)
}

#[derive(Clone, Eq, PartialEq)]
pub enum Value {
    /// The value at the specified index is not provided by the endpoint the object was retrieved from
    NotProvided,
    /// The specified index has the given string value
    Value(String),
}

pub enum ValueError<E>
{
    IndexOutOfBounds,
    NoValue,
    Parse(E),
}

pub struct RawObject {
    values: Vec<Value>
}

impl<E> From<E> for ValueError<E>
{
    fn from(err: E) -> Self {
        ValueError::Parse(err)
    }
}

impl RawObject {
    fn new() -> RawObject {
        RawObject {
            values: Vec::new()
        }
    }

    pub fn get<T>(&self, idx: usize) -> Result<T, ValueError<<T as FromStr>::Err>>
        where
            T: FromStr,
    {
        match self.values.get(idx) {
            None => Err(ValueError::IndexOutOfBounds),
            Some(value) => parse(value)
        }
    }

    pub fn get_or<T>(&self, idx: usize, default: T) -> Result<T, ValueError<<T as FromStr>::Err>>
        where
            T: FromStr,
    {
        match self.values.get(idx) {
            None => Ok(default),
            Some(value) => parse(value)
        }
    }

    pub fn get_or_default<T>(&self, idx: usize) -> Result<T, ValueError<<T as FromStr>::Err>>
        where
            T: FromStr + Default,
    {
        self.get_or(idx, Default::default())
    }

    pub fn set(&mut self, idx: usize, string: String) {
        let len = self.values.len();

        if (idx >= len) {
            self.values.extend(iter::repeat(Value::NotProvided).take(idx - len + 1));
        }

        self.values[idx] = Value::Value(string)
    }
}

fn parse<T>(value: &Value) -> Result<T, ValueError<<T as FromStr>::Err>>
    where
        T: FromStr,
{
    match *value {
        Value::Value(ref string) => Ok(string.parse()?),
        Value::NotProvided => Err(ValueError::NoValue)
    }
}

