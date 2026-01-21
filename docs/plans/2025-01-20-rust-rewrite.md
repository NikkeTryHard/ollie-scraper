# Discord Channel Status Scraper (Rust Rewrite) Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan batch by batch.

**Goal:** Rewrite the Discord scraper in Rust as a single, high-performance CLI tool with integrated background execution and real-time monitoring.

**Architecture:** A single binary using Tokio for async concurrency, handling both WebSocket (Gateway) and HTTP (Polling) simultaneously.

**Tech Stack:** Rust, Tokio, Tungstenite (WS), Reqwest (HTTP), Clap (CLI), Serde (JSON).

---

## Batch 1: Project Initialization & Core Types

**Goal:** Initialize the Cargo project and define the data structures for Discord events.

#### Task 1.1: Cargo Init & Manifest
**Files:**
- Create: `Cargo.toml`
- Create: `.env`

**Steps:**
1. Initialize: `cargo init .`
2. Update `Cargo.toml` with dependencies: `tokio`, `tokio-tungstenite`, `serde`, `serde_json`, `reqwest`, `clap`, `dotenvy`, `notify-rust`.
3. Commit: `chore: init rust project`

#### Task 1.2: Model Definitions
**Files:**
- Create: `src/models.rs`
- Test: `src/models.rs`

**Steps:**
1. Define structs for Gateway payloads and Channel objects.
2. Implement Serde traits.
3. Write serialization/deserialization tests.
4. Commit: `feat: define discord data models`

---

## Batch 2: Notification & Audio System (TDD)

**Goal:** Port the looping notification and sound logic to Rust.

#### Task 2.1: Notifier Module
**Files:**
- Create: `src/notifier.rs`

**Steps:**
1. Implement `notify_status_change` calling `notify-send` and `mpv`.
2. Implement looping logic using a Tokio task.
3. Commit: `feat: add notification and audio system`

---

## Batch 3: Discord Gateway & Polling

**Goal:** Implement the core monitoring logic.

#### Task 3.1: REST Polling & WebSocket
**Files:**
- Create: `src/monitor.rs`

**Steps:**
1. Implement Polling loop.
2. Implement WebSocket Listener with Heartbeat.
3. Commit: `feat: implement dual-mode monitoring`

---

## Batch 4: CLI & Daemon Management

**Goal:** Build the integrated CLI for `run`, `stop`, `status`.

#### Task 4.1: CLI Interface
**Files:**
- Create: `src/main.rs`

**Steps:**
1. Define Clap subcommands.
2. Implement Daemonize logic.
3. Commit: `feat: add integrated CLI`
