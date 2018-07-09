// Local
use {Volume, Voxel};
use collide::{VolCollider, Collider};
use collision::{Collidable, CollisionResolution};

// Standard
use std::sync::{Arc, RwLock, RwLockReadGuard, Mutex};
use std::collections::HashMap;

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
    gen_func: Arc<FnGenFunc<V, Output=V>>,
    payload_func: Arc<FnPayloadFunc<V, P, Output=P>>,
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

    pub fn contains(&self, pos: Vec2<i64>) -> bool {
        self.vols.read().unwrap().contains_key(&pos)
    }

    pub fn remove(&self, pos: Vec2<i64>) -> bool {
        self.vols.write().unwrap().remove(&pos).is_some()
    }

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
        self.vols.write().unwrap().insert(pos, Arc::new(RwLock::new(VolState::Exists(vol, payload))));
    }

    pub fn get_voxel_at(&self, pos: Vec3<i64>) -> V::VoxelType {
        let vol_pos = vec2!(
            pos.x.div_euc(self.vol_size),
            pos.y.div_euc(self.vol_size)
        );

        let vox_pos = vec3!(
            pos.x.mod_euc(self.vol_size),
            pos.y.mod_euc(self.vol_size),
            pos.z
        );

        self.vols.read().unwrap()
            .get(&vol_pos)
            .map(|v| match *v.read().unwrap() {
                VolState::Loading => V::VoxelType::empty(),
                VolState::Exists(ref v, _) => v
                    .at(vox_pos)
                    .unwrap_or(V::VoxelType::empty()),
                }
            )
            .unwrap_or(V::VoxelType::empty())
    }
}

impl<V: 'static + Volume, P: Send + Sync + 'static> Collider for VolMgr<V, P> {
    fn get_nearby(&self, pos: Vec3<f32>, radius: Vec3<f32>) -> Vec<Collidable> {
        let scale = vec3!(1.0,1.0,1.0);
        let mut result = Vec::new();
        let area = radius + scale;
        let area = vec3!(area.x as i64, area.y as i64, area.z as i64) + vec3!(1,1,1);
        println!("area {}", area);

        let posi = vec3!(pos.x as i64, pos.y as i64, pos.z as i64);
        let low = posi - area;
        let high = posi + area + vec3!(1,1,1);
        println!("low {}", low);
        println!("high {}", high);

        for z in low.z..high.z {
            for x in low.x..high.x {
                for y in low.y..high.y {
                    if self.get_voxel_at(vec3!(x,y,z).map(|e| e as i64)).is_solid() {
                        let col = Collidable::new_cuboid(vec3!(x as f32 + 0.5, y as f32 + 0.5, z as f32 + 0.5), vec3!(0.5, 0.5, 0.5));
                        result.push(col);
                    }
                }
            }
        }
        return result;
    }
}
