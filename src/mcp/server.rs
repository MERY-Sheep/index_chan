// MCP Server Implementation
// stdio-based JSON-RPC server

use anyhow::Result;
use serde_json::{json, Value};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;

use crate::detector::detect_dead_code;
use crate::graph::CodeGraph;
use crate::mcp::changes::{ChangeManager, FileChange};
use crate::mcp::context::{ContextFormat, ContextGenerator, ContextMode, parse_llm_edits};
use crate::mcp::protocol::*;
use crate::mcp::tools::get_tool_definitions;
use crate::scanner::Scanner;
use std::path::Path;

/// MCP Server with graph caching
pub struct McpServer {
    project_dir: Option<PathBuf>,
    initialized: bool,
    /// Cached graph to avoid re-scanning on each tool call
    graph_cache: Option<CodeGraph>,
}

impl McpServer {
    pub fn new(project_dir: Option<PathBuf>) -> Self {
        Self {
            project_dir,
            initialized: false,
            graph_cache: None,
        }
    }

    /// Get or load graph with caching
    /// Returns a reference to the cached graph, scanning if necessary
    fn get_or_load_graph(&mut self, dir: &Path) -> Result<&CodeGraph, String> {
        let dir_buf = dir.to_path_buf();

        // Check if cache is valid for this directory
        let cache_valid = self.graph_cache.is_some()
            && self.project_dir.as_ref() == Some(&dir_buf);

        if !cache_valid {
            eprintln!("📊 Loading graph for: {}", dir.display());
            let mut scanner = Scanner::new().map_err(|e| e.to_string())?;
            let graph = scanner.scan_directory(dir).map_err(|e| e.to_string())?;
            self.graph_cache = Some(graph);
            self.project_dir = Some(dir_buf);
        } else {
            eprintln!("📊 Using cached graph");
        }

        Ok(self.graph_cache.as_ref().unwrap())
    }

    /// Invalidate the graph cache (call after file modifications)
    #[allow(dead_code)]
    fn invalidate_cache(&mut self) {
        self.graph_cache = None;
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
                tools: Some(ToolsCapability {
                    list_changed: false,
                }),
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
            "search_with_graph" => self.tool_search_with_graph(params.arguments),
            "parse_llm_edits" => self.tool_parse_llm_edits(params.arguments),
            "get_call_chain" => self.tool_get_call_chain(params.arguments),
            "get_call_tree" => self.tool_get_call_tree(params.arguments),
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
        let graph = scanner
            .scan_directory(&directory)
            .map_err(|e| e.to_string())?;

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

        // Save to DB (feature="db")
        #[cfg(feature = "db")]
        {
            let db_path = directory.join(".index-chan").join("graph.db");
            // Clone graph for background thread
            let graph_clone = graph.clone();

            std::thread::spawn(move || {
                let rt = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .unwrap();

                rt.block_on(async {
                    use crate::database::GraphDB;
                    // Ensure directory exists
                    if let Some(parent) = db_path.parent() {
                        let _ = std::fs::create_dir_all(parent);
                    }
                    if let Ok(db) = GraphDB::new(&db_path).await {
                        if let Err(e) = db.save_graph(&graph_clone).await {
                            eprintln!("Failed to save graph to DB: {}", e);
                        }
                    }
                });
            })
            .join()
            .map_err(|_| "Failed to join db thread".to_string())?;
        }

        Ok(serde_json::to_string_pretty(&result).unwrap())
    }

    /// Search tool - code search (redirects to search_with_graph)
    fn tool_search(&mut self, args: Option<Value>) -> Result<String, String> {
        // search ツールは search_with_graph にリダイレクト (DB依存を削除)
        // デフォルト: semantic=false, graph_depth=1
        let mut new_args = args.unwrap_or(json!({}));
        if let Some(obj) = new_args.as_object_mut() {
            // graph_depth が未設定なら 1 に設定
            if !obj.contains_key("graph_depth") {
                obj.insert("graph_depth".to_string(), json!(1));
            }
            // semantic が未設定なら false に設定
            if !obj.contains_key("semantic") {
                obj.insert("semantic".to_string(), json!(false));
            }
        }
        self.tool_search_with_graph(Some(new_args))
    }

    /// Stats tool - project statistics
    /// Enhanced with semantic relation type clustering (Concept Transformer Phase 2)
    fn tool_stats(&mut self, args: Option<Value>) -> Result<String, String> {
        use crate::graph::SemanticRelationType;
        use std::collections::HashMap;

        let directory = self.get_directory_arg(&args)?;

        // Use cached graph
        let graph = self.get_or_load_graph(&directory)?.clone();

        let dead_code = detect_dead_code(&graph);

        // Count edge types and semantic relation types
        let mut edge_type_counts: HashMap<String, usize> = HashMap::new();
        let mut semantic_type_counts: HashMap<String, usize> = HashMap::new();

        for edge in &graph.edges {
            let edge_type_name = format!("{:?}", edge.edge_type);
            *edge_type_counts.entry(edge_type_name).or_insert(0) += 1;

            let semantic_type = edge.edge_type.to_semantic();
            let semantic_name = format!("{:?}", semantic_type);
            *semantic_type_counts.entry(semantic_name).or_insert(0) += 1;
        }

        // Calculate semantic relation summary
        let semantic_summary: Vec<_> = semantic_type_counts.iter()
            .map(|(name, count)| {
                let weight = match name.as_str() {
                    "IsA" => SemanticRelationType::IsA.traversal_weight(),
                    "Has" => SemanticRelationType::Has.traversal_weight(),
                    "Transforms" => SemanticRelationType::Transforms.traversal_weight(),
                    "Uses" => SemanticRelationType::Uses.traversal_weight(),
                    "Creates" => SemanticRelationType::Creates.traversal_weight(),
                    _ => 0.5,
                };
                json!({
                    "type": name,
                    "count": count,
                    "weight": weight
                })
            })
            .collect();

        let result = json!({
            "directory": directory.display().to_string(),
            "total_nodes": graph.nodes.len(),
            "total_edges": graph.edges.len(),
            "dead_code_count": dead_code.len(),
            "files": graph.nodes.values()
                .map(|n| n.file_path.display().to_string())
                .collect::<std::collections::HashSet<_>>()
                .len(),
            "edge_types": edge_type_counts,
            "semantic_relations": semantic_summary
        });

        Ok(serde_json::to_string_pretty(&result).unwrap())
    }

    /// gather_context tool - collect code with dependencies
    fn tool_gather_context(&mut self, args: Option<Value>) -> Result<String, String> {
        let args = args.ok_or("Missing arguments")?;
        let directory = self.get_directory_from_value(&args)?;

        let entry_point = args.get("entry_point").and_then(|v| v.as_str());
        let query = args.get("query").and_then(|v| v.as_str());
        let depth = args.get("depth").and_then(|v| v.as_u64()).unwrap_or(2) as usize;
        let mode = match args.get("mode").and_then(|v| v.as_str()) {
            Some("skeleton") => ContextMode::Skeleton,
            _ => ContextMode::Full,
        };
        let format = match args.get("format").and_then(|v| v.as_str()) {
            Some("llm_edit") => ContextFormat::LlmEdit,
            _ => ContextFormat::Standard,
        };

        // Use cached graph
        let graph = self.get_or_load_graph(&directory)?.clone();
        let generator = ContextGenerator::from_graph(graph);

        let result = generator
            .gather_context(entry_point, query, depth, mode, format)
            .map_err(|e| e.to_string())?;

        // For llm_edit format, return content directly without quality header
        if format == ContextFormat::LlmEdit {
            return Ok(result.content);
        }

        // Build header with quality info (for standard format)
        // Enhanced with Concept Transformer Phase 9b metrics
        let mut content_with_quality = format!(
            "// ===== QUALITY METRICS =====\n\
             // Estimated tokens: {}\n\
             // S/N ratio: {:.2}\n\
             // Concept density: {:.2}\n\
             // Dependencies: {}\n\
             // Entry point ratio: {:.1}%\n\
             // Quality: {}\n",
            result.quality.estimated_tokens,
            result.quality.sn_ratio,
            result.quality.concept_density,
            result.quality.dependency_count,
            result.quality.entry_point_ratio * 100.0,
            result.quality.quality_level
        );
        if result.quality.context_explosion_warning {
            content_with_quality.push_str("// ⚠️ WARNING: Context explosion detected!\n");
        }
        if let Some(ref rec) = result.quality.recommendation {
            content_with_quality.push_str(&format!("// Recommendation: {}\n", rec));
        }
        content_with_quality.push_str("// =============================\n\n");
        content_with_quality.push_str(&result.content);

        Ok(content_with_quality)
    }

    /// get_dependencies tool
    fn tool_get_dependencies(&mut self, args: Option<Value>) -> Result<String, String> {
        let args = args.ok_or("Missing arguments")?;
        let directory = self.get_directory_from_value(&args)?;
        let function_name = args
            .get("function_name")
            .and_then(|v| v.as_str())
            .ok_or("Missing function_name parameter")?;
        let depth = args.get("depth").and_then(|v| v.as_u64()).unwrap_or(1) as usize;

        // Use cached graph
        let graph = self.get_or_load_graph(&directory)?.clone();
        let generator = ContextGenerator::from_graph(graph);

        let deps = generator.get_dependencies(function_name, depth);

        let result = json!({
            "function": function_name,
            "dependencies": deps
        });

        Ok(serde_json::to_string_pretty(&result).unwrap())
    }

    /// get_dependents tool
    fn tool_get_dependents(&mut self, args: Option<Value>) -> Result<String, String> {
        let args = args.ok_or("Missing arguments")?;
        let directory = self.get_directory_from_value(&args)?;
        let function_name = args
            .get("function_name")
            .and_then(|v| v.as_str())
            .ok_or("Missing function_name parameter")?;
        let depth = args.get("depth").and_then(|v| v.as_u64()).unwrap_or(1) as usize;

        // Use cached graph
        let graph = self.get_or_load_graph(&directory)?.clone();
        let generator = ContextGenerator::from_graph(graph);

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
        let dir_str = args
            .get("directory")
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

        let changes: Vec<FileChange> = args
            .get("changes")
            .and_then(|v| serde_json::from_value(v.clone()).ok())
            .ok_or("Invalid changes parameter")?;

        let mut manager = ChangeManager::from_directory(&directory).map_err(|e| e.to_string())?;

        let result = manager
            .validate_changes(&changes)
            .map_err(|e| e.to_string())?;

        Ok(serde_json::to_string_pretty(&result).unwrap())
    }

    /// preview_changes tool
    fn tool_preview_changes(&self, args: Option<Value>) -> Result<String, String> {
        let args = args.ok_or("Missing arguments")?;
        let directory = self.get_directory_from_value(&args)?;

        let changes: Vec<FileChange> = args
            .get("changes")
            .and_then(|v| serde_json::from_value(v.clone()).ok())
            .ok_or("Invalid changes parameter")?;

        let manager = ChangeManager::from_directory(&directory).map_err(|e| e.to_string())?;

        let result = manager
            .preview_changes(&changes)
            .map_err(|e| e.to_string())?;

        Ok(serde_json::to_string_pretty(&result).unwrap())
    }

    /// apply_changes tool
    fn tool_apply_changes(&self, args: Option<Value>) -> Result<String, String> {
        let args = args.ok_or("Missing arguments")?;
        let directory = self.get_directory_from_value(&args)?;

        let changes: Vec<FileChange> = args
            .get("changes")
            .and_then(|v| serde_json::from_value(v.clone()).ok())
            .ok_or("Invalid changes parameter")?;

        let create_backup = args
            .get("create_backup")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        let manager = ChangeManager::from_directory(&directory).map_err(|e| e.to_string())?;

        let result = manager
            .apply_changes(&changes, create_backup)
            .map_err(|e| e.to_string())?;

        Ok(serde_json::to_string_pretty(&result).unwrap())
    }

    /// search_with_graph tool (Phase 7 GraphRAG)
    fn tool_search_with_graph(&mut self, args: Option<Value>) -> Result<String, String> {
        let args = args.ok_or("Missing arguments")?;
        let directory = self.get_directory_from_value(&args)?;
        let query = args
            .get("query")
            .and_then(|v| v.as_str())
            .ok_or("Missing query parameter")?;
        let top_k = args.get("top_k").and_then(|v| v.as_u64()).unwrap_or(3) as usize;
        let graph_depth = args
            .get("graph_depth")
            .and_then(|v| v.as_u64())
            .unwrap_or(2) as usize;
        let use_semantic = args
            .get("semantic")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let filter_generic = args
            .get("filter_generic")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        // Use cached graph
        let graph = self.get_or_load_graph(&directory)?.clone();

        // Use GraphSearcher
        use crate::search::GraphSearcher;
        let searcher = GraphSearcher::new(graph.clone());

        let results = if use_semantic && cfg!(feature = "semantic-search") {
            // Try to use semantic search with cached embeddings
            #[cfg(feature = "semantic-search")]
            {
                use crate::embedding_cache::EmbeddingCache;
                match EmbeddingCache::get_or_create(&graph, &directory) {
                    Ok(cache) => {
                        eprintln!(
                            "Using semantic search with {} embeddings",
                            cache.embeddings.len()
                        );
                        searcher.search_semantic(query, &cache.embeddings, top_k, graph_depth)
                    }
                    Err(e) => {
                        eprintln!(
                            "Failed to load embeddings, falling back to name match: {}",
                            e
                        );
                        searcher.search_with_graph_filtered(query, top_k, graph_depth, filter_generic)
                    }
                }
            }
            #[cfg(not(feature = "semantic-search"))]
            {
                eprintln!("semantic-search feature not enabled, using name match");
                searcher.search_with_graph_filtered(query, top_k, graph_depth, filter_generic)
            }
        } else {
            // Use name-based matching with filtering
            searcher.search_with_graph_filtered(query, top_k, graph_depth, filter_generic)
        };

        let result = json!({
            "query": query,
            "graph_depth": graph_depth,
            "semantic": use_semantic,
            "results": results.iter().map(|r| {
                json!({
                    "name": r.metadata.function_name,
                    "file": r.metadata.file_path.display().to_string(),
                    "score": r.score,
                    "depth": r.depth,
                    "line_range": [r.metadata.start_line, r.metadata.end_line],
                    "match_type": format!("{:?}", r.match_type),
                    "explanation": {
                        "trace": r.explanation.trace.iter().map(|t| {
                            json!({
                                "node": t.node,
                                "node_type": t.node_type,
                                "reason": t.reason,
                                "edge": t.edge,
                                "direction": t.direction
                            })
                        }).collect::<Vec<_>>(),
                        "score_details": {
                            "base_score": r.explanation.score_details.base_score,
                            "decay_factor": r.explanation.score_details.decay_factor,
                            "depth": r.explanation.score_details.depth
                        }
                    }
                })
            }).collect::<Vec<_>>()
        });

        Ok(serde_json::to_string_pretty(&result).unwrap())
    }

    /// parse_llm_edits tool - Parse LLM output into changes array
    fn tool_parse_llm_edits(&self, args: Option<Value>) -> Result<String, String> {
        let args = args.ok_or("Missing arguments")?;
        let content = args
            .get("content")
            .and_then(|v| v.as_str())
            .ok_or("Missing content parameter")?;

        let changes = parse_llm_edits(content).map_err(|e| e.to_string())?;

        let result = json!({
            "changes": changes.iter().map(|c| {
                json!({
                    "file_path": c.file_path,
                    "start_line": c.start_line,
                    "end_line": c.end_line,
                    "modified_content": c.modified_content
                })
            }).collect::<Vec<_>>()
        });

        Ok(serde_json::to_string_pretty(&result).unwrap())
    }

    /// get_call_chain tool - Find path between two functions
    fn tool_get_call_chain(&mut self, args: Option<Value>) -> Result<String, String> {
        let args = args.ok_or("Missing arguments")?;
        let directory = self.get_directory_from_value(&args)?;
        let from = args
            .get("from")
            .and_then(|v| v.as_str())
            .ok_or("Missing 'from' parameter")?;
        let to = args
            .get("to")
            .and_then(|v| v.as_str())
            .ok_or("Missing 'to' parameter")?;
        let max_depth = args.get("max_depth").and_then(|v| v.as_u64()).unwrap_or(10) as usize;

        // Use cached graph
        let graph = self.get_or_load_graph(&directory)?.clone();
        let generator = ContextGenerator::from_graph(graph);
        let result = generator.get_call_chain(from, to, max_depth);

        // Format output with visual representation
        let visual = if result.found {
            result.chain.iter()
                .enumerate()
                .map(|(i, step)| {
                    let indent = "  ".repeat(i);
                    format!("{}→ {} ({}:{})", indent, step.name, step.file_path, step.line)
                })
                .collect::<Vec<_>>()
                .join("\n")
        } else {
            format!("No path found from '{}' to '{}'", from, to)
        };

        let output = json!({
            "from": result.from,
            "to": result.to,
            "found": result.found,
            "chain": result.chain,
            "visual": visual
        });

        Ok(serde_json::to_string_pretty(&output).unwrap())
    }

    /// get_call_tree tool - Visualize call hierarchy
    fn tool_get_call_tree(&mut self, args: Option<Value>) -> Result<String, String> {
        let args = args.ok_or("Missing arguments")?;
        let directory = self.get_directory_from_value(&args)?;
        let function_name = args
            .get("function_name")
            .and_then(|v| v.as_str())
            .ok_or("Missing function_name parameter")?;
        let depth = args.get("depth").and_then(|v| v.as_u64()).unwrap_or(3) as usize;
        let direction = args.get("direction").and_then(|v| v.as_str()).unwrap_or("callee");

        // Use cached graph
        let graph = self.get_or_load_graph(&directory)?.clone();
        let generator = ContextGenerator::from_graph(graph);
        let tree = generator.get_call_tree(function_name, depth, direction);

        // Format output with visual tree representation
        let visual = tree.iter()
            .map(|node| {
                let indent = "  ".repeat(node.depth);
                let prefix = if node.depth == 0 { "●" } else { "├─" };
                format!("{}{} {} ({}:{})", indent, prefix, node.name, node.file_path, node.line)
            })
            .collect::<Vec<_>>()
            .join("\n");

        let output = json!({
            "function": function_name,
            "direction": direction,
            "depth": depth,
            "nodes": tree,
            "visual": visual
        });

        Ok(serde_json::to_string_pretty(&output).unwrap())
    }
}

impl Default for McpServer {
    fn default() -> Self {
        Self::new(None)
    }
}
