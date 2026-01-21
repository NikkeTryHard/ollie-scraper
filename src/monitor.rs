//! Dual-mode Discord channel monitoring via REST polling and WebSocket Gateway.
//!
//! This module provides two concurrent monitoring strategies:
//! - REST polling: Periodically fetches channel info via Discord API
//! - WebSocket: Real-time updates via Discord Gateway

use crate::models::{Channel, GatewayMessage, HelloPayload, IdentifyPayload, IdentifyProperties};
use crate::notifier::Notifier;
use futures_util::{SinkExt, StreamExt};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio_tungstenite::{connect_async, tungstenite::Message};

const DISCORD_API_BASE: &str = "https://discord.com/api/v9";
const DISCORD_GATEWAY_URL: &str = "wss://gateway.discord.gg/?v=9&encoding=json";
const USER_AGENT: &str = "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36";

/// Check for channel name changes and notify if changed.
///
/// This helper extracts the common pattern used in both poll_loop and websocket_loop
/// to avoid code duplication.
async fn check_and_notify_change(
    new_name: Option<String>,
    last_name: &Arc<RwLock<Option<String>>>,
    notifier: &Arc<Notifier>,
    source: &str,
) {
    let last = last_name.read().await;
    if *last != new_name {
        drop(last);
        let mut last_write = last_name.write().await;
        *last_write = new_name.clone();
        drop(last_write);
        if let Some(ref name) = new_name {
            println!("[{}] Channel name changed to: {}", source, name);
            notifier.start_alarm(name).await;
        }
    }
}

/// Fetch channel name from Discord REST API.
///
/// Returns `Ok(Some(name))` if the channel exists and has a name,
/// `Ok(None)` if the channel exists but has no name (e.g., DM channels),
/// or an error if the request fails.
pub async fn fetch_channel_name(
    token: &str,
    channel_id: &str,
) -> Result<Option<String>, reqwest::Error> {
    let client = reqwest::Client::new();
    let url = format!("{}/channels/{}", DISCORD_API_BASE, channel_id);

    let response = client
        .get(&url)
        .header("Authorization", token)
        .header("User-Agent", USER_AGENT)
        .send()
        .await?
        .error_for_status()?;

    let channel: Channel = response.json().await?;
    Ok(channel.name)
}

/// Poll Discord REST API for channel name changes.
///
/// This loop runs indefinitely, checking for channel name changes
/// at the specified interval. When a change is detected, it triggers
/// the notifier alarm.
pub async fn poll_loop(
    token: String,
    channel_id: String,
    poll_interval: f64,
    notifier: Arc<Notifier>,
    last_name: Arc<RwLock<Option<String>>>,
) {
    let interval = Duration::from_secs_f64(poll_interval);

    loop {
        tokio::time::sleep(interval).await;

        match fetch_channel_name(&token, &channel_id).await {
            Ok(current_name) => {
                check_and_notify_change(current_name, &last_name, &notifier, "POLL").await;
            }
            Err(e) => {
                eprintln!("[POLL] Failed to fetch channel: {}", e);
            }
        }
    }
}

/// Connect to Discord Gateway and listen for CHANNEL_UPDATE events.
///
/// This function:
/// 1. Connects to the Discord WebSocket Gateway
/// 2. Handles the Hello message and extracts heartbeat interval
/// 3. Sends Identify payload with browser spoofing
/// 4. Spawns a heartbeat task
/// 5. Listens for CHANNEL_UPDATE events and triggers alarms on changes
pub async fn websocket_loop(
    token: String,
    channel_id: String,
    notifier: Arc<Notifier>,
    last_name: Arc<RwLock<Option<String>>>,
) {
    loop {
        println!("[WS] Connecting to Discord Gateway...");

        match connect_async(DISCORD_GATEWAY_URL).await {
            Ok((ws_stream, _)) => {
                println!("[WS] Connected to Gateway");

                let (mut write, mut read) = ws_stream.split();

                // Wait for Hello message (op 10)
                let heartbeat_interval = match read.next().await {
                    Some(Ok(Message::Text(text))) => {
                        match serde_json::from_str::<GatewayMessage>(&text) {
                            Ok(msg) if msg.op == 10 => {
                                if let Some(d) = msg.d {
                                    match serde_json::from_value::<HelloPayload>(d) {
                                        Ok(hello) => {
                                            println!(
                                                "[WS] Received Hello, heartbeat_interval: {}ms",
                                                hello.heartbeat_interval
                                            );
                                            hello.heartbeat_interval
                                        }
                                        Err(e) => {
                                            eprintln!("[WS] Failed to parse Hello payload: {}", e);
                                            continue;
                                        }
                                    }
                                } else {
                                    eprintln!("[WS] Hello message missing 'd' field");
                                    continue;
                                }
                            }
                            Ok(msg) => {
                                eprintln!("[WS] Expected op 10, got op {}", msg.op);
                                continue;
                            }
                            Err(e) => {
                                eprintln!("[WS] Failed to parse Gateway message: {}", e);
                                continue;
                            }
                        }
                    }
                    Some(Ok(_)) => {
                        eprintln!("[WS] Expected text message for Hello");
                        continue;
                    }
                    Some(Err(e)) => {
                        eprintln!("[WS] WebSocket error: {}", e);
                        continue;
                    }
                    None => {
                        eprintln!("[WS] Connection closed before Hello");
                        continue;
                    }
                };

                // Send Identify (op 2)
                let identify = GatewayMessage {
                    op: 2,
                    t: None,
                    d: Some(
                        serde_json::to_value(IdentifyPayload {
                            token: token.clone(),
                            properties: IdentifyProperties {
                                os: "linux".to_string(),
                                browser: "Chrome".to_string(),
                                device: "Chrome".to_string(),
                            },
                        })
                        .expect("Failed to serialize identify properties"),
                    ),
                };

                let identify_json = serde_json::to_string(&identify)
                    .expect("Failed to serialize identify payload");
                if let Err(e) = write.send(Message::Text(identify_json)).await {
                    eprintln!("[WS] Failed to send Identify: {}", e);
                    continue;
                }
                println!("[WS] Sent Identify payload");

                // Spawn heartbeat task
                let heartbeat_interval_ms = heartbeat_interval;
                let (heartbeat_tx, mut heartbeat_rx) = tokio::sync::mpsc::channel::<()>(1);

                let heartbeat_handle = tokio::spawn(async move {
                    let interval = Duration::from_millis(heartbeat_interval_ms);
                    loop {
                        tokio::time::sleep(interval).await;
                        if heartbeat_tx.send(()).await.is_err() {
                            break;
                        }
                    }
                });

                // Main event loop
                let channel_id_clone = channel_id.clone();
                let notifier_clone = Arc::clone(&notifier);
                let last_name_clone = Arc::clone(&last_name);

                loop {
                    tokio::select! {
                        // Handle heartbeat
                        Some(()) = heartbeat_rx.recv() => {
                            let heartbeat = GatewayMessage {
                                op: 1,
                                t: None,
                                d: None,
                            };
                            let heartbeat_json = serde_json::to_string(&heartbeat)
                                .expect("Failed to serialize heartbeat payload");
                            if let Err(e) = write.send(Message::Text(heartbeat_json)).await {
                                eprintln!("[WS] Failed to send heartbeat: {}", e);
                                break;
                            }
                        }

                        // Handle incoming messages
                        msg = read.next() => {
                            match msg {
                                Some(Ok(Message::Text(text))) => {
                                    if let Ok(gateway_msg) = serde_json::from_str::<GatewayMessage>(&text) {
                                        // Handle CHANNEL_UPDATE (op 0, t: "CHANNEL_UPDATE")
                                        if gateway_msg.op == 0 {
                                            if let Some(ref t) = gateway_msg.t {
                                                if t == "CHANNEL_UPDATE" {
                                                    if let Some(d) = gateway_msg.d {
                                                        if let Ok(channel) = serde_json::from_value::<Channel>(d) {
                                                            if channel.id == channel_id_clone {
                                                                check_and_notify_change(
                                                                    channel.name,
                                                                    &last_name_clone,
                                                                    &notifier_clone,
                                                                    "WS",
                                                                ).await;
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                        // Handle heartbeat ACK (op 11) - just acknowledge
                                        else if gateway_msg.op == 11 {
                                            // Heartbeat acknowledged, continue
                                        }
                                    }
                                }
                                Some(Ok(Message::Close(_))) => {
                                    println!("[WS] Connection closed by server");
                                    break;
                                }
                                Some(Err(e)) => {
                                    eprintln!("[WS] WebSocket error: {}", e);
                                    break;
                                }
                                None => {
                                    println!("[WS] Connection closed");
                                    break;
                                }
                                _ => {}
                            }
                        }
                    }
                }

                // Clean up heartbeat task
                heartbeat_handle.abort();
            }
            Err(e) => {
                eprintln!("[WS] Failed to connect: {}", e);
            }
        }

        // Wait before reconnecting
        println!("[WS] Reconnecting in 5 seconds...");
        tokio::time::sleep(Duration::from_secs(5)).await;
    }
}

/// Run the complete dual-mode monitoring system.
///
/// This function:
/// 1. Fetches the initial channel name
/// 2. Runs both polling and WebSocket loops concurrently
pub async fn run_monitor(token: String, channel_id: String, sound_path: String) {
    let notifier = Arc::new(Notifier::new(sound_path));
    let last_name: Arc<RwLock<Option<String>>> = Arc::new(RwLock::new(None));

    // Fetch initial channel name
    println!("Fetching initial channel state...");
    match fetch_channel_name(&token, &channel_id).await {
        Ok(name) => {
            println!("Initial channel name: {:?}", name);
            let mut last = last_name.write().await;
            *last = name;
        }
        Err(e) => {
            eprintln!("Failed to fetch initial channel state: {}", e);
        }
    }

    // Run both monitoring modes concurrently
    let poll_token = token.clone();
    let poll_channel_id = channel_id.clone();
    let poll_notifier = Arc::clone(&notifier);
    let poll_last_name = Arc::clone(&last_name);

    let ws_token = token;
    let ws_channel_id = channel_id;
    let ws_notifier = Arc::clone(&notifier);
    let ws_last_name = Arc::clone(&last_name);

    println!("Starting dual-mode monitoring (REST polling + WebSocket)...");

    tokio::join!(
        poll_loop(poll_token, poll_channel_id, 1.5, poll_notifier, poll_last_name),
        websocket_loop(ws_token, ws_channel_id, ws_notifier, ws_last_name)
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constants() {
        assert!(DISCORD_API_BASE.starts_with("https://"));
        assert!(DISCORD_GATEWAY_URL.starts_with("wss://"));
        assert!(USER_AGENT.contains("Mozilla"));
    }

    #[test]
    fn test_api_url_construction() {
        let channel_id = "123456789";
        let url = format!("{}/channels/{}", DISCORD_API_BASE, channel_id);
        assert_eq!(url, "https://discord.com/api/v9/channels/123456789");
    }

    #[tokio::test]
    async fn test_last_name_rwlock_behavior() {
        let last_name: Arc<RwLock<Option<String>>> = Arc::new(RwLock::new(None));

        // Initial state
        {
            let read = last_name.read().await;
            assert!(read.is_none());
        }

        // Write new value
        {
            let mut write = last_name.write().await;
            *write = Some("test-channel".to_string());
        }

        // Read new value
        {
            let read = last_name.read().await;
            assert_eq!(*read, Some("test-channel".to_string()));
        }
    }
}
