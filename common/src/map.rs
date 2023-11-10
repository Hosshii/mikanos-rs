use core::hash::{BuildHasher, BuildHasherDefault, Hasher};
use core::ops::{Index, IndexMut};
use core::{borrow::Borrow, hash::Hash};

pub const DEFAULT_MAP_SIZE: usize = 256;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    LackOfCapacity,
}

pub struct FixedMap<K, V, const N: usize = DEFAULT_MAP_SIZE, H = DefaultHashBuilder> {
    arr: [Option<(K, V)>; N],
    _hash_builder: H,
}

impl<K, V, const N: usize, H> FixedMap<K, V, N, H> {
    pub fn len(&self) -> usize {
        self.arr.iter().flatten().count()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl<K, V, const N: usize> FixedMap<K, V, N, DefaultHashBuilder> {
    pub fn new() -> Self {
        Self {
            arr: [(); N].map(|_| None),
            _hash_builder: DefaultHashBuilder::default(),
        }
    }
}

impl<K, V, const N: usize> Default for FixedMap<K, V, N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<K, V, const N: usize, H> FixedMap<K, V, N, H>
where
    K: Eq + Hash,
    H: BuildHasher,
{
    fn find_index<Q>(&self, k: &Q) -> Option<usize>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        self.arr
            .iter()
            .position(|v| v.as_ref().map(|(k, _)| k.borrow()) == Some(k))
    }

    pub fn get<Q>(&self, k: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        let idx = self.find_index(k)?;
        self.arr.index(idx).as_ref().map(|(_, v)| v)
    }

    pub fn get_mut<Q>(&mut self, k: &Q) -> Option<&mut V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        let idx = self.find_index(k)?;
        self.arr.index_mut(idx).as_mut().map(|(_, v)| v)
    }

    pub fn contains_key<Q>(&self, k: &Q) -> bool
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        self.get(k).is_some()
    }

    pub fn insert(&mut self, k: K, mut v: V) -> Result<Option<V>> {
        if let Some(inner) = self.get_mut(&k) {
            core::mem::swap(inner, &mut v);
            Ok(Some(v))
        } else if self.len() >= N {
            return Err(Error::LackOfCapacity);
        } else {
            let entry = self.arr.iter_mut().find(|v| v.is_none()).unwrap();
            debug_assert!(entry.is_none());
            *entry = Some((k, v));
            Ok(None)
        }
    }

    pub fn remove<Q>(&mut self, k: &Q) -> Option<V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        let idx = self.find_index(k)?;
        let elem = core::mem::take(self.arr.index_mut(idx));
        elem.map(|(_, v)| v)
    }
}

type DefaultHashBuilder = BuildHasherDefault<DefaultHasher>;

pub struct DefaultHasher;

impl DefaultHasher {
    pub fn new() -> Self {
        Self {}
    }
}

impl Hasher for DefaultHasher {
    fn finish(&self) -> u64 {
        0 // ダミー実装
    }

    fn write(&mut self, _bytes: &[u8]) {
        // ダミー実装
    }
}

impl Default for DefaultHasher {
    fn default() -> Self {
        DefaultHasher
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        let mut map: FixedMap<u32, i32, 2> = FixedMap::new();

        assert!(map.is_empty());

        map.insert(1, 2).unwrap();
        assert_eq!(map.len(), 1);
        assert_eq!(map.get(&1), Some(&2));
        assert_eq!(map.get(&2), None);
        *map.get_mut(&1).unwrap() = 3;
        assert_eq!(map.get(&1), Some(&3));

        assert!(map.contains_key(&1));
        assert!(!map.contains_key(&2));

        assert_eq!(map.insert(1, 4), Ok(Some(3)));
        assert_eq!(map.insert(2, 2), Ok(None));
        assert_eq!(map.insert(3, 1), Err(Error::LackOfCapacity));
        assert_eq!(map.len(), 2);

        assert_eq!(map.remove(&2), Some(2));
        assert_eq!(map.remove(&2), None);
        assert_eq!(map.len(), 1);

        assert_eq!(map.insert(3, 1), Ok(None));
        assert_eq!(map.len(), 2);
    }
}
