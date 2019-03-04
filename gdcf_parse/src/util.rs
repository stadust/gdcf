use std::str::FromStr;

pub struct SelfZip<I> {
    iter: I,
}

impl<I: Iterator> Iterator for SelfZip<I> {
    type Item = (I::Item, I::Item);

    fn next(&mut self) -> Option<Self::Item> {
        match (self.iter.next(), self.iter.next()) {
            (Some(a), Some(b)) => Some((a, b)),
            _ => None,
        }
    }
}

pub trait SelfZipExt: Iterator {
    fn self_zip(self) -> SelfZip<Self>
    where
        Self: Sized,
    {
        SelfZip { iter: self }
    }
}

impl<I> SelfZipExt for I where I: Iterator {}

pub fn default_to_none<T>(value: T) -> Option<T>
where
    T: FromStr + Default + PartialEq,
{
    if value == Default::default() {
        None
    } else {
        Some(value)
    }
}

pub fn int_to_bool(value: u8) -> bool {
    gdcf::convert::to::bool(value)
}

pub fn into_option<T>(value: T) -> Option<T> {
    Some(value)
}
