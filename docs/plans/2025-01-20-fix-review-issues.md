# Fix Review Issues Plan

> **For Claude:** Use superpowers:executing-plans to implement this plan.

**Goal:** Fix all Critical and Important issues identified in the end-of-worktree review.

**Approach:** Batch related fixes together for efficiency.

---

## Batch 1: Critical Fixes

**Goal:** Fix all 3 critical issues that block merge.

#### Task 1.1: Fix Hardcoded Path

**Files:** `src/main.rs`

- Replace hardcoded `/home/nikketryhard/...` with executable-relative path
- Use `std::env::current_exe()` to find binary location
- Fallback to current directory + "boom.mp3"

#### Task 1.2: Remove Unused Import

**Files:** `src/main.rs`

- Remove `use std::os::unix::process::CommandExt;` from `stop_daemon()`

#### Task 1.3: Replace Unwraps with Expect

**Files:** `src/monitor.rs`

- Replace `.unwrap()` on lines 165, 168, 203 with `.expect("...")` with context

**Commit:** `fix: resolve critical review issues`

---

## Batch 2: DRY & Cleanup

**Goal:** Extract duplicated code and remove unused dependencies.

#### Task 2.1: Extract Channel Change Detection

**Files:** `src/monitor.rs`

- Create helper function `check_and_notify_change()`
- Use in both `poll_loop` and `websocket_loop`

#### Task 2.2: Remove Unused Dependencies

**Files:** `Cargo.toml`

- Remove `notify-rust = "4"` (using shell command instead)
- Remove `url = "2"` (never imported)

#### Task 2.3: Centralize Sound Path Loading

**Files:** `src/main.rs`

- Create `get_sound_path()` helper function
- Use in `load_config()` and `test_notification()`

**Commit:** `refactor: extract helpers and remove unused deps`

---

## Batch 3: Async & Error Handling

**Goal:** Fix blocking I/O and improve error types.

#### Task 3.1: Use Tokio Process for Sound

**Files:** `src/notifier.rs`

- Replace `std::process::Command` with `tokio::process::Command`
- Make `play_sound()` async

#### Task 3.2: Add Sequence Number Support

**Files:** `src/models.rs`, `src/monitor.rs`

- Add `s: Option<u64>` to `GatewayMessage`
- Track last sequence in websocket_loop
- Include in heartbeat payload

#### Task 3.3: Extract Constants

**Files:** `src/monitor.rs`

- `POLL_INTERVAL_SECS: f64 = 1.5`
- `RECONNECT_DELAY_SECS: u64 = 5`

**Commit:** `fix: async process, sequence tracking, constants`

---

## Batch 4: Final Polish

**Goal:** Address remaining important issues.

#### Task 4.1: Add Graceful Shutdown

**Files:** `src/main.rs`, `src/monitor.rs`

- Add `tokio::signal::ctrl_c()` handler for foreground mode
- Clean shutdown of monitor loops

**Commit:** `feat: add graceful shutdown handling`
