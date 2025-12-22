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
            Language::Rust => matches!(kind, "function_item" | "function_signature_item"),
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
        let signature = self.extract_signature(node, source);

        Some(FunctionInfo {
            name,
            line_range: (start_line, end_line),
            is_exported,
            signature,
        })
    }

    /// Extract function signature (everything before the body block)
    fn extract_signature(&self, node: Node, source: &str) -> String {
        let node_text = node.utf8_text(source.as_bytes()).unwrap_or("");

        // Find the position of the opening brace for the body
        // For Rust: fn name(params) -> Type { ... }
        // For TS: function name(params): Type { ... }
        if let Some(brace_pos) = node_text.find('{') {
            let sig = node_text[..brace_pos].trim();
            // Clean up and return single line signature
            sig.lines()
                .map(|l| l.trim())
                .collect::<Vec<_>>()
                .join(" ")
        } else {
            // No body (e.g., function signature in trait)
            node_text.lines()
                .map(|l| l.trim())
                .collect::<Vec<_>>()
                .join(" ")
        }
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

        match self.language {
            Language::TypeScript => {
                if kind == "call_expression" {
                    if let Some(call_info) = self.extract_call_info(node, source) {
                        calls.push(call_info);
                    }
                }
            }
            Language::Rust => {
                // call_expression: 通常の関数/メソッド呼び出し
                if kind == "call_expression" {
                    let rust_calls = self.extract_rust_calls(node, source);
                    calls.extend(rust_calls);
                }
                // macro_invocation: マクロ呼び出し (println!, vec!, 等)
                if kind == "macro_invocation" {
                    if let Some(call_info) = self.extract_macro_call(node, source) {
                        calls.push(call_info);
                    }
                }
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
                                "log"
                                    | "error"
                                    | "warn"
                                    | "info"
                                    | "push"
                                    | "pop"
                                    | "map"
                                    | "filter"
                                    | "reduce"
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

    /// Rust呼び出し専用: メソッドチェーン、scoped_identifier を全て検出
    fn extract_rust_calls(&self, node: Node, source: &str) -> Vec<CallInfo> {
        let caller_line = node.start_position().row + 1;
        let mut calls = Vec::new();

        // call_expression の関数部分を取得
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            match child.kind() {
                // 単純な識別子: foo()
                "identifier" => {
                    if let Ok(callee_name) = child.utf8_text(source.as_bytes()) {
                        calls.push(CallInfo {
                            caller_line,
                            callee_name: callee_name.to_string(),
                        });
                    }
                }
                // フィールド式: self.graph.method() - チェーン全体を解析
                "field_expression" => {
                    let chain = self.extract_method_chain(child, source);
                    for method_name in chain {
                        calls.push(CallInfo {
                            caller_line,
                            callee_name: method_name,
                        });
                    }
                }
                // スコープ付き識別子: Vec::new(), Result::Ok()
                "scoped_identifier" => {
                    if let Ok(full_path) = child.utf8_text(source.as_bytes()) {
                        // フルパスから最後の識別子を取得
                        if let Some(name) = full_path.rsplit("::").next() {
                            calls.push(CallInfo {
                                caller_line,
                                callee_name: name.to_string(),
                            });
                        }
                        // 型名も記録 (Vec, Result など)
                        if let Some(type_name) = full_path.split("::").next() {
                            if type_name != "self" && type_name != "crate" && type_name != "super" {
                                calls.push(CallInfo {
                                    caller_line,
                                    callee_name: type_name.to_string(),
                                });
                            }
                        }
                    }
                }
                // ジェネリック関数: function::<T>()
                "generic_function" => {
                    // 内部の識別子を探す
                    let mut inner_cursor = child.walk();
                    for inner in child.children(&mut inner_cursor) {
                        if inner.kind() == "identifier" || inner.kind() == "scoped_identifier" {
                            if let Ok(name) = inner.utf8_text(source.as_bytes()) {
                                let final_name = name.rsplit("::").next().unwrap_or(name);
                                calls.push(CallInfo {
                                    caller_line,
                                    callee_name: final_name.to_string(),
                                });
                            }
                            break;
                        } else if inner.kind() == "field_expression" {
                            let chain = self.extract_method_chain(inner, source);
                            for method_name in chain {
                                calls.push(CallInfo {
                                    caller_line,
                                    callee_name: method_name,
                                });
                            }
                            break;
                        }
                    }
                }
                _ => {}
            }
        }

        calls
    }

    /// マクロ呼び出しを抽出: println!, vec! 等
    fn extract_macro_call(&self, node: Node, source: &str) -> Option<CallInfo> {
        let caller_line = node.start_position().row + 1;

        // マクロ名を取得 (最初の子ノードが識別子)
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "identifier" || child.kind() == "scoped_identifier" {
                if let Ok(name) = child.utf8_text(source.as_bytes()) {
                    // パス付きマクロ (std::println!) の場合は最後の部分を使用
                    let macro_name = name.rsplit("::").next().unwrap_or(name);
                    return Some(CallInfo {
                        caller_line,
                        callee_name: macro_name.to_string(),
                    });
                }
            }
        }
        None
    }

    /// メソッドチェーンを再帰的に解析: self.graph.traverse_from() → ["traverse_from", "graph"]
    fn extract_method_chain(&self, node: Node, source: &str) -> Vec<String> {
        let mut methods = Vec::new();

        if node.kind() == "field_expression" {
            // フィールド名 (メソッド名) を取得
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                if child.kind() == "field_identifier" {
                    if let Ok(name) = child.utf8_text(source.as_bytes()) {
                        methods.push(name.to_string());
                    }
                } else if child.kind() == "field_expression" || child.kind() == "call_expression" {
                    // 再帰的にチェーンを辿る
                    methods.extend(self.extract_method_chain(child, source));
                }
            }
        } else if node.kind() == "call_expression" {
            // チェーン内の call_expression
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                if child.kind() == "field_expression" {
                    methods.extend(self.extract_method_chain(child, source));
                }
            }
        }

        methods
    }
}

#[derive(Debug, Clone)]
pub struct FunctionInfo {
    pub name: String,
    pub line_range: (usize, usize),
    pub is_exported: bool,
    pub signature: String,
}

#[derive(Debug, Clone)]
pub struct ImportInfo {
    pub module_name: String,
    pub aliases: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct CallInfo {
    pub caller_line: usize,
    pub callee_name: String,
}

impl CodeParser {
    // ... existing extract_calls ...

    pub fn extract_imports(&self, tree: &tree_sitter::Tree, source: &str) -> Vec<ImportInfo> {
        let mut imports = Vec::new();
        let root_node = tree.root_node();
        self.traverse_imports(root_node, source, &mut imports);
        imports
    }

    fn traverse_imports(&self, node: Node, source: &str, imports: &mut Vec<ImportInfo>) {
        let kind = node.kind();
        let is_import = match self.language {
            Language::TypeScript => kind == "import_statement",
            Language::Rust => kind == "use_declaration",
        };

        if is_import {
            if let Some(info) = self.extract_import_info(node, source) {
                imports.push(info);
            }
        }

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.traverse_imports(child, source, imports);
        }
    }

    fn extract_import_info(&self, node: Node, source: &str) -> Option<ImportInfo> {
        match self.language {
            Language::TypeScript => {
                // import { Foo } from "./bar";
                // import * as Foo from "./bar";
                let mut module_name = String::new();
                let mut aliases = Vec::new();

                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    if child.kind() == "string" {
                        // Module source
                        if let Ok(text) = child.utf8_text(source.as_bytes()) {
                            module_name = text.trim_matches(|c| c == '\'' || c == '"').to_string();
                        }
                    } else if child.kind() == "import_clause" {
                        // Named imports or default import
                        let mut clause_cursor = child.walk();
                        for clause_child in child.children(&mut clause_cursor) {
                            if clause_child.kind() == "identifier" {
                                // Default import
                                if let Ok(text) = clause_child.utf8_text(source.as_bytes()) {
                                    aliases.push(text.to_string());
                                }
                            } else if clause_child.kind() == "named_imports" {
                                // { Foo, Bar as Baz }
                                let mut named_cursor = clause_child.walk();
                                for named_child in clause_child.children(&mut named_cursor) {
                                    if named_child.kind() == "import_specifier" {
                                        let mut spec_cursor = named_child.walk();
                                        for spec_child in named_child.children(&mut spec_cursor) {
                                            if spec_child.kind() == "identifier" {
                                                if let Ok(text) =
                                                    spec_child.utf8_text(source.as_bytes())
                                                {
                                                    aliases.push(text.to_string());
                                                    // For simple cases, we take both name and alias as separate entries to be safe
                                                    // Ideal would be to track alias -> original connection
                                                }
                                            }
                                        }
                                    }
                                }
                            } else if clause_child.kind() == "namespace_import" {
                                // * as Foo
                                let mut ns_cursor = clause_child.walk();
                                for ns_child in clause_child.children(&mut ns_cursor) {
                                    if ns_child.kind() == "identifier" {
                                        if let Ok(text) = ns_child.utf8_text(source.as_bytes()) {
                                            aliases.push(text.to_string());
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                if !module_name.is_empty() {
                    return Some(ImportInfo {
                        module_name,
                        aliases,
                    });
                }
                None
            }
            Language::Rust => {
                // use crate::foo::Bar;
                // use std::collections::{HashMap, HashSet};
                // Rust handling is complex, simplified version: try to get the last identifier
                let _cursor = node.walk();
                let _module_path = String::new();

                // Helper to extract text from generic nodes
                fn get_text(node: Node, source: &str) -> String {
                    node.utf8_text(source.as_bytes()).unwrap_or("").to_string()
                }

                // Simplified: just get the full text of the use declaration to parse manually or get last part
                // Real implementation requires recursive descent on use_tree

                // For now, let's just grab leaf identifiers
                // Not perfect but better than nothing
                let mut aliases = Vec::new();

                // Recursive function to find identifiers in use tree
                fn find_idents(node: Node, source: &str, idents: &mut Vec<String>) {
                    if node.kind() == "identifier" {
                        idents.push(get_text(node, source));
                    }
                    let mut cursor = node.walk();
                    for child in node.children(&mut cursor) {
                        find_idents(child, source, idents);
                    }
                }

                find_idents(node, source, &mut aliases);

                // Assuming the last ones are the imported items
                // This is a naive heuristic
                if !aliases.is_empty() {
                    // Module name is roughly the first part
                    let module_name = aliases.first().cloned().unwrap_or_default();
                    return Some(ImportInfo {
                        module_name,
                        aliases,
                    });
                }
                None
            }
        }
    }
}

// Legacy TypeScriptParser for backward compatibility
#[allow(dead_code)]
pub struct TypeScriptParser {
    inner: CodeParser,
}

#[allow(dead_code)]
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
