pub mod r#trait;

#[cfg(feature = "push_to_talk")]
pub mod push_to_talk;

#[cfg(feature = "push_to_talk")]
pub use push_to_talk::PushToTalk;

pub use r#trait::Trigger;

// Re-export error types from vv-core
pub use vv_core::{VvResult, VvError};
