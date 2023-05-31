use strum::{Display, EnumIter};

use super::*;

#[derive(Clone, Copy, Default, Display, EnumIter)]
#[strum(serialize_all = "snake_case")]
pub enum GomokuTextures {
    #[default]
    White
}

impl Into<u32> for GomokuTextures {
    fn into(self) -> u32 {
        self as u32
    }
}

impl Textures for GomokuTextures {}

pub type Txts = GomokuTextures;