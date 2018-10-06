// Local
use super::Voxel;

#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Block {
    Air,
    Stone,
}

impl Voxel for Block {
    fn empty() -> Self { Block::Air }

    fn solid() -> Self { Block::Stone }

    fn is_empty(&self) -> bool { *self == Block::Air }
}
