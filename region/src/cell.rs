use Voxel;

#[repr(u16)]
#[derive(Copy, Clone, PartialEq)]
pub enum CellMaterial {
    GlossySmooth(u8),
    GlossyRough(u8),
    MatteSmooth(u8),
    MatteRough(u8),
    MetallicSmooth(u8),
    MetallicRough(u8),
    Empty,
}

#[derive(Copy, Clone)]
pub struct Cell {
    mat: CellMaterial,
}

impl Voxel for Cell {
    type Material = CellMaterial;

    fn new(mat: Self::Material) -> Self { Cell { mat } }

    fn empty() -> Self { Cell { mat: CellMaterial::Empty } }

    fn is_solid(&self) -> bool { self.mat != CellMaterial::Empty }

    fn material(&self) -> Self::Material { self.mat }
}
