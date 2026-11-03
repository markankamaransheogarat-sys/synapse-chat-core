use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::crdt::LWWElementSet;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitLogEntry {
    pub sequence: u64,
    pub channel_id: String,
    pub node_id: String,
    pub operation: String, // "ADD" or "REMOVE"
    pub element_id: String,
    pub content: String,
    pub timestamp: u64,
}

#[derive(Debug, Default)]
pub struct CommitLog {
    entries: Vec<CommitLogEntry>,
    sequence_counter: u64,
}

impl CommitLog {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            sequence_counter: 0,
        }
    }

    pub fn append(
        &mut self,
        channel_id: &str,
        node_id: &str,
        operation: &str,
        element_id: &str,
        content: &str,
        timestamp: u64,
    ) -> u64 {
        self.sequence_counter += 1;
        let entry = CommitLogEntry {
            sequence: self.sequence_counter,
            channel_id: channel_id.to_string(),
            node_id: node_id.to_string(),
            operation: operation.to_string(),
            element_id: element_id.to_string(),
            content: content.to_string(),
            timestamp,
        };
        self.entries.push(entry);
        self.sequence_counter
    }

    /// Replay entries starting from a given sequence to reconstruct state
    pub fn replay_from(&self, from_seq: u64) -> HashMap<String, LWWElementSet> {
        let mut state: HashMap<String, LWWElementSet> = HashMap::new();

        for entry in self.entries.iter().filter(|e| e.sequence > from_seq) {
            let ch_set = state.entry(entry.channel_id.clone()).or_default();
            if entry.operation == "ADD" {
                ch_set.add(&entry.element_id, &entry.content, entry.timestamp, &entry.node_id);
            } else if entry.operation == "REMOVE" {
                ch_set.remove(&entry.element_id, entry.timestamp, &entry.node_id);
            }
        }

        state
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}
