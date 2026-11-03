use std::collections::HashMap;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::crdt::{LWWElementSet, VectorClock};
use crate::protocol::{OpCode, SynapseFrame};
use super::session::ClientSession;

#[derive(Default)]
pub struct ChannelState {
    pub crdt_messages: LWWElementSet,
    pub vector_clock: VectorClock,
    pub subscribers: Vec<String>, // session_ids
}

pub struct SynapseRouter {
    pub sessions: Arc<RwLock<HashMap<String, ClientSession>>>,
    pub channels: Arc<RwLock<HashMap<String, ChannelState>>>,
    pub global_seq: AtomicU32,
}

impl SynapseRouter {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            channels: Arc::new(RwLock::new(HashMap::new())),
            global_seq: AtomicU32::new(1),
        }
    }

    pub fn next_sequence(&self) -> u32 {
        self.global_seq.fetch_add(1, Ordering::SeqCst)
    }

    pub async fn register_session(&self, session: ClientSession) {
        let mut sessions = self.sessions.write().await;
        sessions.insert(session.session_id.clone(), session);
    }

    pub async fn unregister_session(&self, session_id: &str) {
        let mut sessions = self.sessions.write().await;
        if let Some(session) = sessions.get_mut(session_id) {
            session.is_connected = false;
        }
    }

    pub async fn subscribe(&self, session_id: &str, channel_id: &str) {
        {
            let mut sessions = self.sessions.write().await;
            if let Some(session) = sessions.get_mut(session_id) {
                session.subscribe(channel_id);
            }
        }
        {
            let mut channels = self.channels.write().await;
            let ch = channels.entry(channel_id.to_string()).or_default();
            if !ch.subscribers.contains(&session_id.to_string()) {
                ch.subscribers.push(session_id.to_string());
            }
        }
    }

    /// Publish a message to a channel: updates channel CRDT state, vector clock, and dispatches to all subscribers
    pub async fn publish_message(
        &self,
        from_node: &str,
        channel_id: &str,
        message_id: &str,
        content: &str,
        timestamp: u64,
    ) -> usize {
        let seq = self.next_sequence();

        // 1. Update CRDT state and VectorClock
        let (frame, subscribers) = {
            let mut channels = self.channels.write().await;
            let ch = channels.entry(channel_id.to_string()).or_default();

            ch.crdt_messages.add(message_id, content, timestamp, from_node);
            ch.vector_clock.increment(from_node);

            let payload = serde_json::to_vec(&serde_json::json!({
                "message_id": message_id,
                "sender": from_node,
                "content": content,
                "timestamp": timestamp,
                "vector_clock": ch.vector_clock,
            })).unwrap_or_default();

            let frame = SynapseFrame::new(OpCode::Message, seq, channel_id, payload);
            (frame, ch.subscribers.clone())
        };

        // 2. Broadcast to all active subscribers
        let mut sessions = self.sessions.write().await;
        let mut delivered = 0;

        for sid in subscribers {
            if let Some(session) = sessions.get_mut(&sid) {
                if session.deliver(frame.clone()).await {
                    delivered += 1;
                }
            }
        }

        delivered
    }

    /// Merge incoming CRDT delta from client (e.g. after offline reconnection)
    pub async fn merge_channel_crdt(
        &self,
        channel_id: &str,
        incoming_crdt: &LWWElementSet,
        incoming_clock: &VectorClock,
    ) {
        let mut channels = self.channels.write().await;
        let ch = channels.entry(channel_id.to_string()).or_default();
        ch.crdt_messages.merge(incoming_crdt);
        ch.vector_clock.merge(incoming_clock);
    }
}
