// Local
use super::{
    Container, FnGenFunc, FnPayloadFunc, Key, PersState, VolContainer, VolConverter, VolGen, VolPers, Volume, Voxel,
};
use collision::{Collider, Primitive};

// Standard
use std::{collections::HashSet, sync::Arc};

// Library
use coord::prelude::*;
use parking_lot::{Mutex, RwLock};
use threadpool::ThreadPool;

pub enum VolState<C: VolContainer, P: Send + Sync + 'static> {
    Loading,
    Exists(Arc<Container<C, P>>),
}

lazy_static! {
    static ref POOL: Mutex<ThreadPool> = Mutex::new(ThreadPool::new(2));
}

impl Key for Vec3<i64> {
    fn print(&self) -> String { return format!("c{},{},{}", self.x, self.y, self.z).to_string(); }
}

pub struct VolMgr<
    V: 'static + Volume,
    C: VolContainer<VoxelType = V::VoxelType>,
    VC: VolConverter<C>,
    P: Send + Sync + 'static,
> {
    vol_size: Vec3<i64>,
    pending: Arc<RwLock<HashSet<Vec3<i64>>>>,
    pers: VolPers<Vec3<i64>, C, VC, P>,
    gen: VolGen<V, C, P>,
}

impl<V: 'static + Volume, C: VolContainer<VoxelType = V::VoxelType>, VC: VolConverter<C>, P: Send + Sync + 'static>
    VolMgr<V, C, VC, P>
{
    pub fn new(vol_size: Vec3<i64>, gen: VolGen<V, C, P>) -> VolMgr<V, C, VC, P> {
        VolMgr {
            vol_size,
            pending: Arc::new(RwLock::new(HashSet::new())),
            pers: VolPers::new(),
            gen,
        }
    }

    pub fn at(&self, pos: Vec3<i64>) -> Option<VolState<C, P>> {
        if let Some(con) = self.pers.get(&pos) {
            return Some(VolState::Exists(con));
        } else if self.pending.read().get(&pos).is_some() {
            return Some(VolState::Loading);
        }
        return None;
    }

    pub fn persistence<'a>(&'a self) -> &'a VolPers<Vec3<i64>, C, VC, P> { &self.pers }

    pub fn contains(&self, pos: Vec3<i64>) -> bool {
        return self.pers.get(&pos).is_some() || self.pending.read().get(&pos).is_some();
    }

    pub fn loaded(&self, pos: Vec3<i64>) -> bool {
        return self.pending.read().get(&pos).is_none() && self.pers.get(&pos).is_some();
    }

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

    pub fn get_voxel_at(&self, pos: Vec3<i64>) -> V::VoxelType {
        let vol_pos = pos.div_euc(self.vol_size);
        let vox_pos = vec3!(
            pos.x.mod_euc(self.vol_size.x),
            pos.y.mod_euc(self.vol_size.y),
            pos.z.mod_euc(self.vol_size.z)
        );
        let ref data_ref = self.pers.hot();
        if let Some(container) = data_ref.get(&vol_pos) {
            if let Some(any_vol) = container.vols().get(PersState::Raw) {
                //TODO: also allow for other datasets other than Raw, e.g. Rle
                return any_vol.at(vox_pos).unwrap_or(V::VoxelType::empty());
            };
        };
        return V::VoxelType::empty();
    }
}

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
