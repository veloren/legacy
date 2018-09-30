use super::super::{container::VolContainer, PersState, Volume};
use chunk::{Block, Chunk, ChunkFile, ChunkRle};

pub struct ChunkContainer {
    raw: Option<Chunk>,
    rle: Option<ChunkRle>,
    file: Option<ChunkFile>,
}

impl VolContainer for ChunkContainer {
    type VoxelType = Block;

    fn new() -> ChunkContainer {
        ChunkContainer {
            raw: None,
            rle: None,
            file: None,
        }
    }

    fn contains(&self, state: PersState) -> bool {
        match state {
            PersState::Raw => self.raw.is_some(),
            PersState::Rle => self.rle.is_some(),
            PersState::File => self.file.is_some(),
        }
    }

    fn insert<V: Volume<VoxelType = Block>>(&mut self, mut vol: V, state: PersState) {
        match state {
            PersState::Raw => {
                let raw: &mut Chunk = vol.as_any_mut().downcast_mut::<Chunk>().expect("Should be Chunk");
                self.raw = Some(raw.clone());
            },
            PersState::Rle => {
                let rle: &mut ChunkRle = vol.as_any_mut().downcast_mut::<ChunkRle>().expect("Should be ChunkRle");
                self.rle = Some(rle.clone());
            },
            PersState::File => {
                let file: &mut ChunkFile = vol
                    .as_any_mut()
                    .downcast_mut::<ChunkFile>()
                    .expect("Should be ChunkFile");
                self.file = Some(file.clone());
            },
        }
    }

    fn remove(&mut self, state: PersState) {
        match state {
            PersState::Raw => self.raw = None,
            PersState::Rle => self.rle = None,
            PersState::File => self.file = None,
        }
    }

    fn get<'a>(&'a self, state: PersState) -> Option<&'a dyn Volume<VoxelType = Block>> {
        return match state {
            PersState::Raw => self.raw.as_ref().map(|c| c as &dyn Volume<VoxelType = Block>),
            PersState::Rle => self.rle.as_ref().map(|c| c as &dyn Volume<VoxelType = Block>),
            PersState::File => self.file.as_ref().map(|c| c as &dyn Volume<VoxelType = Block>),
        };
    }

    fn get_mut<'a>(&'a mut self, state: PersState) -> Option<&'a mut dyn Volume<VoxelType = Block>> {
        return match state {
            PersState::Raw => self.raw.as_mut().map(|c| c as &mut dyn Volume<VoxelType = Block>),
            PersState::Rle => self.rle.as_mut().map(|c| c as &mut dyn Volume<VoxelType = Block>),
            PersState::File => self.file.as_mut().map(|c| c as &mut dyn Volume<VoxelType = Block>),
        };
    }
}
