use crate::{Trigger, VvError, VvResult};
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::sync::oneshot;
use tracing::{debug, info};

/// Global hotkey trigger that records while the hotkey is held.
///
/// This uses `rdev` to listen for keyboard events globally.
/// When the hotkey combination is detected, recording starts/stops.
///
/// # Platform Requirements
/// - **macOS**: Requires Accessibility API permission (prompted on first use)
/// - **Linux**: User must be in the `input` group
#[derive(Debug)]
pub struct GlobalHotkey {
    hotkey_string: String,
}

impl GlobalHotkey {
    /// Create a new `GlobalHotkey` trigger.
    ///
    /// # Arguments
    /// * `hotkey_string` - Hotkey combination (e.g., "Alt+Shift+V", "Ctrl+Shift+V")
    ///
    /// # Returns
    /// A Result containing the `GlobalHotkey` or a `VvError`
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let hotkey = GlobalHotkey::new("Alt+Shift+V")?;
    /// ```
    /// # Errors
    /// `Trigger` if hotkey format is invalid
    pub fn new(hotkey_string: &str) -> VvResult<Self> {
        // Validate hotkey string format
        Self::validate_hotkey(hotkey_string)?;

        Ok(Self {
            hotkey_string: hotkey_string.to_string(),
        })
    }

    /// Validate hotkey string format
    fn validate_hotkey(hotkey: &str) -> VvResult<()> {
        if hotkey.is_empty() {
            return Err(VvError::Trigger(
                "Hotkey string cannot be empty".to_string(),
            ));
        }

        // Check for valid modifiers and key
        let parts: Vec<&str> = hotkey.split('+').collect();
        if parts.is_empty() || parts.len() > 4 {
            return Err(VvError::Trigger(format!(
                "Invalid hotkey format: '{}'. Expected format: Mod1+Mod2+Key (e.g., Alt+Shift+V)",
                hotkey
            )));
        }

        // Validate each part
        let valid_modifiers = ["Ctrl", "Alt", "Shift", "Cmd", "Win", "Meta"];
        for (i, part) in parts.iter().enumerate() {
            let is_last = i == parts.len() - 1;
            let is_valid_modifier = valid_modifiers.contains(&part.trim());

            if !is_last && !is_valid_modifier {
                return Err(VvError::Trigger(format!(
                    "Invalid modifier '{}'. Valid modifiers: {:?}",
                    part, valid_modifiers
                )));
            }

            // Last part should be the key (can be anything)
            if is_last && part.trim().is_empty() {
                return Err(VvError::Trigger("Hotkey cannot end with a '+'".to_string()));
            }
        }

        Ok(())
    }

    /// Parse hotkey string into rdev::Key and modifiers
    fn parse_hotkey(&self) -> VvResult<(rdev::Key, Vec<rdev::Key>)> {
        let parts: Vec<&str> = self.hotkey_string.split('+').collect();

        // Last part is the main key
        let key_str = parts
            .last()
            .ok_or_else(|| VvError::Trigger("Hotkey must have a key component".to_string()))?;

        let key = Self::str_to_key(key_str)?;

        // Earlier parts are modifiers
        let mut modifiers = Vec::new();
        for part in &parts[..parts.len() - 1] {
            let modifier = Self::str_to_modifier(part)?;
            modifiers.push(modifier);
        }

        Ok((key, modifiers))
    }

    /// Convert string key to rdev::Key
    fn str_to_key(s: &str) -> VvResult<rdev::Key> {
        let s_lower = s.trim().to_lowercase();

        // Map common key names to rdev::Key
        let key = match s_lower.as_str() {
            // Letters
            "a" => rdev::Key::KeyA,
            "b" => rdev::Key::KeyB,
            "c" => rdev::Key::KeyC,
            "d" => rdev::Key::KeyD,
            "e" => rdev::Key::KeyE,
            "f" => rdev::Key::KeyF,
            "g" => rdev::Key::KeyG,
            "h" => rdev::Key::KeyH,
            "i" => rdev::Key::KeyI,
            "j" => rdev::Key::KeyJ,
            "k" => rdev::Key::KeyK,
            "l" => rdev::Key::KeyL,
            "m" => rdev::Key::KeyM,
            "n" => rdev::Key::KeyN,
            "o" => rdev::Key::KeyO,
            "p" => rdev::Key::KeyP,
            "q" => rdev::Key::KeyQ,
            "r" => rdev::Key::KeyR,
            "s" => rdev::Key::KeyS,
            "t" => rdev::Key::KeyT,
            "u" => rdev::Key::KeyU,
            "v" => rdev::Key::KeyV,
            "w" => rdev::Key::KeyW,
            "x" => rdev::Key::KeyX,
            "y" => rdev::Key::KeyY,
            "z" => rdev::Key::KeyZ,
            // Numbers
            "0" => rdev::Key::Num0,
            "1" => rdev::Key::Num1,
            "2" => rdev::Key::Num2,
            "3" => rdev::Key::Num3,
            "4" => rdev::Key::Num4,
            "5" => rdev::Key::Num5,
            "6" => rdev::Key::Num6,
            "7" => rdev::Key::Num7,
            "8" => rdev::Key::Num8,
            "9" => rdev::Key::Num9,
            // Function keys
            "f1" => rdev::Key::F1,
            "f2" => rdev::Key::F2,
            "f3" => rdev::Key::F3,
            "f4" => rdev::Key::F4,
            "f5" => rdev::Key::F5,
            "f6" => rdev::Key::F6,
            "f7" => rdev::Key::F7,
            "f8" => rdev::Key::F8,
            "f9" => rdev::Key::F9,
            "f10" => rdev::Key::F10,
            "f11" => rdev::Key::F11,
            "f12" => rdev::Key::F12,
            // Special keys
            "space" => rdev::Key::Space,
            "enter" | "return" => rdev::Key::Return,
            "tab" => rdev::Key::Tab,
            "escape" | "esc" => rdev::Key::Escape,
            "backspace" => rdev::Key::Backspace,
            // Arrow keys
            "up" => rdev::Key::UpArrow,
            "down" => rdev::Key::DownArrow,
            "left" => rdev::Key::LeftArrow,
            "right" => rdev::Key::RightArrow,
            _ => {
                return Err(VvError::Trigger(format!(
                    "Unsupported key '{}'. Supported: A-Z, 0-9, F1-F12, Space, Enter, Tab, Esc, Arrows",
                    s
                )));
            },
        };

        Ok(key)
    }

    /// Convert string modifier to rdev::Key
    fn str_to_modifier(s: &str) -> VvResult<rdev::Key> {
        let s_lower = s.trim().to_lowercase();

        let key = match s_lower.as_str() {
            "ctrl" | "control" => rdev::Key::ControlLeft,
            "alt" => rdev::Key::Alt,
            "shift" => rdev::Key::ShiftLeft,
            "cmd" | "command" | "meta" | "win" => rdev::Key::MetaLeft,
            _ => {
                return Err(VvError::Trigger(format!(
                    "Invalid modifier '{}'. Valid modifiers: Ctrl, Alt, Shift, Cmd/Meta",
                    s
                )));
            },
        };

        Ok(key)
    }
}

#[async_trait::async_trait]
impl Trigger for GlobalHotkey {
    /// Wait for the global hotkey to be pressed (key down).
    async fn wait_for_start(&self) -> VvResult<()> {
        info!("Waiting for global hotkey: {}", self.hotkey_string);

        #[cfg(target_os = "macos")]
        {
            info!("Global hotkey on macOS (may require Accessibility permission)");
        }

        #[cfg(target_os = "linux")]
        {
            info!("Global hotkey on Linux (requires user to be in 'input' group)");
        }

        // Create oneshot channel for signaling
        let (tx, rx) = oneshot::channel();
        let tx = Arc::new(Mutex::new(Some(tx)));
        let done = Arc::new(AtomicBool::new(false));

        // Parse the hotkey
        let (target_key, target_modifiers) = self.parse_hotkey()?;
        debug!(
            "Listening for hotkey: key={:?}, modifiers={:?}",
            target_key, target_modifiers
        );

        // Clone for the callback
        let done_clone = Arc::clone(&done);

        // Spawn a thread to listen for keyboard events
        let _handle = std::thread::spawn(move || {
            // Track modifier state
            let ctrl_pressed = Arc::new(AtomicBool::new(false));
            let alt_pressed = Arc::new(AtomicBool::new(false));
            let shift_pressed = Arc::new(AtomicBool::new(false));
            let meta_pressed = Arc::new(AtomicBool::new(false));

            let ctrl_clone = Arc::clone(&ctrl_pressed);
            let alt_clone = Arc::clone(&alt_pressed);
            let shift_clone = Arc::clone(&shift_pressed);
            let meta_clone = Arc::clone(&meta_pressed);

            let callback = move |event: rdev::Event| {
                if done_clone.load(Ordering::Relaxed) {
                    return;
                }

                match event.event_type {
                    rdev::EventType::KeyPress(key) => {
                        // Track modifier state
                        match key {
                            rdev::Key::ControlLeft | rdev::Key::ControlRight => {
                                ctrl_clone.store(true, Ordering::Relaxed);
                            },
                            rdev::Key::Alt => {
                                alt_clone.store(true, Ordering::Relaxed);
                            },
                            rdev::Key::ShiftLeft | rdev::Key::ShiftRight => {
                                shift_clone.store(true, Ordering::Relaxed);
                            },
                            rdev::Key::MetaLeft | rdev::Key::MetaRight => {
                                meta_clone.store(true, Ordering::Relaxed);
                            },
                            _ => {},
                        }

                        // Check if all required modifiers are pressed and target key matches
                        let modifiers_match = target_modifiers.iter().all(|m| {
                            matches!(m, rdev::Key::ControlLeft | rdev::Key::ControlRight if ctrl_clone.load(Ordering::Relaxed))
                                || matches!(m, rdev::Key::Alt if alt_clone.load(Ordering::Relaxed))
                                || matches!(m, rdev::Key::ShiftLeft | rdev::Key::ShiftRight if shift_clone.load(Ordering::Relaxed))
                                || matches!(m, rdev::Key::MetaLeft | rdev::Key::MetaRight if meta_clone.load(Ordering::Relaxed))
                        });

                        if modifiers_match && key == target_key {
                            debug!("Hotkey pressed: {:?}", key);
                            done_clone.store(true, Ordering::Relaxed);
                            if let Some(tx) = tx.lock().unwrap().take() {
                                let _ = tx.send(());
                            }
                        }
                    },
                    rdev::EventType::KeyRelease(key) => {
                        // Update modifier state on release
                        match key {
                            rdev::Key::ControlLeft | rdev::Key::ControlRight => {
                                ctrl_clone.store(false, Ordering::Relaxed);
                            },
                            rdev::Key::Alt => {
                                alt_clone.store(false, Ordering::Relaxed);
                            },
                            rdev::Key::ShiftLeft | rdev::Key::ShiftRight => {
                                shift_clone.store(false, Ordering::Relaxed);
                            },
                            rdev::Key::MetaLeft | rdev::Key::MetaRight => {
                                meta_clone.store(false, Ordering::Relaxed);
                            },
                            _ => {},
                        }
                    },
                    _ => {},
                }
            };

            // Use a small timeout to allow checking the done flag
            // rdev::listen is blocking, so we use a thread with a timeout
            let _ = rdev::listen(callback);
        });

        // Wait for the signal
        match rx.await {
            Ok(_) => {
                info!("Global hotkey pressed: {}", self.hotkey_string);
                Ok(())
            },
            Err(e) => Err(VvError::Trigger(format!("Hotkey channel error: {}", e))),
        }
    }

    /// Wait for the global hotkey to be released (key up).
    async fn wait_for_stop(&self) -> VvResult<()> {
        info!("Waiting for hotkey release: {}", self.hotkey_string);

        // Create oneshot channel for signaling
        let (tx, rx) = oneshot::channel();
        let tx = Arc::new(Mutex::new(Some(tx)));
        let done = Arc::new(AtomicBool::new(false));

        // Parse the hotkey
        let (target_key, _) = self.parse_hotkey()?;

        // Clone for the callback
        let done_clone = Arc::clone(&done);

        // Spawn a thread to listen for keyboard events
        let _handle = std::thread::spawn(move || {
            let callback = move |event: rdev::Event| {
                if done_clone.load(Ordering::Relaxed) {
                    return;
                }

                if let rdev::EventType::KeyRelease(key) = event.event_type {
                    if key == target_key {
                        debug!("Hotkey released: {:?}", key);
                        done_clone.store(true, Ordering::Relaxed);
                        if let Some(tx) = tx.lock().unwrap().take() {
                            let _ = tx.send(());
                        }
                    }
                }
            };

            let _ = rdev::listen(callback);
        });

        // Wait for the signal
        match rx.await {
            Ok(_) => {
                info!("Global hotkey released: {}", self.hotkey_string);
                Ok(())
            },
            Err(e) => Err(VvError::Trigger(format!("Hotkey channel error: {}", e))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_global_hotkey_creation() {
        let hotkey = GlobalHotkey::new("Ctrl+Shift+V").unwrap();
        assert_eq!(hotkey.hotkey_string, "Ctrl+Shift+V");
    }

    #[test]
    fn test_hotkey_validation() {
        // Valid hotkeys
        assert!(GlobalHotkey::new("Ctrl+V").is_ok());
        assert!(GlobalHotkey::new("Alt+Shift+A").is_ok());
        assert!(GlobalHotkey::new("Cmd+Space").is_ok());

        // Invalid hotkeys
        assert!(GlobalHotkey::new("").is_err()); // Empty
        assert!(GlobalHotkey::new("Invalid+Key").is_err()); // Invalid modifier
    }

    #[test]
    fn test_key_parsing() {
        let hotkey = GlobalHotkey::new("Ctrl+Shift+V").unwrap();
        let (key, modifiers) = hotkey.parse_hotkey().unwrap();

        assert_eq!(key, rdev::Key::KeyV);
        assert_eq!(modifiers.len(), 2);
    }

    #[test]
    fn test_send_sync() {
        // Verify GlobalHotkey implements Send + Sync
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<GlobalHotkey>();
    }
}
