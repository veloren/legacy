use bincode;

use std::{fs::File, io::prelude::*, u8};
use terrain::{
    chunk::{Block, BlockRle, Chunk, HomogeneousData, HeterogeneousData, RleData, BLOCK_RLE_MAX_CNT},
    Key, PersState, VolCluster, Volume, ReadVolume, ReadWriteVolume, Voxel, ConvertVolume, ConstructVolume,
};
use vek::*;

impl ConvertVolume for HeterogeneousData {
    fn convert<Chunk>(&self, state: &PersState, con: &mut Chunk) where Chunk: VolCluster<VoxelType = Block>,
    {
        match state {
            PersState::Homo => {
                let mut homo = HomogeneousData::new();
                *homo.mut_size() = self.size();
                *homo.mut_voxel() = self.at_unsafe(Vec3::new(0, 0, 0));
                con.insert(homo);
            },
            PersState::Hetero => {
            },
            PersState::Rle => {
                let mut homo = RleData::new();
                *homo.mut_size() = self.size();
                *homo.mut_voxel() = Vec::new();
                con.insert(homo);
            },
            PersState::File => {
            },
        }
    }
}

impl ConvertVolume for HomogeneousData {
    fn convert<Chunk>(&self, state: &PersState, con: &mut Chunk) where Chunk: VolCluster<VoxelType = Block>,
    {
        match state {
            PersState::Homo => {
            },
            PersState::Hetero => {
                let mut hetero = HeterogeneousData::new();
                *hetero.mut_size() = self.size();
                hetero.fill(self.at_unsafe(Vec3::new(0, 0, 0)));
                con.insert(hetero);
            },
            PersState::Rle => {
            },
            PersState::File => {
            },
        }
    }
}

impl ConvertVolume for RleData {
    fn convert<Chunk>(&self, state: &PersState, con: &mut Chunk) where Chunk: VolCluster<VoxelType = Block>,
    {
        match state {
            PersState::Homo => {
            },
            PersState::Hetero => {
                let mut hetero = HeterogeneousData::new();
                *hetero.mut_size() = self.size();
                hetero.fill(self.at_unsafe(Vec3::new(0, 0, 0)));
                con.insert(hetero);
            },
            PersState::Rle => {
            },
            PersState::File => {
            },
        }
    }
}

/*
+ This is some ugly code, it covers the conversion from any persistent state into any other persistent state. e.g. the code for transforming a RleChunk into a RawChunk
* thats why it needs to make downcasts inside this coding. The ugly part is always transfering the state of how blocks are saved in each struct.
+ Dont bother to much with this code, adjust it if you decide to change the representation of a chunk.
+ This file is tested well with tests to ensure the algorithms work
*/

/*
impl VolConverter<ChunkContainer> for ChunkConverter {
    fn convert<K: Key>(key: &K, container: &mut ChunkContainer, state: PersState) {
        let x = Homogeneous::new();
        match state {
            PersState::Raw => {
                if container.get_mut(PersState::Rle).is_none() && container.get_mut(PersState::File).is_some() {
                    // In case we have File and want Raw, recursive call ourself to generate Rle first
                    Self::convert(key, container, PersState::Rle);
                };
                if let Some(rle) = container.get_mut(PersState::Rle) {
                    //convert Rle to Raw
                    let from: &mut ChunkRle = rle.as_any_mut().downcast_mut::<ChunkRle>().expect("Should be ChunkRle");
                    let size = from.size();
                    let mut raw = Chunk::new();
                    // copy properties from rle to raw
                    raw.set_size(size);
                    raw.set_offset(from.offset());
                    let ref voxels = from.voxels_mut();
                    //unfold Rle into Raw format
                    for x in 0..size.x {
                        for y in 0..size.y {
                            let mut old_z: i64 = 0;
                            let ref stack = voxels[(x * size.y + y) as usize];
                            for b in stack {
                                let new_z = old_z + (b.num_minus_one + 1) as i64;
                                for z in old_z..new_z {
                                    let pos = Vec3::<i64>::new(x, y, z as i64);
                                    raw.set(pos, b.block);
                                }
                                old_z = new_z;
                            }
                            for z in old_z..size.z {
                                let pos = Vec3::<i64>::new(x, y, z as i64);
                                raw.set(pos, Block::empty());
                            }
                        }
                    }
                    container.insert(raw, PersState::Raw);
                }

                // Rle -> Raw
                // File -> Rle -> Raw
            },
            PersState::Rle => {
                if let Some(raw) = container.get_mut(PersState::Raw) {
                    let from: &mut Chunk = raw.as_any_mut().downcast_mut::<Chunk>().expect("Should be Chunk");
                    let size = from.size();
                    let mut rle = ChunkRle::new();
                    rle.set_size(size);
                    rle.set_offset(from.offset());
                    let ref mut voxels = rle.voxels_mut();
                    for x in 0..size.x {
                        for y in 0..size.y {
                            let mut old_z: i64 = 0;
                            let ref mut xy = voxels[(x * size.y + y) as usize];
                            xy.clear();
                            let mut last_block = from.at(Vec3::new(x, y, 0)).unwrap().material();
                            //start converting the pillar x,y
                            for z in 1..size.z {
                                let block = from.at(Vec3::new(x, y, z)).unwrap().material();
                                // check the block if its the same like the last one or a diffrernt one, if its a different one, we need to save the last one
                                if block != last_block {
                                    let zcnt = z - old_z;
                                    old_z = z;
                                    let high = ((zcnt as f32) / (BLOCK_RLE_MAX_CNT as f32 + 1.0)).ceil() as usize;
                                    let lastsize = zcnt % (BLOCK_RLE_MAX_CNT as i64 + 1);
                                    //println!("zcnt {} high {}", zcnt, high);
                                    for i in 0..high {
                                        //we add n blocks with the same type
                                        xy.push(BlockRle::new(
                                            Block::new(last_block),
                                            if i == (high - 1) {
                                                (lastsize - 1) as u8
                                            } else {
                                                BLOCK_RLE_MAX_CNT
                                            },
                                        ));
                                    }
                                    last_block = block;
                                }
                            }
                            // same coding as above to handle the last block outside the array
                            if old_z != size.z && last_block != Block::empty().material() {
                                //println!("END last_block {:?}, old_z {}", last_block, old_z);
                                let zcnt = size.z - old_z;
                                let high = ((zcnt as f32) / (BLOCK_RLE_MAX_CNT as f32 + 1.0)).ceil() as usize;
                                let lastsize = zcnt % (BLOCK_RLE_MAX_CNT as i64 + 1);
                                //println!("zcnt {} high {}", zcnt, high);
                                for i in 0..high {
                                    //println!("add {:?}", last_block);
                                    xy.push(BlockRle::new(
                                        Block::new(last_block),
                                        if i == (high - 1) {
                                            (lastsize - 1) as u8
                                        } else {
                                            BLOCK_RLE_MAX_CNT
                                        },
                                    ));
                                }
                            }
                            //println!("pillar done");
                        }
                    }
                    container.insert(rle, PersState::Rle);
                }
                if let Some(file) = container.get_mut(PersState::File) {
                    let from: &mut ChunkFile = file
                        .as_any_mut()
                        .downcast_mut::<ChunkFile>()
                        .expect("Should be ChunkFile");
                    // if we have a file we need to deserialize it into a RleChunk
                    let filename = from.file();
                    let mut datfile = File::open(filename).unwrap();
                    let mut content = Vec::<u8>::new();
                    datfile
                        .read_to_end(&mut content)
                        .expect(&format!("read of file {} failed", filename));
                    debug!("read from file: {}, bytes: {}", filename, content.len());
                    let mut rle: ChunkRle = bincode::deserialize(&content).expect("Cannot Load Chunk from File");
                    rle.set_size(from.size());
                    rle.set_offset(from.offset());
                    container.insert(rle, PersState::Rle);
                }
            },
            PersState::File => {
                if container.get_mut(PersState::Rle).is_none() && container.get_mut(PersState::Raw).is_some() {
                    // If we have Raw, convert it to Rle first, then continue
                    Self::convert(key, container, PersState::Rle);
                };
                if let Some(rle) = container.get_mut(PersState::Rle) {
                    // If we have Rle, we can serialize it and then save it to a file
                    let from: &mut ChunkRle = rle.as_any_mut().downcast_mut::<ChunkRle>().expect("Should be ChunkRle");
                    let mut file = ChunkFile::new(from.size());
                    file.set_offset(from.offset());
                    let filename = key.print() + ".dat";
                    let content = bincode::serialize(&from).expect("Cannot searialize Chunk");
                    let filepath = "./saves/".to_owned() + &(filename);
                    *file.file_mut() = (filepath).to_string();
                    let mut datfile = File::create(filepath).unwrap();
                    datfile.write_all(&content).unwrap();
                    debug!("write to file: {}, bytes: {}", filename, content.len());
                    container.insert(file, PersState::File);
                }
            },
        }
    }
}
*/
