use seahash::SeaHasher;
use std::hash::{Hash, Hasher};


pub(crate) fn hash<H: Hash>(h: &H) -> u64 {
    let mut hasher = SeaHasher::default();
    h.hash(&mut hasher);
    hasher.finish()
}