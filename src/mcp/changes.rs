// Batch Changes for MCP
// validate_changes, apply_changes, preview_changes の実装

use std::path::{Path, PathBuf};
use std::collections::HashMap;
use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};

use crate::graph::CodeGraph;
use crate::scanner::Scanner;

/// Change validation result
#[derive(Debug, Clone, Serialize)]
pub struct ValidationResult {
    pub status: ValidationStatus,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
    pub affected_files: Vec<String>,
    pub affected_functions: Vec<String>,
    pub new_functions: Vec<String>,
    pub import_issues: Vec<ImportIssue>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub enum ValidationStatus {
    Ok,
    Warning,
    Error,
}

/// Import validation issue
#[derive(Debug, Clone, Serialize)]
pub struct ImportIssue {
    pub file: String,
    pub line: usize,
    pub import_path: String,
    pub issue: String,
}

/// File change
#[derive(Debug, Clone, Deserialize)]
pub struct FileChange {
    pub file_path: String,
    pub original_content: Option<String>,
    pub modified_content: String,
    pub start_line: Option<usize>,
    pub end_line: Option<usize>,
}

/// Change application result
#[derive(Debug, Clone, Serialize)]
pub struct ApplyResult {
    pub success: bool,
    pub applied_files: Vec<String>,
    pub failed_files: Vec<FailedFile>,
    pub backup_dir: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct FailedFile {
    pub file: String,
    pub error: String,
}

/// Preview result
#[derive(Debug, Clone, Serialize)]
pub struct PreviewResult {
    pub diffs: Vec<FileDiff>,
    pub total_additions: usize,
    pub total_deletions: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct FileDiff {
    pub file_path: String,
    pub diff: String,
    pub additions: usize,
    pub deletions: usize,
}

/// Change manager
pub struct ChangeManager {
    project_dir: PathBuf,
    graph: Option<CodeGraph>,
}

impl ChangeManager {
    /// Create from directory
    pub fn from_directory(directory: &Path) -> Result<Self> {
        Ok(Self {
            project_dir: directory.to_path_buf(),
            graph: None,
        })
    }

    /// Load graph for validation
    fn ensure_graph(&mut self) -> Result<&CodeGraph> {
        if self.graph.is_none() {
            let mut scanner = Scanner::new()?;
            self.graph = Some(scanner.scan_directory(&self.project_dir)?);
        }
        Ok(self.graph.as_ref().unwrap())
    }

    /// Validate changes
    pub fn validate_changes(&mut self, changes: &[FileChange]) -> Result<ValidationResult> {
        let mut result = ValidationResult {
            status: ValidationStatus::Ok,
            warnings: Vec::new(),
            errors: Vec::new(),
            affected_files: Vec::new(),
            affected_functions: Vec::new(),
            new_functions: Vec::new(),
            import_issues: Vec::new(),
        };

        // Load graph for dependency analysis
        self.ensure_graph()?;
        let graph = self.graph.as_ref().unwrap();

        for change in changes {
            let file_path = self.project_dir.join(&change.file_path);
            
            // Check file exists
            if change.original_content.is_none() && !file_path.exists() {
                result.errors.push(format!("File not found: {}", change.file_path));
                result.status = ValidationStatus::Error;
                continue;
            }

            result.affected_files.push(change.file_path.clone());

            // Syntax check
            if let Err(e) = self.check_syntax(&change.modified_content) {
                result.errors.push(format!("Syntax error in {}: {}", change.file_path, e));
                result.status = ValidationStatus::Error;
            }

            // Import validation (最優先)
            let import_issues = self.validate_imports(&change.modified_content, graph);
            if !import_issues.is_empty() {
                result.import_issues.extend(import_issues);
                result.status = ValidationStatus::Error;
            }

            // Detect new functions
            let new_funcs = self.detect_new_functions(&change.modified_content, &file_path)?;
            if !new_funcs.is_empty() {
                result.warnings.push(format!("New functions added in {}: {}", 
                    change.file_path, new_funcs.join(", ")));
                result.new_functions.extend(new_funcs);
            }

            // Detect modified functions
            let modified_funcs = self.detect_modified_functions(&change.modified_content, &file_path)?;
            if !modified_funcs.is_empty() {
                result.warnings.push(format!("Functions modified in {}: {}", 
                    change.file_path, modified_funcs.join(", ")));
                result.affected_functions.extend(modified_funcs);
            }
        }

        // Set status based on errors
        if !result.errors.is_empty() {
            result.status = ValidationStatus::Error;
        } else if !result.warnings.is_empty() {
            result.status = ValidationStatus::Warning;
        }

        Ok(result)
    }

    /// Check TypeScript syntax
    fn check_syntax(&self, content: &str) -> Result<()> {
        // Basic syntax checks
        let mut brace_count = 0;
        let mut paren_count = 0;
        let mut bracket_count = 0;

        for ch in content.chars() {
            match ch {
                '{' => brace_count += 1,
                '}' => brace_count -= 1,
                '(' => paren_count += 1,
                ')' => paren_count -= 1,
                '[' => bracket_count += 1,
                ']' => bracket_count -= 1,
                _ => {}
            }
        }

        if brace_count != 0 {
            return Err(anyhow!("Unmatched braces: {}", brace_count));
        }
        if paren_count != 0 {
            return Err(anyhow!("Unmatched parentheses: {}", paren_count));
        }
        if bracket_count != 0 {
            return Err(anyhow!("Unmatched brackets: {}", bracket_count));
        }

        Ok(())
    }

    /// Validate imports against dependency graph
    fn validate_imports(&self, content: &str, graph: &CodeGraph) -> Vec<ImportIssue> {
        let mut issues = Vec::new();

        // Extract import statements
        for (line_num, line) in content.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.starts_with("import ") || trimmed.starts_with("from ") {
                // Parse import path
                if let Some(import_path) = self.extract_import_path(trimmed) {
                    // Check if import exists in graph
                    if !self.import_exists_in_graph(&import_path, graph) {
                        issues.push(ImportIssue {
                            file: "".to_string(),
                            line: line_num + 1,
                            import_path: import_path.clone(),
                            issue: format!("Import '{}' not found in dependency graph", import_path),
                        });
                    }
                }
            }
        }

        issues
    }

    /// Extract import path from import statement
    fn extract_import_path(&self, line: &str) -> Option<String> {
        // Simple regex-like extraction
        if let Some(from_pos) = line.find("from ") {
            let after_from = &line[from_pos + 5..];
            if let Some(quote_start) = after_from.find(|c| c == '"' || c == '\'') {
                let quote_char = after_from.chars().nth(quote_start).unwrap();
                let path_start = quote_start + 1;
                if let Some(quote_end) = after_from[path_start..].find(quote_char) {
                    return Some(after_from[path_start..path_start + quote_end].to_string());
                }
            }
        }
        None
    }

    /// Check if import exists in dependency graph
    fn import_exists_in_graph(&self, import_path: &str, graph: &CodeGraph) -> bool {
        // Check if any node's file path matches the import
        // This is a simplified check - in production, you'd want more sophisticated resolution
        
        // Skip node_modules and built-in imports
        if import_path.starts_with('.') || import_path.contains("node_modules") {
            return true; // Assume relative imports are valid
        }

        // Check if any file in graph matches
        graph.nodes.values().any(|node| {
            node.file_path.to_string_lossy().contains(import_path)
        })
    }

    /// Detect new functions in modified content
    fn detect_new_functions(&self, content: &str, file_path: &Path) -> Result<Vec<String>> {
        let mut new_functions = Vec::new();

        // Simple function detection
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("function ") || trimmed.contains("const ") && trimmed.contains(" = ") {
                if let Some(func_name) = self.extract_function_name(trimmed) {
                    // Check if it exists in current file
                    if !self.function_exists_in_file(&func_name, file_path)? {
                        new_functions.push(func_name);
                    }
                }
            }
        }

        Ok(new_functions)
    }

    /// Detect modified functions
    fn detect_modified_functions(&self, content: &str, file_path: &Path) -> Result<Vec<String>> {
        let mut modified_functions = Vec::new();

        // Simple function detection
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("function ") || trimmed.contains("const ") && trimmed.contains(" = ") {
                if let Some(func_name) = self.extract_function_name(trimmed) {
                    // Check if it exists in current file
                    if self.function_exists_in_file(&func_name, file_path)? {
                        modified_functions.push(func_name);
                    }
                }
            }
        }

        Ok(modified_functions)
    }

    /// Extract function name from line
    fn extract_function_name(&self, line: &str) -> Option<String> {
        if line.starts_with("function ") {
            let after_func = &line[9..];
            if let Some(paren_pos) = after_func.find('(') {
                return Some(after_func[..paren_pos].trim().to_string());
            }
        } else if line.contains("const ") {
            if let Some(const_pos) = line.find("const ") {
                let after_const = &line[const_pos + 6..];
                if let Some(eq_pos) = after_const.find('=') {
                    return Some(after_const[..eq_pos].trim().to_string());
                }
            }
        }
        None
    }

    /// Check if function exists in file
    fn function_exists_in_file(&self, func_name: &str, file_path: &Path) -> Result<bool> {
        if !file_path.exists() {
            return Ok(false);
        }

        let content = std::fs::read_to_string(file_path)?;
        Ok(content.contains(&format!("function {}", func_name)) ||
           content.contains(&format!("const {} =", func_name)))
    }

    /// Preview changes (generate diffs)
    pub fn preview_changes(&self, changes: &[FileChange]) -> Result<PreviewResult> {
        let mut diffs = Vec::new();
        let mut total_additions = 0;
        let mut total_deletions = 0;

        for change in changes {
            let file_path = self.project_dir.join(&change.file_path);
            
            let original = if let Some(orig) = &change.original_content {
                orig.clone()
            } else if file_path.exists() {
                std::fs::read_to_string(&file_path)?
            } else {
                String::new()
            };

            let diff = self.generate_diff(&original, &change.modified_content);
            let (additions, deletions) = self.count_changes(&diff);

            total_additions += additions;
            total_deletions += deletions;

            diffs.push(FileDiff {
                file_path: change.file_path.clone(),
                diff,
                additions,
                deletions,
            });
        }

        Ok(PreviewResult {
            diffs,
            total_additions,
            total_deletions,
        })
    }

    /// Generate unified diff
    fn generate_diff(&self, original: &str, modified: &str) -> String {
        let mut diff = String::new();
        let orig_lines: Vec<&str> = original.lines().collect();
        let mod_lines: Vec<&str> = modified.lines().collect();

        diff.push_str("--- original\n");
        diff.push_str("+++ modified\n");

        let max_len = orig_lines.len().max(mod_lines.len());
        for i in 0..max_len {
            let orig_line = orig_lines.get(i).copied().unwrap_or("");
            let mod_line = mod_lines.get(i).copied().unwrap_or("");

            if orig_line != mod_line {
                if !orig_line.is_empty() {
                    diff.push_str(&format!("-{}\n", orig_line));
                }
                if !mod_line.is_empty() {
                    diff.push_str(&format!("+{}\n", mod_line));
                }
            }
        }

        diff
    }

    /// Count additions and deletions in diff
    fn count_changes(&self, diff: &str) -> (usize, usize) {
        let mut additions = 0;
        let mut deletions = 0;

        for line in diff.lines() {
            if line.starts_with('+') && !line.starts_with("+++") {
                additions += 1;
            } else if line.starts_with('-') && !line.starts_with("---") {
                deletions += 1;
            }
        }

        (additions, deletions)
    }

    /// Apply changes to files
    pub fn apply_changes(&self, changes: &[FileChange], create_backup: bool) -> Result<ApplyResult> {
        let mut applied_files = Vec::new();
        let mut failed_files = Vec::new();
        let backup_dir = if create_backup {
            Some(self.create_backup_dir()?)
        } else {
            None
        };

        for change in changes {
            let file_path = self.project_dir.join(&change.file_path);
            
            match self.apply_single_change(change, &file_path, backup_dir.as_ref()) {
                Ok(_) => {
                    applied_files.push(change.file_path.clone());
                }
                Err(e) => {
                    failed_files.push(FailedFile {
                        file: change.file_path.clone(),
                        error: e.to_string(),
                    });
                }
            }
        }

        Ok(ApplyResult {
            success: failed_files.is_empty(),
            applied_files,
            failed_files,
            backup_dir: backup_dir.map(|p| p.display().to_string()),
        })
    }

    /// Create backup directory
    fn create_backup_dir(&self) -> Result<PathBuf> {
        let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
        let backup_dir = self.project_dir.join(".index-chan").join("backups").join(timestamp.to_string());
        std::fs::create_dir_all(&backup_dir)?;
        Ok(backup_dir)
    }

    /// Apply single file change
    fn apply_single_change(&self, change: &FileChange, file_path: &Path, backup_dir: Option<&PathBuf>) -> Result<()> {
        // Create backup if requested
        if let Some(backup) = backup_dir {
            if file_path.exists() {
                let relative = file_path.strip_prefix(&self.project_dir)?;
                let backup_file = backup.join(relative);
                if let Some(parent) = backup_file.parent() {
                    std::fs::create_dir_all(parent)?;
                }
                std::fs::copy(file_path, backup_file)?;
            }
        }

        // Apply change
        if let (Some(start), Some(end)) = (change.start_line, change.end_line) {
            // Partial file update
            let original = std::fs::read_to_string(file_path)?;
            let mut lines: Vec<&str> = original.lines().collect();
            
            let new_lines: Vec<&str> = change.modified_content.lines().collect();
            lines.splice(start.saturating_sub(1)..end, new_lines);
            
            std::fs::write(file_path, lines.join("\n"))?;
        } else {
            // Full file replacement
            if let Some(parent) = file_path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::write(file_path, &change.modified_content)?;
        }

        Ok(())
    }
}

/// Parse context file format (for LLM responses)
pub fn parse_context_file(content: &str) -> Result<HashMap<String, FileChange>> {
    let mut changes = HashMap::new();
    let mut current_file: Option<String> = None;
    let mut current_content = String::new();
    let mut in_file = false;

    for line in content.lines() {
        if line.starts_with("// ===== FILE: ") {
            // Save previous file
            if let Some(file) = current_file.take() {
                changes.insert(file.clone(), FileChange {
                    file_path: file,
                    original_content: None,
                    modified_content: current_content.clone(),
                    start_line: None,
                    end_line: None,
                });
                current_content.clear();
            }

            // Extract new file path
            let file_path = line.trim_start_matches("// ===== FILE: ")
                .trim_end_matches(" =====")
                .trim();
            current_file = Some(file_path.to_string());
            in_file = true;
        } else if line.starts_with("// ===== END") {
            // Save last file
            if let Some(file) = current_file.take() {
                changes.insert(file.clone(), FileChange {
                    file_path: file,
                    original_content: None,
                    modified_content: current_content.clone(),
                    start_line: None,
                    end_line: None,
                });
            }
            in_file = false;
        } else if in_file && !line.starts_with("// Lines:") {
            current_content.push_str(line);
            current_content.push('\n');
        }
    }

    Ok(changes)
}
