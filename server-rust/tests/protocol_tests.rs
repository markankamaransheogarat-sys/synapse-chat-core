use synapse_chat_core::{CodecError, OpCode, SynapseCodec, SynapseFrame};

#[test]
fn test_frame_encoding_and_decoding_roundtrip() {
    let frame = SynapseFrame::new(
        OpCode::Publish,
        1001,
        "team_dev_core",
        b"{\"action\":\"typing\",\"user\":\"alice\"}".to_vec(),
    );

    let bytes = SynapseCodec::encode(&frame);
    assert!(bytes.len() > 17);

    let (decoded, consumed) = SynapseCodec::decode(&bytes).expect("Decoding failed");
    assert_eq!(consumed, bytes.len());
    assert_eq!(decoded.opcode, OpCode::Publish);
    assert_eq!(decoded.sequence, 1001);
    assert_eq!(decoded.channel_id, "team_dev_core");
    assert_eq!(decoded.payload, frame.payload);
}

#[test]
fn test_crc32_checksum_corruption_detection() {
    let frame = SynapseFrame::new(
        OpCode::Message,
        42,
        "general",
        b"Critical financial transaction payload".to_vec(),
    );

    let mut bytes = SynapseCodec::encode(&frame);

    // Corrupt a single payload byte
    let corrupt_idx = bytes.len() - 6;
    bytes[corrupt_idx] ^= 0xFF;

    let res = SynapseCodec::decode(&bytes);
    match res {
        Err(CodecError::ChecksumMismatch { expected, computed }) => {
            assert_ne!(expected, computed);
        }
        _ => panic!("Expected ChecksumMismatch error on corrupted buffer"),
    }
}

#[test]
fn test_incomplete_buffer_rejection() {
    let frame = SynapseFrame::new(OpCode::Message, 1, "test", b"short".to_vec());
    let bytes = SynapseCodec::encode(&frame);

    // Truncate buffer
    let truncated = &bytes[..10];
    let res = SynapseCodec::decode(truncated);
    assert_eq!(res, Err(CodecError::IncompleteFrame));
}
