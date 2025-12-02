use crate::detector::DeadCode;
use anyhow::Result;
use std::path::Path;

pub struct Annotator {
    dry_run: bool,
    llm_analyses: Option<std::collections::HashMap<String, LLMAnalysisData>>,
}

#[derive(Clone)]
pub struct LLMAnalysisData {
    pub should_delete: bool,
    pub confidence: f32,
    pub reason: String,
    #[allow(dead_code)]
    pub category: String,
}

impl Annotator {
    pub fn new(dry_run: bool) -> Self {
        Self {
            dry_run,
            llm_analyses: None,
        }
    }

    pub fn with_llm_analyses(
        mut self,
        analyses: std::collections::HashMap<String, LLMAnalysisData>,
    ) -> Self {
        self.llm_analyses = Some(analyses);
        self
    }

    pub fn annotate(&self, dead_code: &[DeadCode]) -> Result<AnnotationResult> {
        let mut annotated_count = 0;
        let mut skipped_count = 0;

        for code in dead_code {
            // Only annotate code that should be kept
            if self.should_annotate(code) {
                if !self.dry_run {
                    self.add_annotation(code)?;
                }
                annotated_count += 1;
            } else {
                skipped_count += 1;
            }
        }

        Ok(AnnotationResult {
            annotated_count,
            skipped_count,
        })
    }

    fn should_annotate(&self, code: &DeadCode) -> bool {
        // If we have LLM analysis, use it
        if let Some(analyses) = &self.llm_analyses {
            let key = format!("{}:{}", code.node.file_path.display(), code.node.name);
            if let Some(analysis) = analyses.get(&key) {
                // Annotate if LLM says to keep it (not delete)
                // and confidence is high enough
                return !analysis.should_delete && analysis.confidence >= 0.75;
            }
        }

        // Fallback to rule-based
        // Annotate if it's probably safe or needs review
        // (i.e., might be used in the future)
        matches!(
            code.safety_level,
            crate::detector::SafetyLevel::ProbablySafe | crate::detector::SafetyLevel::NeedsReview
        )
    }

    fn add_annotation(&self, code: &DeadCode) -> Result<()> {
        let file_path = &code.node.file_path;
        let content = std::fs::read_to_string(file_path)?;
        let lines: Vec<&str> = content.lines().collect();

        let start_line = code.node.line_range.0 - 1; // 0-indexed

        // Determine annotation based on file extension
        let annotation = self.get_annotation(file_path, &code.reason);

        // Insert annotation before the function
        let mut new_lines = lines[..start_line].to_vec();
        new_lines.push(&annotation);
        new_lines.extend_from_slice(&lines[start_line..]);

        let new_content = new_lines.join("\n");
        std::fs::write(file_path, new_content)?;

        Ok(())
    }

    fn get_annotation(&self, file_path: &Path, reason: &str) -> String {
        let ext = file_path.extension().and_then(|s| s.to_str());

        // Get LLM reason if available
        let annotation_reason = if let Some(analyses) = &self.llm_analyses {
            // Try to find matching analysis
            analyses
                .values()
                .find(|a| !a.should_delete)
                .map(|a| a.reason.clone())
                .unwrap_or_else(|| reason.to_string())
        } else {
            reason.to_string()
        };

        match ext {
            Some("rs") => {
                format!("#[allow(dead_code)] // index-chan: {}", annotation_reason)
            }
            Some("ts") | Some("tsx") | Some("js") | Some("jsx") => {
                format!("// @ts-ignore - index-chan: {}", annotation_reason)
            }
            Some("py") => {
                format!("# noqa: F841 - index-chan: {}", annotation_reason)
            }
            _ => {
                format!("// index-chan: {}", annotation_reason)
            }
        }
    }
}

pub struct AnnotationResult {
    pub annotated_count: usize,
    pub skipped_count: usize,
}
