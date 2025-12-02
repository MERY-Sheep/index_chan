use crate::detector::{DeadCode, SafetyLevel};
use colored::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ScanReport {
    pub summary: Summary,
    pub dead_code: Vec<DeadCodeEntry>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Summary {
    pub total_files: usize,
    pub total_functions: usize,
    pub dead_code_count: usize,
    pub dead_code_lines: usize,
    pub reduction_percent: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeadCodeEntry {
    pub file: String,
    pub name: String,
    pub line_start: usize,
    pub line_end: usize,
    pub safety_level: String,
    pub reason: String,
}

pub fn print_report(dead_code: &[DeadCode], total_files: usize, total_functions: usize) {
    let dead_count = dead_code.len();
    let dead_lines: usize = dead_code
        .iter()
        .map(|dc| dc.node.line_range.1 - dc.node.line_range.0 + 1)
        .sum();

    println!("\n{}", "🔍 デッドコード検出結果".bold());
    println!("{}", "━".repeat(50));
    println!();
    println!("📁 総ファイル数: {}", total_files);
    println!("📊 総関数数: {}", total_functions);
    println!("🗑️  未使用関数: {}個 ({}行)", dead_count, dead_lines);
    println!();

    // Group by safety level
    let definitely_safe: Vec<_> = dead_code
        .iter()
        .filter(|dc| matches!(dc.safety_level, SafetyLevel::DefinitelySafe))
        .collect();

    let probably_safe: Vec<_> = dead_code
        .iter()
        .filter(|dc| matches!(dc.safety_level, SafetyLevel::ProbablySafe))
        .collect();

    let needs_review: Vec<_> = dead_code
        .iter()
        .filter(|dc| matches!(dc.safety_level, SafetyLevel::NeedsReview))
        .collect();

    if !definitely_safe.is_empty() {
        println!(
            "{} {}個",
            "[確実に安全]".green().bold(),
            definitely_safe.len()
        );
        for dc in definitely_safe.iter().take(5) {
            print_dead_code_entry(dc);
        }
        if definitely_safe.len() > 5 {
            println!("└─ ... 他{}個", definitely_safe.len() - 5);
        }
        println!();
    }

    if !probably_safe.is_empty() {
        println!(
            "{} {}個",
            "[おそらく安全]".yellow().bold(),
            probably_safe.len()
        );
        for dc in probably_safe.iter().take(5) {
            print_dead_code_entry(dc);
        }
        if probably_safe.len() > 5 {
            println!("└─ ... 他{}個", probably_safe.len() - 5);
        }
        println!();
    }

    if !needs_review.is_empty() {
        println!("{} {}個", "[要確認]".red().bold(), needs_review.len());
        for dc in needs_review.iter().take(5) {
            print_dead_code_entry(dc);
        }
        if needs_review.len() > 5 {
            println!("└─ ... 他{}個", needs_review.len() - 5);
        }
        println!();
    }

    let reduction_percent = if total_functions > 0 {
        (dead_count as f64 / total_functions as f64) * 100.0
    } else {
        0.0
    };

    println!(
        "💾 削減可能な行数: {}行 (全体の {:.1}%)",
        dead_lines, reduction_percent
    );
    println!("💰 削減可能なトークン数: 約 {}トークン", dead_lines * 20);
}

fn print_dead_code_entry(dc: &DeadCode) {
    let path = dc.node.file_path.display();
    let range = format!("{}:{}-{}", path, dc.node.line_range.0, dc.node.line_range.1);
    println!("├─ {} ({})", range, dc.node.name);
}

pub fn generate_json_report(
    dead_code: &[DeadCode],
    total_files: usize,
    total_functions: usize,
) -> ScanReport {
    let dead_count = dead_code.len();
    let dead_lines: usize = dead_code
        .iter()
        .map(|dc| dc.node.line_range.1 - dc.node.line_range.0 + 1)
        .sum();

    let reduction_percent = if total_functions > 0 {
        (dead_count as f64 / total_functions as f64) * 100.0
    } else {
        0.0
    };

    let summary = Summary {
        total_files,
        total_functions,
        dead_code_count: dead_count,
        dead_code_lines: dead_lines,
        reduction_percent,
    };

    let dead_code_entries: Vec<DeadCodeEntry> = dead_code
        .iter()
        .map(|dc| DeadCodeEntry {
            file: dc.node.file_path.to_string_lossy().to_string(),
            name: dc.node.name.clone(),
            line_start: dc.node.line_range.0,
            line_end: dc.node.line_range.1,
            safety_level: format!("{:?}", dc.safety_level).to_lowercase(),
            reason: dc.reason.clone(),
        })
        .collect();

    ScanReport {
        summary,
        dead_code: dead_code_entries,
    }
}
