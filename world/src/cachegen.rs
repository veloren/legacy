// Standard
use std::hash::{Hash, Hasher};

// Library
use fnv::FnvHasher;
use parking_lot::RwLock;

// Local
use crate::Gen;

pub struct CacheGen<T, I, O>
where
    I: Eq + Hash,
    O: 'static,
{
    cache: Vec<RwLock<Option<(I, O)>>>,
    gen: T,
}

impl<T, I, O> CacheGen<T, I, O>
where
    I: Eq + Hash,
    O: 'static,
{
    pub fn new(gen: T, cache_size: usize) -> Self {
        let mut cache = vec![];
        for _ in 0..cache_size {
            cache.push(RwLock::new(None));
        }

        Self { cache, gen }
    }

    pub fn internal(&self) -> &T { &self.gen }
}

impl<S, T: Gen<S>> Gen<S> for CacheGen<T, T::In, T::Out>
where
    T::In: Eq + Hash,
    T::Out: 'static,
{
    type In = T::In;
    type Out = T::Out;

    fn sample<'a>(&'a self, i: Self::In, supplement: &'a S) -> Self::Out {
        let mut hasher = FnvHasher::with_key(0);
        i.hash(&mut hasher);

        let idx = hasher.finish() as usize % self.cache.len();

        if let Some(Some(o)) = self.cache.get(idx).and_then(|c| {
            c.read().clone().map(|item| {
                let (cached_i, o) = item.clone();
                if cached_i == i {
                    Some(o)
                } else {
                    None
                }
            })
        }) {
            o
        } else {
            let samp = self.gen.sample(i.clone(), supplement);
            self.cache.get(idx).map(|c| *c.write() = Some((i, samp.clone())));
            samp
        }
    }
}
