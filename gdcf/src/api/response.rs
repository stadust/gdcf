use model::GDObject;

use std::iter;
use std::iter::Once;
use std::slice::Iter;
use std::vec::IntoIter;

pub enum ProcessedResponse {
    One(GDObject),
    Many(Vec<GDObject>),
}

impl ProcessedResponse {
    pub fn iter(&self) -> RespIter {
        match *self {
            ProcessedResponse::One(ref obj) => RespIter::One(iter::once(obj)),
            ProcessedResponse::Many(ref objs) => RespIter::Many(objs.iter())
        }
    }
}

impl IntoIterator for ProcessedResponse {
    type Item = GDObject;
    type IntoIter = RespIntoIter;

    fn into_iter(self) -> RespIntoIter {
        match self {
            ProcessedResponse::One(obj) => RespIntoIter::One(iter::once(obj)),
            ProcessedResponse::Many(objs) => RespIntoIter::Many(objs.into_iter())
        }
    }
}

impl<'a> IntoIterator for &'a ProcessedResponse {
    type Item = &'a GDObject;
    type IntoIter = RespIter<'a>;

    fn into_iter(self) -> RespIter<'a> {
        self.iter()
    }
}

pub enum RespIntoIter {
    One(Once<GDObject>),
    Many(IntoIter<GDObject>),
}

impl Iterator for RespIntoIter {
    type Item = GDObject;

    fn next(&mut self) -> Option<<Self as Iterator>::Item> {
        match *self {
            RespIntoIter::One(ref mut once) => once.next(),
            RespIntoIter::Many(ref mut i) => i.next()
        }
    }
}

pub enum RespIter<'a> {
    One(Once<&'a GDObject>),
    Many(Iter<'a, GDObject>),
}

impl<'a> Iterator for RespIter<'a> {
    type Item = &'a GDObject;

    fn next(&mut self) -> Option<<Self as Iterator>::Item> {
        match *self {
            RespIter::One(ref mut once) => once.next(),
            RespIter::Many(ref mut i) => i.next()
        }
    }
}