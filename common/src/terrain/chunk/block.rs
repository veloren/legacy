use super::super::Voxel;

#[repr(u16)]
#[derive(Copy, Clone, PartialEq, EnumMap, Debug, Serialize, Deserialize)]
pub enum BlockMaterial {
    Air,
    Grass,
    Sand,
    Earth,
    Stone,
    Water,
    Snow,
    Log,
    Leaves,
    Gold,
}

#[derive(Copy, Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Block {
    mat: BlockMaterial,
}

impl Block {
    pub fn from_byte(byte: u8) -> Self {
        Self {
            mat: match byte {
                0 => BlockMaterial::Air,
                1 => BlockMaterial::Grass,
                2 => BlockMaterial::Sand,
                3 => BlockMaterial::Earth,
                4 => BlockMaterial::Stone,
                5 => BlockMaterial::Water,
                6 => BlockMaterial::Snow,
                7 => BlockMaterial::Log,
                8 => BlockMaterial::Leaves,
                9 => BlockMaterial::Gold,
                _ => BlockMaterial::Stone,
            },
        }
    }
}

impl Voxel for Block {
    type Material = BlockMaterial;

    fn new(mat: Self::Material) -> Self { Block { mat } }

    fn empty() -> Self {
        Block {
            mat: BlockMaterial::Air,
        }
    }

    fn is_solid(&self) -> bool { self.mat != BlockMaterial::Air }

    fn material(&self) -> Self::Material { self.mat }
}
