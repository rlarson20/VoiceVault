use std::path::{Path, PathBuf};
use vv_core::{Note, VaultConfig, VvError, VvResult};
use chrono::Local;
use tracing::{debug, info};

/// Writes transcribed notes to an Obsidian vault
///
/// # Examples
///
/// ```ignore
/// use vv_vault::VaultWriter;
/// use vv_core::VaultConfig;
///
/// let config = VaultConfig {
///     path: PathBuf::from("/Users/you/Notes"),
///     folder: "captures".to_string(),
///     filename_format: "%Y-%m-%dT%H-%M-%S".to_string(),
/// };
/// let writer = VaultWriter::new(&config)?;
/// ```
#[derive(Debug, Clone)]
pub struct VaultWriter {
    /// Path to the vault directory
    vault_path: PathBuf,
    /// Folder name within vault for recordings
    folder: String,
    /// Filename format (strftime-compatible)
    filename_format: String,
}

impl VaultWriter {
    /// Creates a new VaultWriter from a VaultConfig
    ///
    /// # Arguments
    /// * `config` - Vault configuration containing path, folder, and filename format
    ///
    /// # Returns
    /// A Result containing the VaultWriter or a VvError
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let writer = VaultWriter::new(&config)?;
    /// ```
    pub fn new(config: &VaultConfig) -> VvResult<Self> {
        // Validate that vault path exists
        if !config.path.exists() {
            return Err(VvError::VaultNotFound(config.path.clone()));
        }

        Ok(Self {
            vault_path: config.path.clone(),
            folder: config.folder.clone(),
            filename_format: config.filename_format.clone(),
        })
    }

    /// Writes a note to the vault as a Markdown file
    ///
    /// This method:
    /// 1. Creates the target folder if it doesn't exist
    /// 2. Generates a filename using the configured format
    /// 3. Writes the note content to a temporary file
    /// 4. Atomically renames the temp file to the final filename
    ///
    /// The atomic write ensures that Obsidian's file watcher never sees
    /// a partially written file.
    ///
    /// # Arguments
    /// * `note` - The note to write
    ///
    /// # Returns
    /// A Result containing the path to the written file or a VvError
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let note = Note::new(
    ///     "Hello world".to_string(),
    ///     Local::now(),
    ///     5.2,
    ///     TriggerKind::GlobalHotkey,
    /// );
    /// let path = writer.write(&note)?;
    /// info!("Note saved to: {}", path.display());
    /// ```
    pub fn write(&self, note: &Note) -> VvResult<PathBuf> {
        // Create the target folder if it doesn't exist
        let target_dir = self.vault_path.join(&self.folder);
        std::fs::create_dir_all(&target_dir).map_err(|e| {
            VvError::VaultWrite(format!(
                "Failed to create directory {}: {}",
                target_dir.display(),
                e
            ))
        })?;

        debug!("Target directory ensured: {}", target_dir.display());

        // Generate filename using the configured format
        let filename = note
            .recorded_at
            .format(&self.filename_format)
            .to_string();
        let file_path = target_dir.join(format!("{}.md", filename));
        let temp_path = target_dir.join(format!("{}.md.tmp", filename));

        debug!("Writing note to temporary file: {}", temp_path.display());

        // Write to temporary file first
        let markdown_content = note.to_markdown();
        std::fs::write(&temp_path, &markdown_content).map_err(|e| {
            VvError::VaultWrite(format!(
                "Failed to write temporary file {}: {}",
                temp_path.display(),
                e
            ))
        })?;

        debug!("Atomically renaming to final path: {}", file_path.display());

        // Atomically rename to final filename
        std::fs::rename(&temp_path, &file_path).map_err(|e| {
            // Clean up temp file if rename fails
            let _ = std::fs::remove_file(&temp_path);
            VvError::VaultWrite(format!(
                "Failed to rename {} to {}: {}",
                temp_path.display(),
                file_path.display(),
                e
            ))
        })?;

        info!("Note written successfully to: {}", file_path.display());

        Ok(file_path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use vv_core::TriggerKind;
    use tempfile::TempDir;

    #[test]
    fn test_vault_writer_creates_folder() {
        // Create a temporary vault directory
        let temp_vault = TempDir::new().expect("Failed to create temp dir");
        let vault_path = temp_vault.path();

        // Create a VaultConfig with a non-existent subfolder
        let config = VaultConfig {
            path: vault_path.to_path_buf(),
            folder: "captures".to_string(),
            filename_format: "%Y-%m-%dT%H-%M-%S".to_string(),
        };

        // Create VaultWriter
        let writer = VaultWriter::new(&config).expect("Failed to create VaultWriter");

        // Create a test note
        let note = Note::new(
            "Test note content".to_string(),
            Local::now(),
            5.0,
            TriggerKind::GlobalHotkey,
        );

        // Write the note
        let result = writer.write(&note);
        assert!(result.is_ok(), "Write should succeed");

        // Verify the folder was created
        let target_dir = vault_path.join("captures");
        assert!(target_dir.exists(), "Target directory should be created");
        assert!(target_dir.is_dir(), "Target should be a directory");
    }

    #[test]
    fn test_vault_writer_atomic_write() {
        // Create a temporary vault directory
        let temp_vault = TempDir::new().expect("Failed to create temp dir");
        let vault_path = temp_vault.path();

        // Create a VaultConfig
        let config = VaultConfig {
            path: vault_path.to_path_buf(),
            folder: "notes".to_string(),
            filename_format: "%Y-%m-%dT%H-%M-%S".to_string(),
        };

        // Create VaultWriter
        let writer = VaultWriter::new(&config).expect("Failed to create VaultWriter");

        // Create a test note
        let note = Note::new(
            "Atomic write test content".to_string(),
            Local::now(),
            3.5,
            TriggerKind::PushToTalk,
        );

        // Write the note
        let file_path = writer.write(&note).expect("Write should succeed");

        // Verify the final file exists
        assert!(file_path.exists(), "Final file should exist");
        assert!(file_path.is_file(), "Final path should be a file");

        // Verify the .tmp file was cleaned up
        let temp_path = file_path.with_extension("md.tmp");
        assert!(!temp_path.exists(), "Temporary file should be cleaned up");

        // Verify the content is correct
        let content = std::fs::read_to_string(&file_path)
            .expect("Failed to read written file");
        assert!(content.contains("Atomic write test content"));
        assert!(content.contains("---"));
        assert!(content.contains("trigger: Push-to-Talk"));
        assert!(content.contains("duration_seconds: 3.5"));
    }

    #[test]
    fn test_vault_writer_nonexistent_vault() {
        // Create a VaultConfig pointing to a non-existent path
        let config = VaultConfig {
            path: PathBuf::from("/nonexistent/path/that/does/not/exist"),
            folder: "notes".to_string(),
            filename_format: "%Y-%m-%dT%H-%M-%S".to_string(),
        };

        // Attempt to create VaultWriter
        let result = VaultWriter::new(&config);

        // Should return an error
        assert!(result.is_err(), "Should fail with non-existent vault path");

        if let Err(VvError::VaultNotFound(path)) = result {
            assert_eq!(
                path,
                PathBuf::from("/nonexistent/path/that/does/not/exist")
            );
        } else {
            panic!("Expected VaultNotFound error");
        }
    }
}
