# ⚡ SYNAPSE: High-Concurrency Chat Gateway & Offline-First CRDT Engine

[![License: MIT](https://img.shields.io/badge/License-MIT-00ff66.svg?style=flat-square)](LICENSE)
[![Core: Rust](https://img.shields.io/badge/Core-Rust%202021%20%7C%20Tokio-dea584.svg?style=flat-square)](https://www.rust-lang.org/)
[![Client: Kotlin](https://img.shields.io/badge/Client-Kotlin%20%7C%20Coroutines-7f52ff.svg?style=flat-square)](https://kotlinlang.org/)
[![Throughput](https://img.shields.io/badge/Throughput-145%2C294%20msg%2Fsec-00e5ff.svg?style=flat-square)](#)
[![Routing Latency](https://img.shields.io/badge/Routing%20Latency-6.88%20%C2%B5s%2Fmsg-ff0055.svg?style=flat-square)](#)
[![Tests](https://img.shields.io/badge/Tests-10%2F10%20Passing-00ff66.svg?style=flat-square)](#)

```
  ██████▓██   ██▓ ███▄    █  ▄▄▄       ██▓███    ██████ ▓█████ 
▒██    ▒ ▒██  ██▒ ██ ▀█   █ ▒████▄    ▓██░  ██▒▒██    ▒ ▓█   ▀ 
░ ▓██▄    ▒██ ██░▓██  ▀█ ██▒▒██  ▀█▄  ▓██░ ██▓▒░ ▓██▄   ▒███   
  ▒   ██▒ ░ ▐██▓░▓██▒  ▐▌██▒░██▄▄▄▄██ ▒██▄█▓▒ ▒  ▒   ██▒▒▓█  ▄ 
▒██████▒▒ ░ ██▒▓░▒██░   ▓██░ ▓█   ▓██▒▒██▒ ░  ░▒██████▒▒░▒████▒
▒ ▒▓▒ ▒ ░  ██▒▒▒ ░ ▒░   ▒ ▒  ▒▒   ▓▒█░▒▓▒░ ░  ░▒ ▒▓▒ ▒ ░░░ ▒░ ░
░ ░▒  ░  ░██ ░▒░ ░ ░░   ░ ▒░  ▒   ▒▒ ░░▒ ░     ░ ░▒  ░ ░ ░ ░  ░
░  ░  ░  ░ ▒ ░░     ░   ░ ░   ░   ▒   ░░       ░  ░  ░     ░    
      ░  ░ ░              ░       ░  ░                   ░  ░   
         ░ ░                                                    
```

> **Designed for the extreme concurrency requirements of enterprise messengers (Wildberries WB Chat) and zero-connectivity resilience (Windi Messenger / Mango FZCO).**

**Synapse** is an industrial-grade distributed chat gateway and offline-first state synchronization engine. It unites a **high-throughput asynchronous Rust broker (145,000+ msg/sec, 6.88 µs routing latency)** with a **native Kotlin offline-first client architecture** powered by Conflict-free Replicated Data Types (CRDTs).

---

## 🌟 Architectural Highlights

1. **High-Concurrency Tokio Broker**:
   - Zero-allocation binary frame parser with IEEE 802.3 CRC-32 integrity validation.
   - Lock-free / Read-heavy concurrent session routing across millions of active subscriber topics.
   - Bounded ring-buffer offline queues with automatic playback on client reconnection.
2. **Conflict-Free Replicated State Synchronization (CRDT)**:
   - **Vector Clocks**: Partial order tracking that accurately detects concurrent causality conflicts ($V_A \parallel V_B$) without physical clock skew vulnerabilities.
   - **LWW-Element-Set (Last-Write-Wins)**: Add-set and remove-set (tombstone) join-semilattice for conflict-free message creation, editing, and deletion across flaky networks.
   - **Positive-Negative Counters (PN-Counter)**: Multi-peer convergence for read receipts and emoji reactions.
3. **Enterprise Mobile Client (Kotlin & Coroutines)**:
   - State-machine driven offline queue (`PENDING_LOCAL` $\to$ `SENDING` $\to$ `SYNCED`).
   - SQLite / SQLCipher local persistence layer with optimistic UI rendering.
   - Automatic delta reconciliation upon network interface reconnection.

---

## 📐 Mathematical Foundations of Strong Eventual Consistency (SEC)

### 1. Join-Semilattice Properties
A distributed state replica $S$ converges to an identical state across all nodes regardless of message arrival ordering if the merge operator $\sqcup$ satisfies:

$$\begin{aligned}
\text{Commutativity: } & x \sqcup y = y \sqcup x \\
\text{Associativity: } & (x \sqcup y) \sqcup z = x \sqcup (y \sqcup z) \\
\text{Idempotence: } & x \sqcup x = x
\end{aligned}$$

### 2. Vector Clock Causality
For two vector clocks $V_A$ and $V_B$ over node set $\mathcal{N}$:

$$\begin{aligned}
V_A \le V_B &\iff \forall k \in \mathcal{N}, V_A[k] \le V_B[k] \\
V_A < V_B &\iff V_A \le V_B \land V_A \ne V_B \quad (\text{Happened-Before}) \\
V_A \parallel V_B &\iff \neg(V_A \le V_B) \land \neg(V_B \le V_A) \quad (\text{Concurrent Conflict})
\end{aligned}$$

When concurrent edits occur ($V_A \parallel V_B$), Synapse resolves ties deterministically via:
$$(t_1, n_1) \succ (t_2, n_2) \iff (t_1 > t_2) \lor (t_1 = t_2 \land n_1 >_{\text{lex}} n_2)$$

---

## 📦 Binary Wire Protocol Layout (RFC 2026-SYN-01)

Packets are packed into a compact 17-byte minimum binary frame format:

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

| Field | Size | Description |
|:---|:---|:---|
| `MAGIC` | 2 bytes | Protocol sync word: `0x5359` (`"SY"`) |
| `VERSION` | 1 byte | Protocol version (`0x01`) |
| `OPCODE` | 1 byte | `0x01` Handshake, `0x02` Publish, `0x03` Message, `0x04` Ack, `0x05` SyncReq, `0x06` SyncResp |
| `SEQUENCE` | 4 bytes | Monotonic 32-bit packet identifier |
| `CHAN_LEN` | 1 byte | Channel string length |
| `PAYLOAD_LEN` | 4 bytes | Payload byte length |
| `CRC-32` | 4 bytes | IEEE 802.3 checksum |

---

## ⚡ High-Load Benchmark Results

Benchmarked under high message volume over concurrent in-memory session pipelines:

```
========================================================================
 ⚡ SYNAPSE: High-Concurrency Chat Gateway & Offline CRDT Engine 
========================================================================
[*] Initializing 100 concurrent sessions across 10 channels...
[*] Starting high-throughput broadcast benchmark: 10,000 messages...

[+] BENCHMARK COMPLETE!
  Total Messages Processed:  10,000
  Elapsed Duration:          68.83 ms
  Throughput Rate:           145,294 msg/sec
  Average Routing Latency:   6.88 µs/msg

--- Wire Protocol Verification ---
  Encoded Wire Frame Size:   61 bytes
  Decoded OpCode:            Message
  Decoded Sequence:          #42
  Decoded Channel:           channel_0
  CRC-32 Checksum Verified:  Valid IEEE 802.3
```

---

## 🗂 Repository Structure

```
synapse-chat-core/
├── Cargo.toml
├── LICENSE
├── README.md
├── architecture/
│   ├── PROTOCOL_SPEC_RFC.md          # Complete binary framing & opcode RFC
│   └── CRDT_CONVERGENCE_PROOFS.md    # Formal join-semilattice & causality proofs
├── server-rust/                      # High-Concurrency Async Gateway Core
│   ├── Cargo.toml
│   ├── src/
│   │   ├── crdt/
│   │   │   ├── vector_clock.rs       # Vector clock causality comparison engine
│   │   │   ├── lww_set.rs            # Last-Write-Wins Element Set CRDT
│   │   │   └── pn_counter.rs         # Positive-Negative Counter CRDT
│   │   ├── protocol/
│   │   │   ├── frame.rs              # Frame definitions & opcode representations
│   │   │   └── codec.rs              # Zero-allocation encoder/decoder & IEEE CRC-32
│   │   ├── broker/
│   │   │   ├── session.rs            # Client session & bounded offline ring-buffer
│   │   │   └── router.rs             # Topic router & concurrent dispatch engine
│   │   ├── storage/
│   │   │   └── event_log.rs          # Append-only WAL commit log & snapshot replayer
│   │   ├── telemetry/
│   │   │   └── metrics.rs            # Microsecond execution timer
│   │   ├── lib.rs
│   │   └── main.rs                   # End-to-end benchmark showcase CLI
│   └── tests/
│       ├── crdt_tests.rs             # 5 unit tests for CRDT properties
│       ├── protocol_tests.rs         # 3 unit tests for binary framing & corruption
│       └── broker_tests.rs           # 2 integration tests for offline replay & merge
└── client-kotlin/                    # Android / Mobile Offline-First Engine
    └── src/main/kotlin/org/synapse/chat/
        ├── model/
        │   └── ChatMessage.kt        # Chat entity with SyncState & VectorClock
        ├── crdt/
        │   ├── VectorClock.kt        # Kotlin VectorClock matching Rust wire format
        │   └── LWWMessageSet.kt      # Client-side CRDT merge & deduplication store
        └── sync/
            ├── OfflineSyncEngine.kt  # Coroutines state machine for offline queueing
            └── LocalStore.kt         # Interface for encrypted SQLite / Room database
```

---

## 🚀 Quickstart

### 1. Run Automated Test Suite (Rust)
```bash
git clone https://github.com/markankamaransheogarat-sys/synapse-chat-core.git
cd synapse-chat-core

# Run all 10 unit and integration tests
cargo test
```

### 2. Run the High-Throughput Benchmark
```bash
cargo run --release --bin synapse-chat-core
```

---

## 📜 License & Attribution

Designed and engineered by **Konstantin ([@markankamaransheogarat-sys](https://github.com/markankamaransheogarat-sys))**.  
Released under the [MIT License](LICENSE).
