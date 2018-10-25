use terrain::{VolCluster, PersState, Volume, ReadVolume, ReadWriteVolume, AnyVolume, SerializeVolume, ConvertVolume, Container};
use terrain::chunk::{Block, HomogeneousData, HeterogeneousData, RleData};
use parking_lot::{RwLock, RwLockReadGuard, RwLockWriteGuard};

pub enum Chunk {
    Homo(HomogeneousData),
    Hetero(HeterogeneousData),
    Rle(RleData),
    HeteroAndRle(HeterogeneousData, RleData),
}

pub struct ChunkContainer<P> {
    data: RwLock<Chunk>,
    payload: RwLock<Option<P>>,
}

impl<P> ChunkContainer<P> {
    pub fn new(chunk: Chunk) -> Self {
        ChunkContainer {
            data: RwLock::new(chunk),
            payload: RwLock::new(None),
        }
    }
}

impl<P> Container for ChunkContainer<P> {
    type Payload = P;
    type Cluster = Chunk;

    fn payload(&self) -> RwLockReadGuard<Option<Self::Payload>> { self.payload.read() }
    fn payload_mut(&self) -> RwLockWriteGuard<Option<Self::Payload>> { self.payload.write() }
    fn payload_try(&self) -> Option<RwLockReadGuard<Option<Self::Payload>>> { self.payload.try_read() }
    fn payload_try_mut(&self) -> Option<RwLockWriteGuard<Option<Self::Payload>>> { self.payload.try_write() }
    fn data(&self) -> RwLockReadGuard<Self::Cluster> { self.data.read() }
    fn data_mut(&self) -> RwLockWriteGuard<Self::Cluster> { self.data.write() }
    fn data_try(&self) -> Option<RwLockReadGuard<Self::Cluster>> { self.data.try_read() }
    fn data_try_mut(&self) -> Option<RwLockWriteGuard<Self::Cluster>> { self.data.try_write() }
}

impl VolCluster for Chunk {
    type VoxelType = Block;

/*
    fn new() -> Chunk {
        Chunk::Homogeneous {
                homo: None,
        }
    }
*/
    fn contains(&self, state: PersState) -> bool {
        match self {
            Chunk::Homo( _ ) => state == PersState::Homo,
            Chunk::Hetero( _ ) => state == PersState::Hetero,
            Chunk::Rle( _ ) => state == PersState::Rle,
            Chunk::HeteroAndRle( _, _ ) => state == PersState::Hetero || state == PersState::Rle,
        }
    }

    fn insert<V: Volume<VoxelType = Block> + AnyVolume>(&mut self, mut vol: V) {
        let homo: Option<&mut HomogeneousData> = vol.as_any_mut().downcast_mut::<HomogeneousData>();
        if let Some(homo) = homo {
            *self = Chunk::Homo(homo.clone());
            return;
        }
        let heterodata: Option<&mut HeterogeneousData> = vol.as_any_mut().downcast_mut::<HeterogeneousData>();
        if let Some(heterodata) = heterodata {
            match self {
                Chunk::HeteroAndRle(ref mut hetero, _) => *hetero = heterodata.clone(),
                Chunk::Hetero( ref mut hetero ) => *hetero = heterodata.clone(),
                Chunk::Rle( rle ) => *self = Chunk::HeteroAndRle(heterodata.clone(), rle.clone()/*TODO: optimize clone away*/),
                _ => *self = Chunk::Hetero(heterodata.clone()),
            };
        }
        let rledata: Option<&mut RleData> = vol.as_any_mut().downcast_mut::<RleData>();
        if let Some(rledata) = rledata {
            match self {
                Chunk::HeteroAndRle( _, ref mut rle) => *rle = rledata.clone(),
                Chunk::Rle( ref mut rle ) => *rle = rledata.clone(),
                Chunk::Hetero( hetero ) => *self = Chunk::HeteroAndRle(hetero.clone()/*TODO: optimize clone away*/, rledata.clone()),
                _ => *self = Chunk::Rle(rledata.clone()),
            };
        }
        panic!("Cannot Store Vol of type {:?}: ", vol);
    }

    fn remove(&mut self, state: PersState) {
        match self {
            Chunk::HeteroAndRle(hetero, rle) => {
                match state {
                    PersState::Hetero => *self = Chunk::Rle(rle.clone()/*TODO: optimize clone away*/),
                    PersState::Rle => *self = Chunk::Hetero(hetero.clone()/*TODO: optimize clone away*/),
                    _ => panic!("Cannot remove vol of type {:?}: ", state),
                };
            },
            _ => panic!("Cannot remove vol of type {:?}: ", state),
        };
    }

    fn get<'a>(&'a self, state: PersState) -> Option<&'a dyn ReadVolume<VoxelType = Block>> {
        return match state {
            PersState::Homo => {
                match self {
                    Chunk::Homo( homo ) => Some(homo as &dyn ReadVolume<VoxelType = Block>),
                    _ => None,
                }
            },
            PersState::Hetero => {
                match self {
                    Chunk::Hetero( hetero ) => Some(hetero as &dyn ReadVolume<VoxelType = Block>),
                    Chunk::HeteroAndRle( hetero, _ ) => Some(hetero as &dyn ReadVolume<VoxelType = Block>),
                    _ => None,
                }
            },
            PersState::Rle => {
                match self {
                    Chunk::Rle( rle ) => Some(rle as &dyn ReadVolume<VoxelType = Block>),
                    Chunk::HeteroAndRle( _, rle ) => Some(rle as &dyn ReadVolume<VoxelType = Block>),
                    _ => None,
                }
            },
        };
    }

    fn get_mut<'a>(&'a mut self, state: PersState) -> Option<&'a mut dyn ReadWriteVolume<VoxelType = Block>> {
        return match state {
            PersState::Homo => None,
            PersState::Hetero => {
                match self {
                    Chunk::Hetero( ref mut hetero ) => Some(hetero as &mut dyn ReadWriteVolume<VoxelType = Block>),
                    Chunk::HeteroAndRle( ref mut hetero, _ ) => Some(hetero as &mut dyn ReadWriteVolume<VoxelType = Block>),
                    _ => None,
                }
            },
            PersState::Rle => None,
        };
    }

    fn get_vol<'a>(&'a self, state: PersState) -> Option<&'a dyn Volume<VoxelType = Block>> {
        return match state {
            PersState::Homo => {
                match self {
                    Chunk::Homo( homo ) => Some(homo as &dyn Volume<VoxelType = Block>),
                    _ => None,
                }
            },
            PersState::Hetero => {
                match self {
                    Chunk::Hetero( hetero ) => Some(hetero as &dyn Volume<VoxelType = Block>),
                    Chunk::HeteroAndRle( hetero, _ ) => Some(hetero as &dyn Volume<VoxelType = Block>),
                    _ => None,
                }
            },
            PersState::Rle => {
                match self {
                    Chunk::Rle( rle ) => Some(rle as &dyn Volume<VoxelType = Block>),
                    Chunk::HeteroAndRle( _, rle ) => Some(rle as &dyn Volume<VoxelType = Block>),
                    _ => None,
                }
            },
        };
    }

    fn get_serializeable<'a>(&'a self, state: PersState) -> Option<&'a dyn SerializeVolume<VoxelType = Block>> {
        return match state {
            PersState::Homo => {
                match self {
                    Chunk::Homo( homo ) => Some(homo as &dyn SerializeVolume<VoxelType = Block>),
                    _ => None,
                }
            },
            PersState::Hetero => None,
            PersState::Rle => {
                match self {
                    Chunk::Rle( rle ) => Some(rle as &dyn SerializeVolume<VoxelType = Block>),
                    Chunk::HeteroAndRle( _, rle ) => Some(rle as &dyn SerializeVolume<VoxelType = Block>),
                    _ => None,
                }
            },
        };
    }

    fn get_any<'a>(&'a self, state: PersState) -> Option<&'a dyn AnyVolume> {
        return match state {
            PersState::Homo => {
                match self {
                    Chunk::Homo( homo ) => Some(homo as &dyn AnyVolume),
                    _ => None,
                }
            },
            PersState::Hetero => {
                match self {
                    Chunk::Hetero( hetero ) => Some(hetero as &dyn AnyVolume),
                    Chunk::HeteroAndRle( hetero, _ ) => Some(hetero as &dyn AnyVolume),
                    _ => None,
                }
            },
            PersState::Rle => {
                match self {
                    Chunk::Rle( rle ) => Some(rle as &dyn AnyVolume),
                    Chunk::HeteroAndRle( _, rle ) => Some(rle as &dyn AnyVolume),
                    _ => None,
                }
            },
        };
    }
}
