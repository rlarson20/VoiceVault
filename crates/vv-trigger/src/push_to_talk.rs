use std::io::{self, Write};
use crate::{Trigger, VvResult, VvError};

/// Push-to-talk trigger that waits for Enter key presses.
///
/// This is a simple marker type - all state is managed via stdin interactions.
#[derive(Debug, Clone, Copy)]
pub struct PushToTalk;

impl PushToTalk {
    /// Create a new PushToTalk trigger.
    pub fn new() -> Self {
        Self
    }
}

impl Default for PushToTalk {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl Trigger for PushToTalk {
    /// Wait for the user to press Enter to start recording.
    async fn wait_for_start(&self) -> VvResult<()> {
        print!("Press Enter to start recording... ");
        io::stdout().flush().unwrap();

        let mut line = String::new();
        io::stdin().read_line(&mut line)
            .map_err(|e| VvError::Trigger(format!("Failed to read stdin: {}", e)))?;
        Ok(())
    }

    /// Wait for the user to press Enter to stop recording.
    async fn wait_for_stop(&self) -> VvResult<()> {
        print!("Recording... Press Enter to stop");
        io::stdout().flush().unwrap();

        let mut line = String::new();
        io::stdin().read_line(&mut line)
            .map_err(|e| VvError::Trigger(format!("Failed to read stdin: {}", e)))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_push_to_talk_creation() {
        let trigger = PushToTalk::new();
        let default_trigger = PushToTalk::default();

        // Both should create valid instances
        let _ = trigger;
        let _ = default_trigger;
    }

    #[test]
    fn test_push_to_talk_send_sync() {
        // Verify PushToTalk implements Send + Sync
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<PushToTalk>();
    }
}
