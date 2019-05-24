#[cfg(feature = "serde_support")]
use serde_derive::{Serialize, Deserialize};

#[derive(Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Clone)]
#[cfg_attr(feature = "serde_support", derive(Serialize, Deserialize))]
pub struct TextData {
    pub text: String,
}
