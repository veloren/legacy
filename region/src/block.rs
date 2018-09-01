use Voxel;

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
