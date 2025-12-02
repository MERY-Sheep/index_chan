use anyhow::Result;
use git2::Repository;
use std::path::Path;
use crate::graph::CodeNode;

pub struct ContextCollector {
    repo: Option<Repository>,
}

impl ContextCollector {
    pub fn new(project_path: &Path) -> Self {
        let repo = Repository::discover(project_path).ok();
        Self { repo }
    }
    
    pub fn collect_context(&self, node: &CodeNode) -> String {
        let mut context = String::new();
        
        // Add file information
        context.push_str(&format!("ファイル: {}\n", node.file_path.display()));
        context.push_str(&format!("行: {}-{}\n", node.line_range.0, node.line_range.1));
        context.push_str(&format!("エクスポート: {}\n", node.is_exported));
        context.push_str("\n");
        
        // Add git history if available
        if let Some(git_info) = self.get_git_history(&node.file_path) {
            context.push_str("Git履歴:\n");
            context.push_str(&git_info);
            context.push_str("\n");
        }
        
        context
    }
    
    fn get_git_history(&self, file_path: &Path) -> Option<String> {
        let repo = self.repo.as_ref()?;
        
        // Get last commit for this file
        let mut revwalk = repo.revwalk().ok()?;
        revwalk.push_head().ok()?;
        
        for oid in revwalk.take(5) {
            let oid = oid.ok()?;
            let commit = repo.find_commit(oid).ok()?;
            
            let message = commit.message().unwrap_or("");
            let time = commit.time();
            
            // Simple heuristic: check if commit message mentions the file
            if message.contains(&file_path.file_name()?.to_string_lossy().to_string()) {
                return Some(format!(
                    "最終更新: {} 秒前\nコミットメッセージ: {}",
                    time.seconds(),
                    message
                ));
            }
        }
        
        Some("Git履歴なし".to_string())
    }
}
