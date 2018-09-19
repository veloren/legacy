// Standard
use std::{
    sync::RwLock,
    collections::HashMap,
    hash::Hash,
};

pub struct Tub<K: Copy + Eq + Hash, T> {
    map: RwLock<HashMap<K, RwLock<T>>>,
}

impl<K: Copy + Eq + Hash, T> Tub<K, T> {
    pub fn new() -> Self {
        Self {
            map: RwLock::new(HashMap::new()),
        }
    }

    pub fn add(&self, key: K, item: T) -> K {
        self.map.write().unwrap().insert(key, RwLock::new(item));
        key
    }

    pub fn remove(&self, key: K) -> Option<T> {
        self.map.write().unwrap().remove(&key).map(|i| i.into_inner().unwrap())
    }


    pub fn do_for<R, F: FnOnce(&T) -> R>(&self, key: K, f: F) -> Option<R> {
        self.map.read().unwrap().get(&key).map(|i| f(&i.read().unwrap()))
    }

    pub fn do_for_mut<R, F: FnOnce(&mut T) -> R>(&self, key: K, f: F) -> Option<R> {
        self.map.read().unwrap().get(&key).map(|i| f(&mut i.write().unwrap()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tub() {
        let tub = Tub::new();

        let id = tub.add(0, 5);
        assert_eq!(id, 0);

        assert_eq!(tub.do_for(id, |i| *i), Some(5));

        assert_eq!(tub.do_for_mut(id, |i| {*i = 6; *i}), Some(6));

        assert_eq!(tub.remove(id), Some(6));
        assert_eq!(tub.remove(id), None);

        assert_eq!(tub.do_for(id, |i| *i), None);
    }
}
