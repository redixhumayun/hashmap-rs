#![allow(dead_code)]
#![allow(unused_variables)]
use rand::Rng;

pub trait HashMapBehavior<K, V> {
    fn new(capacity: usize) -> Self;
    fn insert(&mut self, key: K, value: V) -> anyhow::Result<()>;
    fn get(&self, key: K) -> anyhow::Result<Option<V>>;
    fn delete(&mut self, key: K) -> anyhow::Result<()>;
}

// Implement for both HashMap variants
impl<K: crate::chaining::Key, V: crate::chaining::Value> HashMapBehavior<K, V>
    for crate::chaining::HashMap<K, V>
{
    fn new(capacity: usize) -> Self {
        Self::new(capacity)
    }
    fn insert(&mut self, key: K, value: V) -> anyhow::Result<()> {
        self.insert(key, value)
    }
    fn get(&self, key: K) -> anyhow::Result<Option<V>> {
        self.get(key)
    }
    fn delete(&mut self, key: K) -> anyhow::Result<()> {
        self.delete(key)
    }
}

impl<K: crate::open_addressing::Key, V: crate::open_addressing::Value> HashMapBehavior<K, V>
    for crate::open_addressing::HashMap<K, V>
{
    fn new(capacity: usize) -> Self {
        Self::new(capacity)
    }
    fn insert(&mut self, key: K, value: V) -> anyhow::Result<()> {
        self.insert(key, value)
    }
    fn get(&self, key: K) -> anyhow::Result<Option<V>> {
        self.get(key)
    }
    fn delete(&mut self, key: K) -> anyhow::Result<()> {
        self.delete(key)
    }
}

pub struct LoadFactorWorkload {
    pub size: usize,
    pub value_size: usize,
}

#[derive(Clone)]
pub enum KeyPattern {
    Uniform,
    Clustered,
    Sequential,
}

pub struct KeyDistributionWorkload {
    pub size: usize,
    pub pattern: KeyPattern,
}

pub struct OperationMixWorkload {
    pub initial_size: usize,
    pub operations: usize,
    pub read_pct: u8,
    pub write_pct: u8, // delete_pct is implied as 100 - (read_pct + write_pct)
}

pub mod generators {
    use super::*;

    pub fn run_load_factor_workload<M: HashMapBehavior<String, String>>(
        workload: &LoadFactorWorkload,
    ) {
        let mut map = M::new(16);
        for i in 0..workload.size {
            map.insert(format!("key_{}", i), "x".repeat(workload.value_size))
                .unwrap();
        }
    }

    pub fn run_key_distribution_workload<M: HashMapBehavior<String, String>>(
        workload: &KeyDistributionWorkload,
    ) {
        let mut map = M::new(workload.size);
        let mut rng = rand::thread_rng();

        match workload.pattern {
            KeyPattern::Uniform => {
                for _ in 0..workload.size {
                    map.insert(format!("key_{}", rng.gen::<u64>()), "value".to_string())
                        .unwrap();
                }
            }
            KeyPattern::Clustered => {
                for i in 0..workload.size {
                    let cluster = i / (workload.size / 10); // 10 clusters
                    map.insert(format!("cluster_{}_{}", cluster, i), "value".to_string())
                        .unwrap();
                }
            }
            KeyPattern::Sequential => {
                for i in 0..workload.size {
                    map.insert(format!("{:020}", i), "value".to_string())
                        .unwrap();
                }
            }
        }
    }

    pub fn run_operation_mix_workload<M: HashMapBehavior<String, String>>(
        workload: &OperationMixWorkload,
    ) {
        // Returns map and operations performed
        let mut map = M::new(workload.initial_size);
        let mut rng = rand::thread_rng();
        let mut ops_performed = 0;

        // Pre-populate
        for i in 0..workload.initial_size {
            map.insert(format!("key_{}", i), "initial".to_string())
                .unwrap();
            ops_performed += 1;
        }

        // Run mixed workload
        for _ in 0..workload.operations {
            let op = rng.gen::<u8>() % 100;
            let key_idx = rng.gen::<usize>() % workload.initial_size;

            if op < workload.read_pct {
                let _ = map.get(format!("key_{}", key_idx));
            } else if op < (workload.read_pct + workload.write_pct) {
                let _ = map.insert(format!("key_{}", key_idx), "updated".to_string());
            } else {
                let _ = map.delete(format!("key_{}", key_idx));
            }
            ops_performed += 1;
        }
    }
}
