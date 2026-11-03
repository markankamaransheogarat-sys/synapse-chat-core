# RFC 2026-SYN-01: Synapse Binary Wire Protocol Specification

**Author:** Konstantin ([@markankamaransheogarat-sys](https://github.com/markankamaransheogarat-sys))  
**Target:** Wildberries High-Concurrency Messenger / Windi Offline-First Gateway  
**Date:** November 2026  

---

## 1. Frame Binary Layout

To guarantee ultra-low memory overhead and eliminate JSON allocation churn over millions of concurrent TCP/WebSocket connections, all packets adhere to the 17-byte minimum binary frame format:

```
 0                   1                   2                   3
 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|       MAGIC (0x5359)          |    VERSION    |    OPCODE     |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|                       SEQUENCE NUMBER                         |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|  CHAN_LEN     |  CHANNEL IDENTIFIER UTF-8 (0..255 BYTES) ...  |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|                       PAYLOAD LENGTH                          |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|                 BINARY PAYLOAD (CRDT DELTA / MSG)             |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|                       CRC-32 CHECKSUM                         |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
```

### 1.1 Field Descriptions
- `MAGIC`: 2 bytes (`0x5359` — ASCII `"SY"`).
- `VERSION`: 1 byte (`0x01`).
- `OPCODE`: 1 byte operation identifier:
  - `0x01`: `Handshake` (Authentication token + client capabilities)
  - `0x02`: `Publish` (Inbound message publish request)
  - `0x03`: `Message` (Outbound broadcast to channel subscribers)
  - `0x04`: `Ack` (Message delivery confirmation)
  - `0x05`: `SyncRequest` (Client sends local Vector Clock state)
  - `0x06`: `SyncResponse` (Server replies with missing CRDT deltas)
  - `0x07`: `HeartbeatPing` / `0x08`: `HeartbeatPong`
- `SEQUENCE`: 4 bytes unsigned big-endian monotonic packet identifier.
- `CHANNEL_LEN`: 1 byte indicating length of Channel ID string.
- `CHANNEL_ID`: Variable UTF-8 string bytes.
- `PAYLOAD_LEN`: 4 bytes unsigned big-endian payload length.
- `PAYLOAD`: Serialized CRDT state delta or message content.
- `CRC-32`: 4 bytes IEEE 802.3 checksum covering all bytes preceding it.
