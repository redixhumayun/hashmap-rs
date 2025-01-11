#![allow(dead_code)]
use std::{
    fmt::Display,
    hash::{DefaultHasher, Hash, Hasher},
};

use anyhow::{anyhow, bail};

pub trait Key = Hash + Clone + PartialEq + Display;
pub trait Value = Clone;

const LOAD_FACTOR_LIMIT: f64 = 0.7;

#[derive(Clone)]
enum Entry<K, V> {
    Empty,
    Deleted(K),
    Occupied(K, V),
}

pub struct HashMap<K, V>
where
    K: Key,
    V: Value,
{
    data: Vec<Entry<K, V>>,
    capacity: usize,
    size: usize,
}

impl<K, V> HashMap<K, V>
where
    K: Key,
    V: Value,
{
    pub fn new(capacity: usize) -> Self {
        let initial_capacity = 16.max(capacity.next_power_of_two());
        let data = vec![Entry::Empty; initial_capacity];
        Self {
            data,
            capacity: initial_capacity,
            size: 0,
        }
    }

    fn hash(&self, key: &K) -> usize
    where
        K: Key,
    {
        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        (hasher.finish() as usize) % self.capacity
    }

    pub fn get(&self, key: K) -> anyhow::Result<Option<V>>
    where
        V: Value,
    {
        let index = self.hash(&key);
        let mut current_index = index;
        loop {
            match self.data.get(current_index) {
                Some(Entry::Empty) => return anyhow::Ok(None),
                Some(Entry::Occupied(k, v)) => {
                    if *k == key {
                        return anyhow::Ok(Some(v.clone()));
                    }
                    current_index = (current_index + 1) % self.capacity;
                }
                Some(Entry::Deleted(_)) => {
                    current_index = (current_index + 1) % self.capacity;
                }
                None => {
                    bail!("entry at {index} cannot be found. seems like an issue with the hash function")
                }
            };
            if current_index == index {
                return anyhow::Ok(None);
            }
        }
    }

    fn get_load_factor(&self) -> f64 {
        self.size as f64 / self.capacity as f64
    }

    pub fn insert(&mut self, key: K, value: V) -> anyhow::Result<()> {
        if self.get_load_factor() >= LOAD_FACTOR_LIMIT {
            self.resize();
        }

        let index = self.hash(&key);
        let mut current_index = index;
        loop {
            match self.data.get(current_index) {
                Some(Entry::Empty) => {
                    self.data[current_index] = Entry::Occupied(key, value);
                    self.size += 1;
                    return Ok(());
                }
                Some(Entry::Deleted(_)) => {
                    self.data[current_index] = Entry::Occupied(key, value);
                    self.size += 1;
                    return Ok(());
                }
                Some(Entry::Occupied(_, _)) => {
                    current_index = (current_index + 1) % self.capacity;
                }
                None => {
                    bail!("entry at {index} cannot be found. seems like an issue with the hash function");
                }
            };
            if current_index == index {
                bail!("entry for key {key} cannot be inserted. seems like an issue with the hash function");
            }
        }
    }

    fn resize(&mut self) {
        let old_capacity = self.capacity;
        let new_capacity = old_capacity << 1;

        // Calculate sizes
        // let entry_size = std::mem::size_of::<Entry<K, V>>();
        // let vec_size = new_capacity * entry_size;
        // println!("Resize Stats:");
        // println!(
        //     "  Old capacity: {}, New capacity: {}",
        //     old_capacity, new_capacity
        // );
        // println!("  Entry size: {} bytes", entry_size);
        // println!("  New vec size: {} bytes", vec_size);
        // println!("  Current size (items): {}", self.size);
        // println!(
        //     "  Actual old vec size: {} bytes",
        //     self.data.len() * entry_size
        // );
        // let old_entries: Vec<Entry<K, V>> = self.data.drain(..).collect();

        let new_data: Vec<Entry<K, V>> = vec![Entry::Empty; new_capacity];
        let old_data = std::mem::replace(&mut self.data, new_data);
        self.capacity = new_capacity;
        for entry in old_data {
            if let Entry::Occupied(k, v) = entry {
                let mut index = self.hash(&k);
                while let Some(Entry::Occupied(_, _)) = self.data.get(index) {
                    index = (index + 1) % self.capacity;
                }
                self.data[index] = Entry::Occupied(k, v);
            }
        }
        // println!("Done resizing!!!");
    }

    pub fn delete(&mut self, key: K) -> anyhow::Result<()> {
        let index = self.hash(&key);
        let mut current_index = index;
        loop {
            match self.data.get_mut(current_index) {
                Some(Entry::Empty) => return anyhow::Ok(()),
                Some(Entry::Deleted(k)) => {
                    if *k == key {
                        return anyhow::Ok(());
                    }
                    current_index = (current_index + 1) % self.capacity;
                }
                Some(Entry::Occupied(k, _v)) => {
                    if *k == key {
                        self.data[current_index] = Entry::Deleted(key);
                        self.size -= 1;
                        return anyhow::Ok(());
                    }
                    current_index = (current_index + 1) % self.capacity;
                }
                None => {
                    bail!("entry at {index} cannot be found. seems like an issue with the hash function")
                }
            };
            if current_index == index {
                return Err(anyhow!(
                    "entry for key {key} cannot be found, so it was not deleted"
                ));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hashmap() {
        let mut map: HashMap<String, String> = HashMap::new(10);
        map.insert("key".to_string(), "value".to_string()).unwrap();
        assert_eq!(
            map.get("key".to_string()).unwrap(),
            Some("value".to_string())
        );
    }

    #[test]
    fn test_resizing() {
        let mut map: HashMap<String, String> = HashMap::new(10);
        for i in 0..100 {
            let key = format!("Key{i}");
            let value = format!("Value{i}");
            map.insert(key, value).unwrap();
        }

        for i in 0..100 {
            let key = format!("Key{i}");
            let value = map.get(key).unwrap();
            assert_eq!(value, Some(format!("Value{i}")));
        }
    }

    #[test]
    fn test_mix_workload() {
        let mut map: HashMap<String, String> = HashMap::new(10);
        for i in 0..100 {
            let key = format!("Key{i}");
            let value = format!("Value{i}");
            map.insert(key.clone(), value.clone()).unwrap();
        }
        //  delete every 5th key
        for i in 0..100 {
            if i % 5 == 0 {
                let key = format!("Key{i}");
                map.delete(key).unwrap();
            }
        }
        //  check if remaining keys exist
        for i in 0..100 {
            if i % 5 == 0 {
                let key = format!("Key{i}");
                assert_eq!(map.get(key).unwrap(), None);
            } else {
                let key = format!("Key{i}");
                assert_eq!(map.get(key).unwrap(), Some(format!("Value{i}")));
            }
        }
    }

    #[test]
    fn profile_memory_patterns() {
        let mut map: HashMap<String, String> = HashMap::new(16);

        // Three distinct workload phases to stress different patterns

        // Phase 1: Controlled growth causing resizes
        for i in 0..1_000_000 {
            map.insert(format!("key_{}", i), "x".repeat(1000)).unwrap();
        }

        // Phase 2: Heavy updates/overwrites
        for i in 0..50_000 {
            map.insert(format!("key_{}", i), "y".repeat(200)).unwrap();
        }

        // Phase 3: Mixed deletes and inserts
        for i in 0..75_000 {
            if i % 2 == 0 {
                map.delete(format!("key_{}", i)).unwrap();
            } else {
                map.insert(format!("key_new_{}", i), "z".repeat(150))
                    .unwrap();
            }
        }
    }
}
