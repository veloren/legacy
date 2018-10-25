// Standard
use std::{
    hash::{Hash, Hasher},
    cell::Cell,
    collections::hash_map::DefaultHasher,
};

// Library
use fnv::FnvHasher;
use parking_lot::RwLock;

// Local
use Gen;

pub struct CacheGen<T: Gen> where T::In: Eq + Hash, T::Out: 'static {
    cache: Vec<RwLock<Option<(T::In, T::Out)>>>,
    gen: T,
}

impl<T: Gen> CacheGen<T> where T::In: Eq + Hash, T::Out: 'static {
    pub fn new(gen: T, cache_size: usize) -> Self {
        let mut cache = vec![];
        for _ in 0..cache_size {
            cache.push(RwLock::new(None));
        }

        Self {
            cache,
            gen,
        }
    }

    pub fn internal(&self) -> &T {
        &self.gen
    }
}

impl<T: Gen> Gen for CacheGen<T> where T::In: Eq + Hash, T::Out: 'static {
    type In = T::In;
    type Out = T::Out;

    fn sample(&self, i: Self::In) -> Self::Out {
        let mut hasher = FnvHasher::with_key(0);
        i.hash(&mut hasher);

        let idx = hasher.finish() as usize % self.cache.len();

        if let Some(Some(o)) = self.cache.get(idx).and_then(|c| c.read().clone().map(|item| {
            let (cached_i, o) = item.clone();
            if cached_i == i { Some(o) } else { None }
        })) {
            o
        } else {
            let samp = self.gen.sample(i.clone());
            self.cache.get(idx).map(|c| *c.write() = Some((i, samp.clone())));
            samp
        }
    }
}
