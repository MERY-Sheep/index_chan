use crate::graph::CodeNode;
use std::path::Path;

pub struct ContextCollector {}

impl ContextCollector {
    pub fn new(_project_path: &Path) -> Self {
        Self {}
    }

    pub fn collect_context(&self, node: &CodeNode) -> String {
        let mut context = String::new();

        // Add file information
        context.push_str(&format!("ファイル: {}\n", node.file_path.display()));
        context.push_str(&format!(
            "行: {}-{}\n",
            node.line_range.0, node.line_range.1
        ));
        context.push_str(&format!("エクスポート: {}\n", node.is_exported));
        context.push_str("\n");

        context
    }
}
