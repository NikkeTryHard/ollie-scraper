//! CLI interface for the Discord channel status monitor.
//!
//! Provides commands for running, stopping, and monitoring the scraper daemon.

mod models;
mod monitor;
mod notifier;

use clap::{Parser, Subcommand};
use notifier::Notifier;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

const PID_FILE: &str = "scraper.pid";

/// Get the default sound path by searching relative to the executable.
fn get_default_sound_path() -> String {
    // Try to find boom.mp3 relative to the executable
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            let sound_path = exe_dir.join("boom.mp3");
            if sound_path.exists() {
                return sound_path.to_string_lossy().to_string();
            }
            // Also check parent directory (for target/release/ollie-scraper)
            if let Some(parent) = exe_dir.parent() {
                if let Some(grandparent) = parent.parent() {
                    let sound_path = grandparent.join("boom.mp3");
                    if sound_path.exists() {
                        return sound_path.to_string_lossy().to_string();
                    }
                }
            }
        }
    }
    // Fallback to current directory
    "boom.mp3".to_string()
}

#[derive(Parser)]
#[command(name = "ollie-scraper")]
#[command(about = "Discord channel status monitor")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start monitoring (foreground or background)
    Run {
        /// Run as a background daemon
        #[arg(long)]
        daemon: bool,
    },
    /// Stop the daemon
    Stop,
    /// Show status (running/stopped, PID, uptime)
    Status,
    /// Test notification (play sound + show popup once)
    Test,
}

/// Get the path to the PID file (in the same directory as the executable).
fn get_pid_file_path() -> PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
        .unwrap_or_else(|| PathBuf::from("."))
        .join(PID_FILE)
}

/// Check if a process with the given PID is running.
fn is_process_running(pid: u32) -> bool {
    // On Linux, check if /proc/<pid> exists
    PathBuf::from(format!("/proc/{}", pid)).exists()
}

/// Read PID from the PID file.
fn read_pid() -> Option<u32> {
    let pid_path = get_pid_file_path();
    fs::read_to_string(&pid_path)
        .ok()
        .and_then(|s| s.trim().parse().ok())
}

/// Write PID to the PID file.
fn write_pid(pid: u32) -> std::io::Result<()> {
    let pid_path = get_pid_file_path();
    fs::write(&pid_path, pid.to_string())
}

/// Delete the PID file.
fn delete_pid_file() -> std::io::Result<()> {
    let pid_path = get_pid_file_path();
    if pid_path.exists() {
        fs::remove_file(&pid_path)
    } else {
        Ok(())
    }
}

/// Load configuration from environment variables.
fn load_config() -> Result<(String, String, String), String> {
    // Load .env file if it exists
    dotenvy::dotenv().ok();

    let token = std::env::var("DISCORD_TOKEN")
        .map_err(|_| "DISCORD_TOKEN environment variable not set")?;

    let channel_id = std::env::var("CHANNEL_ID")
        .map_err(|_| "CHANNEL_ID environment variable not set")?;

    // Use default sound path if not specified
    let sound_path =
        std::env::var("SOUND_PATH").unwrap_or_else(|_| get_default_sound_path());

    Ok((token, channel_id, sound_path))
}

/// Run the monitor in the foreground.
async fn run_foreground(token: String, channel_id: String, sound_path: String) {
    println!("Starting ollie-scraper in foreground mode...");
    println!("Sound path: {}", sound_path);
    println!("Channel ID: {}", channel_id);
    println!("Press Ctrl+C to stop.");
    println!();

    monitor::run_monitor(token, channel_id, sound_path).await;
}

/// Run the monitor as a background daemon.
fn run_daemon() -> Result<(), String> {
    // Check if already running
    if let Some(pid) = read_pid() {
        if is_process_running(pid) {
            return Err(format!("Daemon already running with PID {}", pid));
        }
    }

    // Get the current executable path
    let exe_path = std::env::current_exe().map_err(|e| format!("Failed to get executable path: {}", e))?;

    // Get log file path (same directory as executable)
    let log_path = exe_path.parent()
        .map(|p| p.join("scraper.log"))
        .unwrap_or_else(|| PathBuf::from("scraper.log"));

    // Open log file for writing
    let log_file = fs::File::create(&log_path)
        .map_err(|e| format!("Failed to create log file: {}", e))?;

    // Fork to background using nohup and disown pattern
    let child = Command::new(&exe_path)
        .args(["run"])
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::from(log_file.try_clone().unwrap()))
        .stderr(std::process::Stdio::from(log_file))
        .spawn()
        .map_err(|e| format!("Failed to spawn daemon: {}", e))?;

    let pid = child.id();
    write_pid(pid).map_err(|e| format!("Failed to write PID file: {}", e))?;

    println!("Daemon started with PID {}", pid);
    println!("Log file: {:?}", log_path);
    println!("PID file: {:?}", get_pid_file_path());

    Ok(()
)}

/// Stop the running daemon.
fn stop_daemon() -> Result<(), String> {
    let pid = read_pid().ok_or("No PID file found. Is the daemon running?")?;

    if !is_process_running(pid) {
        delete_pid_file().ok();
        return Err(format!("Process {} is not running. Cleaned up stale PID file.", pid));
    }

    // Send SIGTERM
    #[cfg(unix)]
    {
        let status = Command::new("kill")
            .args(["-TERM", &pid.to_string()])
            .status()
            .map_err(|e| format!("Failed to send SIGTERM: {}", e))?;

        if !status.success() {
            return Err(format!("Failed to stop process {}", pid));
        }
    }

    #[cfg(not(unix))]
    {
        return Err("Stop command is only supported on Unix systems".to_string());
    }

    delete_pid_file().map_err(|e| format!("Failed to delete PID file: {}", e))?;

    println!("Stopped daemon (PID {})", pid);
    Ok(())
}

/// Show the daemon status with verbose information.
fn show_status() {
    println!("========================================");
    println!("   OLLIE SCRAPER STATUS");
    println!("========================================");
    println!();

    match read_pid() {
        Some(pid) => {
            if is_process_running(pid) {
                println!("STATUS:    RUNNING");
                println!("PID:       {}", pid);

                // Try to get process stats from /proc
                #[cfg(unix)]
                {
                    // Memory usage from /proc/[pid]/status
                    if let Ok(status) = fs::read_to_string(format!("/proc/{}/status", pid)) {
                        for line in status.lines() {
                            if line.starts_with("VmRSS:") {
                                let parts: Vec<&str> = line.split_whitespace().collect();
                                if parts.len() >= 2 {
                                    if let Ok(kb) = parts[1].parse::<f64>() {
                                        println!("MEMORY:    {:.1} MB", kb / 1024.0);
                                    }
                                }
                            }
                        }
                    }

                    // CPU usage (snapshot)
                    if let Ok(stat) = fs::read_to_string(format!("/proc/{}/stat", pid)) {
                        let parts: Vec<&str> = stat.split_whitespace().collect();
                        if parts.len() > 21 {
                            // Field 22 is starttime in clock ticks since boot
                            if let Ok(starttime) = parts[21].parse::<u64>() {
                                // Get system uptime
                                if let Ok(uptime_str) = fs::read_to_string("/proc/uptime") {
                                    if let Some(uptime_secs) = uptime_str
                                        .split_whitespace()
                                        .next()
                                        .and_then(|s| s.parse::<f64>().ok())
                                    {
                                        let ticks_per_sec = 100u64;
                                        let process_start_secs = starttime / ticks_per_sec;
                                        let process_uptime =
                                            uptime_secs as u64 - process_start_secs;

                                        let hours = process_uptime / 3600;
                                        let minutes = (process_uptime % 3600) / 60;
                                        let seconds = process_uptime % 60;

                                        println!("UPTIME:    {}h {}m {}s", hours, minutes, seconds);
                                    }
                                }
                            }
                        }
                    }
                }

                // Try to read channel info from log file
                println!();
                println!("----------------------------------------");
                println!("   CHANNEL INFO");
                println!("----------------------------------------");

                // Look for log file in same directory as executable
                let log_path = get_pid_file_path().parent()
                    .map(|p| p.join("scraper.log"))
                    .unwrap_or_else(|| PathBuf::from("scraper.log"));

                if let Ok(log_content) = fs::read_to_string(&log_path) {
                    // Find current channel name
                    let mut channel_found = false;
                    for line in log_content.lines() {
                        if line.contains("Initial channel name:") {
                            // Parse: Initial channel name: Some("〖start-order-❌〗")
                            if let Some(start) = line.find("Some(\"") {
                                if let Some(end) = line.rfind("\")") {
                                    let name = &line[start + 6..end];
                                    println!("CHANNEL:   {}", name);
                                    channel_found = true;
                                }
                            }
                        } else if line.contains("Channel ID:") {
                            if let Some(id) = line.split("Channel ID:").nth(1) {
                                println!("CHANNEL ID: {}", id.trim());
                            }
                        }
                    }
                    if !channel_found {
                        println!("CHANNEL:   (waiting for initial fetch)");
                    }

                    println!();
                    println!("----------------------------------------");
                    println!("   STATISTICS");
                    println!("----------------------------------------");

                    // Count events
                    let ws_events = log_content.lines().filter(|l| l.contains("[WS]") && l.contains("changed")).count();
                    let poll_events = log_content.lines().filter(|l| l.contains("[POLL]")).count();
                    let heartbeats = log_content.lines().filter(|l| l.contains("Heartbeat ACK")).count();
                    let alarms = log_content.lines().filter(|l| l.contains("ALARM") || l.contains("start_alarm")).count();

                    println!("WebSocket Events:  {}", ws_events);
                    println!("Poll Detections:   {}", poll_events);
                    println!("Heartbeats:        {}", heartbeats);
                    println!("Alarms Triggered:  {}", alarms);

                    println!();
                    println!("----------------------------------------");
                    println!("   LAST 5 LOG ENTRIES");
                    println!("----------------------------------------");

                    let lines: Vec<&str> = log_content.lines().collect();
                    let start = if lines.len() > 5 { lines.len() - 5 } else { 0 };
                    for line in &lines[start..] {
                        println!("{}", line);
                    }
                } else {
                    println!("CHANNEL:   (no log file found)");
                }

                println!();
                println!("========================================");
            } else {
                println!("STATUS:    STOPPED (stale PID file)");
                println!("PID:       {} (not running)", pid);
                println!();
                println!("Run 'ollie-scraper stop' to clean up the stale PID file.");
            }
        }
        None => {
            println!("STATUS:    STOPPED");
            println!("PID:       -");
            println!();
            println!("========================================");
        }
    }
}

/// Test the notification system.
async fn test_notification() {
    println!("Testing notification system...");
    println!();

    let sound_path = std::env::var("SOUND_PATH").unwrap_or_else(|_| get_default_sound_path());

    // Check if sound file exists
    if !PathBuf::from(&sound_path).exists() {
        eprintln!("Warning: Sound file not found at {}", sound_path);
    }

    let notifier = Notifier::new(sound_path.clone());

    // Send notification
    println!("Sending test notification...");
    match Notifier::send_notification("TEST-CHANNEL").await {
        Ok(_) => println!("  Notification sent successfully"),
        Err(e) => eprintln!("  Failed to send notification: {}", e),
    }

    // Play sound
    println!("Playing test sound: {}", sound_path);
    match notifier.play_sound().await {
        Ok(_) => println!("  Sound played successfully"),
        Err(e) => eprintln!("  Failed to play sound: {}", e),
    }

    println!();
    println!("Test complete.");
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Run { daemon } => {
            if daemon {
                if let Err(e) = run_daemon() {
                    eprintln!("Error: {}", e);
                    std::process::exit(1);
                }
            } else {
                match load_config() {
                    Ok((token, channel_id, sound_path)) => {
                        run_foreground(token, channel_id, sound_path).await;
                    }
                    Err(e) => {
                        eprintln!("Configuration error: {}", e);
                        eprintln!();
                        eprintln!("Please set the following environment variables:");
                        eprintln!("  DISCORD_TOKEN - Your Discord user token");
                        eprintln!("  CHANNEL_ID    - The channel ID to monitor");
                        eprintln!("  SOUND_PATH    - (optional) Path to alarm sound file");
                        std::process::exit(1);
                    }
                }
            }
        }
        Commands::Stop => {
            if let Err(e) = stop_daemon() {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
        Commands::Status => {
            show_status();
        }
        Commands::Test => {
            test_notification().await;
        }
    }
}
