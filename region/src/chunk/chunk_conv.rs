use bincode;

use super::super::{Key, PersState, VolContainer, VolConverter, Volume, Voxel};
use chunk::{Block, BlockRle, Chunk, ChunkContainer, ChunkFile, ChunkRle, BLOCK_RLE_MAX_CNT};
use coord::prelude::*;
use parking_lot::{RwLock, RwLockReadGuard, RwLockWriteGuard};
use std::{fs::File, io::prelude::*, u8};

pub struct ChunkConverter {}

impl VolConverter<ChunkContainer> for ChunkConverter {
    fn convert<K: Key>(key: &K, container: &mut ChunkContainer, state: PersState) {
        match state {
            PersState::Raw => {
                if container.get_mut(PersState::Rle).is_none() && container.get_mut(PersState::File).is_some() {
                    Self::convert(key, container, PersState::Rle);
                };
                if let Some(rle) = container.get_mut(PersState::Rle) {
                    let from: &mut ChunkRle = rle.as_any_mut().downcast_mut::<ChunkRle>().expect("Should be ChunkRle");
                    let size = from.size();
                    let mut raw = Chunk::new();
                    raw.set_size(size);
                    raw.set_offset(from.offset());
                    let ref voxels = from.voxels_mut();
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
                            //println!("start pillar {}/{}", x,y);
                            for z in 1..size.z {
                                let block = from.at(Vec3::new(x, y, z)).unwrap().material();
                                //println!("block: {:?}, last_block {:?}, z {}, old_z {}", block, last_block, z, old_z);
                                if block != last_block {
                                    let zcnt = z - old_z;
                                    old_z = z;
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
                                    last_block = block;
                                }
                            }
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
                    let filename = from.file();
                    let mut datfile = File::open(filename).unwrap();
                    let mut content = Vec::<u8>::new();
                    datfile
                        .read_to_end(&mut content)
                        .expect(&format!("read of file {} failed", filename));
                    debug!("write to file: {}, bytes: {}", filename, content.len());
                    let mut rle: ChunkRle = bincode::deserialize(&content).expect("Cannot Load Chunk from File");
                    rle.set_size(from.size());
                    rle.set_offset(from.offset());
                    container.insert(rle, PersState::Rle);
                }
                // Raw -> Rle
                // File -> Rle
            },
            PersState::File => {
                if container.get_mut(PersState::Rle).is_none() && container.get_mut(PersState::Raw).is_some() {
                    Self::convert(key, container, PersState::Rle);
                };
                if let Some(rle) = container.get_mut(PersState::Rle) {
                    let from: &mut ChunkRle = rle.as_any_mut().downcast_mut::<ChunkRle>().expect("Should be ChunkRle");
                    let mut file = ChunkFile::new(from.size());
                    file.set_offset(from.offset());
                    let filename = key.print() + ".dat";
                    let content = bincode::serialize(&from).expect("Cannot Store Chunk in File");
                    let filepath = "./saves/".to_owned() + &(filename);
                    *file.file_mut() = (filepath).to_string();
                    let mut datfile = File::create(filepath).unwrap();
                    datfile.write_all(&content).unwrap();
                    container.insert(file, PersState::File);
                }
                // Rle -> File
                // Raw -> Rle -> File
            },
        }
    }
}
