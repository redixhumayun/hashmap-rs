#![allow(dead_code)]
use std::hash::{Hash, Hasher};
use std::{fmt::Display, hash::DefaultHasher};

use anyhow::Ok;

pub trait Key = Hash + Clone + PartialEq + Display;
pub trait Value = Clone + Display;

const LOAD_FACTOR_LIMIT: f64 = 0.7;

#[derive(Clone)]
struct Node<K, V>
where
    K: Key,
    V: Value,
{
    key: K,
    value: V,
    next: Option<Box<Node<K, V>>>,
}

impl<K, V> Node<K, V>
where
    K: Key,
    V: Value,
{
    fn new(key: K, value: V) -> Self {
        Self {
            key,
            value,
            next: None,
        }
    }
}

#[derive(Clone)]
struct LinkedList<K, V>
where
    K: Key,
    V: Value,
{
    head: Option<Box<Node<K, V>>>,
}

impl<K, V> LinkedList<K, V>
where
    K: Key,
    V: Value,
{
    fn new() -> Self {
        Self { head: None }
    }

    fn get(&self, key: K) -> anyhow::Result<Option<V>> {
        let mut current = &self.head;
        while let Some(node) = current {
            if node.key == key {
                return Ok(Some(node.value.clone()));
            }
            current = &node.next;
        }
        return Ok(None);
    }

    //  Does an insert on the LinkedList and returns false if not a new insert
    //  Returns true otherwise
    fn insert(&mut self, key: K, value: V) -> anyhow::Result<bool> {
        let mut current = &mut self.head;
        while let Some(node) = current {
            if node.key == key {
                node.value = value;
                return Ok(false);
            }
            current = &mut node.next;
        }
        *current = Some(Box::new(Node::new(key, value)));
        Ok(true)
    }

    fn delete(&mut self, key: K) -> anyhow::Result<()> {
        let mut current = &mut self.head;
        while let Some(node) = current.take() {
            if node.key == key {
                *current = node.next;
                return Ok(());
            }
            *current = Some(node);
            current = &mut current.as_mut().unwrap().next;
        }
        Ok(())
    }

    fn iter(&self) -> LinkedListIterator<K, V> {
        LinkedListIterator {
            current: self.head.as_deref(),
        }
    }
}

struct LinkedListIterator<'a, K, V>
where
    K: Key,
    V: Value,
{
    current: Option<&'a Node<K, V>>,
}

impl<'a, K, V> Iterator for LinkedListIterator<'a, K, V>
where
    K: Key,
    V: Value,
{
    type Item = (K, V);
    fn next(&mut self) -> Option<Self::Item> {
        self.current.map(|node| {
            self.current = node.next.as_deref();
            (node.key.clone(), node.value.clone())
        })
    }
}

pub struct HashMap<K, V>
where
    K: Key,
    V: Value,
{
    buckets: Vec<LinkedList<K, V>>,
    size: usize,
    capacity: usize,
}

impl<K, V> HashMap<K, V>
where
    K: Key,
    V: Value,
{
    pub fn new(capacity: usize) -> Self {
        let initial_capacity = 16.max(capacity.next_power_of_two());
        let buckets = vec![LinkedList::new(); initial_capacity];
        Self {
            buckets,
            size: 0,
            capacity: initial_capacity,
        }
    }

    fn hash(&self, key: &K) -> usize {
        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        hasher.finish() as usize % self.capacity
    }

    pub fn get(&self, key: K) -> anyhow::Result<Option<V>> {
        let index = self.hash(&key);
        self.buckets[index].get(key)
    }

    fn get_load_factor(&self) -> f64 {
        self.size as f64 / self.capacity as f64
    }

    fn resize(&mut self) -> anyhow::Result<()> {
        let new_capacity = self.capacity * 2;
        let new_buckets: Vec<LinkedList<K, V>> = vec![LinkedList::new(); new_capacity];
        let old_buckets = std::mem::replace(&mut self.buckets, new_buckets);
        self.capacity = new_capacity;
        self.size = 0;

        for bucket in old_buckets {
            for (key, value) in bucket.iter() {
                self.insert(key, value)?;
            }
        }

        anyhow::Ok(())
    }

    pub fn insert(&mut self, key: K, value: V) -> anyhow::Result<()> {
        if self.get_load_factor() >= LOAD_FACTOR_LIMIT {
            self.resize()?;
        }
        let index = self.hash(&key);
        self.buckets
            .get_mut(index)
            .map(|bucket| {
                let result = bucket.insert(key, value).unwrap();
                if result {
                    self.size += 1;
                }
                return anyhow::Ok(());
            })
            .transpose()
            .and(anyhow::Ok(()))
    }

    pub fn delete(&mut self, key: K) -> anyhow::Result<()> {
        let index = self.hash(&key);
        self.buckets
            .get_mut(index)
            .map(|bucket| bucket.delete(key))
            .transpose()
            .and(anyhow::Ok(()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hashmap() {
        let mut map: HashMap<String, String> = HashMap::new(16);
        map.insert("key".to_string(), "value".to_string()).unwrap();
        let value = map.get("key".to_string());
        assert_eq!(value.unwrap().unwrap(), "value".to_string());
    }

    #[test]
    fn test_resizing() {
        let mut map: HashMap<String, String> = HashMap::new(16);
        for i in 0..25 {
            let key = format!("key_{}", i);
            let value = format!("value_{}", i);
            map.insert(key, value).unwrap();
        }
        for i in 0..25 {
            let key = format!("key_{}", i);
            let value = format!("value_{}", i);
            let result = map.get(key).unwrap();
            assert_eq!(result.unwrap(), value);
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
}
