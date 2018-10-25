// Standard
use std::{collections::{HashSet, HashMap}, sync::Arc, thread, time::Duration};
use std::ops::Deref;

// Library
use parking_lot::{Mutex, RwLock, RwLockReadGuard};
use threadpool::ThreadPool;
use vek::*;

// Local
use terrain::{Container, Key, PersState, VoxelRelVec, VoxelAbsVec, VolumeIdxVec, VolPers, VolGen, Volume, ReadVolume, VolCluster, Voxel};
use terrain::chunk::{ChunkContainer, Block, ChunkSample, Chunk, HomogeneousData};
use terrain;

lazy_static! {
    static ref POOL: Mutex<ThreadPool> = Mutex::new(ThreadPool::new(2));
}

impl Key for VolumeIdxVec {
    fn print(&self) -> String { return format!("c{},{},{}", self.x, self.y, self.z).to_string(); }
}

#[derive(Debug, PartialEq)]
pub enum ChunkSampleError {
  ChunkMissing,
  CannotGetLock,
  NoContent,
}

pub struct ChunkMgr<P: Send + Sync + 'static> {
    vol_size: VoxelRelVec,
    pending: Arc<RwLock<HashMap<VolumeIdxVec, Arc<Mutex<Option<ChunkContainer<P>>>>>>>, // Mutex is only needed for compiler, we dont acces it in multiple threads
    pers: VolPers<VolumeIdxVec, ChunkContainer<P>>,
    gen: VolGen<VolumeIdxVec, ChunkContainer<P>>,
}

impl<P: Send + Sync + 'static> ChunkMgr<P> {
    pub fn new(vol_size: VoxelRelVec, gen: VolGen<VolumeIdxVec, ChunkContainer<P>>) -> ChunkMgr<P> {
        ChunkMgr {
            vol_size,
            pending: Arc::new(RwLock::new(HashMap::new())),
            pers: VolPers::new(),
            gen,
        }
    }

    pub fn exists_block(&self, pos: VoxelAbsVec) -> bool {
        self.exists_chunk(terrain::voxabs_to_volidx(pos, self.vol_size))
    }

    pub fn exists_chunk(&self, pos: VolumeIdxVec) -> bool {
        self.pers.map().get(&pos).is_some()
    }

    pub fn get_block(&self, pos: VoxelAbsVec) -> Option<Block> {
        let chunk = terrain::voxabs_to_volidx(pos, self.vol_size);
        let off = terrain::voxabs_to_voxrel(pos, self.vol_size);
        let map = self.pers.map();
        let chunk = map.get(&chunk);
        if let Some(chunk) = chunk {
            let lock = chunk.data();
            let hetero = lock.get(PersState::Hetero);
            if let Some(hetero) = hetero {
                return hetero.at(off)
            }
        }
        None
    }

    // Tries getting a Sample
    pub fn try_get_sample(&self, from: VoxelAbsVec, to: VoxelAbsVec) -> Result<ChunkSample, ChunkSampleError> {
        let mut c = 0;
        while true {
            match self.get_sample(from, to) {
                 Ok(sample) => return Ok(sample),
                 Err(e) => {
                     if e == ChunkSampleError::CannotGetLock {
                         c += 1;
                         if c > 10 {
                             warn!("Long waiting chunk sample {}", c)
                         }
                         thread::sleep(Duration::from_millis(10));
                     } else {
                         return Err(e);
                     }
                 }
            }
        }
        panic!("unreachable");
    }

    pub fn get_sample(&self, from: VoxelAbsVec, to: VoxelAbsVec) -> Result<ChunkSample, ChunkSampleError> {
        let mut map = HashMap::new();
        let chunk_from = terrain::voxabs_to_volidx(from, self.vol_size);
        let chunk_to = terrain::voxabs_to_volidx(to, self.vol_size);
        let lock = self.pers.map();
        for x in chunk_from.x .. chunk_to.x + 1 {
            for y in chunk_from.y .. chunk_to.y + 1 {
                for z in chunk_from.z .. chunk_to.z + 1 {
                    let key = Vec3::new(x,y,z);
                    let cc = lock.get(&key).map(|v| v.clone());
                    if let Some(cc) = cc {
                        if cc.data_try().take().map(|value| map.insert(key, Arc::new(value))).is_none() {
                            return Err(ChunkSampleError::CannotGetLock);
                        }
                        let v = map.get(&key).unwrap();
                    } else {
                        debug!("Chunk does not exist: {}", &key);
                        return Err(ChunkSampleError::ChunkMissing);
                    }

                }
            }
        }
        Ok(ChunkSample::new_internal(self.vol_size, from, to, map))
    }

    pub fn gen(&self, pos: VolumeIdxVec) {
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

    // regually call this to copy over generated chunks
    pub fn maintain(&self) {
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
                        self.pers.map_mut().insert(pos, arc);
                    },
                    Err(con_arc) => {
                        map.insert(pos, con_arc);
                    }
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

    pub fn remove(&self, pos: VolumeIdxVec) -> bool { self.pers.map_mut().remove(&pos).is_some() }

    pub fn pending_chunk_cnt(&self) -> usize { self.pending.read().len() }

    pub fn map(&self) -> HashMap<VolumeIdxVec, Arc<ChunkContainer<P>>> {
        // I just dont want to give access to the real persistency here
        let lock = self.pers.map();
        let mut new_map = HashMap::new();
        for (k, a) in lock.iter() {
            new_map.insert(*k, a.clone());
        }
        return new_map;
    }
}
