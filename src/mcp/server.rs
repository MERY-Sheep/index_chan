// MCP Server Implementation
// stdio-based JSON-RPC server

use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use anyhow::Result;
use serde_json::{json, Value};

use crate::mcp::protocol::*;
use crate::mcp::tools::get_tool_definitions;
use crate::mcp::context::{ContextGenerator, ContextMode};
use crate::mcp::changes::{ChangeManager, FileChange};
use crate::scanner::Scanner;
use crate::detector::detect_dead_code;

/// MCP Server
pub struct McpServer {
    project_dir: Option<PathBuf>,
    initialized: bool,
}

impl McpServer {
    pub fn new() -> Self {
        Self {
            project_dir: None,
            initialized: false,
        }
    }

    /// Run the server (stdio mode)
    pub fn run(&mut self) -> Result<()> {
        let stdin = std::io::stdin();
        let mut stdout = std::io::stdout();
        let reader = BufReader::new(stdin.lock());

        eprintln!("index-chan MCP server started");

        for line in reader.lines() {
            let line = line?;
            // Remove BOM and trim whitespace
            let line = line.trim_start_matches('\u{feff}').trim();
            if line.is_empty() {
                continue;
            }

            match serde_json::from_str::<JsonRpcRequest>(line) {
                Ok(request) => {
                    let response = self.handle_request(request);
                    let response_json = serde_json::to_string(&response)?;
                    writeln!(stdout, "{}", response_json)?;
                    stdout.flush()?;
                }
                Err(e) => {
                    let response = JsonRpcResponse::error(
                        None,
                        McpError::PARSE_ERROR,
                        &format!("Parse error: {}", e),
                    );
                    let response_json = serde_json::to_string(&response)?;
                    writeln!(stdout, "{}", response_json)?;
                    stdout.flush()?;
                }
            }
        }

        Ok(())
    }

    /// Handle a single request
    fn handle_request(&mut self, request: JsonRpcRequest) -> JsonRpcResponse {
        match request.method.as_str() {
            "initialize" => self.handle_initialize(request),
            "initialized" => JsonRpcResponse::success(request.id, json!({})),
            "tools/list" => self.handle_tools_list(request),
            "tools/call" => self.handle_tools_call(request),
            _ => JsonRpcResponse::error(
                request.id,
                McpError::METHOD_NOT_FOUND,
                &format!("Method not found: {}", request.method),
            ),
        }
    }

    /// Handle initialize request
    fn handle_initialize(&mut self, request: JsonRpcRequest) -> JsonRpcResponse {
        self.initialized = true;

        let result = InitializeResult {
            protocol_version: "2024-11-05".to_string(),
            capabilities: ServerCapabilities {
                tools: Some(ToolsCapability { list_changed: false }),
            },
            server_info: ServerInfo {
                name: "index-chan".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
            },
        };

        JsonRpcResponse::success(request.id, serde_json::to_value(result).unwrap())
    }

    /// Handle tools/list request
    fn handle_tools_list(&self, request: JsonRpcRequest) -> JsonRpcResponse {
        let tools = get_tool_definitions();
        JsonRpcResponse::success(request.id, json!({ "tools": tools }))
    }

    /// Handle tools/call request
    fn handle_tools_call(&mut self, request: JsonRpcRequest) -> JsonRpcResponse {
        let params: CallToolParams = match request.params {
            Some(p) => match serde_json::from_value(p) {
                Ok(params) => params,
                Err(e) => {
                    return JsonRpcResponse::error(
                        request.id,
                        McpError::INVALID_PARAMS,
                        &format!("Invalid params: {}", e),
                    );
                }
            },
            None => {
                return JsonRpcResponse::error(
                    request.id,
                    McpError::INVALID_PARAMS,
                    "Missing params",
                );
            }
        };

        let result = match params.name.as_str() {
            "scan" => self.tool_scan(params.arguments),
            "search" => self.tool_search(params.arguments),
            "stats" => self.tool_stats(params.arguments),
            "gather_context" => self.tool_gather_context(params.arguments),
            "get_dependencies" => self.tool_get_dependencies(params.arguments),
            "get_dependents" => self.tool_get_dependents(params.arguments),
            "validate_changes" => self.tool_validate_changes(params.arguments),
            "preview_changes" => self.tool_preview_changes(params.arguments),
            "apply_changes" => self.tool_apply_changes(params.arguments),
            _ => Err(format!("Unknown tool: {}", params.name)),
        };

        match result {
            Ok(content) => {
                let call_result = CallToolResult {
                    content: vec![ToolContent::Text { text: content }],
                    is_error: false,
                };
                JsonRpcResponse::success(request.id, serde_json::to_value(call_result).unwrap())
            }
            Err(e) => {
                let call_result = CallToolResult {
                    content: vec![ToolContent::Text { text: e }],
                    is_error: true,
                };
                JsonRpcResponse::success(request.id, serde_json::to_value(call_result).unwrap())
            }
        }
    }

    // ===== Tool Implementations =====

    /// Scan tool - dead code detection
    fn tool_scan(&self, args: Option<Value>) -> Result<String, String> {
        let directory = self.get_directory_arg(&args)?;

        let mut scanner = Scanner::new().map_err(|e| e.to_string())?;
        let graph = scanner.scan_directory(&directory).map_err(|e| e.to_string())?;

        let dead_code = detect_dead_code(&graph);

        let result = json!({
            "total_functions": graph.nodes.len(),
            "dead_code_count": dead_code.len(),
            "dead_code": dead_code.iter().map(|d| {
                json!({
                    "name": d.node.name,
                    "file": d.node.file_path.display().to_string(),
                    "line": d.node.line_range.0,
                    "safety": format!("{:?}", d.safety_level)
                })
            }).collect::<Vec<_>>()
        });

        Ok(serde_json::to_string_pretty(&result).unwrap())
    }

    /// Search tool - code search
    fn tool_search(&self, args: Option<Value>) -> Result<String, String> {
        let args = args.ok_or("Missing arguments")?;
        let query = args.get("query")
            .and_then(|v| v.as_str())
            .ok_or("Missing query parameter")?;
        let _top_k = args.get("top_k")
            .and_then(|v| v.as_u64())
            .unwrap_or(10) as usize;

        // TODO: Implement search using index
        Ok(format!("Search for '{}' - index not loaded. Use 'index-chan index <dir>' first.", query))
    }

    /// Stats tool - project statistics
    fn tool_stats(&self, args: Option<Value>) -> Result<String, String> {
        let directory = self.get_directory_arg(&args)?;

        let mut scanner = Scanner::new().map_err(|e| e.to_string())?;
        let graph = scanner.scan_directory(&directory).map_err(|e| e.to_string())?;

        let dead_code = detect_dead_code(&graph);

        let result = json!({
            "directory": directory.display().to_string(),
            "total_nodes": graph.nodes.len(),
            "total_edges": graph.edges.len(),
            "dead_code_count": dead_code.len(),
            "files": graph.nodes.values()
                .map(|n| n.file_path.display().to_string())
                .collect::<std::collections::HashSet<_>>()
                .len()
        });

        Ok(serde_json::to_string_pretty(&result).unwrap())
    }

    /// gather_context tool - collect code with dependencies
    fn tool_gather_context(&self, args: Option<Value>) -> Result<String, String> {
        let args = args.ok_or("Missing arguments")?;
        let directory = self.get_directory_from_value(&args)?;

        let entry_point = args.get("entry_point").and_then(|v| v.as_str());
        let query = args.get("query").and_then(|v| v.as_str());
        let depth = args.get("depth").and_then(|v| v.as_u64()).unwrap_or(2) as usize;
        let mode = match args.get("mode").and_then(|v| v.as_str()) {
            Some("skeleton") => ContextMode::Skeleton,
            _ => ContextMode::Full,
        };

        let generator = ContextGenerator::from_directory(&directory)
            .map_err(|e| e.to_string())?;

        let result = generator.gather_context(entry_point, query, depth, mode)
            .map_err(|e| e.to_string())?;

        Ok(result.content)
    }

    /// get_dependencies tool
    fn tool_get_dependencies(&self, args: Option<Value>) -> Result<String, String> {
        let args = args.ok_or("Missing arguments")?;
        let directory = self.get_directory_from_value(&args)?;
        let function_name = args.get("function_name")
            .and_then(|v| v.as_str())
            .ok_or("Missing function_name parameter")?;
        let depth = args.get("depth").and_then(|v| v.as_u64()).unwrap_or(1) as usize;

        let generator = ContextGenerator::from_directory(&directory)
            .map_err(|e| e.to_string())?;

        let deps = generator.get_dependencies(function_name, depth);

        let result = json!({
            "function": function_name,
            "dependencies": deps
        });

        Ok(serde_json::to_string_pretty(&result).unwrap())
    }

    /// get_dependents tool
    fn tool_get_dependents(&self, args: Option<Value>) -> Result<String, String> {
        let args = args.ok_or("Missing arguments")?;
        let directory = self.get_directory_from_value(&args)?;
        let function_name = args.get("function_name")
            .and_then(|v| v.as_str())
            .ok_or("Missing function_name parameter")?;
        let depth = args.get("depth").and_then(|v| v.as_u64()).unwrap_or(1) as usize;

        let generator = ContextGenerator::from_directory(&directory)
            .map_err(|e| e.to_string())?;

        let deps = generator.get_dependents(function_name, depth);

        let result = json!({
            "function": function_name,
            "dependents": deps
        });

        Ok(serde_json::to_string_pretty(&result).unwrap())
    }

    // ===== Helper Methods =====

    fn get_directory_arg(&self, args: &Option<Value>) -> Result<PathBuf, String> {
        let args = args.as_ref().ok_or("Missing arguments")?;
        self.get_directory_from_value(args)
    }

    fn get_directory_from_value(&self, args: &Value) -> Result<PathBuf, String> {
        let dir_str = args.get("directory")
            .and_then(|v| v.as_str())
            .ok_or("Missing directory parameter")?;
        
        let path = PathBuf::from(dir_str);
        if !path.exists() {
            return Err(format!("Directory not found: {}", dir_str));
        }
        Ok(path)
    }

    /// validate_changes tool
    fn tool_validate_changes(&self, args: Option<Value>) -> Result<String, String> {
        let args = args.ok_or("Missing arguments")?;
        let directory = self.get_directory_from_value(&args)?;

        let changes: Vec<FileChange> = args.get("changes")
            .and_then(|v| serde_json::from_value(v.clone()).ok())
            .ok_or("Invalid changes parameter")?;

        let mut manager = ChangeManager::from_directory(&directory)
            .map_err(|e| e.to_string())?;

        let result = manager.validate_changes(&changes)
            .map_err(|e| e.to_string())?;

        Ok(serde_json::to_string_pretty(&result).unwrap())
    }

    /// preview_changes tool
    fn tool_preview_changes(&self, args: Option<Value>) -> Result<String, String> {
        let args = args.ok_or("Missing arguments")?;
        let directory = self.get_directory_from_value(&args)?;

        let changes: Vec<FileChange> = args.get("changes")
            .and_then(|v| serde_json::from_value(v.clone()).ok())
            .ok_or("Invalid changes parameter")?;

        let manager = ChangeManager::from_directory(&directory)
            .map_err(|e| e.to_string())?;

        let result = manager.preview_changes(&changes)
            .map_err(|e| e.to_string())?;

        Ok(serde_json::to_string_pretty(&result).unwrap())
    }

    /// apply_changes tool
    fn tool_apply_changes(&self, args: Option<Value>) -> Result<String, String> {
        let args = args.ok_or("Missing arguments")?;
        let directory = self.get_directory_from_value(&args)?;

        let changes: Vec<FileChange> = args.get("changes")
            .and_then(|v| serde_json::from_value(v.clone()).ok())
            .ok_or("Invalid changes parameter")?;

        let create_backup = args.get("create_backup")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        let manager = ChangeManager::from_directory(&directory)
            .map_err(|e| e.to_string())?;

        let result = manager.apply_changes(&changes, create_backup)
            .map_err(|e| e.to_string())?;

        Ok(serde_json::to_string_pretty(&result).unwrap())
    }
}

impl Default for McpServer {
    fn default() -> Self {
        Self::new()
    }
}
