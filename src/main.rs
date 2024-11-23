#![feature(trait_alias)]
use std::{
    fmt::Display,
    hash::{DefaultHasher, Hash, Hasher},
};

use anyhow::{anyhow, bail};

trait Key = Hash + Clone + PartialEq + Display;
trait Value = Clone;

const LOAD_FACTOR_LIMIT: f64 = 0.7;

#[derive(Clone)]
enum Entry<K, V> {
    Empty,
    Deleted(K),
    Occupied(K, V),
}

struct HashMap<K, V>
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
    fn new(capacity: usize) -> Self {
        let data = vec![Entry::Empty; capacity];
        Self {
            data,
            capacity,
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

    fn get(&self, key: K) -> anyhow::Result<Option<V>>
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

    fn insert(&mut self, key: K, value: V) -> anyhow::Result<()> {
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
        let new_capacity = self.capacity * 2;
        
        // Calculate sizes
        let entry_size = std::mem::size_of::<Entry<K, V>>();
        let vec_size = new_capacity * entry_size;
        
        eprintln!("Resize Stats:");
        eprintln!("  Old capacity: {}, New capacity: {}", old_capacity, new_capacity);
        eprintln!("  Entry size: {} bytes", entry_size);
        eprintln!("  New vec size: {} bytes", vec_size);
        eprintln!("  Current size (items): {}", self.size);
        
        // Track existing data
        let old_entries: Vec<Entry<K, V>> = self.data.drain(..).collect();
        eprintln!("  Actual old vec size: {} bytes", old_entries.len() * entry_size);

        //  do the resizing
        self.capacity = new_capacity;
        let mut new_data: Vec<Entry<K, V>> = vec![Entry::Empty; new_capacity];
        let old_entries: Vec<Entry<K, V>> = self.data.drain(..).collect();
        for entry in old_entries {
            if let Entry::Occupied(k, v) = entry {
                let mut index = self.hash(&k);
                while let Some(Entry::Occupied(_, _)) = new_data.get(index) {
                    index = (index + 1) % self.capacity;
                }
                new_data[index] = Entry::Occupied(k, v);
            }
        }
        self.data = new_data;
    }

    fn delete(&mut self, key: K) -> anyhow::Result<()> {
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

fn main() {
    let mut hash_map: HashMap<String, String> = HashMap::new(10);
    let key = "key".to_string();
    let value = "value".to_string();
    hash_map.insert(key.clone(), value.clone()).unwrap();
    let retrieved_value = hash_map.get(key.clone()).unwrap();
    assert_eq!(retrieved_value, Some(value.clone()));
    hash_map.delete(key.clone()).unwrap();
    assert_eq!(hash_map.get(key.clone()).unwrap(), None);
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
        eprintln!("Test starting");
        let mut map: HashMap<String, String> = HashMap::new(16);

        // Three distinct workload phases to stress different patterns

        // Phase 1: Controlled growth causing resizes
        println!("starting insertions");
        for i in 0..1_000_000 {
            map.insert(format!("key_{}", i), "x".repeat(1000)).unwrap();
        }
        println!("phase 1 complete");

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
