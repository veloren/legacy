// Local
use super::{Container, Key, PersState, VolContainer, VolConverter, VolPers, Volume, Voxel};
use collision::{Collider, Primitive};

// Standard
use std::{collections::HashSet, sync::Arc};

// Library
use parking_lot::{Mutex, RwLock};
use threadpool::ThreadPool;
use vek::*;

pub trait FnGenFunc<V: Volume, C: VolContainer<VoxelType = V::VoxelType>, P: Send + Sync + 'static>:
    Fn(Vec3<i64>, &Container<C, P>) + Send + Sync + 'static
{
}
impl<
        V: Volume,
        C: VolContainer<VoxelType = V::VoxelType>,
        P: Send + Sync + 'static,
        T: Fn(Vec3<i64>, &Container<C, P>),
    > FnGenFunc<V, C, P> for T
where
    T: Send + Sync + 'static,
{}

pub trait FnPayloadFunc<V: Volume, C: VolContainer<VoxelType = V::VoxelType>, P: Send + Sync + 'static>:
    Fn(Vec3<i64>, &Container<C, P>) + Send + Sync + 'static
{
}
impl<
        V: Volume,
        C: VolContainer<VoxelType = V::VoxelType>,
        P: Send + Sync + 'static,
        T: Fn(Vec3<i64>, &Container<C, P>),
    > FnPayloadFunc<V, C, P> for T
where
    T: Send + Sync + 'static,
{}

pub struct VolGen<V: Volume, C: VolContainer<VoxelType = V::VoxelType>, P: Send + Sync + 'static> {
    pub gen_func: Arc<FnGenFunc<V, C, P, Output = ()> + Send + Sync + 'static>,
    pub payload_func: Arc<FnPayloadFunc<V, C, P, Output = ()>>,
}

impl<V: Volume, C: VolContainer<VoxelType = V::VoxelType>, P: Send + Sync + 'static> VolGen<V, C, P> {
    pub fn new<GF: FnGenFunc<V, C, P> + Send + Sync + 'static, PF: FnPayloadFunc<V, C, P>>(
        gen_func: GF,
        payload_func: PF,
    ) -> VolGen<V, C, P> {
        VolGen {
            gen_func: Arc::new(gen_func),
            payload_func: Arc::new(payload_func),
        }
    }
}

/*
  - offload
  -  gen
*/
