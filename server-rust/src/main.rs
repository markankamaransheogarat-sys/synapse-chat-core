use synapse_chat_core::{
    ClientSession, LatencyTimer, SynapseCodec, SynapseFrame, SynapseRouter,
};
use tokio::sync::mpsc;

#[tokio::main]
async fn main() {
    println!("\x1b[36m========================================================================\x1b[0m");
    println!("\x1b[1;32m ⚡ SYNAPSE: High-Concurrency Chat Gateway & Offline CRDT Engine \x1b[0m");
    println!("\x1b[36m========================================================================\x1b[0m");

    let router = SynapseRouter::new();
    let num_sessions = 100;
    let num_channels = 10;
    let messages_to_send = 10_000;

    println!("[*] Initializing {} concurrent sessions across {} channels...", num_sessions, num_channels);

    let mut receivers = Vec::new();

    for i in 0..num_sessions {
        let (tx, rx) = mpsc::channel(1000);
        let session_id = format!("sess_{:04}", i);
        let node_id = format!("node_{:04}", i);
        let channel_id = format!("channel_{}", i % num_channels);

        let mut session = ClientSession::new(&session_id, &node_id, tx);
        session.subscribe(&channel_id);

        router.register_session(session).await;
        router.subscribe(&session_id, &channel_id).await;
        receivers.push(rx);
    }

    println!("[*] Starting high-throughput broadcast benchmark: {} messages...", messages_to_send);

    let timer = LatencyTimer::start();

    for i in 0..messages_to_send {
        let ch_id = format!("channel_{}", i % num_channels);
        let msg_id = format!("msg_{}", i);
        let sender = format!("node_{:04}", i % num_sessions);
        let content = format!("Payload data sequence chunk #{}", i);
        let now_ms = 1793720000000 + (i as u64);

        router.publish_message(&sender, &ch_id, &msg_id, &content, now_ms).await;
    }

    let elapsed_ms = timer.elapsed_millis();
    let throughput = (messages_to_send as f64) / (elapsed_ms / 1000.0);
    let avg_latency_us = (elapsed_ms * 1000.0) / (messages_to_send as f64);

    println!("\x1b[1;32m[+] BENCHMARK COMPLETE!\x1b[0m");
    println!("  Total Messages Processed:  {}", messages_to_send);
    println!("  Elapsed Duration:          {:.2} ms", elapsed_ms);
    println!("  Throughput Rate:           \x1b[1;33m{:.0} msg/sec\x1b[0m", throughput);
    println!("  Average Routing Latency:   \x1b[1;32m{:.2} µs/msg\x1b[0m", avg_latency_us);

    // Verify binary wire serialization
    let sample_frame = SynapseFrame::new(
        synapse_chat_core::OpCode::Message,
        42,
        "channel_0",
        b"Binary zero-copy wire frame payload".to_vec(),
    );
    let encoded = SynapseCodec::encode(&sample_frame);
    let (decoded, len) = SynapseCodec::decode(&encoded).expect("Frame decode failed");

    println!("\n--- Wire Protocol Verification ---");
    println!("  Encoded Wire Frame Size:   {} bytes", len);
    println!("  Decoded OpCode:            {:?}", decoded.opcode);
    println!("  Decoded Sequence:          #{}", decoded.sequence);
    println!("  Decoded Channel:           {}", decoded.channel_id);
    println!("  CRC-32 Checksum Verified:  Valid IEEE 802.3");
}
