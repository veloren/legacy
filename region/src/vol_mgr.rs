// Local
use collision::{Collider, Primitive};
use vol_per::{Container, PersState, VolPers, VolumeConverter};
use Volume;
use Voxel;

// Standard
use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, Mutex, RwLock, RwLockReadGuard},
};

// Library
use coord::prelude::*;
use threadpool::ThreadPool;

pub enum VolState<V: Voxel, P: Send + Sync + 'static> {
    Loading,
    Exists(Arc<RwLock<Container<V, P>>>),
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

pub struct VolMgr<V: 'static + Volume, VC: VolumeConverter<V::VoxelType>, P: Send + Sync + 'static> {
    vol_size: i64,
    pending: Arc<RwLock<HashSet<Vec2<i64>>>>,
    pers: VolPers<Vec2<i64>, V::VoxelType, VC, P>,
    gen: VolGen<V, P>,
}

impl<V: 'static + Volume, VC: VolumeConverter<V::VoxelType>, P: Send + Sync + 'static> VolMgr<V, VC, P> {
    pub fn new(vol_size: i64, gen: VolGen<V, P>) -> VolMgr<V, VC, P> {
        VolMgr {
            vol_size,
            pending: Arc::new(RwLock::new(HashSet::new())),
            pers: VolPers::new(),
            gen,
        }
    }

    pub fn at(&self, pos: Vec2<i64>) -> Option<VolState<V::VoxelType, P>> {
        let p = self.pers.get(&pos);
        if let Some(p) = p {
            return Some(VolState::Exists(p));
        } else {
            if self.pending.read().unwrap().get(&pos).is_some() {
                return Some(VolState::Loading);
            }
        }
        return None;
    }

    pub fn persistence<'a>(&'a self) -> &'a VolPers<Vec2<i64>, V::VoxelType, VC, P> { &self.pers }

    pub fn contains(&self, pos: Vec2<i64>) -> bool {
        let p = self.pers.get(&pos);
        if let Some(p) = p {
            return true;
        } else {
            if self.pending.read().unwrap().get(&pos).is_some() {
                return true;
            }
        }
        return false;
    }

    pub fn loaded(&self, pos: Vec2<i64>) -> bool {
        if self.pending.read().unwrap().get(&pos).is_some() {
            return false;
        } else {
            let p = self.pers.get(&pos);
            return p.is_some();
        }
    }

    pub fn remove(&self, pos: Vec2<i64>) -> bool {
        let o = self.pers.data_mut().remove(&pos);
        return o.is_some();
    }

    pub fn gen(&self, pos: Vec2<i64>) {
        if self.contains(pos) {
            return; // Don't try to generate the same chunk twice
        }
        //TODO: this is not thread safe, calling it really fast might end up being 2 threads in this path!
        let gen_func = self.gen.gen_func.clone();
        let payload_func = self.gen.payload_func.clone();
        let pen = self.pending.clone();
        pen.write().unwrap().insert(pos);
        let con = Arc::new(RwLock::new(Container::new()));
        self.pers.data_mut().insert(pos, con.clone());
        POOL.lock().unwrap().execute(move || {
            let vol = gen_func(pos);
            let payload = payload_func(&vol);
            *con.write().unwrap().payload_mut() = Some(payload);
            *con.write().unwrap().get_mut(PersState::Raw) = Some(Box::new(vol));
            pen.write().unwrap().remove(&pos);
        });
    }

    pub fn set(&self, pos: Vec2<i64>, vol: V, payload: P) {
        let mut con = Container::new();
        *con.payload_mut() = Some(payload);
        *con.get_mut(PersState::Raw) = Some(Box::new(vol));
        self.pers.data_mut().insert(pos, Arc::new(RwLock::new(con)));
    }

    pub fn get_voxel_at(&self, pos: Vec3<i64>) -> V::VoxelType {
        let vol_pos = vec2!(pos.x.div_euc(self.vol_size), pos.y.div_euc(self.vol_size));

        let vox_pos = vec3!(pos.x.mod_euc(self.vol_size), pos.y.mod_euc(self.vol_size), pos.z);

        let ref d = self.pers.data();
        let ref v = d.get(&vol_pos);
        if let Some(v) = v {
            let con = v.read().unwrap();
            let any_vol = con.get(PersState::Raw); //TODO: allow any vol here for performance which suppoerts at
            if let Some(any_vol) = any_vol {
                return any_vol.at(vox_pos).unwrap_or(V::VoxelType::empty());
            };
        };
        return V::VoxelType::empty();
    }
}

pub struct VolMgrIter<'a, V: 'static + Volume, VC: 'a + VolumeConverter<V::VoxelType>, P: Send + Sync + 'static> {
    cur: Vec3<i64>,
    low: Vec3<i64>,
    high: Vec3<i64>,
    mgr: &'a VolMgr<V, VC, P>,
}

impl<'a, V: 'static + Volume, VC: VolumeConverter<V::VoxelType>, P: Send + Sync + 'static> Iterator
    for VolMgrIter<'a, V, VC, P>
{
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

impl<'a, V: 'static + Volume, VC: 'a + VolumeConverter<V::VoxelType>, P: Send + Sync + 'static> Collider<'a>
    for VolMgr<V, VC, P>
{
    type Iter = VolMgrIter<'a, V, VC, P>;

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
