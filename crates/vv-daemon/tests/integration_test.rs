//! Integration tests for VoiceVault
//!
//! These tests verify the full pipeline from note creation to vault writing.

use std::fs;
use tempfile::TempDir;
use vv_core::{Note, TriggerKind};
use vv_vault::VaultWriter;

#[test]
fn test_note_frontmatter_format() {
    use chrono::Local;

    // Create a fixed timestamp for reproducible output
    let timestamp = "2025-02-19T10:30:00+00:00";
    let recorded_at: chrono::DateTime<Local> = timestamp
        .parse::<chrono::DateTime<chrono::Utc>>()
        .unwrap()
        .with_timezone(&Local);

    let note = Note::new(
        "This is a test transcription".to_string(),
        recorded_at,
        5.2,
        TriggerKind::PushToTalk,
    );

    let markdown = note.to_markdown();

    // Verify frontmatter structure
    assert!(markdown.starts_with("---\n"));
    assert!(markdown.contains("trigger: Push-to-Talk"));
    assert!(markdown.contains("duration_seconds: 5.2"));
    assert!(markdown.contains("recorded_at:"));
    assert!(markdown.contains("---\n\n"));
    assert!(markdown.ends_with("This is a test transcription"));
}

#[test]
fn test_all_trigger_kinds() {
    use chrono::Local;

    let timestamp = "2025-02-19T10:30:00+00:00";
    let recorded_at: chrono::DateTime<Local> = timestamp
        .parse::<chrono::DateTime<chrono::Utc>>()
        .unwrap()
        .with_timezone(&Local);

    // Test all trigger kinds
    let triggers = vec![
        TriggerKind::GlobalHotkey,
        TriggerKind::PushToTalk,
        TriggerKind::HttpEndpoint,
        TriggerKind::UnixSocket,
    ];

    for trigger in triggers {
        let note = Note::new("Test".to_string(), recorded_at, 1.0, trigger.clone());
        let markdown = note.to_markdown();

        assert!(
            markdown.contains(&format!("trigger: {}", trigger)),
            "Trigger kind {} should be in frontmatter",
            trigger
        );
    }
}

#[test]
fn test_vault_writer_creates_file() {
    use chrono::Local;

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let vault_path = temp_dir.path();
    let notes_folder = vault_path.join("voice-notes");

    fs::create_dir_all(&notes_folder).expect("Failed to create notes folder");

    let writer =
        VaultWriter::new_with_path(notes_folder.as_path()).expect("Failed to create writer");

    let note = Note::new(
        "Integration test transcription".to_string(),
        Local::now(),
        2.5,
        TriggerKind::PushToTalk,
    );

    let note_path = writer.write(&note).expect("Failed to write note");

    // Verify file was created
    assert!(note_path.exists());
    assert!(note_path.starts_with(&notes_folder));

    // Verify file content
    let content = fs::read_to_string(&note_path).expect("Failed to read note file");
    assert!(content.contains("---"));
    assert!(content.contains("trigger: Push-to-Talk"));
    assert!(content.contains("Integration test transcription"));
}

#[test]
fn test_vault_writer_with_all_trigger_types() {
    use chrono::Local;

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let vault_path = temp_dir.path();
    let notes_folder = vault_path.join("voice-notes");

    fs::create_dir_all(&notes_folder).expect("Failed to create notes folder");

    let writer =
        VaultWriter::new_with_path(notes_folder.as_path()).expect("Failed to create writer");

    // Test writing notes with each trigger type
    let trigger_tests = vec![
        (TriggerKind::GlobalHotkey, "Hotkey test"),
        (TriggerKind::PushToTalk, "PushToTalk test"),
        (TriggerKind::HttpEndpoint, "HTTP test"),
        (TriggerKind::UnixSocket, "Unix socket test"),
    ];

    for (trigger, text) in &trigger_tests {
        let note = Note::new(text.to_string(), Local::now(), 1.0, trigger.clone());
        let note_path = writer.write(&note).expect("Failed to write note");

        // Verify file was created and contains the trigger type
        assert!(note_path.exists());
        let content = fs::read_to_string(&note_path).expect("Failed to read note file");
        assert!(content.contains(&format!("trigger: {}", trigger)));
        assert!(content.contains(text));
    }
}

#[test]
fn test_note_timestamp_format() {
    use chrono::Local;

    let now = Local::now();
    let note = Note::new(
        "Test content".to_string(),
        now,
        std::f64::consts::PI,
        TriggerKind::HttpEndpoint,
    );

    let markdown = note.to_markdown();

    // Verify timestamp is present and formatted
    assert!(markdown.contains("recorded_at:"));
    assert!(markdown.contains("Test content"));
    assert!(markdown.contains("trigger: HTTP Endpoint"));
    assert!(markdown.contains("duration_seconds: 3.14"));
}
