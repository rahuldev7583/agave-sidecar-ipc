# agave-sidecar-ipc

This repo explores how Agave validator can expose internal state
(Bank, AccountsDB, slot progress) to an external RPC sidecar **without blocking
replay and without locks.**

The goal is to understand how a validator-like process can expose internal state
to a sidecar using **shared memory**, **atomic ordering**, and **zero-copy data structures**.


-  Shared memory setup using `mmap`
-  Lock-free SPSC queue (via `anza-xyz/shaq`)
-  Zero-copy allocator in shared memory (`anza-xyz/rts-alloc`)

