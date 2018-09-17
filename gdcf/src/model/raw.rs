use error::ValueError;
use std::{iter, str::FromStr};
use failure::Fail;

#[derive(Clone, Eq, PartialEq, Debug)]
pub enum Value {
    /// The value at the specified index is not provided by the endpoint the
    /// object was retrieved from
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
        <T as FromStr>::Err: Fail,
    {
        match self.values.get(idx) {
            None => Err(ValueError::NoValue(idx)),
            Some(value) =>
                match *value {
                    Value::Value(ref string) =>
                        match string.parse() {
                            Ok(parsed) => Ok(parsed),
                            Err(err) => Err(ValueError::Parse(idx, string.clone(), Box::new(err))),
                        },
                    Value::NotProvided => Err(ValueError::NoValue(idx)),
                },
        }
    }

    pub fn get_with<T, E, F>(&self, idx: usize, f: F) -> Result<T, ValueError>
    where
        E: Fail,
        F: Fn(&str) -> Result<T, E>,
    {
        match self.values.get(idx) {
            None => Err(ValueError::NoValue(idx)),
            Some(value) =>
                match *value {
                    Value::Value(ref string) => f(string).map_err(|err| ValueError::Parse(idx, string.clone(), Box::new(err))),
                    Value::NotProvided => Err(ValueError::NoValue(idx)),
                },
        }
    }

    pub fn get_with_or<T, E, F>(&self, idx: usize, f: F, default: T) -> Result<T, ValueError>
    where
        E: Fail,
        F: Fn(&str) -> Result<T, E>,
    {
        match self.values.get(idx) {
            None => Ok(default),
            Some(value) =>
                match *value {
                    Value::Value(ref string) => f(string).map_err(|err| ValueError::Parse(idx, string.clone(), Box::new(err))),
                    Value::NotProvided => Ok(default),
                },
        }
    }

    pub fn get_with_or_default<T, E, F>(&self, idx: usize, f: F) -> Result<T, ValueError>
    where
        T: Default,
        E: Fail,
        F: Fn(&str) -> Result<T, E>,
    {
        match self.values.get(idx) {
            None => Ok(Default::default()),
            Some(value) =>
                match *value {
                    Value::Value(ref string) => f(string).map_err(|err| ValueError::Parse(idx, string.clone(), Box::new(err))),
                    Value::NotProvided => Ok(Default::default()),
                },
        }
    }

    pub fn get_or<T>(&self, idx: usize, default: T) -> Result<T, ValueError>
    where
        T: FromStr + 'static,
        <T as FromStr>::Err: Fail,
    {
        match self.values.get(idx) {
            None => Ok(default),
            Some(value) =>
                match *value {
                    Value::Value(ref string) =>
                        match string.parse() {
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
        <T as FromStr>::Err: Fail,
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

impl Default for RawObject {
    fn default() -> Self {
        RawObject { values: Vec::new() }
    }
}
