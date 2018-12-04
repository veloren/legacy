use crate::terrain::{
    chunk::{
        rle::{BlockRle, BLOCK_RLE_MAX_NUM},
        Block, HeterogeneousData, HomogeneousData, RleData,
    },
    AnyVolume, ConstructVolume, PersState, PhysicalVolume, ReadVolume, ReadWriteVolume, SerializeVolume, VolCluster,
    Volume, VoxRel, Voxel,
};
use vek::*;

pub enum Chunk {
    Homo(HomogeneousData),
    Hetero(HeterogeneousData),
    Rle(RleData),
    HeteroAndRle(HeterogeneousData, RleData),
}

impl VolCluster for Chunk {
    type VoxelType = Block;

    fn contains(&self, state: PersState) -> bool {
        match self {
            Chunk::Homo(_) => state == PersState::Homo,
            Chunk::Hetero(_) => state == PersState::Hetero,
            Chunk::Rle(_) => state == PersState::Rle,
            Chunk::HeteroAndRle(_, _) => state == PersState::Hetero || state == PersState::Rle,
        }
    }

    fn convert(&mut self, state: PersState) {
        match state {
            PersState::Homo => {
                match self {
                    Chunk::Homo(_) => return,
                    Chunk::Hetero(hetero) => {
                        let t = hetero.at_unchecked(Vec3::new(0, 0, 0));
                        // check if possible!
                        for e in hetero.voxels_mut().iter() {
                            if *e != t {
                                return;
                            }
                        }
                        let homo = HomogeneousData::filled(hetero.size(), t);
                        self.insert(homo);
                    },
                    Chunk::Rle(rle) | Chunk::HeteroAndRle(_, rle) => {
                        let t = rle.at_unchecked(Vec3::new(0, 0, 0));
                        for e in rle.voxels_mut().iter() {
                            for e in e.iter() {
                                if e.block != t {
                                    return;
                                }
                            }
                        }
                        // check if possible!
                        let homo = HomogeneousData::filled(rle.size(), t);
                        self.insert(homo);
                    },
                }
            },
            PersState::Hetero => {
                match self {
                    Chunk::Homo(homo) => {
                        let hetero = HeterogeneousData::filled(homo.size(), homo.at_unchecked(Vec3::new(0, 0, 0)));
                        self.insert(hetero);
                    },
                    Chunk::Hetero(_) | Chunk::HeteroAndRle(_, _) => return,
                    Chunk::Rle(rle) => {
                        let size = rle.size();
                        let mut hetero = HeterogeneousData::empty(size);
                        let ref voxels = rle.voxels_mut();
                        //unfold Rle into Raw format
                        for x in 0..size.x {
                            for y in 0..size.y {
                                let mut old_z: VoxRel = 0;
                                let ref stack = voxels[(x * size.y + y) as usize];
                                for b in stack {
                                    let new_z = old_z + (b.num_minus_one + 1) as VoxRel;
                                    for z in old_z..new_z {
                                        let pos = Vec3::new(x, y, z);
                                        hetero.replace_at_unchecked(pos, b.block);
                                    }
                                    old_z = new_z;
                                }
                                for z in old_z..size.z {
                                    let pos = Vec3::new(x, y, z);
                                    hetero.replace_at_unchecked(pos, Block::empty());
                                }
                            }
                        }
                        self.insert(hetero);
                    },
                }
            },
            PersState::Rle => {
                match self {
                    Chunk::Homo(homo) => {
                        let rle = RleData::filled(homo.size(), homo.at_unchecked(Vec3::new(0, 0, 0)));
                        self.insert(rle);
                    },
                    Chunk::Hetero(hetero) => {
                        let size = hetero.size();
                        let mut rle = RleData::empty(size);
                        let ref mut voxels = rle.voxels_mut();
                        for x in 0..size.x {
                            for y in 0..size.y {
                                let mut old_z: VoxRel = 0;
                                let ref mut xy = voxels[(x * size.y + y) as usize];
                                xy.clear();
                                let mut last_block = hetero.at_unchecked(Vec3::new(x, y, 0)).material();
                                //start converting the pillar x,y
                                for z in 1..size.z + 1 {
                                    let lastelem = z == size.z;
                                    let block = if lastelem {
                                        hetero.at_unchecked(Vec3::new(x, y, z - 1)).material()
                                    } else {
                                        hetero.at_unchecked(Vec3::new(x, y, z)).material()
                                    };
                                    // check the block if its the same like the last one or a diffrernt one, if its a different one, we need to save the last one

                                    if (!lastelem && block != last_block)
                                        || (lastelem && old_z != size.z && last_block != Block::empty().material())
                                    {
                                        let zcnt = z - old_z;
                                        old_z = z;
                                        let high = (zcnt / (BLOCK_RLE_MAX_NUM)) as usize;
                                        let lastsize = zcnt % (BLOCK_RLE_MAX_NUM);
                                        for i in 0..high + 1 {
                                            //we add n blocks with the same type
                                            xy.push(BlockRle::new(
                                                Block::new(last_block),
                                                if i == (high) {
                                                    (lastsize - 1) as u8
                                                } else {
                                                    (BLOCK_RLE_MAX_NUM - 1) as u8
                                                },
                                            ));
                                        }
                                        last_block = block;
                                    }
                                }
                                //println!("pillar done");
                            }
                        }
                        self.insert(rle);
                    },
                    Chunk::Rle(_) | Chunk::HeteroAndRle(_, _) => return,
                }
            },
        };
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
                Chunk::Hetero(ref mut hetero) => *hetero = heterodata.clone(),
                Chunk::Rle(rle) => {
                    *self = Chunk::HeteroAndRle(heterodata.clone(), rle.clone() /*TODO: optimize clone away*/)
                },
                _ => *self = Chunk::Hetero(heterodata.clone()),
            };
            return;
        }
        let rledata: Option<&mut RleData> = vol.as_any_mut().downcast_mut::<RleData>();
        if let Some(rledata) = rledata {
            match self {
                Chunk::HeteroAndRle(_, ref mut rle) => *rle = rledata.clone(),
                Chunk::Rle(ref mut rle) => *rle = rledata.clone(),
                Chunk::Hetero(hetero) => {
                    *self = Chunk::HeteroAndRle(hetero.clone() /*TODO: optimize clone away*/, rledata.clone())
                },
                _ => *self = Chunk::Rle(rledata.clone()),
            };
            return;
        }
        panic!("Cannot Store Vol of type {:?}: ", vol);
    }

    fn remove(&mut self, state: PersState) {
        match self {
            Chunk::HeteroAndRle(hetero, rle) => {
                match state {
                    PersState::Hetero => *self = Chunk::Rle(rle.clone() /*TODO: optimize clone away*/),
                    PersState::Rle => *self = Chunk::Hetero(hetero.clone() /*TODO: optimize clone away*/),
                    _ => panic!("Cannot remove vol of type {:?}: ", state),
                };
            },
            _ => panic!("Cannot remove vol of type {:?}: ", state),
        };
    }

    fn get<'a>(&'a self, state: PersState) -> Option<&'a dyn ReadVolume<VoxelType = Block>> {
        return match state {
            PersState::Homo => match self {
                Chunk::Homo(homo) => Some(homo as &dyn ReadVolume<VoxelType = Block>),
                _ => None,
            },
            PersState::Hetero => match self {
                Chunk::Hetero(hetero) => Some(hetero as &dyn ReadVolume<VoxelType = Block>),
                Chunk::HeteroAndRle(hetero, _) => Some(hetero as &dyn ReadVolume<VoxelType = Block>),
                _ => None,
            },
            PersState::Rle => match self {
                Chunk::Rle(rle) => Some(rle as &dyn ReadVolume<VoxelType = Block>),
                Chunk::HeteroAndRle(_, rle) => Some(rle as &dyn ReadVolume<VoxelType = Block>),
                _ => None,
            },
        };
    }

    fn get_mut<'a>(&'a mut self, state: PersState) -> Option<&'a mut dyn ReadWriteVolume<VoxelType = Block>> {
        return match state {
            PersState::Homo => None,
            PersState::Hetero => match self {
                Chunk::Hetero(ref mut hetero) => Some(hetero as &mut dyn ReadWriteVolume<VoxelType = Block>),
                Chunk::HeteroAndRle(ref mut hetero, _) => Some(hetero as &mut dyn ReadWriteVolume<VoxelType = Block>),
                _ => None,
            },
            PersState::Rle => None,
        };
    }

    fn get_vol<'a>(&'a self, state: PersState) -> Option<&'a dyn Volume<VoxelType = Block>> {
        return match state {
            PersState::Homo => match self {
                Chunk::Homo(homo) => Some(homo as &dyn Volume<VoxelType = Block>),
                _ => None,
            },
            PersState::Hetero => match self {
                Chunk::Hetero(hetero) => Some(hetero as &dyn Volume<VoxelType = Block>),
                Chunk::HeteroAndRle(hetero, _) => Some(hetero as &dyn Volume<VoxelType = Block>),
                _ => None,
            },
            PersState::Rle => match self {
                Chunk::Rle(rle) => Some(rle as &dyn Volume<VoxelType = Block>),
                Chunk::HeteroAndRle(_, rle) => Some(rle as &dyn Volume<VoxelType = Block>),
                _ => None,
            },
        };
    }

    fn get_physical<'a>(&'a self, state: PersState) -> Option<&'a dyn PhysicalVolume<VoxelType = Block>> {
        return match state {
            PersState::Homo => match self {
                Chunk::Homo(homo) => Some(homo as &dyn PhysicalVolume<VoxelType = Block>),
                _ => None,
            },
            PersState::Hetero => match self {
                Chunk::Hetero(hetero) => Some(hetero as &dyn PhysicalVolume<VoxelType = Block>),
                Chunk::HeteroAndRle(hetero, _) => Some(hetero as &dyn PhysicalVolume<VoxelType = Block>),
                _ => None,
            },
            PersState::Rle => match self {
                Chunk::Rle(rle) => Some(rle as &dyn PhysicalVolume<VoxelType = Block>),
                Chunk::HeteroAndRle(_, rle) => Some(rle as &dyn PhysicalVolume<VoxelType = Block>),
                _ => None,
            },
        };
    }

    fn get_serializeable<'a>(&'a self, state: PersState) -> Option<&'a dyn SerializeVolume<VoxelType = Block>> {
        return match state {
            PersState::Homo => match self {
                Chunk::Homo(homo) => Some(homo as &dyn SerializeVolume<VoxelType = Block>),
                _ => None,
            },
            PersState::Hetero => None,
            PersState::Rle => match self {
                Chunk::Rle(rle) => Some(rle as &dyn SerializeVolume<VoxelType = Block>),
                Chunk::HeteroAndRle(_, rle) => Some(rle as &dyn SerializeVolume<VoxelType = Block>),
                _ => None,
            },
        };
    }

    fn get_any<'a>(&'a self, state: PersState) -> Option<&'a dyn AnyVolume> {
        return match state {
            PersState::Homo => match self {
                Chunk::Homo(homo) => Some(homo as &dyn AnyVolume),
                _ => None,
            },
            PersState::Hetero => match self {
                Chunk::Hetero(hetero) => Some(hetero as &dyn AnyVolume),
                Chunk::HeteroAndRle(hetero, _) => Some(hetero as &dyn AnyVolume),
                _ => None,
            },
            PersState::Rle => match self {
                Chunk::Rle(rle) => Some(rle as &dyn AnyVolume),
                Chunk::HeteroAndRle(_, rle) => Some(rle as &dyn AnyVolume),
                _ => None,
            },
        };
    }

    fn prefered<'a>(&'a self) -> Option<&'a dyn ReadVolume<VoxelType = Block>> {
        self.get(match self {
            Chunk::Homo(_) => PersState::Homo,
            Chunk::Hetero(_) => PersState::Hetero,
            Chunk::Rle(_) => PersState::Rle,
            Chunk::HeteroAndRle(_, _) => PersState::Hetero,
        })
    }

    fn prefered_mut<'a>(&'a mut self) -> Option<&'a mut dyn ReadWriteVolume<VoxelType = Block>> {
        self.get_mut(match self {
            Chunk::Homo(_) => PersState::Homo,
            Chunk::Hetero(_) => PersState::Hetero,
            Chunk::Rle(_) => PersState::Rle,
            Chunk::HeteroAndRle(_, _) => PersState::Hetero,
        })
    }

    fn prefered_vol<'a>(&'a self) -> Option<&'a dyn Volume<VoxelType = Block>> {
        self.get_vol(match self {
            Chunk::Homo(_) => PersState::Homo,
            Chunk::Hetero(_) => PersState::Hetero,
            Chunk::Rle(_) => PersState::Rle,
            Chunk::HeteroAndRle(_, _) => PersState::Hetero,
        })
    }

    fn prefered_physical<'a>(&'a self) -> Option<&'a dyn PhysicalVolume<VoxelType = Block>> {
        self.get_physical(match self {
            Chunk::Homo(_) => PersState::Homo,
            Chunk::Hetero(_) => PersState::Hetero,
            Chunk::Rle(_) => PersState::Rle,
            Chunk::HeteroAndRle(_, _) => PersState::Hetero,
        })
    }

    fn prefered_serializeable<'a>(&'a self) -> Option<&'a dyn SerializeVolume<VoxelType = Block>> {
        self.get_serializeable(match self {
            Chunk::Homo(_) => PersState::Homo,
            Chunk::Hetero(_) => PersState::Hetero,
            Chunk::Rle(_) => PersState::Rle,
            Chunk::HeteroAndRle(_, _) => PersState::Rle,
        })
    }

    fn prefered_any<'a>(&'a self) -> Option<&'a dyn AnyVolume> {
        self.get_any(match self {
            Chunk::Homo(_) => PersState::Homo,
            Chunk::Hetero(_) => PersState::Hetero,
            Chunk::Rle(_) => PersState::Rle,
            Chunk::HeteroAndRle(_, _) => PersState::Hetero,
        })
    }

    fn to_bytes(&mut self) -> Result<Vec<u8>, ()> {
        let mut content = vec![];
        let mut ser = self.prefered_serializeable();
        if ser.is_none() {
            self.convert(PersState::Rle);
            ser = self.prefered_serializeable();
        }
        if let Some(ser) = ser {
            let mut bytes = Vec::<u8>::new();
            if self.contains(PersState::Rle) {
                bytes.push(2);
            } else {
                if self.contains(PersState::Homo) {
                    bytes.push(1);
                } else {
                    panic!("what the heck!, this state wasnt planed!")
                }
            }
            let to_bytes = ser.to_bytes();
            if let Ok(to_bytes) = to_bytes {
                bytes.extend(&to_bytes);
                content.extend_from_slice(&bytes);
                return Ok(content);
            }
        }
        Err(())
    }

    fn from_bytes(data: &[u8]) -> Result<Self, ()> {
        let state = data[0];

        if state == 1 {
            let vol: Result<HomogeneousData, ()> = SerializeVolume::from_bytes(&data[1..]);
            if let Ok(vol) = vol {
                return Ok(Chunk::Homo(vol));
            }
        } else {
            let vol: Result<RleData, ()> = SerializeVolume::from_bytes(&data[1..]);
            if let Ok(vol) = vol {
                return Ok(Chunk::Rle(vol));
            }
        }
        Err(())
    }
}
