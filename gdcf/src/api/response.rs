use model::RawObject;

use std::iter;
use std::iter::Once;
use std::slice::Iter;

pub enum ProcessedResponse {
    One(RawObject),
    Many(Vec<RawObject>),
}

impl ProcessedResponse {
    pub fn iter(&self) -> RespIter {
        match *self {
            ProcessedResponse::One(ref obj) => RespIter::One(iter::once(obj)),
            ProcessedResponse::Many(ref objs) => RespIter::Many(objs.iter())
        }
    }
}

pub enum RespIter<'a> {
    One(Once<&'a RawObject>),
    Many(Iter<'a, RawObject>),
}

impl<'a> Iterator for RespIter<'a> {
    type Item = &'a RawObject;

    fn next(&mut self) -> Option<<Self as Iterator>::Item> {
        match *self {
            RespIter::One(ref mut once) => once.next(),
            RespIter::Many(ref mut i) => i.next()
        }
    }
}