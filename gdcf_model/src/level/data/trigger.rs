#[cfg(feature = "serde_support")]
use serde_derive::{Serialize, Deserialize};

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
#[cfg_attr(feature = "serde_support", derive(Serialize, Deserialize))]
pub struct ColorTriggerData {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub blending_enabled: bool,
}
