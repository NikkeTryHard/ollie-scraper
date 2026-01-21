//! Notification and audio alarm system for channel status changes.
//!
//! This module provides desktop notifications via `notify-send` and audio alerts
//! via `mpv` that loop until explicitly stopped.

use tokio::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

/// Notifier handles desktop notifications and looping audio alarms.
pub struct Notifier {
    sound_path: String,
    running: Arc<AtomicBool>,
}

impl Notifier {
    /// Create a new Notifier with the specified sound file path.
    pub fn new(sound_path: String) -> Self {
        Self {
            sound_path,
            running: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Get a clone of the running flag for external control.
    pub fn running_flag(&self) -> Arc<AtomicBool> {
        Arc::clone(&self.running)
    }

    /// Check if the alarm is currently running.
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }

    /// Send a desktop notification using notify-send.
    pub async fn send_notification(channel_name: &str) -> std::io::Result<std::process::Output> {
        Command::new("notify-send")
            .args(["-u", "critical", "CHANNEL OPEN", &format!("Channel is now: {}", channel_name)])
            .output()
            .await
    }

    /// Play the alarm sound once using mpv.
    pub async fn play_sound(&self) -> std::io::Result<std::process::Output> {
        Command::new("mpv")
            .args(["--no-video", "--really-quiet", &self.sound_path])
            .output()
            .await
    }

    /// Build the notify-send command arguments (for testing).
    pub fn build_notification_args(channel_name: &str) -> Vec<String> {
        vec![
            "-u".to_string(),
            "critical".to_string(),
            "CHANNEL OPEN".to_string(),
            format!("Channel is now: {}", channel_name),
        ]
    }

    /// Build the mpv command arguments (for testing).
    pub fn build_sound_args(&self) -> Vec<String> {
        vec![
            "--no-video".to_string(),
            "--really-quiet".to_string(),
            self.sound_path.clone(),
        ]
    }

    /// Start the alarm loop. Sends notification once, then loops audio every 3 seconds.
    /// This runs until `stop()` is called.
    pub async fn start_alarm(&self, channel_name: &str) {
        // Set running flag
        self.running.store(true, Ordering::SeqCst);

        // Send notification once at the start
        if let Err(e) = Self::send_notification(channel_name).await {
            eprintln!("Failed to send notification: {}", e);
        }

        // Loop playing the sound until stopped
        while self.running.load(Ordering::SeqCst) {
            if let Err(e) = self.play_sound().await {
                eprintln!("Failed to play sound: {}", e);
            }

            // Wait 3 seconds before playing again, but check running flag more frequently
            for _ in 0..30 {
                if !self.running.load(Ordering::SeqCst) {
                    break;
                }
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
        }
    }

    /// Stop the alarm loop.
    pub fn stop(&self) {
        self.running.store(false, Ordering::SeqCst);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_notification_args_construction() {
        let args = Notifier::build_notification_args("test-channel");

        assert_eq!(args.len(), 4);
        assert_eq!(args[0], "-u");
        assert_eq!(args[1], "critical");
        assert_eq!(args[2], "CHANNEL OPEN");
        assert_eq!(args[3], "Channel is now: test-channel");
    }

    #[test]
    fn test_notification_args_with_special_characters() {
        let args = Notifier::build_notification_args("voice-chat-123");

        assert_eq!(args[3], "Channel is now: voice-chat-123");
    }

    #[test]
    fn test_sound_args_construction() {
        let notifier = Notifier::new("/path/to/sound.mp3".to_string());
        let args = notifier.build_sound_args();

        assert_eq!(args.len(), 3);
        assert_eq!(args[0], "--no-video");
        assert_eq!(args[1], "--really-quiet");
        assert_eq!(args[2], "/path/to/sound.mp3");
    }

    #[test]
    fn test_notifier_creation() {
        let notifier = Notifier::new("/test/path/boom.mp3".to_string());

        assert_eq!(notifier.sound_path, "/test/path/boom.mp3");
        assert!(!notifier.is_running());
    }

    #[test]
    fn test_stop_sets_running_to_false() {
        let notifier = Notifier::new("/test/boom.mp3".to_string());

        // Manually set running to true
        notifier.running.store(true, Ordering::SeqCst);
        assert!(notifier.is_running());

        // Stop should set it to false
        notifier.stop();
        assert!(!notifier.is_running());
    }

    #[test]
    fn test_running_flag_is_shared() {
        let notifier = Notifier::new("/test/boom.mp3".to_string());
        let flag = notifier.running_flag();

        // Initially not running
        assert!(!flag.load(Ordering::SeqCst));

        // Set via notifier
        notifier.running.store(true, Ordering::SeqCst);

        // Flag should reflect the change
        assert!(flag.load(Ordering::SeqCst));

        // Stop via flag
        flag.store(false, Ordering::SeqCst);

        // Notifier should reflect the change
        assert!(!notifier.is_running());
    }

    #[tokio::test]
    async fn test_alarm_can_be_stopped() {
        let notifier = Arc::new(Notifier::new("/nonexistent/path.mp3".to_string()));
        let notifier_clone = Arc::clone(&notifier);

        // Start alarm in background
        let handle = tokio::spawn(async move {
            notifier_clone.start_alarm("test-channel").await;
        });

        // Give it a moment to start
        tokio::time::sleep(Duration::from_millis(50)).await;

        // Verify it's running
        assert!(notifier.is_running());

        // Stop it
        notifier.stop();

        // Wait for the task to complete (should be quick now)
        let result = tokio::time::timeout(Duration::from_secs(1), handle).await;
        assert!(result.is_ok(), "Alarm should stop within timeout");

        // Verify it's stopped
        assert!(!notifier.is_running());
    }
}
