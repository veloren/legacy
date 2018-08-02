// Local
use collision::{Collider, Primitive};
use Volume;
use Voxel;

// Standard
use std::{
    collections::HashMap,
    sync::{Arc, Mutex, RwLock, RwLockReadGuard},
};

// Library
use coord::prelude::*;
use threadpool::ThreadPool;

pub enum VolState<V: Volume, P> {
    Loading,
    Exists(V, P),
}

lazy_static! {
    static ref POOL: Mutex<ThreadPool> = Mutex::new(ThreadPool::new(2));
}

pub trait FnGenFunc<V>: Fn(Vec2<i64>) -> V + Send + Sync + 'static {}
impl<V, T: Fn(Vec2<i64>) -> V + Send + Sync + 'static> FnGenFunc<V> for T {}

pub trait FnPayloadFunc<V, P: Send + Sync + 'static>: Fn(&V) -> P + Send + Sync + 'static {}
impl<V, P: Send + Sync + 'static, T: Fn(&V) -> P + Send + Sync + 'static> FnPayloadFunc<V, P> for T {}

pub struct VolGen<V: Volume, P: Send + Sync + 'static> {
    gen_func: Arc<FnGenFunc<V, Output = V>>,
    payload_func: Arc<FnPayloadFunc<V, P, Output = P>>,
}

impl<V: Volume, P: Send + Sync + 'static> VolGen<V, P> {
    pub fn new<GF: FnGenFunc<V>, PF: FnPayloadFunc<V, P>>(gen_func: GF, payload_func: PF) -> VolGen<V, P> {
        VolGen {
            gen_func: Arc::new(gen_func),
            payload_func: Arc::new(payload_func),
        }
    }
}

pub struct VolMgr<V: 'static + Volume, P: Send + Sync + 'static> {
    vol_size: i64,
    vols: RwLock<HashMap<Vec2<i64>, Arc<RwLock<VolState<V, P>>>>>,
    gen: VolGen<V, P>,
}

impl<V: 'static + Volume, P: Send + Sync + 'static> VolMgr<V, P> {
    pub fn new(vol_size: i64, gen: VolGen<V, P>) -> VolMgr<V, P> {
        VolMgr {
            vol_size,
            vols: RwLock::new(HashMap::new()),
            gen,
        }
    }

    pub fn at(&self, pos: Vec2<i64>) -> Option<Arc<RwLock<VolState<V, P>>>> {
        self.vols.read().unwrap().get(&pos).map(|v| v.clone())
    }

    pub fn volumes<'a>(&'a self) -> RwLockReadGuard<'a, HashMap<Vec2<i64>, Arc<RwLock<VolState<V, P>>>>> {
        self.vols.read().unwrap()
    }

    pub fn contains(&self, pos: Vec2<i64>) -> bool { self.vols.read().unwrap().contains_key(&pos) }

    pub fn loaded(&self, pos: Vec2<i64>) -> bool {
        self.at(pos)
            .map(|v| {
                if let VolState::Loading = *v.read().unwrap() {
                    false
                } else {
                    true
                }
            })
            .unwrap_or(false)
    }

    pub fn remove(&self, pos: Vec2<i64>) -> bool { self.vols.write().unwrap().remove(&pos).is_some() }

    pub fn gen(&self, pos: Vec2<i64>) {
        if self.contains(pos) {
            return; // Don't try to generate the same chunk twice
        }

        let gen_func = self.gen.gen_func.clone();
        let payload_func = self.gen.payload_func.clone();
        let vol_state = Arc::new(RwLock::new(VolState::Loading));
        self.vols.write().unwrap().insert(pos, vol_state.clone());
        POOL.lock().unwrap().execute(move || {
            let vol = gen_func(pos);
            let payload = payload_func(&vol);
            *vol_state.write().unwrap() = VolState::Exists(vol, payload);
        });
    }

    pub fn set(&self, pos: Vec2<i64>, vol: V, payload: P) {
        self.vols
            .write()
            .unwrap()
            .insert(pos, Arc::new(RwLock::new(VolState::Exists(vol, payload))));
    }

    pub fn get_voxel_at(&self, pos: Vec3<i64>) -> V::VoxelType {
        let vol_pos = vec2!(pos.x.div_euc(self.vol_size), pos.y.div_euc(self.vol_size));

        let vox_pos = vec3!(pos.x.mod_euc(self.vol_size), pos.y.mod_euc(self.vol_size), pos.z);

        self.vols
            .read()
            .unwrap()
            .get(&vol_pos)
            .map(|v| match *v.read().unwrap() {
                VolState::Loading => V::VoxelType::empty(),
                VolState::Exists(ref v, _) => v.at(vox_pos).unwrap_or(V::VoxelType::empty()),
            })
            .unwrap_or(V::VoxelType::empty())
    }
}

pub struct VolMgrIter<'a, V: 'static + Volume, P: Send + Sync + 'static> {
    cur: Vec3<i64>,
    low: Vec3<i64>,
    high: Vec3<i64>,
    mgr: &'a VolMgr<V, P>,
}

impl<'a, V: 'static + Volume, P: Send + Sync + 'static> Iterator for VolMgrIter<'a, V, P> {
    type Item = Primitive;

    fn next(&mut self) -> Option<Self::Item> {
        while self.cur.z < self.high.z {
            while self.cur.y < self.high.y {
                while self.cur.x < self.high.x {
                    if self.mgr.get_voxel_at(self.cur).is_solid() {
                        let col = Primitive::new_cuboid(
                            vec3!(
                                self.cur.x as f32 + 0.5,
                                self.cur.y as f32 + 0.5,
                                self.cur.z as f32 + 0.5
                            ),
                            vec3!(0.5, 0.5, 0.5),
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

impl<'a, V: 'static + Volume, P: Send + Sync + 'static> Collider<'a> for VolMgr<V, P> {
    type Iter = VolMgrIter<'a, V, P>;

    fn get_nearby(&'a self, col: &Primitive) -> Self::Iter {
        let scale = vec3!(1.0, 1.0, 1.0);
        let area = col.col_approx_abc() + scale;

        let pos = col.col_center();
        let low = pos - area;
        let high = pos + area;
        // ceil the low and floor the high for dat performance improve
        let low = low.map(|e| e.ceil() as i64);
        let high = high.map(|e| (e.floor() as i64) + 1); // +1 is for the for loop

        return VolMgrIter {
            cur: low,
            low,
            high,
            mgr: self,
        };
    }

    fn get_nearby_dir(&'a self, col: &Primitive, dir: Vec3<f32>) -> Self::Iter {
        //one might optimze this later on
        let scale = vec3!(1.0, 1.0, 1.0);
        let dirabs = vec3!(dir.x.abs(), dir.y.abs(), dir.z.abs()) / 2.0;
        let area = col.col_approx_abc() + dirabs + scale;

        let pos = col.col_center() + dir / 2.0;
        let low = pos - area;
        let high = pos + area;
        // ceil the low and floor the high for dat performance improve
        let low = low.map(|e| e.ceil() as i64);
        let high = high.map(|e| (e.floor() as i64) + 1); // +1 is for the for loop

        return VolMgrIter {
            cur: low,
            low,
            high,
            mgr: self,
        };
    }
}
