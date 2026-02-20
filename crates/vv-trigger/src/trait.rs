use async_trait::async_trait;

/// Trait for determining when to start/stop recording.
#[async_trait]
pub trait Trigger: Send + Sync {
    /// Returns true when recording should start.
    async fn should_start(&self) -> bool;

    /// Returns true when recording should stop.
    async fn should_stop(&self) -> bool;
}
