use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CausalOrder {
    Equal,
    HappenedBefore, // self < other
    HappenedAfter,  // self > other
    Concurrent,     // self || other (conflict!)
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct VectorClock {
    pub clock: HashMap<String, u64>,
}

impl VectorClock {
    pub fn new() -> Self {
        Self {
            clock: HashMap::new(),
        }
    }

    pub fn increment(&mut self, node_id: &str) -> u64 {
        let counter = self.clock.entry(node_id.to_string()).or_insert(0);
        *counter += 1;
        *counter
    }

    pub fn get(&self, node_id: &str) -> u64 {
        *self.clock.get(node_id).unwrap_or(&0)
    }

    pub fn set(&mut self, node_id: &str, counter: u64) {
        self.clock.insert(node_id.to_string(), counter);
    }

    /// Merge two vector clocks: V_merged[k] = max(V_A[k], V_B[k])
    /// Satisfies Join-Semilattice: Commutative, Associative, Idempotent
    pub fn merge(&mut self, other: &VectorClock) {
        for (node_id, &counter) in &other.clock {
            let entry = self.clock.entry(node_id.clone()).or_insert(0);
            *entry = (*entry).max(counter);
        }
    }

    /// Compare causal relationship between two clocks
    pub fn compare(&self, other: &VectorClock) -> CausalOrder {
        let mut self_has_greater = false;
        let mut other_has_greater = false;

        let mut all_keys: std::collections::HashSet<&String> = self.clock.keys().collect();
        all_keys.extend(other.clock.keys());

        for key in all_keys {
            let v_self = self.get(key);
            let v_other = other.get(key);

            if v_self > v_other {
                self_has_greater = true;
            } else if v_other > v_self {
                other_has_greater = true;
            }
        }

        match (self_has_greater, other_has_greater) {
            (false, false) => CausalOrder::Equal,
            (true, false) => CausalOrder::HappenedAfter,
            (false, true) => CausalOrder::HappenedBefore,
            (true, true) => CausalOrder::Concurrent,
        }
    }
}
