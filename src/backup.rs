use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

/// Backup manifest for tracking file changes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupManifest {
    pub timestamp: DateTime<Utc>,
    pub operation: String,
    pub changes: Vec<FileChange>,
}

/// Type of file change
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ChangeType {
    Modified,
    Created,
    Deleted,
}

/// Individual file change record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileChange {
    pub change_type: ChangeType,
    pub path: PathBuf,
    pub backup_path: Option<PathBuf>,
}

impl BackupManifest {
    /// Create a new backup manifest
    pub fn new(operation: &str) -> Self {
        Self {
            timestamp: Utc::now(),
            operation: operation.to_string(),
            changes: Vec::new(),
        }
    }

    /// Add a file change to the manifest
    pub fn add_change(&mut self, change_type: ChangeType, path: PathBuf, backup_path: Option<PathBuf>) {
        self.changes.push(FileChange {
            change_type,
            path,
            backup_path,
        });
    }

    /// Save manifest to file
    pub fn save(&self, backup_dir: &Path) -> Result<()> {
        let manifest_path = backup_dir.join("manifest.json");
        let json = serde_json::to_string_pretty(self)
            .context("Failed to serialize manifest")?;
        fs::write(&manifest_path, json)
            .context("Failed to write manifest file")?;
        Ok(())
    }

    /// Load manifest from file
    pub fn load(backup_dir: &Path) -> Result<Self> {
        let manifest_path = backup_dir.join("manifest.json");
        let json = fs::read_to_string(&manifest_path)
            .context("Failed to read manifest file")?;
        let manifest = serde_json::from_str(&json)
            .context("Failed to parse manifest")?;
        Ok(manifest)
    }
}

/// Backup manager for creating and restoring backups
pub struct BackupManager {
    backup_root: PathBuf,
}

impl BackupManager {
    /// Create a new backup manager
    pub fn new(project_root: &Path) -> Self {
        Self {
            backup_root: project_root.join(".index-chan").join("backups"),
        }
    }

    /// Create a new backup directory with timestamp
    pub fn create_backup_dir(&self, operation: &str) -> Result<(PathBuf, BackupManifest)> {
        let timestamp = Utc::now().format("%Y%m%d_%H%M%S").to_string();
        let backup_dir = self.backup_root.join(&timestamp);
        
        fs::create_dir_all(&backup_dir)
            .context("Failed to create backup directory")?;
        
        let manifest = BackupManifest::new(operation);
        
        Ok((backup_dir, manifest))
    }

    /// Backup a file before modification
    pub fn backup_file(&self, file_path: &Path, backup_dir: &Path) -> Result<PathBuf> {
        let file_name = file_path
            .file_name()
            .context("Invalid file path")?
            .to_string_lossy();
        
        let backup_path = backup_dir.join(format!("{}.bak", file_name));
        
        fs::copy(file_path, &backup_path)
            .context(format!("Failed to backup file: {}", file_path.display()))?;
        
        Ok(backup_path)
    }

    /// Get the most recent backup directory
    pub fn get_latest_backup(&self) -> Result<Option<PathBuf>> {
        if !self.backup_root.exists() {
            return Ok(None);
        }

        let mut backups: Vec<_> = fs::read_dir(&self.backup_root)?
            .filter_map(|e| e.ok())
            .filter(|e| e.path().is_dir())
            .collect();

        if backups.is_empty() {
            return Ok(None);
        }

        backups.sort_by_key(|e| e.file_name());
        Ok(Some(backups.last().unwrap().path()))
    }

    /// List all backup directories
    pub fn list_backups(&self) -> Result<Vec<PathBuf>> {
        if !self.backup_root.exists() {
            return Ok(Vec::new());
        }

        let mut backups: Vec<_> = fs::read_dir(&self.backup_root)?
            .filter_map(|e| e.ok())
            .filter(|e| e.path().is_dir())
            .map(|e| e.path())
            .collect();

        backups.sort();
        Ok(backups)
    }

    /// Restore from a backup using manifest
    pub fn restore(&self, backup_dir: &Path) -> Result<RestoreResult> {
        let manifest = BackupManifest::load(backup_dir)?;
        
        let mut restored_count = 0;
        let mut failed_files = Vec::new();

        for change in &manifest.changes {
            match self.restore_change(change, backup_dir) {
                Ok(_) => restored_count += 1,
                Err(e) => {
                    eprintln!("⚠️  Failed to restore {}: {}", change.path.display(), e);
                    failed_files.push(change.path.clone());
                }
            }
        }

        Ok(RestoreResult {
            restored_count,
            failed_files,
            manifest,
        })
    }

    /// Restore a single file change
    fn restore_change(&self, change: &FileChange, backup_dir: &Path) -> Result<()> {
        match change.change_type {
            ChangeType::Modified => {
                // Restore from backup
                if let Some(backup_path) = &change.backup_path {
                    let full_backup_path = backup_dir.join(backup_path);
                    fs::copy(&full_backup_path, &change.path)
                        .context("Failed to restore modified file")?;
                }
            }
            ChangeType::Created => {
                // Delete the created file
                if change.path.exists() {
                    fs::remove_file(&change.path)
                        .context("Failed to delete created file")?;
                }
            }
            ChangeType::Deleted => {
                // Restore from backup
                if let Some(backup_path) = &change.backup_path {
                    let full_backup_path = backup_dir.join(backup_path);
                    if let Some(parent) = change.path.parent() {
                        fs::create_dir_all(parent)?;
                    }
                    fs::copy(&full_backup_path, &change.path)
                        .context("Failed to restore deleted file")?;
                }
            }
        }
        Ok(())
    }
}

/// Result of a restore operation
pub struct RestoreResult {
    pub restored_count: usize,
    pub failed_files: Vec<PathBuf>,
    #[allow(dead_code)]
    pub manifest: BackupManifest,
}
