use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Positive-Negative Counter (PN-Counter) CRDT
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PNCounter {
    pub positive: HashMap<String, u64>,
    pub negative: HashMap<String, u64>,
}

impl PNCounter {
    pub fn new() -> Self {
        Self {
            positive: HashMap::new(),
            negative: HashMap::new(),
        }
    }

    pub fn increment(&mut self, node_id: &str, delta: u64) {
        let entry = self.positive.entry(node_id.to_string()).or_insert(0);
        *entry += delta;
    }

    pub fn decrement(&mut self, node_id: &str, delta: u64) {
        let entry = self.negative.entry(node_id.to_string()).or_insert(0);
        *entry += delta;
    }

    pub fn value(&self) -> i64 {
        let p_sum: u64 = self.positive.values().sum();
        let n_sum: u64 = self.negative.values().sum();
        (p_sum as i64) - (n_sum as i64)
    }

    pub fn merge(&mut self, other: &PNCounter) {
        for (node, &val) in &other.positive {
            let entry = self.positive.entry(node.clone()).or_insert(0);
            *entry = (*entry).max(val);
        }
        for (node, &val) in &other.negative {
            let entry = self.negative.entry(node.clone()).or_insert(0);
            *entry = (*entry).max(val);
        }
    }
}
