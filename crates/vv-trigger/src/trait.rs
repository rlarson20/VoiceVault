use async_trait::async_trait;
use crate::VvResult;

/// Trait for determining when to start/stop recording.
#[async_trait]
pub trait Trigger: Send + Sync {
    /// Wait until recording should start.
    async fn wait_for_start(&self) -> VvResult<()>;

    /// Wait until recording should stop.
    async fn wait_for_stop(&self) -> VvResult<()>;
}
