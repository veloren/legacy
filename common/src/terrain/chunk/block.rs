use serde_derive::{Deserialize, Serialize};

use super::super::Voxel;

#[derive(Copy, Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct BlockMat {
    pub grad: u8, // 0x0 - 0xFE = gradient, 0xFF = palette mode
    pub index: u8,
}

impl BlockMat {
    pub fn get_palette(&self) -> u16 { ((self.grad as u16) << 8) | (self.index as u16) }

    #[allow(dead_code)]
    pub fn grad(&self) -> u8 { self.grad }
    #[allow(dead_code)]
    pub fn index(&self) -> u8 { self.index }
}

#[derive(Copy, Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Block {
    mat: BlockMat,
}

impl Block {
    pub const AIR: Block = Block::from_byte(0);
    pub const GRASS: Block = Block::from_byte(26);
    pub const SAND: Block = Block::from_byte(131);
    pub const EARTH: Block = Block::from_byte(98);
    pub const STONE: Block = Block::from_byte(1);
    pub const WATER: Block = Block::from_byte(3);
    pub const SNOW: Block = Block::from_byte(7);
    pub const LOG: Block = Block::from_byte(77);
    pub const LEAF: Block = Block::from_byte(34);
    pub const GOLD: Block = Block::from_byte(95);
    pub const LIGHT_COBBLE: Block = Block::from_byte(109);
    pub const MID_COBBLE: Block = Block::from_byte(83);
    pub const DARK_COBBLE: Block = Block::from_byte(163);

    pub const GRAD2_A_GRASS: u8 = 0;
    pub const GRAD2_A_LEAF0: u8 = 1;
    pub const GRAD2_B_STONE: u8 = 0;
    pub const GRAD2_B_DRY_GRASS: u8 = 1;
    pub const GRAD2_B_LEAF1: u8 = 2;

    pub const GRAD3_O_STONE: u8 = 0;
    pub const GRAD3_O_EARTH: u8 = 1;
    pub const GRAD3_A_GRASS: u8 = 0;
    pub const GRAD3_B_SAND: u8 = 0;
    pub const GRAD3_B_SNOW: u8 = 1;

    pub fn gradient2(idx_a: u8, idx_b: u8, grad: u8) -> Self {
        Self {
            mat: BlockMat {
                grad: 0x40 | grad.min(0x3F),
                index: (idx_a & 0xF) | ((idx_b & 0xF) << 4),
            },
        }
    }

    pub fn gradient3(idx_o: u8, idx_a: u8, idx_b: u8, grad_ab: u8, grad_o: u8) -> Self {
        Self {
            mat: BlockMat {
                grad: 0xC0 | grad_o.min(0x3F),
                index: (idx_o & 0x1) | ((idx_a & 0x1) << 1) | ((idx_b & 0x1) << 2) | (grad_ab.min(0x1F) << 3),
            },
        }
    }

    pub const fn from_byte(byte: u8) -> Self {
        Self {
            mat: BlockMat {
                grad: 0x80,
                index: byte,
            },
        }
    }
}

impl Voxel for Block {
    type Material = BlockMat;

    fn new(mat: Self::Material) -> Self { Block { mat } }

    fn empty() -> Self { Self::AIR }

    fn is_solid(&self) -> bool { *self != Self::AIR }

    fn material(&self) -> Self::Material { self.mat }
}
