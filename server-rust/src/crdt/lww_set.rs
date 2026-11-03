use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LWWRecord {
    pub content: String,
    pub timestamp: u64,
    pub node_id: String,
}

impl LWWRecord {
    pub fn new(content: &str, timestamp: u64, node_id: &str) -> Self {
        Self {
            content: content.to_string(),
            timestamp,
            node_id: node_id.to_string(),
        }
    }

    /// Deterministic tie-breaking: higher timestamp wins; if equal, lexicographically greater node_id wins
    pub fn dominates(&self, other: &LWWRecord) -> bool {
        if self.timestamp != other.timestamp {
            self.timestamp > other.timestamp
        } else {
            self.node_id > other.node_id
        }
    }
}

/// Last-Write-Wins Element Set (LWW-Element-Set) CRDT for distributed message state
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LWWElementSet {
    pub add_set: HashMap<String, LWWRecord>,
    pub remove_set: HashMap<String, LWWRecord>, // Tombstones for deleted messages
}

impl LWWElementSet {
    pub fn new() -> Self {
        Self {
            add_set: HashMap::new(),
            remove_set: HashMap::new(),
        }
    }

    /// Add or update a message
    pub fn add(&mut self, id: &str, content: &str, timestamp: u64, node_id: &str) {
        let record = LWWRecord::new(content, timestamp, node_id);
        match self.add_set.get(id) {
            Some(existing) if !record.dominates(existing) => {}
            _ => {
                self.add_set.insert(id.to_string(), record);
            }
        }
    }

    /// Remove (tombstone) a message
    pub fn remove(&mut self, id: &str, timestamp: u64, node_id: &str) {
        let record = LWWRecord::new("", timestamp, node_id);
        match self.remove_set.get(id) {
            Some(existing) if !record.dominates(existing) => {}
            _ => {
                self.remove_set.insert(id.to_string(), record);
            }
        }
    }

    /// Check if a message is currently active (exists and not tombstoned)
    pub fn contains(&self, id: &str) -> bool {
        match self.add_set.get(id) {
            Some(add_record) => match self.remove_set.get(id) {
                Some(rem_record) => add_record.dominates(rem_record),
                None => true,
            },
            None => false,
        }
    }

    /// Get current active content of a message
    pub fn get(&self, id: &str) -> Option<&LWWRecord> {
        if self.contains(id) {
            self.add_set.get(id)
        } else {
            None
        }
    }

    /// Returns all currently active elements sorted by timestamp
    pub fn get_active_elements(&self) -> Vec<(String, LWWRecord)> {
        let mut elements = Vec::new();
        for (id, record) in &self.add_set {
            if self.contains(id) {
                elements.push((id.clone(), record.clone()));
            }
        }
        elements.sort_by(|a, b| {
            if a.1.timestamp != b.1.timestamp {
                a.1.timestamp.cmp(&b.1.timestamp)
            } else {
                a.1.node_id.cmp(&b.1.node_id)
            }
        });
        elements
    }

    /// Strong Eventual Consistency (SEC) State-based Merge
    /// Commutative, Associative, Idempotent join-semilattice
    pub fn merge(&mut self, other: &LWWElementSet) {
        // Merge Add Set
        for (id, other_rec) in &other.add_set {
            match self.add_set.get(id) {
                Some(self_rec) if !other_rec.dominates(self_rec) => {}
                _ => {
                    self.add_set.insert(id.clone(), other_rec.clone());
                }
            }
        }

        // Merge Remove Set
        for (id, other_rec) in &other.remove_set {
            match self.remove_set.get(id) {
                Some(self_rec) if !other_rec.dominates(self_rec) => {}
                _ => {
                    self.remove_set.insert(id.clone(), other_rec.clone());
                }
            }
        }
    }
}
