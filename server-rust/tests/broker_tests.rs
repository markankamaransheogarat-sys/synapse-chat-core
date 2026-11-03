use synapse_chat_core::{ClientSession, LWWElementSet, SynapseRouter, VectorClock};
use tokio::sync::mpsc;

#[tokio::test]
async fn test_offline_session_queuing_and_replay() {
    let (tx, mut rx) = mpsc::channel(100);
    let mut session = ClientSession::new("sess_offline", "client_1", tx);
    session.subscribe("channel_alerts");

    // Client goes offline (network drop)
    session.is_connected = false;

    // Send messages while offline
    let frame1 = synapse_chat_core::SynapseFrame::new(
        synapse_chat_core::OpCode::Message,
        1,
        "channel_alerts",
        b"Offline alert 1".to_vec(),
    );
    let frame2 = synapse_chat_core::SynapseFrame::new(
        synapse_chat_core::OpCode::Message,
        2,
        "channel_alerts",
        b"Offline alert 2".to_vec(),
    );

    let delivered1 = session.deliver(frame1).await;
    let delivered2 = session.deliver(frame2).await;

    assert!(!delivered1);
    assert!(!delivered2);
    assert_eq!(session.offline_buffer.len(), 2);

    // Client reconnects
    session.is_connected = true;
    let flushed = session.flush_offline_queue().await;
    assert_eq!(flushed, 2);
    assert_eq!(session.offline_buffer.len(), 0);

    // Verify messages arrive in channel
    let msg1 = rx.recv().await.expect("Expected frame 1");
    let msg2 = rx.recv().await.expect("Expected frame 2");
    assert_eq!(msg1.sequence, 1);
    assert_eq!(msg2.sequence, 2);
}

#[tokio::test]
async fn test_router_crdt_state_merge() {
    let router = SynapseRouter::new();

    // Client A went offline and generated local edits
    let mut client_a_crdt = LWWElementSet::new();
    client_a_crdt.add("msg_offline_1", "Written in subway", 1000, "client_A");

    let mut client_a_clock = VectorClock::new();
    client_a_clock.set("client_A", 1);

    // Reconnection merge into router
    router
        .merge_channel_crdt("channel_subway", &client_a_crdt, &client_a_clock)
        .await;

    let channels = router.channels.read().await;
    let ch = channels.get("channel_subway").unwrap();
    assert!(ch.crdt_messages.contains("msg_offline_1"));
    assert_eq!(ch.vector_clock.get("client_A"), 1);
}
