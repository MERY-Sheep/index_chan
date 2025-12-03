use anyhow::{Context, Result};
use std::fs;
use std::io::{self, Write};
use std::path::Path;

use crate::detector::{DeadCode, SafetyLevel};
use crate::backup::{BackupManager, ChangeType};

pub struct Cleaner {
    dry_run: bool,
    auto: bool,
    safe_only: bool,
}

impl Cleaner {
    pub fn new(dry_run: bool, auto: bool, safe_only: bool) -> Self {
        Self {
            dry_run,
            auto,
            safe_only,
        }
    }

    pub fn clean(&self, dead_code: &[DeadCode]) -> Result<CleanResult> {
        self.clean_with_backup(dead_code, None)
    }

    pub fn clean_with_backup(&self, dead_code: &[DeadCode], project_root: Option<&Path>) -> Result<CleanResult> {
        let mut result = CleanResult {
            deleted_count: 0,
            skipped_count: 0,
            deleted_lines: 0,
        };

        // バックアップマネージャーとマニフェストを準備
        let (backup_manager, backup_dir, mut manifest) = if !self.dry_run && project_root.is_some() {
            let manager = BackupManager::new(project_root.unwrap());
            let (dir, manifest) = manager.create_backup_dir("clean")?;
            (Some(manager), Some(dir), Some(manifest))
        } else {
            (None, None, None)
        };

        for dc in dead_code {
            // safe_onlyモードでは確実に安全なもののみ
            if self.safe_only && !matches!(dc.safety_level, SafetyLevel::DefinitelySafe) {
                result.skipped_count += 1;
                continue;
            }

            // 自動モードでない場合は確認
            if !self.auto && !self.confirm_delete(dc)? {
                result.skipped_count += 1;
                continue;
            }

            // バックアップを作成
            if let (Some(ref manager), Some(ref backup_dir), Some(ref mut manifest)) = 
                (&backup_manager, &backup_dir, &mut manifest) {
                if dc.node.file_path.exists() {
                    let backup_path = manager.backup_file(&dc.node.file_path, backup_dir)?;
                    let relative_backup = backup_path.strip_prefix(backup_dir)
                        .unwrap_or(&backup_path)
                        .to_path_buf();
                    manifest.add_change(ChangeType::Modified, dc.node.file_path.clone(), Some(relative_backup));
                }
            }

            // 削除実行
            if self.delete_code(dc)? {
                result.deleted_count += 1;
                result.deleted_lines += dc.node.line_range.1 - dc.node.line_range.0 + 1;
            } else {
                result.skipped_count += 1;
            }
        }

        // マニフェストを保存
        if let (Some(ref backup_dir), Some(manifest)) = (&backup_dir, manifest) {
            manifest.save(backup_dir)?;
            println!("\n📦 バックアップを作成しました: {}", backup_dir.display());
        }

        Ok(result)
    }

    fn confirm_delete(&self, dc: &DeadCode) -> Result<bool> {
        println!("\nDeletion candidate:");
        println!("  File: {}", dc.node.file_path.display());
        println!("  Function: {}", dc.node.name);
        println!(
            "  Line range: {}-{}",
            dc.node.line_range.0, dc.node.line_range.1
        );
        println!("  Safety level: {:?}", dc.safety_level);
        println!("  Reason: {}", dc.reason);

        print!("Delete? (y/n): ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        Ok(input.trim().eq_ignore_ascii_case("y"))
    }

    fn delete_code(&self, dc: &DeadCode) -> Result<bool> {
        if self.dry_run {
            println!(
                "  [DRY RUN] Delete: {}:{}-{}",
                dc.node.file_path.display(),
                dc.node.line_range.0,
                dc.node.line_range.1
            );
            return Ok(true);
        }

        // Read file
        let content = fs::read_to_string(&dc.node.file_path).context(format!(
            "Failed to read file: {}",
            dc.node.file_path.display()
        ))?;

        let lines: Vec<&str> = content.lines().collect();

        // Verify line range
        let start = dc.node.line_range.0.saturating_sub(1);
        let end = dc.node.line_range.1;

        if end > lines.len() {
            eprintln!(
                "  ⚠️  Invalid line range: {}-{} (file has {} lines)",
                dc.node.line_range.0,
                dc.node.line_range.1,
                lines.len()
            );
            return Ok(false);
        }

        // Create content after deletion
        let mut new_lines = Vec::new();
        new_lines.extend_from_slice(&lines[..start]);
        new_lines.extend_from_slice(&lines[end..]);

        let new_content = new_lines.join("\n");

        // Write to file
        fs::write(&dc.node.file_path, new_content).context(format!(
            "Failed to write file: {}",
            dc.node.file_path.display()
        ))?;

        println!(
            "  ✅ Deleted: {}:{}-{}",
            dc.node.file_path.display(),
            dc.node.line_range.0,
            dc.node.line_range.1
        );

        Ok(true)
    }
}

pub struct CleanResult {
    pub deleted_count: usize,
    pub skipped_count: usize,
    pub deleted_lines: usize,
}
