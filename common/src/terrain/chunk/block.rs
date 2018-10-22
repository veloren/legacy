use super::super::Voxel;

#[derive(Copy, Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct BlockMat {
    pub grad: u8, // 0x0 - 0xFE = gradient, 0xFF = palette mode
    pub index: u8,
}

impl BlockMat {
    pub fn get_palette(&self) -> u16 {
        ((self.grad as u16) << 8) | (self.index as u16)
    }
}

#[derive(Copy, Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Block {
    mat: BlockMat,
}

impl Block {
    pub const AIR: Block   = Block::from_byte(0);
    pub const GRASS: Block = Block::from_byte(26);
    pub const SAND: Block  = Block::from_byte(122);
    pub const EARTH: Block = Block::from_byte(98);
    pub const STONE: Block = Block::from_byte(1);
    pub const WATER: Block = Block::from_byte(206);
    pub const SNOW: Block  = Block::from_byte(7);
    pub const LOG: Block   = Block::from_byte(77);
    pub const LEAF: Block  = Block::from_byte(34);
    pub const GOLD: Block  = Block::from_byte(95);

    pub const fn from_byte(byte: u8) -> Self {
        Self {
            mat: BlockMat {
                grad: 0xFF,
                index: byte,
            },
        }
    }
}

impl Voxel for Block {
    type Material = BlockMat;

    fn new(mat: Self::Material) -> Self { Block { mat } }

    fn empty() -> Self {
        Self::AIR
    }

    fn is_solid(&self) -> bool { *self != Self::AIR }

    fn material(&self) -> Self::Material { self.mat }
}
