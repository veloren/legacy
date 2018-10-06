use terrain::{VolCluster, PersState, Volume, ReadVolume, ReadWriteVolume, AnyVolume, SerializeVolume, ConvertVolume, Container};
use terrain::chunk::{Block, HomogeneousData, HeterogeneousData, RleData};
use parking_lot::{RwLock, RwLockReadGuard, RwLockWriteGuard};

pub enum Chunk {
    Homogeneous{
        homo: Option<HomogeneousData>,
    },
    Heterogeneous {
        hetero: Option<HeterogeneousData>,
        rle: Option<RleData>,
    },
}

pub struct ChunkContainer<P> {
    data: RwLock<Chunk>,
    payload: RwLock<Option<P>>,
}

impl<P> ChunkContainer<P> {
    pub fn new() -> Self {
        ChunkContainer {
            data: RwLock::new(Chunk::new()),
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

    fn new() -> Chunk {
        Chunk::Homogeneous {
                homo: None,
        }
    }

    fn contains(&self, state: PersState) -> bool {
        match self {
            Chunk::Homogeneous{ homo } => {if state == PersState::Homo {return homo.is_some()}},
            Chunk::Heterogeneous{ hetero, rle} => {
                match state {
                    PersState::Homo => return false,
                    PersState::Hetero => return hetero.is_some(),
                    PersState::Rle => return rle.is_some(),
                    PersState::File => return false,
                }
            },
        }
        false
    }


    fn convert(&mut self, state: PersState) -> bool {
        /*
        let try = if let Chunk::Homogeneous{ref homo} = self {
            homo.
            if let Some(e) = homo {
                e.convert(&state, self);
                return true
            };
        }
        if let Chunk::Heterogeneous{ref hetero, ref rle} = self {
            if let Some(e) = hetero {
                e.convert(&state, self);
                return true
            };
            if let Some(e) = rle {
                e.convert(&state, self);
                return true
            };
        }*/
        false

    }

    fn insert<V: Volume<VoxelType = Block> + AnyVolume>(&mut self, mut vol: V) {
        let homo: Option<&mut HomogeneousData> = vol.as_any_mut().downcast_mut::<HomogeneousData>();
        if let Some(homo) = homo {
            *self = Chunk::Homogeneous{homo: Some(homo.clone())};
            return;
        }
        let heterodata: Option<&mut HeterogeneousData> = vol.as_any_mut().downcast_mut::<HeterogeneousData>();
        if let Some(heterodata) = heterodata {
            if let Chunk::Heterogeneous{ref mut hetero, ref mut rle} = self {
                *hetero = Some(heterodata.clone());
            } else {
                *self = Chunk::Heterogeneous{
                    hetero: Some(heterodata.clone()),
                    rle: None,
                };
            }
            return;
        }
        let rledata: Option<&mut RleData> = vol.as_any_mut().downcast_mut::<RleData>();
        if let Some(rledata) = rledata {
            if let Chunk::Heterogeneous{ref mut hetero, ref mut rle} = self {
                *rle = Some(rledata.clone());
            } else {
                *self = Chunk::Heterogeneous{
                    hetero: None,
                    rle: Some(rledata.clone()),
                };
            }
            return;
        }
        panic!("Cannot Store Vol of this type {:?}: ", vol);
    }

    fn remove(&mut self, state: PersState) {
        match state {
            PersState::Homo => {
                if let Chunk::Homogeneous{ref mut homo} = self {
                    *homo = None;
                }
            },
            PersState::Hetero => {
                if let Chunk::Heterogeneous{ref mut hetero, ref mut rle} = self {
                    *hetero = None;
                }
            },
            PersState::Rle => {
                if let Chunk::Heterogeneous{ref mut hetero, ref mut rle} = self {
                    *rle = None;
                }
            },
            PersState::File => return,
        }
    }

/*
    fn getty<'a, HeterogeneousData>(&'a self) -> Option<&'a HeterogeneousData> {
        None,
    }

    fn getty<'a, RleData>(&'a self) -> Option<&'a RleData> {
        None,
    }*/

    fn get<'a>(&'a self, state: PersState) -> Option<&'a dyn ReadVolume<VoxelType = Block>> {
        //TODO: simplify this like below!
        match state {
            PersState::Homo => {
                if let Chunk::Homogeneous{homo} = self {
                    return homo.as_ref().map(|c| c as &dyn ReadVolume<VoxelType = Block>);
                }
            },
            PersState::Hetero => {
                if let Chunk::Heterogeneous{hetero, rle} = self {
                    return hetero.as_ref().map(|c| c as &dyn ReadVolume<VoxelType = Block>);
                }
            },
            PersState::Rle => {
                if let Chunk::Heterogeneous{hetero, rle} = self {
                    return rle.as_ref().map(|c| c as &dyn ReadVolume<VoxelType = Block>);
                }
            },
            PersState::File => return None,
        }
        /*
        return match state {
            PersState::Raw => self.raw.as_ref().map(|c| c as &dyn Volume<VoxelType = Block>),
            PersState::Rle => self.rle.as_ref().map(|c| c as &dyn Volume<VoxelType = Block>),
            PersState::File => self.file.as_ref().map(|c| c as &dyn Volume<VoxelType = Block>),
        };
        */
        None
    }

    fn get_mut<'a>(&'a mut self, state: PersState) -> Option<&'a mut dyn ReadWriteVolume<VoxelType = Block>> {
        //TODO: simplify this like below!
        match state {
            PersState::Homo => return None,
            PersState::Hetero => {
                if let Chunk::Heterogeneous{hetero, rle} = self {
                    return hetero.as_mut().map(|c| c as &mut dyn ReadWriteVolume<VoxelType = Block>);
                }
            },
            PersState::Rle => return None,
            PersState::File => return None,
        }
        /*
        return match state {
            PersState::Raw => self.raw.as_mut().map(|c| c as &mut dyn Volume<VoxelType = Block>),
            PersState::Rle => self.rle.as_mut().map(|c| c as &mut dyn Volume<VoxelType = Block>),
            PersState::File => self.file.as_mut().map(|c| c as &mut dyn Volume<VoxelType = Block>),
        };*/
        None
    }

    fn get_vol<'a>(&'a self, state: PersState) -> Option<&'a dyn Volume<VoxelType = Block>> {
        //TODO: simplify this like below!
        match state {
            PersState::Homo => {
                if let Chunk::Homogeneous{homo} = self {
                    return homo.as_ref().map(|c| c as &dyn Volume<VoxelType = Block>);
                }
            },
            PersState::Hetero => {
                if let Chunk::Heterogeneous{hetero, rle} = self {
                    return hetero.as_ref().map(|c| c as &dyn Volume<VoxelType = Block>);
                }
            },
            PersState::Rle => {
                if let Chunk::Heterogeneous{hetero, rle} = self {
                    return rle.as_ref().map(|c| c as &dyn Volume<VoxelType = Block>);
                }
            },
            PersState::File => return None,
        }
        /*
        return match state {
            PersState::Raw => self.raw.as_ref().map(|c| c as &dyn Volume<VoxelType = Block>),
            PersState::Rle => self.rle.as_ref().map(|c| c as &dyn Volume<VoxelType = Block>),
            PersState::File => self.file.as_ref().map(|c| c as &dyn Volume<VoxelType = Block>),
        };
        */
        None
    }

    fn get_serializeable<'a>(&'a self, state: PersState) -> Option<&'a dyn SerializeVolume<VoxelType = Block>> {
        //TODO: simplify this like below!
        match state {
            PersState::Homo => {
                if let Chunk::Homogeneous{homo} = self {
                    return homo.as_ref().map(|c| c as &dyn SerializeVolume<VoxelType = Block>);
                }
            },
            PersState::Hetero => return None,
            PersState::Rle => {
                if let Chunk::Heterogeneous{hetero, rle} = self {
                    return rle.as_ref().map(|c| c as &dyn SerializeVolume<VoxelType = Block>);
                }
            },
            PersState::File => return None,
        }
        /*
        return match state {
            PersState::Raw => self.raw.as_ref().map(|c| c as &dyn Volume<VoxelType = Block>),
            PersState::Rle => self.rle.as_ref().map(|c| c as &dyn Volume<VoxelType = Block>),
            PersState::File => self.file.as_ref().map(|c| c as &dyn Volume<VoxelType = Block>),
        };
        */
        None
    }

    fn get_any<'a>(&'a self, state: PersState) -> Option<&'a dyn AnyVolume> {
        //TODO: simplify this like below!
        match state {
            PersState::Homo => {
                if let Chunk::Homogeneous{homo} = self {
                    return homo.as_ref().map(|c| c as &dyn AnyVolume);
                }
            },
            PersState::Hetero => {
                if let Chunk::Heterogeneous{hetero, rle} = self {
                    return hetero.as_ref().map(|c| c as &dyn AnyVolume);
                }
            },
            PersState::Rle => {
                if let Chunk::Heterogeneous{hetero, rle} = self {
                    return rle.as_ref().map(|c| c as &dyn AnyVolume);
                }
            },
            PersState::File => return None,
        }
        /*
        return match state {
            PersState::Raw => self.raw.as_ref().map(|c| c as &dyn Volume<VoxelType = Block>),
            PersState::Rle => self.rle.as_ref().map(|c| c as &dyn Volume<VoxelType = Block>),
            PersState::File => self.file.as_ref().map(|c| c as &dyn Volume<VoxelType = Block>),
        };
        */
        None
    }
}
