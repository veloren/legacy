use common::terrain::{
    chunk::{Block, BlockMaterial, HeterogeneousData, HomogeneousData},
    figure::{Cell, CellMaterial, Figure},
    PhysicalVolume, ReadVolume, Voxel,
};
use voxel::{Material, MaterialKind, RenderMaterial};

pub trait RenderVoxel: Voxel {
    fn get_color(&self) -> u8;
    fn get_mat(&self) -> RenderMaterial;
    fn is_opaque(&self) -> bool;
    fn is_occupied(&self) -> bool;
    fn should_add(&self, other_opaque: bool) -> bool { !self.is_occupied() || !self.is_opaque() && other_opaque }
}

pub trait RenderVolume: ReadVolume + PhysicalVolume
where
    Self::VoxelType: RenderVoxel,
{
}

// Implementations for common structures

impl RenderVoxel for Block {
    fn get_color(&self) -> u8 {
        let color_map = enum_map! {
            BlockMaterial::Air => 255,
            BlockMaterial::Grass => 150,
            BlockMaterial::Sand  => 151,
            BlockMaterial::Earth =>152,
            BlockMaterial::Stone => 153,
            BlockMaterial::Water => 154,
            BlockMaterial::Snow => 155,
            BlockMaterial::Log => 156,
            BlockMaterial::Leaves =>157,
            BlockMaterial::Gold => 158,
        };

        color_map[self.material()]
    }

    fn get_mat(&self) -> RenderMaterial {
        let mat_map = enum_map! {
            BlockMaterial::Air => RenderMaterial::new(Material::Empty, MaterialKind::Empty),
            BlockMaterial::Grass => RenderMaterial::new(Material::Grass, MaterialKind::Solid),
            BlockMaterial::Sand => RenderMaterial::new(Material::Sand, MaterialKind::Solid),
            BlockMaterial::Earth => RenderMaterial::new(Material::Earth, MaterialKind::Solid),
            BlockMaterial::Stone => RenderMaterial::new(Material::Stone, MaterialKind::Solid),
            BlockMaterial::Water => RenderMaterial::new(Material::Water, MaterialKind::Water),
            BlockMaterial::Snow => RenderMaterial::new(Material::Snow, MaterialKind::Solid),
            BlockMaterial::Log => RenderMaterial::new(Material::Log, MaterialKind::Solid),
            BlockMaterial::Leaves => RenderMaterial::new(Material::Leaves, MaterialKind::Translucent),
            BlockMaterial::Gold => RenderMaterial::new(Material::MetallicRough, MaterialKind::Solid),
        };

        mat_map[self.material()]
    }

    fn is_opaque(&self) -> bool { self.get_mat().is_opaque() }

    fn is_occupied(&self) -> bool { self.material() != BlockMaterial::Air }
}

impl RenderVoxel for Cell {
    fn get_color(&self) -> u8 {
        match self.material() {
            CellMaterial::Empty => 0,
            CellMaterial::GlossySmooth(c)
            | CellMaterial::GlossyRough(c)
            | CellMaterial::MatteSmooth(c)
            | CellMaterial::MatteRough(c)
            | CellMaterial::MetallicSmooth(c)
            | CellMaterial::MetallicRough(c) => c,
        }
    }

    fn get_mat(&self) -> RenderMaterial {
        match self.material() {
            CellMaterial::Empty => RenderMaterial::new(Material::Empty, MaterialKind::Empty),
            CellMaterial::GlossySmooth(_) => RenderMaterial::new(Material::GlossySmooth, MaterialKind::Solid),
            CellMaterial::GlossyRough(_) => RenderMaterial::new(Material::GlossyRough, MaterialKind::Solid),
            CellMaterial::MatteSmooth(_) => RenderMaterial::new(Material::MatteSmooth, MaterialKind::Solid),
            CellMaterial::MatteRough(_) => RenderMaterial::new(Material::MatteRough, MaterialKind::Solid),
            CellMaterial::MetallicSmooth(_) => RenderMaterial::new(Material::MetallicSmooth, MaterialKind::Solid),
            CellMaterial::MetallicRough(_) => RenderMaterial::new(Material::MetallicRough, MaterialKind::Solid),
        }
    }

    fn is_opaque(&self) -> bool { self.get_mat().is_opaque() }

    fn is_occupied(&self) -> bool { self.get_mat().is_opaque() }
}

impl<B, V: PhysicalVolume<VoxelType = B> + ReadVolume<VoxelType = B>> RenderVolume for V where B: RenderVoxel {}
//the trait `voxel::render_volume::RenderVolume` is not implemented for `dyn common::terrain::PhysicalVolume<VoxelType=common::terrain::chunk::Block>`
