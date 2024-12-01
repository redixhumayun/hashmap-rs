#![feature(trait_alias)]

mod open_addressing;
use open_addressing::HashMap;

mod chaining;
// use chaining::HashMap;

fn main() {
    println!("Test starting");
    let mut map: HashMap<String, String> = HashMap::new(16);

    // Three distinct workload phases to stress different patterns

    // Phase 1: Controlled growth causing resizes
    let keys: Vec<String> = (0..500_000).map(|i| format!("key_{}", i)).collect();
    let values: Vec<String> = (0..500_000).map(|_| "x".repeat(1000)).collect();
    for (key, value) in keys.into_iter().zip(values.into_iter()) {
        map.insert(key, value).unwrap();
    }
    println!("phase 1 complete");

    // Phase 2: Heavy updates/overwrites
    let keys: Vec<String> = (0..500_000).map(|i| format!("key_{}", i)).collect();
    let values: Vec<String> = (0..500_000).map(|_| "y".repeat(200)).collect();
    for (key, value) in keys.into_iter().zip(values.into_iter()) {
        map.insert(key, value).unwrap();
    }

    // Phase 3: Mixed deletes and inserts
    for i in 0..75_000 {
        if i % 2 == 0 {
            let key = format!("key_{}", i);
            map.delete(key).unwrap();
        } else {
            let new_key = format!("key_new_{}", i);
            let new_value = "z".repeat(150);
            map.insert(new_key, new_value).unwrap();
        }
    }
}
