use anyhow::{Context, Result};
use std::fs;
use std::io::{self, Write};

use crate::detector::{DeadCode, SafetyLevel};

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
        let mut result = CleanResult {
            deleted_count: 0,
            skipped_count: 0,
            deleted_lines: 0,
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

            // 削除実行
            if self.delete_code(dc)? {
                result.deleted_count += 1;
                result.deleted_lines += dc.node.line_range.1 - dc.node.line_range.0 + 1;
            } else {
                result.skipped_count += 1;
            }
        }

        Ok(result)
    }

    fn confirm_delete(&self, dc: &DeadCode) -> Result<bool> {
        println!("\n削除候補:");
        println!("  ファイル: {}", dc.node.file_path.display());
        println!("  関数名: {}", dc.node.name);
        println!(
            "  行範囲: {}-{}",
            dc.node.line_range.0, dc.node.line_range.1
        );
        println!("  安全性: {:?}", dc.safety_level);
        println!("  理由: {}", dc.reason);

        print!("削除しますか? (y/n): ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        Ok(input.trim().eq_ignore_ascii_case("y"))
    }

    fn delete_code(&self, dc: &DeadCode) -> Result<bool> {
        if self.dry_run {
            println!(
                "  [DRY RUN] 削除: {}:{}-{}",
                dc.node.file_path.display(),
                dc.node.line_range.0,
                dc.node.line_range.1
            );
            return Ok(true);
        }

        // ファイルを読み込み
        let content = fs::read_to_string(&dc.node.file_path).context(format!(
            "Failed to read file: {}",
            dc.node.file_path.display()
        ))?;

        let lines: Vec<&str> = content.lines().collect();

        // 削除する行範囲を確認
        let start = dc.node.line_range.0.saturating_sub(1);
        let end = dc.node.line_range.1;

        if end > lines.len() {
            eprintln!(
                "  ⚠️  行範囲が不正: {}-{} (ファイルは{}行)",
                dc.node.line_range.0,
                dc.node.line_range.1,
                lines.len()
            );
            return Ok(false);
        }

        // 削除後の内容を作成
        let mut new_lines = Vec::new();
        new_lines.extend_from_slice(&lines[..start]);
        new_lines.extend_from_slice(&lines[end..]);

        let new_content = new_lines.join("\n");

        // ファイルに書き込み
        fs::write(&dc.node.file_path, new_content).context(format!(
            "Failed to write file: {}",
            dc.node.file_path.display()
        ))?;

        println!(
            "  ✅ 削除完了: {}:{}-{}",
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
