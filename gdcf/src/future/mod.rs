use futures::Future;

pub mod extend;
pub mod process;
pub mod refresh;
pub mod stream;

pub trait GdcfFuture: Future {
    fn has_result_cached(&self) -> bool;
    fn into_cached(self) -> Option<Self::Item>;
}
