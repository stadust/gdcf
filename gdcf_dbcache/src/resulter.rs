//! Utility method for dealing with iterators over results

pub trait Resulter {
    fn partition_results<T, E, B, C>(self) -> (B, C)
    where
        Self: Sized + Iterator<Item = Result<T, E>>,
        B: Default + Extend<T>,
        C: Default + Extend<E>,
    {
        let mut oks = B::default();
        let mut errs = C::default();

        for result in self {
            match result {
                Ok(ok) => oks.extend(Some(ok)),
                Err(err) => errs.extend(Some(err)),
            }
        }

        (oks, errs)
    }

    fn flatten_results<T, E>(self) -> FlattenResults<Self>
    where
        Self: Sized + Iterator<Item = Result<Result<T, E>, E>>,
    {
        FlattenResults { iter: self }
    }

    fn collect2<T, E, B, C>(self) -> Result<B, C>
    where
        Self: Sized + Iterator<Item = Result<T, E>>,
        B: Default + Extend<T>,
        C: Default + Extend<E>,
    {
        let mut oks = B::default();
        let mut errs = C::default();

        let mut has_err = false;

        for result in self {
            match result {
                Ok(ok) => oks.extend(Some(ok)),
                Err(err) => {
                    errs.extend(Some(err));
                    has_err = true;
                },
            }
        }

        if has_err {
            Err(errs)
        } else {
            Ok(oks)
        }
    }

    fn oks<T, E>(self) -> Oks<Self>
    where
        Self: Sized + Iterator<Item = Result<T, E>>,
    {
        Oks { iter: self }
    }

    fn errs<T, E>(self) -> Errs<Self>
    where
        Self: Sized + Iterator<Item = Result<T, E>>,
    {
        Errs { iter: self }
    }
}

#[derive(Debug, Clone)]
pub struct Oks<I> {
    iter: I,
}

impl<T, E, I> Iterator for Oks<I>
where
    I: Iterator<Item = Result<T, E>>,
{
    type Item = T;

    fn next(&mut self) -> Option<T> {
        loop {
            match self.iter.next() {
                Some(Ok(t)) => return Some(t),
                None => return None,
                _ => continue,
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct Errs<I> {
    iter: I,
}

impl<T, E, I> Iterator for Errs<I>
where
    I: Iterator<Item = Result<T, E>>,
{
    type Item = E;

    fn next(&mut self) -> Option<E> {
        loop {
            match self.iter.next() {
                Some(Err(e)) => return Some(e),
                None => return None,
                _ => continue,
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct FlattenResults<I> {
    iter: I,
}

impl<T, E, I> Iterator for FlattenResults<I>
where
    I: Iterator<Item = Result<Result<T, E>, E>>,
{
    type Item = Result<T, E>;

    fn next(&mut self) -> Option<Result<T, E>> {
        match self.iter.next() {
            Some(Ok(result)) => Some(result),
            Some(Err(err)) => Some(Err(err)),
            None => None,
        }
    }
}

impl<T, E, I> Resulter for I where I: Iterator<Item = Result<T, E>> {}
