// Standard
use std::{collections::{HashSet, HashMap}, sync::Arc, thread, time::Duration};

// Library
use parking_lot::{Mutex, RwLock, RwLockReadGuard};
use threadpool::ThreadPool;
use vek::*;

// Local
use terrain::{Container, Key, PersState, VoxelRelVec, VoxelAbsVec, VolumeIdxVec, VolPers, VolGen, Volume, ReadVolume, VolCluster, Voxel};
use terrain::chunk::{ChunkContainer, Block, ChunkSample, Chunk};
use terrain;
//use physics::collision::{Collider, Primitive};

lazy_static! {
    static ref POOL: Mutex<ThreadPool> = Mutex::new(ThreadPool::new(2));
}

impl Key for VolumeIdxVec {
    fn print(&self) -> String { return format!("c{},{},{}", self.x, self.y, self.z).to_string(); }
}

/*
 --> Get absolute
 --> Trigger creation
 --> Trigger Payload Generation ?
 --> It currently tries to abstract all Chunks away and only return ChunkSample!


*/


pub struct ChunkMgr<P: Send + Sync + 'static> {
    vol_size: VoxelRelVec,
    pending: Arc<RwLock<HashSet<VolumeIdxVec>>>,
    pers: VolPers<VolumeIdxVec, ChunkContainer<P>>,
    gen: VolGen<VolumeIdxVec, ChunkContainer<P>>,
}

impl<P: Send + Sync + 'static> ChunkMgr<P> {
    pub fn new(vol_size: VoxelRelVec, gen: VolGen<VolumeIdxVec, ChunkContainer<P>>) -> ChunkMgr<P> {
        ChunkMgr {
            vol_size,
            pending: Arc::new(RwLock::new(HashSet::new())),
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

    pub fn get_sample(&self, from: VoxelAbsVec, to: VoxelAbsVec) -> ChunkSample<P> {
        let mut c = 0;
        while true {
            if let Some(sample) = self.try_get_sample(from, to) {
                return sample;
            } else {
                c += 1;
                if c > 10 {
                    warn!("Long waiting chunk sample {}", c)
                }
                thread::sleep(Duration::from_millis(10));
            }
        }
        panic!("unreachable");
    }

    /*

    impl<'a, P> ChunkSample<'a, P> {
        pub fn new(
            size: VoxelRelVec,
            block_from_abs: VoxelAbsVec,
            block_to_abs: VoxelAbsVec,
            map: HashMap<VolumeIdxVec, (&'a ChunkContainer<P>, RwLockReadGuard<'a, Chunk>)>
        ) -> Self {
    */

    pub fn try_get_sample(&self, from: VoxelAbsVec, to: VoxelAbsVec) -> Option<ChunkSample<P>> {
        let mut map = HashMap::new();
        let chunk_from = terrain::voxabs_to_volidx(from, self.vol_size);
        let chunk_to = terrain::voxabs_to_volidx(to, self.vol_size);
        let lock = self.pers.map();
        for x in chunk_from.x .. chunk_to.x + 1 {
            for y in chunk_from.y .. chunk_to.y + 1 {
                for z in chunk_from.z .. chunk_to.z + 1 {
                    let key = Vec3::new(x,y,z);

                    let cc = lock.get(&key).map(|v| v.clone());

/*
                    if cc
                        .and_then(|cc| cc.data_try().map(|value| map.insert(key, Arc::new(value)) ))
                        .is_none() {
                            return None;
                    }
                    */

                    if let Some(cc) = cc {
                        if cc.data_try().take().map(|value| map.insert(key, Arc::new(value))).is_none() {
                            return None;
                        }
                    }

                }
            }
        }
        return Some(ChunkSample::new(self.vol_size, from, to, map));
        None
    }

    pub fn gen(&self, pos: VolumeIdxVec) {
        let gen_func = self.gen.gen_func.clone();
        let payload_func = self.gen.payload_func.clone();
        let pen = self.pending.clone();
        {
            let mut pen_lock = pen.write();
            if pen_lock.get(&pos).is_some() {
                return;
            }
            pen_lock.insert(pos); // the lock above guarantees that no 2 threads can generate the same chunk
        }
        let con = Arc::new(ChunkContainer::new());
        self.pers.map_mut().insert(pos, con.clone());
        // we copied the Arc above and added the blank container to the persistence because those operations are inexpensive
        // and we can now run the chunk generaton which is expensive in a new thread without locking the whole persistence

        POOL.lock().execute(move || {
            gen_func(pos, &con);
            payload_func(pos, &con);
            pen.write().remove(&pos);
        });
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

/*
    pub fn contains(&self, pos: Vec3<i64>) -> bool {
        const dasd: Vec3<i64> = Vec3::new(0,0,0);
        self.pers.get(&pos).is_some() || self.pending.read().get(&pos).is_some()
    }

    pub fn loaded(&self, pos: Vec3<i64>) -> bool {
        self.pending.read().get(&pos).is_none() && self.pers.get(&pos).is_some()
    }

    pub fn pending_cnt(&self) -> usize { return self.pending.read().len(); }

    pub fn remove(&self, pos: Vec3<i64>) -> bool { return self.pers.hot_mut().remove(&pos).is_some(); }

    pub fn gen(&self, pos: Vec3<i64>) {
        let gen_func = self.gen.gen_func.clone();
        let payload_func = self.gen.payload_func.clone();
        let pen = self.pending.clone();
        {
            let mut pen_lock = pen.write();
            if pen_lock.get(&pos).is_some() {
                return;
            }
            pen_lock.insert(pos); // the lock above guarantees that no 2 threads can generate the same chunk
        }
        let con = Arc::new(Container::new());
        self.pers.hot_mut().insert(pos, con.clone());
        // we copied the Arc above and added the blank container to the persistence because those operations are inexpensive
        // and we can now run the chunk generaton which is expensive in a new thread without locking the whole persistence

        POOL.lock().execute(move || {
            gen_func(pos, &con);
            payload_func(pos, &con);
            pen.write().remove(&pos);
        });
    }

    pub fn gen_payload(&self, pos: Vec3<i64>) {
        // regenerate the payload if it got invalid
        let payload_func = self.gen.payload_func.clone();
        let con = self.pers.get(&pos).unwrap().clone();
        POOL.lock().execute(move || {
            payload_func(pos, &con);
        });
    }

    pub fn get_voxel_at(&self, pos: Vec3<i64>) -> Block {
        let vol_pos = Vec3::new(
            pos.x.div_euc(self.vol_size.x),
            pos.y.div_euc(self.vol_size.y),
            pos.z.div_euc(self.vol_size.z),
        );
        let vox_pos = Vec3::new(
            pos.x.mod_euc(self.vol_size.x),
            pos.y.mod_euc(self.vol_size.y),
            pos.z.mod_euc(self.vol_size.z),
        );
        let ref data_ref = self.pers.map();
        if let Some(container) = data_ref.get(&vol_pos) {
            if let Some(any_vol) = container.data().get(PersState::Raw) {
                //TODO: also allow for other datasets other than Raw, e.g. Rle
                return any_vol.at(vox_pos).unwrap_or(Block::empty());
            };
        };
        Block::empty()
    }*/
}

/*
pub struct VolMgrIter<
    'a,
    V: 'static + Volume,
    C: 'a + VolContainer<VoxelType = V::VoxelType>,
    VC: 'a + VolConverter<C>,
    P: Send + Sync + 'static,
> {
    cur: Vec3<i64>,
    low: Vec3<i64>,
    high: Vec3<i64>,
    mgr: &'a VolMgr<V, C, VC, P>,
}

impl<
        'a,
        V: 'static + Volume,
        C: VolContainer<VoxelType = V::VoxelType>,
        VC: VolConverter<C>,
        P: Send + Sync + 'static,
    > Iterator for VolMgrIter<'a, V, C, VC, P>
{
    type Item = Primitive;

    fn next(&mut self) -> Option<Self::Item> {
        while self.cur.z < self.high.z {
            while self.cur.y < self.high.y {
                while self.cur.x < self.high.x {
                    if self.mgr.get_voxel_at(self.cur).is_solid() {
                        let col = Primitive::new_cuboid(
                            Vec3::new(
                                self.cur.x as f32 + 0.5,
                                self.cur.y as f32 + 0.5,
                                self.cur.z as f32 + 0.5,
                            ),
                            Vec3::new(0.5, 0.5, 0.5),
                        );
                        self.cur.x += 1;
                        return Some(col);
                    }
                    self.cur.x += 1;
                }
                self.cur.x = self.low.x;
                self.cur.y += 1;
            }
            self.cur.y = self.low.y;
            self.cur.z += 1;
        }
        None
    }
}

impl<
        'a,
        V: 'static + Volume,
        C: 'a + VolContainer<VoxelType = V::VoxelType>,
        VC: 'a + VolConverter<C>,
        P: Send + Sync + 'static,
    > Collider<'a> for VolMgr<V, C, VC, P>
{
    type Iter = VolMgrIter<'a, V, C, VC, P>;

    fn get_nearby(&'a self, col: &Primitive) -> Self::Iter {
        let scale = Vec3::new(1.0, 1.0, 1.0);
        let area = col.col_approx_abc() + scale;

        let pos = col.col_center();
        let low = pos - area;
        let high = pos + area;
        // ceil the low and floor the high for dat performance improve
        let low = low.map(|e| e.ceil() as i64);
        let high = high.map(|e| (e.floor() as i64) + 1); // +1 is for the for loop

        VolMgrIter {
            cur: low,
            low,
            high,
            mgr: self,
        }
    }

    fn get_nearby_dir(&'a self, col: &Primitive, dir: Vec3<f32>) -> Self::Iter {
        //one might optimze this later on
        let scale = Vec3::new(1.0, 1.0, 1.0);
        let dirabs = Vec3::new(dir.x.abs(), dir.y.abs(), dir.z.abs()) / 2.0;
        let area = col.col_approx_abc() + dirabs + scale;

        let pos = col.col_center() + dir / 2.0;
        let low = pos - area;
        let high = pos + area;
        // ceil the low and floor the high for dat performance improve
        let low = low.map(|e| e.ceil() as i64);
        let high = high.map(|e| (e.floor() as i64) + 1); // +1 is for the for loop

        VolMgrIter {
            cur: low,
            low,
            high,
            mgr: self,
        }
    }
}

*/
