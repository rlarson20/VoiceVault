use chrono::DateTime;
use chrono::Local;
use std::fmt;

/// Represents the type of trigger that initiated a voice recording
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum TriggerKind {
    /// Global hotkey pressed (e.g., Ctrl+Shift+V)
    GlobalHotkey,
    /// Push-to-talk key held down
    PushToTalk,
    /// HTTP endpoint received a request
    HttpEndpoint,
    /// Unix socket received a connection
    UnixSocket,
}

impl fmt::Display for TriggerKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TriggerKind::GlobalHotkey => write!(f, "Global Hotkey"),
            TriggerKind::PushToTalk => write!(f, "Push-to-Talk"),
            TriggerKind::HttpEndpoint => write!(f, "HTTP Endpoint"),
            TriggerKind::UnixSocket => write!(f, "Unix Socket"),
        }
    }
}

/// Represents a voice note with its metadata
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Note {
    /// The transcribed text content
    pub text: String,

    /// When the note was recorded
    pub recorded_at: DateTime<Local>,

    /// Duration of the recording in seconds
    pub duration_secs: f64,

    /// What triggered this recording
    pub trigger: TriggerKind,
}

impl Note {
    /// Creates a new Note
    ///
    /// # Arguments
    /// * `text` - The transcribed text content
    /// * `recorded_at` - When the note was recorded
    /// * `duration_secs` - Duration of the recording in seconds
    /// * `trigger` - What triggered this recording
    #[must_use]
    pub const fn new(
        text: String,
        recorded_at: DateTime<Local>,
        duration_secs: f64,
        trigger: TriggerKind,
    ) -> Self {
        Self {
            text,
            recorded_at,
            duration_secs,
            trigger,
        }
    }

    /// Converts the note to a Markdown string with Obsidian-compatible frontmatter
    ///
    /// # Example
    /// ```ignore
    /// let note = Note::new(
    ///     "This is a test note".to_string(),
    ///     Local::now(),
    ///     5.2,
    ///     TriggerKind::GlobalHotkey,
    /// );
    /// let markdown = note.to_markdown();
    /// ```
    #[must_use]
    pub fn to_markdown(&self) -> String {
        format!(
            "---\ntrigger: {}\nduration_seconds: {}\nrecorded_at: {}\n---\n\n{}",
            self.trigger, self.duration_secs, self.recorded_at, self.text
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_note_to_markdown() {
        // Create a fixed timestamp for reproducible output
        let timestamp = "2025-02-19T10:30:00+00:00";
        let recorded_at: DateTime<Local> = timestamp
            .parse::<DateTime<chrono::Utc>>()
            .unwrap()
            .with_timezone(&Local);

        let note = Note::new(
            "This is a test note".to_string(),
            recorded_at,
            5.2,
            TriggerKind::GlobalHotkey,
        );

        let markdown = note.to_markdown();

        // Check that frontmatter is present
        assert!(markdown.contains("---"));
        assert!(markdown.contains("trigger: Global Hotkey"));
        assert!(markdown.contains("duration_seconds: 5.2"));
        assert!(markdown.contains(&format!("recorded_at: {}", recorded_at)));
        assert!(markdown.contains("This is a test note"));
    }

    #[test]
    fn test_trigger_kind_display() {
        assert_eq!(format!("{}", TriggerKind::GlobalHotkey), "Global Hotkey");
        assert_eq!(format!("{}", TriggerKind::PushToTalk), "Push-to-Talk");
        assert_eq!(format!("{}", TriggerKind::HttpEndpoint), "HTTP Endpoint");
        assert_eq!(format!("{}", TriggerKind::UnixSocket), "Unix Socket");
    }
}
