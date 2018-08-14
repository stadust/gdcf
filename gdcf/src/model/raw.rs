use error::ValueError;
use std::error::Error;
use std::iter;
use std::str::FromStr;

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
}

impl RawObject {
    pub fn new() -> RawObject {
        RawObject::default()
    }

    pub fn get<T>(&self, idx: usize) -> Result<T, ValueError>
        where
            T: FromStr + 'static,
            <T as FromStr>::Err: Error + Send + 'static,
    {
        match self.values.get(idx) {
            None => Err(ValueError::IndexOutOfBounds(idx)),
            Some(value) => match *value {
                Value::Value(ref string) => match string.parse() {
                    Ok(parsed) => Ok(parsed),
                    Err(err) => Err(ValueError::Parse(idx, string.clone(), Box::new(err))),
                },
                Value::NotProvided => Err(ValueError::NoValue(idx)),
            },
        }
    }

    pub fn get_with<T, E, F>(&self, idx: usize, f: F) -> Result<T, ValueError>
        where
            E: Error + Send + 'static,
            F: Fn(&str) -> Result<T, E>,
    {
        match self.values.get(idx) {
            None => Err(ValueError::IndexOutOfBounds(idx)),
            Some(value) => match *value {
                Value::Value(ref string) => {
                    f(string).map_err(|err| ValueError::Parse(idx, string.clone(), Box::new(err)))
                }
                Value::NotProvided => Err(ValueError::NoValue(idx)),
            },
        }
    }

    pub fn get_with_or<T, E, F>(&self, idx: usize, f: F, default: T) -> Result<T, ValueError>
        where
            E: Error + Send + 'static,
            F: Fn(&str) -> Result<T, E>,
    {
        match self.values.get(idx) {
            None => Ok(default),
            Some(value) => match *value {
                Value::Value(ref string) => {
                    f(string).map_err(|err| ValueError::Parse(idx, string.clone(), Box::new(err)))
                }
                Value::NotProvided => Ok(default),
            },
        }
    }

    pub fn get_with_or_default<T, E, F>(&self, idx: usize, f: F) -> Result<T, ValueError>
        where
            T: Default,
            E: Error + Send + 'static,
            F: Fn(&str) -> Result<T, E>,
    {
        match self.values.get(idx) {
            None => Ok(Default::default()),
            Some(value) => match *value {
                Value::Value(ref string) => {
                    f(string).map_err(|err| ValueError::Parse(idx, string.clone(), Box::new(err)))
                }
                Value::NotProvided => Ok(Default::default()),
            },
        }
    }

    pub fn get_or<T>(&self, idx: usize, default: T) -> Result<T, ValueError>
        where
            T: FromStr + 'static,
            <T as FromStr>::Err: Error + Send + 'static,
    {
        match self.values.get(idx) {
            None => Ok(default),
            Some(value) => match *value {
                Value::Value(ref string) => match string.parse() {
                    Ok(parsed) => Ok(parsed),
                    Err(err) => Err(ValueError::Parse(idx, string.clone(), Box::new(err))),
                },
                Value::NotProvided => Ok(default),
            },
        }
    }

    pub fn get_or_default<T>(&self, idx: usize) -> Result<T, ValueError>
        where
            T: FromStr + Default + 'static,
            <T as FromStr>::Err: Error + Send + 'static,
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

impl Default for RawObject {
    fn default() -> Self {
        RawObject {
            values: Vec::new()
        }
    }
}