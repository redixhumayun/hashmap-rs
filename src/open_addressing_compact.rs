#![allow(dead_code)]
use anyhow::{anyhow, bail};
use std::{
    fmt::Display,
    hash::{DefaultHasher, Hash, Hasher},
};

pub trait Key: Hash + Clone + PartialEq + Display + Default {}
impl<T> Key for T where T: Hash + Clone + PartialEq + Display + Default {}

pub trait Value: Clone + Default {}
impl<T> Value for T where T: Clone + Default {}

const LOAD_FACTOR_LIMIT: f64 = 0.7;

// 2 bits per entry: 00 = empty, 01 = deleted, 11 = occupied
const EMPTY: u8 = 0b00;
const DELETED: u8 = 0b01;
const OCCUPIED: u8 = 0b11;

pub struct HashMap<K, V>
where
    K: Key,
    V: Value,
{
    // Each byte stores status for 4 entries (2 bits each)
    status_bits: Vec<u8>,
    // Store key-value pairs contiguously
    entries: Vec<(K, V)>,
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
        let status_size = (initial_capacity + 3) / 4; // Round up to nearest byte

        Self {
            status_bits: vec![0; status_size],
            entries: vec![(K::default(), V::default()); initial_capacity],
            capacity: initial_capacity,
            size: 0,
        }
    }

    fn hash(&self, key: &K) -> usize {
        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        (hasher.finish() as usize) % self.capacity
    }

    fn get_status(&self, index: usize) -> u8 {
        let byte_idx = index / 4;
        let bit_offset = (index % 4) * 2;
        (self.status_bits[byte_idx] >> bit_offset) & 0b11
    }

    fn set_status(&mut self, index: usize, status: u8) {
        let byte_idx = index / 4;
        let bit_offset = (index % 4) * 2;
        // Clear the two bits
        self.status_bits[byte_idx] &= !(0b11 << bit_offset);
        // Set the new status
        self.status_bits[byte_idx] |= (status & 0b11) << bit_offset;
    }

    pub fn get(&self, key: K) -> anyhow::Result<Option<V>> {
        let index = self.hash(&key);
        let mut current_index = index;

        loop {
            match self.get_status(current_index) {
                EMPTY => return Ok(None),
                OCCUPIED => {
                    if self.entries[current_index].0 == key {
                        return Ok(Some(self.entries[current_index].1.clone()));
                    }
                    current_index = (current_index + 1) % self.capacity;
                }
                DELETED => {
                    current_index = (current_index + 1) % self.capacity;
                }
                _ => unreachable!("Invalid status bits"),
            }

            if current_index == index {
                return Ok(None);
            }
        }
    }

    fn get_load_factor(&self) -> f64 {
        self.size as f64 / self.capacity as f64
    }

    pub fn insert(&mut self, key: K, value: V) -> anyhow::Result<()> {
        if self.get_load_factor() >= LOAD_FACTOR_LIMIT {
            self.resize()?;
        }

        let index = self.hash(&key);
        let mut current_index = index;

        loop {
            match self.get_status(current_index) {
                EMPTY | DELETED => {
                    self.entries[current_index] = (key, value);
                    self.set_status(current_index, OCCUPIED);
                    self.size += 1;
                    return Ok(());
                }
                OCCUPIED => {
                    if self.entries[current_index].0 == key {
                        self.entries[current_index].1 = value;
                        return Ok(());
                    }
                    current_index = (current_index + 1) % self.capacity;
                }
                _ => unreachable!("Invalid status bits"),
            }

            if current_index == index {
                bail!("HashMap is full");
            }
        }
    }

    fn resize(&mut self) -> anyhow::Result<()> {
        let new_capacity = self.capacity * 2;
        let new_status_size = (new_capacity + 3) / 4;

        let mut new_status = vec![0; new_status_size];
        let mut new_entries = vec![(K::default(), V::default()); new_capacity];

        // Keep track of old capacity for rehashing
        let old_capacity = self.capacity;
        self.capacity = new_capacity;

        // Rehash all existing entries
        for i in 0..old_capacity {
            if self.get_status(i) == OCCUPIED {
                let (key, value) = std::mem::take(&mut self.entries[i]);
                let mut new_index = self.hash(&key);

                // Find new slot
                while (new_status[new_index / 4] >> ((new_index % 4) * 2)) & 0b11 == OCCUPIED {
                    new_index = (new_index + 1) % new_capacity;
                }

                new_entries[new_index] = (key, value);
                let byte_idx = new_index / 4;
                let bit_offset = (new_index % 4) * 2;
                new_status[byte_idx] |= OCCUPIED << bit_offset;
            }
        }

        self.status_bits = new_status;
        self.entries = new_entries;
        Ok(())
    }

    pub fn delete(&mut self, key: K) -> anyhow::Result<()> {
        let index = self.hash(&key);
        let mut current_index = index;

        loop {
            match self.get_status(current_index) {
                EMPTY => return Ok(()),
                OCCUPIED => {
                    if self.entries[current_index].0 == key {
                        self.set_status(current_index, DELETED);
                        self.size -= 1;
                        return Ok(());
                    }
                    current_index = (current_index + 1) % self.capacity;
                }
                DELETED => {
                    current_index = (current_index + 1) % self.capacity;
                }
                _ => unreachable!("Invalid status bits"),
            }

            if current_index == index {
                return Err(anyhow!("Key not found"));
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
            map.insert(key.clone(), value).unwrap();
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
        // Insert 100 items
        for i in 0..100 {
            let key = format!("Key{i}");
            let value = format!("Value{i}");
            map.insert(key.clone(), value.clone()).unwrap();
        }
        // Delete every 5th key
        for i in 0..100 {
            if i % 5 == 0 {
                let key = format!("Key{i}");
                map.delete(key).unwrap();
            }
        }
        // Verify remaining keys
        for i in 0..100 {
            let key = format!("Key{i}");
            if i % 5 == 0 {
                assert_eq!(map.get(key).unwrap(), None);
            } else {
                assert_eq!(map.get(key).unwrap(), Some(format!("Value{i}")));
            }
        }
    }

    #[test]
    fn test_status_bits() {
        let mut map: HashMap<u64, u64> = HashMap::new(16);

        // Test initial state
        assert_eq!(map.get_status(0), EMPTY);

        // Test setting and getting status
        map.set_status(0, OCCUPIED);
        assert_eq!(map.get_status(0), OCCUPIED);

        map.set_status(1, DELETED);
        assert_eq!(map.get_status(1), DELETED);

        // Test multiple statuses in same byte
        map.set_status(2, OCCUPIED);
        map.set_status(3, EMPTY);
        assert_eq!(map.get_status(2), OCCUPIED);
        assert_eq!(map.get_status(3), EMPTY);
    }
}
