// Local
use terrain::{Container, Key};

// Standard
use std::sync::Arc;

// Library
use vek::*;

pub trait FnGenFunc<K: Key, C: Container>: Fn(K, &C) + Send + Sync + 'static {
}

impl<K: Key, C: Container, T: Fn(K, &C)> FnGenFunc<K, C> for T
    where T: Send + Sync + 'static {

}

pub trait FnPayloadFunc<K: Key, C: Container>: Fn(K, &C) + Send + Sync + 'static {
}

impl<K: Key, C: Container, T: Fn(K, &C)> FnPayloadFunc<K, C> for T
    where T: Send + Sync + 'static {

}

pub struct VolGen<K: Key, C: Container> {
    pub gen_func: Arc<FnGenFunc<K, C, Output = ()>>,
    pub payload_func: Arc<FnPayloadFunc<K, C, Output = ()>>,
}

impl<K: Key, C: Container> VolGen<K, C> {
    pub fn new<GF: FnGenFunc<K, C>, PF: FnPayloadFunc<K, C>>(
        gen_func: GF,
        payload_func: PF,
    ) -> VolGen<K, C> {
        VolGen {
            gen_func: Arc::new(gen_func),
            payload_func: Arc::new(payload_func),
        }
    }
}
