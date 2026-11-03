use tokio::sync::mpsc;
use std::collections::HashSet;
use crate::protocol::SynapseFrame;

#[derive(Debug)]
pub struct ClientSession {
    pub session_id: String,
    pub node_id: String,
    pub subscribed_channels: HashSet<String>,
    pub tx: mpsc::Sender<SynapseFrame>,
    pub offline_buffer: Vec<SynapseFrame>,
    pub max_offline_buffer: usize,
    pub is_connected: bool,
}

impl ClientSession {
    pub fn new(session_id: &str, node_id: &str, tx: mpsc::Sender<SynapseFrame>) -> Self {
        Self {
            session_id: session_id.to_string(),
            node_id: node_id.to_string(),
            subscribed_channels: HashSet::new(),
            tx,
            offline_buffer: Vec::new(),
            max_offline_buffer: 500,
            is_connected: true,
        }
    }

    pub fn subscribe(&mut self, channel: &str) {
        self.subscribed_channels.insert(channel.to_string());
    }

    pub fn unsubscribe(&mut self, channel: &str) {
        self.subscribed_channels.remove(channel);
    }

    /// Deliver a frame to client; if offline or buffer full, queue into offline_buffer
    pub async fn deliver(&mut self, frame: SynapseFrame) -> bool {
        if self.is_connected {
            match self.tx.try_send(frame.clone()) {
                Ok(_) => true,
                Err(_) => {
                    // Channel full or disconnected
                    self.enqueue_offline(frame);
                    false
                }
            }
        } else {
            self.enqueue_offline(frame);
            false
        }
    }

    fn enqueue_offline(&mut self, frame: SynapseFrame) {
        if self.offline_buffer.len() >= self.max_offline_buffer {
            self.offline_buffer.remove(0); // Evict oldest
        }
        self.offline_buffer.push(frame);
    }

    /// Replay queued offline messages on reconnect
    pub async fn flush_offline_queue(&mut self) -> usize {
        let queued = std::mem::take(&mut self.offline_buffer);
        let count = queued.len();
        for frame in queued {
            let _ = self.tx.send(frame).await;
        }
        count
    }
}
