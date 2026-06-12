# agave-sidecar-ipc

Lock-free shared-memory IPC between an Agave validator and an external RPC sidecar,
eliminating RwLock contention on `BankForks` and `AccountsDB`.

## The Problem

Agave validators expose internal state (bank snapshots, account data, slot progress)
to external RPC sidecars through `Arc<RwLock<BankForks>>`. During heavy replay, the
write lock is held for long stretches as banks are ingested. The RPC sidecar trying
to read account state or query bank info contends for that same lock, introducing
latency spikes in RPC responses and blocking validator progress.

This lock contention is fundamentally unnecessary — the validator mutates state,
the sidecar only reads it. They need a way to share data without serializing on
the same mutex.

## The Solution in Three Parts

### 1. Shared Memory (`memmap2` + `rts-alloc`)

Instead of crossing process boundaries with locks, bank state lives in an OS-backed
shared memory region (`mmap`). Both the validator and the sidecar map the same
backing file into their address spaces. Writes by the validator are visible to the
sidecar without any cross-process lock acquisition.

`rts-alloc` places a zero-copy allocator **inside** the mmap. The validator
allocates bank state, account data, and slot metadata directly in shared memory.
The sidecar reads it from its own mapping — no copies, no serialization, no lock.

### 2. Lock-Free Signaling (`anza-xyz/shaq`)

A lock-free SPSC ring buffer lives in the same shared memory region. The
validator (producer) pushes lightweight notifications: "bank at slot N is ready"
or "account at offset X has been updated". The sidecar (consumer) polls these
notifications without ever blocking.

The SPSC queue uses atomic CPU instructions exclusively (fetch-add, CAS,
load-acquire, store-release). No mutexes. No RwLocks. No syscalls.

### 3. `#[repr(C)]` Zero-Copy Structures

All shared types (`Bank`, `Account`, `Transaction`, `ExecutionResult`) are
plain C-layout structs in `ipc_shared`. Their memory representation is stable
and identical on both sides of the IPC boundary. The consumer reads them directly
from shared memory without deserialization — interpreting raw bytes as typed structs.

`MessagePayload` is a C union: the same ring buffer slot carries either a
`Transaction` (validator → sidecar) or an `ExecutionResult` (sidecar → validator),
discriminated by `MessageHeader.msg_type`.

## Architecture

```
                    rts-alloc Shared Heap                     shaq SPSC Ring Buffer
┌─────────────────────────────────────────┐  ┌─────────────────────────────────┐
│  Bank 100      Bank 101      Bank 102   │  │  ┌──────┐ ┌──────┐ ┌──────┐    │
│  ├─ accounts   ├─ accounts   ├─ accounts│  │  │ slot │ │ slot │ │empty │    │
│  ├─ state_root ├─ state_root ├─ state_root│ │  │ 101  │ │ 102  │ │      │    │
│  └─ ...        └─ ...        └─ ...     │  │  │ready │ │ready │ │      │    │
│       ▲                                │  │  └──────┘ └──────┘ └──────┘    │
│       │ Validator writes Bank          │  │    ↑ commit()      ↓ sync()     │
│       │ state directly into shared     │  └─────────────────────────────────┘
│       │ memory via rts-alloc           │
└───────────────────────────────────────┘
          ▲                     ▲
          │                     │
   ┌──────┴──────────┐  ┌──────┴──────────┐
   │    PRODUCER     │  │    CONSUMER     │
   │  (Validator)    │  │  (RPC Sidecar)  │
   │                 │  │                 │
   │ • Replays banks │  │ • Serves RPC    │
   │ • Allocates bank│  │ • Reads bank    │
   │   state in mmap │  │   state from    │
   │ • Pushes "slot  │  │   mmap (no lock)│
   │   ready" to SPSC│  │ • Polls SPSC    │
   └─────────────────┘  └─────────────────┘
```

## Why This Works Where RwLocks Fail

| | RwLock Approach | This Approach |
|---|---|---|
| **Contention point** | Single `Arc<RwLock<BankForks>>` | None — reads never touch validator locks |
| **Blocking** | RPC read blocks on replay write | SPSC reader never waits for producer |
| **Copies** | Bank state cloned for RPC | Zero-copy — sidecar reads from shared memory |
| **Serialization** | Types serialized for cross-process calls | `#[repr(C)]` structs read in-place |
| **Scaling** | Lock contention grows with bank ingest rate | Throughput limited only by memory bandwidth |

## Project Structure

```
agave-sidecar-ipc/
├── ipc_shared/          # Zero-copy data structures shared between sides
│   └── src/lib.rs       #   Account, Bank, Transaction, Message, etc.
├── producer/            # Validator-side binary — writes to shared memory
│   └── src/main.rs      #   Allocates mmap, creates SPSC producer, sends Transactions
├── consumer/            # RPC sidecar binary — reads from shared memory
│   └── src/main.rs      #   Joins mmap, opens SPSC consumer, receives Transactions
├── src/main.rs          # Sandbox: basic mmap experiment
└── Cargo.toml           # Workspace root
```

### Key Dependencies

| Crate | Purpose |
|---|---|
| `memmap2` | Memory-maps a file into both process address spaces |
| `anza-xyz/rts-alloc` | Zero-copy allocator that creates a heap inside the mmap region |
| `anza-xyz/shaq` | Lock-free SPSC ring buffer using atomics, living in the mmap region |

## Current State

The prototype demonstrates a **one-way transaction submission channel**. The
producer sends a hardcoded `Transaction` through the SPSC queue. The consumer
receives and prints it. The shared memory region is 1 MiB with a 2-arena
64 KiB chunk allocator.

This validates the stack end-to-end: mmap setup, rts-alloc initialization,
shaq producer/consumer handshake, and zero-copy message passing with
`#[repr(C)]` types.

## Future Work

- **Bank state in shared memory**: Write full `Bank` snapshots (accounts, state
  roots, slot metadata) into the rts-alloc heap as they're created, not just
  pass lightweight structs through the SPSC.
- **Atomic publication protocol**: Add a sequence lock or generation counter so
  the consumer knows when a bank write is complete and the state is consistent.
- **Bidirectional messaging**: The SPSC queue and `MessagePayload` union already
  support it — the sidecar could return `ExecutionResult` to the validator.
- **Real Agave integration**: Embed the producer into Agave's replay loop,
  allocating bank state in shared memory as banks are finalized.
do 