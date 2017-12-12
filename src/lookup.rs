use std::iter::FromIterator;

#[derive(Clone, Debug)]
pub struct Lookup<K, V, S = RandomState> {
    pub hash_map: HashMap<K, Vec<V>, S>
}

impl<K: Hash + Eq, V> Lookup<K, V, RandomState> {
    pub fn new() -> Self<K, V, RandomState> {
        Lookup { hash_map: HashMap::new() }
    }

    pub fn values(&self) -> Values<K, V> {unimplemented!()}

    pub fn values_mut(&mut self) -> ValuesMut<K, V> {unimplemented!()}

    pub fn iter(&self) -> Iter<K, V> {unimplemented!()}

    pub fn iter_mut(&mut self) -> IterMut<K, V> {unimplemented!()}

    pub fn len(&self) -> usize {unimplemented!()}

    pub fn drain(&mut self) -> Drain<K, V> {unimplemented!()}

    pub fn insert(&mut self, key: K, val: V) {
        let list = self.hash_map.entry(&key).or_insert(Vec::new());
        *list.push(value);
    }

    pub fn retain<F>(&mut self, f: F) {unimplemented!()}


}

impl<K, V> FromIterator for Lookup<K, V> {
    fn from_iter<I: IntoIterator<Item=(K, V)>>(iter: I) -> Self {
        let mut lookup = Lookup::new();

        for (key, value) in iter {
            lookup.insert(key, value);
        }

        lookup
    }
}