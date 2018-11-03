// Local
use terrain::{Container, Key};

// Standard
use std::sync::Arc;

// Library
use parking_lot::Mutex;

pub trait FnGenFunc<K: Key, C: Container>: Fn(K, Arc<Mutex<Option<C>>>) + Send + Sync + 'static {}

impl<K: Key, C: Container, T: Fn(K, Arc<Mutex<Option<C>>>)> FnGenFunc<K, C> for T where T: Send + Sync + 'static {}

pub trait FnDropFunc<K: Key, C: Container>: Fn(K, Arc<C>) + Send + Sync + 'static {}

impl<K: Key, C: Container, T: Fn(K, Arc<C>)> FnDropFunc<K, C> for T where T: Send + Sync + 'static {}

pub struct VolGen<K: Key, C: Container> {
    pub gen_vol: Arc<FnGenFunc<K, C, Output = ()>>,
    pub gen_payload: Arc<FnGenFunc<K, C, Output = ()>>,
    pub drop_vol: Arc<FnDropFunc<K, C, Output = ()>>,
    pub drop_payload: Arc<FnDropFunc<K, C, Output = ()>>,
}

impl<K: Key, C: Container> VolGen<K, C> {
    pub fn new<GV: FnGenFunc<K, C>, GP: FnGenFunc<K, C>, DV: FnDropFunc<K, C>, DP: FnDropFunc<K, C>>(
        gen_vol: GV,
        gen_payload: GP,
        drop_vol: DV,
        drop_payload: DP,
    ) -> VolGen<K, C> {
        VolGen {
            gen_vol: Arc::new(gen_vol),
            gen_payload: Arc::new(gen_payload),
            drop_vol: Arc::new(drop_vol),
            drop_payload: Arc::new(drop_payload),
        }
    }
}
