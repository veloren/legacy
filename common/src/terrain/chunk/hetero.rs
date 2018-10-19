// Library
use noise::{NoiseFn, SuperSimplex, HybridMulti, Seedable, MultiFractal};
use vek::*;
use dot_vox::DotVoxData;

// Local
use terrain::{
    chunk::{Block, BlockMaterial},
    ConstructVolume, PhysicalVolume, ReadVolume, ReadWriteVolume, Volume, VoxAbs, VoxRel, Voxel,
};

#[derive(Clone, Debug, PartialEq)]
pub struct HeterogeneousData {
    size: Vec3<VoxRel>,
    voxels: Vec<Block>,
}

impl From<DotVoxData> for Chunk {
    fn from(vox: DotVoxData) -> Chunk {
        match vox.models.first() {
            Some(model) => {
                let size = Vec3::new(model.size.x, model.size.y, model.size.z).map(|e| e as i64);
                let mut voxels = vec![Block::empty(); (size.x * size.y * size.z) as usize];
                let mut chunk = Chunk {
                    size,
                    offset: Vec3::new(0, 0, 0),
                    voxels,
                };

                for ref v in model.voxels.iter() {
                    let pos = Vec3::new(v.x as i64, v.y as i64, v.z as i64);
                    chunk.set(pos, Block::from_byte(v.i));
                }

                chunk
            },
            None => Chunk {
                size: Vec3::new(1, 1, 1),
                offset: Vec3::new(0, 0, 0),
                voxels: vec![Block::empty()]
            },
        }
    }
}

impl Chunk {
    pub fn empty() -> Self {
        Self {
            size: Vec3::new(0, 0, 0),
            offset: Vec3::new(0, 0, 0),
            voxels: vec![],
        }
    }

    fn calculate_index(&self, off: Vec3<VoxRel>) -> usize {
        (off.x as usize * self.size.y as usize * self.size.z as usize
            + off.y as usize * self.size.z as usize
            + off.z as usize)
    }

    pub(crate) fn voxels_mut(&mut self) -> &mut Vec<Block> { &mut self.voxels }

    pub fn new(size: Vec3<i64>, offset: Vec3<i64>, voxels: Vec<Block>) -> Self {
        Chunk {
            size,
            offset,
            voxels,
        }
    }
}

impl Volume for HeterogeneousData {
    type VoxelType = Block;

    fn size(&self) -> Vec3<VoxRel> { self.size }
}

impl ReadVolume for HeterogeneousData {
    fn at_unchecked(&self, off: Vec3<VoxRel>) -> Block { self.voxels[self.calculate_index(off)] }
}

impl ReadWriteVolume for HeterogeneousData {
    fn replace_at_unchecked(&mut self, off: Vec3<VoxRel>, vox: Self::VoxelType) -> Self::VoxelType {
        let i = self.calculate_index(off);
        let r = self.voxels[i];
        self.voxels[i] = vox;
        r
    }

    fn fill(&mut self, vox: Self::VoxelType) {
        for v in self.voxels.iter_mut() {
            *v = vox;
        }
    }
}

impl ConstructVolume for HeterogeneousData {
    fn filled(size: Vec3<VoxRel>, vox: Self::VoxelType) -> HeterogeneousData {
        HeterogeneousData {
            size,
            voxels: vec![vox; size.map(|e| e as usize).product()],
        }
    }

    fn empty(size: Vec3<VoxRel>) -> HeterogeneousData { Self::filled(size, Block::empty()) }
}

impl PhysicalVolume for HeterogeneousData {}
