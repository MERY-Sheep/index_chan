use anyhow::{Context, Result};
use std::path::Path;
use tree_sitter::{Node, Parser};

// Language enum for multi-language support
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Language {
    TypeScript,
    Rust,
}

impl Language {
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext {
            "ts" | "tsx" => Some(Language::TypeScript),
            "rs" => Some(Language::Rust),
            _ => None,
        }
    }
}

// Unified parser for multiple languages
pub struct CodeParser {
    parser: Parser,
    language: Language,
}

impl CodeParser {
    pub fn new(language: Language) -> Result<Self> {
        let mut parser = Parser::new();
        
        let tree_sitter_lang = match language {
            Language::TypeScript => tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into(),
            Language::Rust => tree_sitter_rust::LANGUAGE.into(),
        };
        
        parser
            .set_language(&tree_sitter_lang)
            .context("Failed to set language")?;

        Ok(Self { parser, language })
    }

    pub fn parse_file(&mut self, path: &Path) -> Result<tree_sitter::Tree> {
        let source_code = std::fs::read_to_string(path)
            .context(format!("Failed to read file: {}", path.display()))?;

        let tree = self
            .parser
            .parse(&source_code, None)
            .context("Failed to parse file")?;

        Ok(tree)
    }

    pub fn extract_functions(&self, tree: &tree_sitter::Tree, source: &str) -> Vec<FunctionInfo> {
        let mut functions = Vec::new();
        let root_node = tree.root_node();

        self.traverse_node(root_node, source, &mut functions);

        functions
    }

    fn traverse_node(&self, node: Node, source: &str, functions: &mut Vec<FunctionInfo>) {
        let kind = node.kind();

        // Check if this is a function declaration based on language
        let is_function = match self.language {
            Language::TypeScript => matches!(
                kind,
                "function_declaration" | "method_definition" | "arrow_function" | "function"
            ),
            Language::Rust => matches!(
                kind,
                "function_item" | "function_signature_item"
            ),
        };

        if is_function {
            if let Some(info) = self.extract_function_info(node, source) {
                functions.push(info);
            }
        }

        // Recursively traverse children
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.traverse_node(child, source, functions);
        }
    }

    fn extract_function_info(&self, node: Node, source: &str) -> Option<FunctionInfo> {
        let name = self.get_function_name(node, source)?;
        let start_line = node.start_position().row + 1;
        let end_line = node.end_position().row + 1;
        let is_exported = self.is_exported(node);

        Some(FunctionInfo {
            name,
            line_range: (start_line, end_line),
            is_exported,
        })
    }

    fn get_function_name(&self, node: Node, source: &str) -> Option<String> {
        let kind = node.kind();

        match self.language {
            Language::TypeScript => {
                // For method definitions, look for property_identifier
                if kind == "method_definition" {
                    let mut cursor = node.walk();
                    for child in node.children(&mut cursor) {
                        if child.kind() == "property_identifier" {
                            return Some(child.utf8_text(source.as_bytes()).ok()?.to_string());
                        }
                    }
                }

                // For function declarations, look for identifier
                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    if child.kind() == "identifier" || child.kind() == "property_identifier" {
                        return Some(child.utf8_text(source.as_bytes()).ok()?.to_string());
                    }
                }
                None
            }
            Language::Rust => {
                // For Rust, look for identifier in function_item
                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    if child.kind() == "identifier" {
                        return Some(child.utf8_text(source.as_bytes()).ok()?.to_string());
                    }
                }
                None
            }
        }
    }

    fn is_exported(&self, node: Node) -> bool {
        match self.language {
            Language::TypeScript => {
                // Check if parent or grandparent is an export statement
                let mut current = node;
                for _ in 0..3 {
                    if let Some(parent) = current.parent() {
                        if parent.kind().contains("export") {
                            return true;
                        }
                        current = parent;
                    } else {
                        break;
                    }
                }
                false
            }
            Language::Rust => {
                // Check for pub keyword
                let mut current = node;
                for _ in 0..3 {
                    if let Some(parent) = current.parent() {
                        let mut cursor = parent.walk();
                        for child in parent.children(&mut cursor) {
                            if child.kind() == "visibility_modifier" {
                                return true;
                            }
                        }
                        current = parent;
                    } else {
                        break;
                    }
                }
                false
            }
        }
    }

    pub fn extract_calls(&self, tree: &tree_sitter::Tree, source: &str) -> Vec<CallInfo> {
        let mut calls = Vec::new();
        let root_node = tree.root_node();

        self.traverse_calls(root_node, source, &mut calls);

        calls
    }

    fn traverse_calls(&self, node: Node, source: &str, calls: &mut Vec<CallInfo>) {
        let kind = node.kind();

        // Check if this is a function call based on language
        let is_call = match self.language {
            Language::TypeScript => kind == "call_expression",
            Language::Rust => kind == "call_expression",
        };

        if is_call {
            if let Some(call_info) = self.extract_call_info(node, source) {
                calls.push(call_info);
            }
        }

        // Recursively traverse children
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.traverse_calls(child, source, calls);
        }
    }

    fn extract_call_info(&self, node: Node, source: &str) -> Option<CallInfo> {
        let caller_line = node.start_position().row + 1;

        match self.language {
            Language::TypeScript => {
                // Get the function being called
                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    if child.kind() == "identifier" {
                        if let Ok(callee_name) = child.utf8_text(source.as_bytes()) {
                            return Some(CallInfo {
                                caller_line,
                                callee_name: callee_name.to_string(),
                            });
                        }
                    } else if child.kind() == "member_expression" {
                        // For member expressions like obj.method(), get just the method name
                        if let Ok(full_name) = child.utf8_text(source.as_bytes()) {
                            let name = full_name.split('.').last().unwrap_or(full_name);
                            // Skip common built-in methods
                            if !matches!(
                                name,
                                "log" | "error" | "warn" | "info" | "push" | "pop" | "map" | "filter" | "reduce"
                            ) {
                                return Some(CallInfo {
                                    caller_line,
                                    callee_name: name.to_string(),
                                });
                            }
                        }
                    }
                }
                None
            }
            Language::Rust => {
                // Get the function being called
                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    if child.kind() == "identifier" {
                        if let Ok(callee_name) = child.utf8_text(source.as_bytes()) {
                            return Some(CallInfo {
                                caller_line,
                                callee_name: callee_name.to_string(),
                            });
                        }
                    } else if child.kind() == "field_expression" {
                        // For field expressions like obj.method()
                        if let Ok(full_name) = child.utf8_text(source.as_bytes()) {
                            let name = full_name.split('.').last().unwrap_or(full_name);
                            return Some(CallInfo {
                                caller_line,
                                callee_name: name.to_string(),
                            });
                        }
                    }
                }
                None
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct FunctionInfo {
    pub name: String,
    pub line_range: (usize, usize),
    pub is_exported: bool,
}

#[derive(Debug, Clone)]
pub struct CallInfo {
    pub caller_line: usize,
    pub callee_name: String,
}

// Legacy TypeScriptParser for backward compatibility
pub struct TypeScriptParser {
    inner: CodeParser,
}

impl TypeScriptParser {
    pub fn new() -> Result<Self> {
        Ok(Self {
            inner: CodeParser::new(Language::TypeScript)?,
        })
    }

    pub fn parse_file(&mut self, path: &Path) -> Result<tree_sitter::Tree> {
        self.inner.parse_file(path)
    }

    pub fn extract_functions(&self, tree: &tree_sitter::Tree, source: &str) -> Vec<FunctionInfo> {
        self.inner.extract_functions(tree, source)
    }

    pub fn extract_calls(&self, tree: &tree_sitter::Tree, source: &str) -> Vec<CallInfo> {
        self.inner.extract_calls(tree, source)
    }
}
