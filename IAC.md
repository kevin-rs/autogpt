# RFC: IAC Protocol for Inter and Intra Agents Communications

![iac-seq-diag](https://github.com/user-attachments/assets/555534b1-84ca-4efc-ac9d-553e4a979d48)

## Introduction

In multi-agent ecosystems such as AutoGPT, agents act autonomously yet must reliably collaborate. The IAC (Inter & Intra Agent Communication) Protocol thus represents a foundational leap forward in agent networking. Building upon traditional IPC concepts, IAC is reinvented through the application of **QUIC (TLS 1.3 over UDP)**, strategic cryptographic identity, and baked-in efficiency layers. The result is an ecosystem-agnostic channel through which agents, whether collocated or distributed globally, communicate with sub-millisecond latency, near-native bandwidth efficiency, and cryptographic authenticity. By turning away from REST, gRPC, and bare TLS/TCP architectures, IAC becomes the foundation of a future where trillions of bounded agents can handshake, exchange, and batch work in a mesh topology with resilience and clarity.

## Motivation

While conventional REST APIs and message queues dominate today's architectures, they expose critical friction points under demanding conditions:

- **Handshake delays**: TCP and TLS incur at least one RTT (round-trip time) to initiate. Frequent connection init cycles add up, particularly in IoT environments.
- **Head-of-Line blocking in TCP**: Even multiplexed frameworks like HTTP/2 can be balked when a delayed packet holds entire streams hostage.
- **Stateless token auth**: Reliance on bearer tokens or API keys lacks granular revocation control, fails cryptographically-proof identity continuity, and scales poorly when agents turn over rapidly.
- **Orchestrator-centric models**: REST and RPC assume a central service; they lack built-in peer-to-peer elasticity, a core requirement for mesh-based systems.

IAC addresses all these limitations in a holistic architecture. QUIC's 0-RTT capability eliminates the handshake cold-start, and its multiplexed streams dodge TCP's halting weakness. Cryptographic signing and identity mitigate token-based attack surfaces. And native peer-to-peer connections allow grids of agents to organize, delegate, and redundantly replicate tasks without centralized bottlenecks.

## Core Design Principles

### Transport Strategy

Derived from the spectral performance of QUIC atop UDP, IAC defaults to QUIC with TLS 1.3 for its handshake and connection lifecycle. Benefits include:

- **Zero RTT setup**: Optionally reuse previous session parameters for near-instant reconnection.
- **Multi-stream concurrency**: No HOL blocking; Each agent stream proceeds independently.
- **Faster loss recovery**: QUIC's ACK-based fast retransmit is more adaptive to packet loss.
- **Integrated congestion control**: Out-of-the-box BBR ensures throughput remains smooth at scale.

Agents gracefully degrade to **TLS/TCP** or **UNIX-domain sockets** as network or deployment requires, preserving compatibility with firewalls or co-located execution.

### Asymmetric Keys

A public-private keypair (Ed25519 seed) is mined at agent runtime. The public key doubles as the agent's unique identity, eliminating ambiguous token systems. During handshake, peers exchange and validate keys; every message thereafter is signed and optionally encrypted. This yields:

- **Unforgeable provenance**: Only the private-key owner can generate valid agent messages.
- **Forward secrecy**: Ephemeral session data cannot be retroactively decrypted by eavesdroppers.

### Efficiency

IAC integrates batch messaging, stream multiplexing, dictionary-driven compression, and congestion pacing. The goal: every byte, every millisecond, optimized with forethought and control.

## Protocol Architecture

### Transport Stack

Every session begins by negotiating the transport stack:

1. **Client** attempts QUIC handshake using native certificate trust roots.
1. **Negotiation** includes compression preference (zstd) and dictionary ID.
1. Upon failure or policy constraints, fallback can be:

   - **TLS/TCP** with full-stream multiplexing.
   - **UNIX socket/shared memory** used within colocated contexts.

1. **Concurrency**: QUIC allows simultaneous handshake, stream-open, and data exchanges via distinct virtual streams.

This layered fallback enables robust deployment across edge nodes, cloud-bound agents, and local orchestrators.

### Message Format

```sh
syntax = "proto3";

package iac;

enum MessageType {
  UNKNOWN = 0;
  PING = 1;
  BROADCAST = 2;
  FILE_TRANSFER = 3;
  COMMAND = 4;
  DELEGATE_TASK = 5;
}

message Message {
  string from = 1;
  string to = 2;
  MessageType msg_type = 3;
  string payload_json = 4;
  uint64 timestamp = 5;
  uint64 msg_id = 6;
  uint64 session_id = 7;
  bytes signature = 8;
  bytes extra_data = 9;
}
```

- `from`, `to`: agent identities as hex-encoded public-keys or DNS-like aliases.
- `msg_type`: designates intent (e.g. MessageType::DELEGATE_TASK, MessageType::PING).
- `payload_json`: UTF-8 serialized metadata or control-object.
- `extra_data`: opaque binary; may carry compressed frames, file chunks, stream tokens.

Supplementing canonical encoding, a binary payload (e.g., `msg.sign()`) is serialized via Protobuf and then optionally compressed. High-frequency workloads dramatically benefit from =85% payload reduction via dictionary reuse.

## Cryptographic Identity

Every message holds a signature using Ed25519 for symmetric environments. The signing flow:

1. Clone the message and zero the `signature` field.
1. Serialize via Protobuf.
1. Sign the serialized data, writing bytes back to `signature`.
1. Receiver performs equivalent zero, serialize, and verify signature.

This ensures authenticated immutability and safeguards inter-agent trust. Channel-level TLS protects data in flight; message-level signatures protect agent locality. With static key rotation allowed, backward traceability and auditability remain intact.

## Communication Model

### Topology Modes

- **Inter-Agent**: global mesh using QUIC; agents broadcast, delegate, or proxy routing messages.
- **Agent-Orchestrator**: hierarchical or peer-centered delegation; orchestrator acts as bootstrap/registry.
- **Intra-Agent**: CPU-local using shared IPC primitives or UDS for low-latency ping/pong.

### Handshake & Key Exchange

Upon initial connection:

1. QUIC handshake authenticates raw channel.
1. Application-level handshake:

   - Exchange of agent IDs / public keys within secure stream.
   - Validation against orchestrator's registry or known set.
   - Compression dictionary selection.
   - Session-level symmetric key derivation if needed.

### Streams and Batching

- Each logical request (e.g., `msg_type = MessageType::DELEGATE_TASK`) uses a new QUIC **unidirectional stream**.
- Low-overhead messages are batched and flushed together.
- Stream includes: Protobuf frame, optional compression header, binary chunk.
- Congestion is paced via BBR controls; handshake prevents bursts.

## Protocol Features

### Frame-Level Compression

- zstd is negotiated on every session.
- Dictionary-based compression allows high gains, especially for repetitive agent payloads.
- Compression is applied post-serialization but pre-stream dispatch.

### Deduplication and Message Reorder

Fields:

- `timestamp`: microsecond precision
- `msg_id`: wrap-around-monotonic counter
- `session_id`: unique per connection

Receiver-side buffer tracks:

- Cached msg_ids to drop duplicates.
- Out-of-order detection with epoch window.
- Adaptive reorder buffer based on network variance.

### File & Stream Transfer

IAC abstracts chunking and receipt acknowledgment:

1. `msg_type=MessageType::FILE_TRANSFER`: includes filename, index, total, checksum in JSON.
1. `extra_data` carries raw chunk (compressed).

## Comparative Analysis

| Feature               | REST / gRPC | IAC (TLS/TCP) | IAC (QUIC)                      |
| --------------------- | ----------- | ------------- | ------------------------------- |
| TLS 0-RTT             | ❌          | ❌            | ✅                              |
| Stream Multiplexing   | ❌          | ❌            | ✅ (ho-free multi-streams)      |
| Agent Identity        | API tokens  | Bearer tokens | ✅ (Key-pair crypto-identity)   |
| Mesh Support          | ❌          | Partial       | ✅ (Built-in P2P delegation)    |
| Compression           | ❌          | ❌            | ✅ (zstd + dictionaries)        |
| File transfer support | ❌          | ❌            | ✅ (chunking + shared-mem)      |
| Dedup/Reorder         | ❌          | ❌            | ✅ (Protocol-level reliability) |

## Conclusion

IAC redefines the foundation of agent communication in a modern, distributed landscape, delivering exceptional performance, cryptographic integrity, and scalable design. It supports use cases ranging from decentralized AutoGPT swarms to ultra-responsive IoT coordination, all while maintaining the robustness and semantic clarity required for global-scale systems. Rather than offering a marginal improvement, IAC represents a fundamental shift, a protocol built not for the past, but for the future of intelligent agents.
