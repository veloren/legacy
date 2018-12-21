// Standard
use std::{collections::HashMap, sync::Arc, thread, time::Duration};

// Library
use lazy_static::lazy_static;
use parking_lot::{Mutex, RwLock, RwLockReadGuard, RwLockWriteGuard};
use threadpool::ThreadPool;
use vek::*;

// Local
use crate::terrain::{
    self,
    chunk::{Block, ChunkContainer, ChunkSample},
    Container, Key, PersState, VolCluster, VolGen, VolOffs, VoxAbs, VoxRel,
};

lazy_static! {
    static ref POOL: Mutex<ThreadPool> = Mutex::new(ThreadPool::new(2));
}

impl Key for Vec3<VolOffs> {
    fn print(&self) -> String { return format!("c{},{},{}", self.x, self.y, self.z).to_string(); }
}

#[derive(Debug, PartialEq)]
pub enum ChunkSampleError {
    ChunkMissing { key: Vec3<VolOffs> },
    CannotGetLock { key: Vec3<VolOffs> },
    NoContent,
}

#[derive(Clone)]
pub struct BlockLoader {
    pub pos: Vec3<VoxAbs>,
    pub size: Vec3<VoxAbs>,
}

pub struct ChunkMgr<P: Send + Sync + 'static> {
    vol_size: Vec3<VoxRel>,
    pending: Arc<RwLock<HashMap<Vec3<VolOffs>, Arc<Mutex<Option<ChunkContainer<P>>>>>>>, // Mutex is only needed for compiler, we dont acces it in multiple threads
    pers: RwLock<HashMap<Vec3<VolOffs>, Arc<ChunkContainer<P>>>>,
    gen: VolGen<Vec3<VolOffs>, ChunkContainer<P>>,
    block_loader: RwLock<Vec<Arc<RwLock<BlockLoader>>>>, //TODO: maybe remove this from CHUNMGR, and just pass it
}

impl<P: Send + Sync + 'static> ChunkMgr<P> {
    pub fn new(vol_size: Vec3<VoxRel>, gen: VolGen<Vec3<VolOffs>, ChunkContainer<P>>) -> ChunkMgr<P> {
        ChunkMgr {
            vol_size,
            pending: Arc::new(RwLock::new(HashMap::new())),
            pers: RwLock::new(HashMap::new()),
            gen,
            block_loader: RwLock::new(Vec::new()),
        }
    }

    pub fn exists_block(&self, pos: Vec3<VoxAbs>) -> bool {
        self.exists_chunk(terrain::voxabs_to_voloffs(pos, self.vol_size))
    }

    pub fn exists_chunk(&self, pos: Vec3<VolOffs>) -> bool { self.pers.read().get(&pos).is_some() }

    pub fn get_block(&self, pos: Vec3<VoxAbs>) -> Option<Block> {
        let chunk = terrain::voxabs_to_voloffs(pos, self.vol_size);
        let off = terrain::voxabs_to_voxrel(pos, self.vol_size);
        if let Some(chunk) = self.pers.read().get(&chunk) {
            let lock = chunk.data();
            let hetero = lock.get(PersState::Hetero);
            if let Some(hetero) = hetero {
                return hetero.at(off);
            }
        }
        None
    }

    // Tries getting a Sample
    pub fn try_get_sample(&self, from: Vec3<VoxAbs>, to: Vec3<VoxAbs>) -> Result<ChunkSample, ChunkSampleError> {
        let mut c = 0;
        loop {
            match self.get_sample(from, to) {
                Ok(sample) => return Ok(sample),
                Err(e) => match e {
                    ChunkSampleError::CannotGetLock { .. } => {
                        c += 1;
                        if c > 10 {
                            warn!("Long waiting chunk sample {}", c)
                        }
                        thread::sleep(Duration::from_millis(1));
                    },
                    _ => {
                        return Err(e);
                    },
                },
            }
        }
    }

    pub fn get_sample(&self, from: Vec3<VoxAbs>, to: Vec3<VoxAbs>) -> Result<ChunkSample, ChunkSampleError> {
        let mut map = HashMap::new();
        let chunk_from = terrain::voxabs_to_voloffs(from, self.vol_size);
        let chunk_to = terrain::voxabs_to_voloffs(to, self.vol_size);
        let lock = self.pers.read();
        for x in chunk_from.x..chunk_to.x + 1 {
            for y in chunk_from.y..chunk_to.y + 1 {
                for z in chunk_from.z..chunk_to.z + 1 {
                    let key = Vec3::new(x, y, z);
                    let cc = lock.get(&key).map(|v| v.clone());
                    if let Some(cc) = cc {
                        if cc
                            .data_try()
                            .take()
                            .map(|value| map.insert(key, Arc::new(value)))
                            .is_none()
                        {
                            debug!("Cannot get lock: {}", &key);
                            return Err(ChunkSampleError::CannotGetLock { key });
                        }
                        let _ = map.get(&key).unwrap();
                    } else {
                        debug!("Chunk does not exist: {}", &key);
                        return Err(ChunkSampleError::ChunkMissing { key });
                    }
                }
            }
        }
        Ok(ChunkSample::new_internal(self.vol_size, from, to, map))
    }

    pub fn gen(&self, pos: Vec3<VolOffs>) {
        // this function must work multithreaded
        let gen_vol = self.gen.gen_vol.clone();
        let gen_payload = self.gen.gen_payload.clone();
        let pen = self.pending.clone();
        let con = Arc::new(Mutex::new(None));
        {
            // the lock below guarantees that no 2 threads can generate the same chunk
            let mut pen_lock = pen.write();
            if pen_lock.get(&pos).is_some() {
                return;
            }
            pen_lock.insert(pos, con.clone());
        }
        // run expensive operations in own thread

        POOL.lock().execute(move || {
            gen_vol(pos, con.clone());
            gen_payload(pos, con.clone());
        });
    }

    pub fn drop(&self, pos: Vec3<VolOffs>) {
        // this function must work multithreaded
        let drop_vol = self.gen.drop_vol.clone();
        let drop_payload = self.gen.drop_payload.clone();

        if let Some(rem) = self.pers.write().remove(&pos) {
            POOL.lock().execute(move || {
                drop_vol(pos, rem.clone());
                drop_payload(pos, rem.clone());
            });
        }
    }

    // regually call this to copy over generated chunks
    pub fn maintain(&self) {
        {
            // handle new generated chunks
            let mut pen_lock = self.pending.write();
            let mut map = HashMap::new();

            // move generated to persistency
            for (pos, con_arc) in pen_lock.drain() {
                if con_arc.lock().is_some() {
                    let m = Arc::try_unwrap(con_arc);
                    match m {
                        Ok(m) => {
                            let opt = m.into_inner();
                            let arc = Arc::new(opt.unwrap());
                            self.pers.write().insert(pos, arc);
                        },
                        Err(con_arc) => {
                            map.insert(pos, con_arc);
                        },
                    }
                } else {
                    map.insert(pos, con_arc);
                }
            }

            // move items back
            for (pos, con_arc) in map.drain() {
                pen_lock.insert(pos, con_arc);
            }
        }

        // generate new chunks
        let mut chunk_map = HashMap::new();
        let block_loader: Vec<BlockLoader> = self.block_loader.read().iter().map(|e| (*e.read()).clone()).collect(); // buffer blockloader

        // Collect chunks around the player
        for bl in block_loader.iter() {
            let pos = bl.pos;
            let size = bl.size;
            let pos_chunk = terrain::voxabs_to_voloffs(pos, self.vol_size);
            let from = terrain::voxabs_to_voloffs(pos - size, self.vol_size);
            let to = terrain::voxabs_to_voloffs(pos + size, self.vol_size);
            for i in from.x..to.x + 1 {
                for j in from.y..to.y + 1 {
                    // Chunks are 64 blocks, and the world limit is 512, so this is 8 chunks
                    // TODO: Don't hard-code this
                    for k in 0..(512i32 / self.vol_size.z as i32) {
                        let ijk = Vec3::new(i, j, k);
                        let diff = (pos_chunk - ijk).map(|e| e.abs()).sum();
                        if let Some(old_diff) = chunk_map.get(&ijk) {
                            if *old_diff < diff {
                                continue;
                            }
                        }
                        chunk_map.insert(ijk, diff);
                    }
                }
            }
        }
        let mut chunks: Vec<(Vec3<VolOffs>, VolOffs)> = chunk_map.iter().map(|pd| (*pd.0, *pd.1)).collect();
        chunks.sort_by(|a, b| a.1.cmp(&b.1));

        // Generate chunks around the player
        const MAX_CHUNKS_IN_QUEUE: usize = 12; // to not overkill the vol_mgr
        for (pos, _diff) in chunks.iter() {
            if !self.exists_chunk(*pos) {
                // generate up to MAX_CHUNKS_IN_QUEUE chunks around the player
                if self.pending_chunk_cnt() < MAX_CHUNKS_IN_QUEUE {
                    self.gen(*pos);
                }
            }
        }

        let diff_till_unload_square: VoxAbs = ((self.vol_size.x as i64)*2).pow(2) /*3 chunks away from everything*/;
        // unload all chunks which have a distance of DIFF_TILL_UNLOAD to a loaded area

        // drop old chunks
        let mut to_remove = Vec::new(); //needed for lock on pers
        for (k, _) in self.pers.read().iter() {
            // skip if exists in HashMap
            if chunk_map.contains_key(k) {
                continue;
            }
            let k_mid = terrain::voloffs_to_voxabs(*k, self.vol_size) + self.vol_size.map(|e| e as i64 / 2);
            let mut lowest_dist = diff_till_unload_square - 1; // bigger than DIFF_TILL_UNLOAD
                                                               // get block distance to nearest blockloader
            for bl in block_loader.iter() {
                let pos = bl.pos;
                let size = bl.size;
                let dist = pos.distance_squared(k_mid);
                if dist - size.magnitude_squared() < lowest_dist {
                    lowest_dist = dist;
                }
            }
            if lowest_dist > diff_till_unload_square {
                to_remove.push(*k);
            }
        }

        for k in to_remove.iter() {
            self.drop(*k);
        }
    }

    pub fn debug(&self) {
        let mut rle = 0;
        let mut homo = 0;
        let mut hetero = 0;
        let mut heteroandrle = 0;
        for (_, a) in self.pers.read().iter() {
            let data = a.data();
            if data.contains(PersState::Homo) {
                homo += 1;
            }
            if data.contains(PersState::Rle) {
                if data.contains(PersState::Hetero) {
                    heteroandrle += 1;
                } else {
                    rle += 1;
                }
            }
            if data.contains(PersState::Hetero) {
                if data.contains(PersState::Rle) {
                    heteroandrle += 1;
                } else {
                    hetero += 1;
                }
            }
        }
        info!(
            "number of chunks; hetero {}, rle {}, homo {}, hetero&rle {}",
            hetero, rle, homo, heteroandrle
        );
    }

    pub fn remove(&self, pos: Vec3<VolOffs>) -> bool { self.pers.write().remove(&pos).is_some() }

    pub fn pending_chunk_cnt(&self) -> usize { self.pending.read().len() }

    pub fn pers<F>(&self, filter: F) -> HashMap<Vec3<VolOffs>, Arc<ChunkContainer<P>>>
    where
        F: Fn(&Vec3<VolOffs>) -> bool,
    {
        //dont give access to the real persistency lock here
        let mut new_map = HashMap::new();
        for (k, a) in self.pers.read().iter() {
            if filter(k) {
                new_map.insert(*k, a.clone());
            }
        }
        return new_map;
    }

    pub fn block_loader(&self) -> RwLockReadGuard<Vec<Arc<RwLock<BlockLoader>>>> { self.block_loader.read() }

    pub fn block_loader_mut(&self) -> RwLockWriteGuard<Vec<Arc<RwLock<BlockLoader>>>> { self.block_loader.write() }
}
