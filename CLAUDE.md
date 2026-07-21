# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What this repo is

A std-only Rust learning workspace for low-level systems interview prep (concurrency, event loops, binary protocols, core data structures). It is **teaching material, not a product**: the same topics exist at three difficulty layers, and the gaps are the point.

**Do not fill in `todo!()`s in `drills/`, `challenges/`, or `rehearsals/`, and do not remove their `#[ignore]` attributes, unless explicitly asked.** Those holes are the user's practice material — "fixing" them destroys the repo's purpose. The same goes for the `_todo: ()` placeholder structs.

## Commands

Quality gates — every commit must pass all four:

```sh
cargo build --workspace
cargo test --workspace              # reference fully green; drills/challenges practice tests are #[ignore]d
cargo clippy --workspace -- -D warnings
cargo fmt --check
```

Other common invocations:

```sh
cargo test -p reference <test_name>              # single test by name
cargo test -p reference --test loom_spsc         # loom model check: SPSC ring
cargo test -p reference --test loom_mpmc         # loom model check: MPMC ring (Vyukov)
cargo test -p reference --test loom_mpsc         # loom model check: MPSC list (Vyukov)
cargo test -p reference --test loom_mpsc_ring    # loom model check: MPSC ring (Vyukov degenerate)
cargo test -p reference --test loom_mpmc_list    # loom model check: MPMC list (Michael-Scott)
cargo test -p reference --test loom_ws_deque     # loom model check: work-stealing deque (Chase-Lev)
cargo test -p reference --test loom_arena        # loom model check: lock-free arena stack
cargo test -p reference --test loom_dsu          # loom model check: lock-free DSU
cargo test -p drills -- --include-ignored        # show which drill tests are red
cargo test -p challenges -- --include-ignored    # show which challenge tests are red
```

## Architecture

Four workspace crates; the first three share module names, organized into four category submodules mirroring `docs/`'s four folders (`async` is a Rust keyword, so the source module is named `runtime`):

- `ds/` — the six single-threaded data-structure modules (`dsu`, `graph`, `lru`, `ring_buffer`, `tree`, `trie`)
- `concurrency/` — `bounded_queue`, `thread_pool`, `sharded_map`, `spsc_ring`, `mpmc_ring`, `mpsc_list`, `arena_lockfree`, `signal_pipeline`; reference-only: `mpsc_ring` (Vyukov degenerate), `mpmc_list` (Michael-Scott), `ws_deque` (Chase-Lev), `rcu_snapshot` (snapshot publication, no unsafe), and the locked/lock-free pairing layer `ds_sync`
- `runtime/` (docs: `docs/async/`) — `executor`, `async_sync`, plus (reference-only) `mini_runtime`
- `io/` — `epoll_sys`, `event_loop`, `fd_registry`, `file_io_offload`, `tcp_echo`, `hw_bridge`

The single-threaded idiom modules stay at crate root and don't span all layers — `iter_mutate` has no challenge, `inplace_leetcode` exists only in reference; `sync_shim` stays at reference's root so `core_impl.rs`'s `crate::sync_shim` path survives moves (see the loom mechanism below). The crates:

- **`reference/`** — complete implementations + tests + teaching comments. The answer key.
- **`drills/`** — same module tree, but core functions are hollowed to `todo!("spec: ...")` with a spec doc comment above each. Skeleton, helpers, and `#[ignore]`d tests are provided.
- **`challenges/`** — only public API signatures + test files + interview-prompt-style module docs (constraints and clarify points, no how). Blank-slate live-coding conditions.
- **`rehearsals/`** — nine timed rehearsal problems simulating the real interview environment (CoderPad: single file, fixed crate list — see `docs/coderpad-constraints.md`). Problem d (tokio_frame_server) uses tokio, the crate's sole dependency; all other problems are std-only. Problems e–h map to predicted question types Q4–Q7 and default to recognition practice rather than full 45-minute runs; problem e2 (fd_registry, the JD's "event registry" sleeper) is worth a full run. The user writes their own tests in-file during rehearsal; `tests/<name>_test.rs` are reference boundary tests, opened only afterward. Verified solutions live in `rehearsals/examples/sol_<name>.rs` (compiled by the gates so they can't rot) — **do not reveal or paste solution content while the user is mid-rehearsal** unless they explicitly ask. The same protection covers `rehearsals/clarify-answers.md` (answer key for the clarify scenario cards in `rehearsals/clarify-cards.md`).

The verified interview environment (user tested the real pad, 2026-07-15) is Rust 1.92 / edition 2024 with tokio available but no libc/mio crates. Hand-written `unsafe extern "C"` raw syscalls do link and run on the pad (verified: epoll_create1/eventfd return fds), so epoll is technically possible there — but impractical to hand-roll in a 45-minute single-file interview, so the epoll-family modules remain deep-dive reading material, not practice targets. In the interview itself the correct move is a 3-line `Poller` trait stub (Abstract the Noise — see `docs/coderpad-constraints.md`), never live FFI. The README's 學習路徑 section encodes this two-tier priority and the recommended reading order.

`drills` and `challenges` depend on `reference` **only as a test harness** (e.g. the tcp_echo challenge uses reference's event_loop as its base; hw_bridge uses reference's server to validate your framer). Challenge/drill bodies must never copy reference code.

Practice tests carry `#[ignore = "..."]` so `cargo test --workspace` stays green. The learning workflow is: read spec → fill the `todo!()` → remove that test's `#[ignore]` → turn green.

`git log --oneline --reverse` is the intended reading order — commits are staged by difficulty (mutex/condvar → single-threaded data structures → atomics/lock-free → executor → event loop → hw_bridge → drills → challenges).

### The sync_shim / loom mechanism

The library is std-only, but loom needs code under test to use *its* atomic/UnsafeCell types. The trick (`reference/src/sync_shim.rs`):

- Lock-free core algorithms live in standalone `core_impl.rs` files (under `concurrency/{spsc_ring,mpsc_ring,mpmc_ring,mpsc_list,mpmc_list,ws_deque,arena_lockfree}/` and `concurrency/ds_sync/dsu_lockfree/`) that only reference `crate::sync_shim as sync`.
- Lib build: `sync_shim` re-exports std types → zero production dependencies.
- Loom tests (`reference/tests/loom_*.rs`): define their own `sync_shim` module re-exporting loom types, then `#[path]`-include **the same** `core_impl.rs` source. Loom verifies the exact logic the lib ships.

Consequences: keep the shim API surface identical on both sides, and remember loom is a **model checker** (exhaustive interleaving enumeration within a preemption bound), not a fuzzer — keep loom models tiny (2 items, capacity 1–2) or state space explodes.

### Conventions

- **std-only library code.** `loom` and `proptest` are `[dev-dependencies]` of `reference` only — never `use` them in library code. The single non-std surface is `epoll_sys`'s hand-written `unsafe extern "C"` syscall bindings (deliberately no libc crate). epoll makes the repo Linux-only.
- **Docs and comments are in Traditional Chinese.** Follow that in new/edited code.
- Every reference module's `//!` doc follows the 5-pillar structure, in order: `[Clarify]` → `[Abstract]` → `[Iterate]` → `[Trade-offs]` → `[Dry-Run]`. Every core function has at least one boundary test whose doc comment hand-traces the execution line by line.
- Every `unsafe` block has a safety-invariant comment above it. Workspace lints deny `unsafe_op_in_unsafe_fn` and `clippy::all`.
- Complexity annotations (Big-O) in docs must match the implementation.
- `docs/` holds per-topic design trade-off write-ups (why the design, not what the code does), sorted into `ds/`, `concurrency/`, `async/`, `io/` mirroring the module stages; interview-craft docs (clarify-playbook, coderpad-constraints, cost-model, rust-five-axis) stay at the root. Update them if a design changes, don't duplicate code into them.
- Data structures prefer index-based designs (indices into a `Vec` arena) over pointer/`Rc<RefCell>` graphs; `tree.rs` shows both side by side deliberately.
