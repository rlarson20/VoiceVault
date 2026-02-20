pub mod r#trait;

#[cfg(feature = "push_to_talk")]
pub mod push_to_talk;

#[cfg(feature = "push_to_talk")]
pub use push_to_talk::PushToTalk;

#[cfg(feature = "global_hotkey")]
pub mod global_hotkey;

#[cfg(feature = "global_hotkey")]
pub use global_hotkey::GlobalHotkey;

#[cfg(feature = "http_endpoint")]
pub mod http_endpoint;

#[cfg(feature = "http_endpoint")]
pub use http_endpoint::HttpEndpoint;

#[cfg(feature = "unix_socket")]
pub mod unix_socket;

#[cfg(feature = "unix_socket")]
pub use unix_socket::UnixSocket;

pub use r#trait::Trigger;

// Re-export error types from vv-core
pub use vv_core::{VvError, VvResult};
