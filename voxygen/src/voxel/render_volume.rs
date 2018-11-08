use common::terrain::{
    chunk::{Block, BlockMat},
    figure::{Cell, CellMaterial},
    PhysicalVolume, ReadVolume, Voxel,
};
use voxel::{Material, MaterialKind, RenderMaterial};

pub trait RenderVoxel: Voxel {
    fn get_palette(&self) -> u16;
    fn get_mat(&self) -> RenderMaterial;
    fn is_opaque(&self) -> bool;
    fn is_occupied(&self) -> bool;
    fn should_add(&self, other_opaque: bool) -> bool { !self.is_occupied() || (!self.is_opaque() && other_opaque) }
}

pub trait RenderVolume: ReadVolume + PhysicalVolume
where
    Self::VoxelType: RenderVoxel,
{
}

// Implementations for common structures

impl RenderVoxel for Block {
    fn get_palette(&self) -> u16 {
        self.material().get_palette()
    }

    fn get_mat(&self) -> RenderMaterial {
        match self.material() {
            // Special case for water
            BlockMat { grad: 0x80, index: 3 } => RenderMaterial::new(Material::Water as u8, MaterialKind::Water),
            // Default material
            m => RenderMaterial::new(Material::MatteSmooth as u8, MaterialKind::Solid),
        }
    }

    fn is_opaque(&self) -> bool { *self != Self::WATER && *self != Self::AIR }

    fn is_occupied(&self) -> bool { *self != Self::AIR }
}

impl RenderVoxel for Cell {
    fn get_palette(&self) -> u16 {
        match self.material() {
            CellMaterial::Empty => 0,
            CellMaterial::GlossySmooth(c)
            | CellMaterial::GlossyRough(c)
            | CellMaterial::MatteSmooth(c)
            | CellMaterial::MatteRough(c)
            | CellMaterial::MetallicSmooth(c)
            | CellMaterial::MetallicRough(c) => c as u16,
        }
    }

    fn get_mat(&self) -> RenderMaterial {
        match self.material() {
            CellMaterial::Empty => RenderMaterial::new(0, MaterialKind::Empty),
            CellMaterial::GlossySmooth(_) => RenderMaterial::new(0, MaterialKind::Solid),
            CellMaterial::GlossyRough(_) => RenderMaterial::new(0, MaterialKind::Solid),
            CellMaterial::MatteSmooth(_) => RenderMaterial::new(0, MaterialKind::Solid),
            CellMaterial::MatteRough(_) => RenderMaterial::new(0, MaterialKind::Solid),
            CellMaterial::MetallicSmooth(_) => RenderMaterial::new(0, MaterialKind::Solid),
            CellMaterial::MetallicRough(_) => RenderMaterial::new(0, MaterialKind::Solid),
        }
    }

    fn is_opaque(&self) -> bool { self.get_mat().is_opaque() }

    fn is_occupied(&self) -> bool { self.get_mat().is_opaque() }
}

impl<B, V: PhysicalVolume<VoxelType = B> + ReadVolume<VoxelType = B>> RenderVolume for V where B: RenderVoxel {}
//the trait `voxel::render_volume::RenderVolume` is not implemented for `dyn common::terrain::PhysicalVolume<VoxelType=common::terrain::chunk::Block>`
