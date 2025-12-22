// MCP Tool Definitions
// index-chanが提供するツール群

use crate::mcp::protocol::ToolDefinition;
use serde_json::json;

/// Get all available tool definitions
pub fn get_tool_definitions() -> Vec<ToolDefinition> {
    vec![
        // Scan tool
        ToolDefinition {
            name: "scan".to_string(),
            description: "Scan directory for dead code detection".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "directory": {
                        "type": "string",
                        "description": "Target directory to scan"
                    }
                },
                "required": ["directory"]
            }),
        },
        // Search tool
        ToolDefinition {
            name: "search".to_string(),
            description: "Search for code in indexed project".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "directory": {
                        "type": "string",
                        "description": "Project directory"
                    },
                    "query": {
                        "type": "string",
                        "description": "Search query"
                    },
                    "top_k": {
                        "type": "integer",
                        "description": "Number of results to return",
                        "default": 10
                    }
                },
                "required": ["query"]
            }),
        },
        // Stats tool
        ToolDefinition {
            name: "stats".to_string(),
            description: "Get project statistics".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "directory": {
                        "type": "string",
                        "description": "Target directory"
                    }
                },
                "required": ["directory"]
            }),
        },
        // gather_context tool (Phase 6 core feature)
        ToolDefinition {
            name: "gather_context".to_string(),
            description: "Gather code context with dependencies for a specific function or query"
                .to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "directory": {
                        "type": "string",
                        "description": "Project directory"
                    },
                    "entry_point": {
                        "type": "string",
                        "description": "Function name to start from"
                    },
                    "query": {
                        "type": "string",
                        "description": "Natural language query to find relevant code"
                    },
                    "depth": {
                        "type": "integer",
                        "description": "Dependency traversal depth",
                        "default": 2
                    },
                    "mode": {
                        "type": "string",
                        "enum": ["full", "skeleton"],
                        "description": "Output mode: full (complete code) or skeleton (signatures only)",
                        "default": "full"
                    },
                    "format": {
                        "type": "string",
                        "enum": ["standard", "llm_edit"],
                        "description": "Output format: standard (comments) or llm_edit (<<<FILE>>> markers for batch editing)",
                        "default": "standard"
                    }
                },
                "required": ["directory"]
            }),
        },
        // get_dependencies tool
        ToolDefinition {
            name: "get_dependencies".to_string(),
            description: "Get functions that the specified function depends on".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "directory": {
                        "type": "string",
                        "description": "Project directory"
                    },
                    "function_name": {
                        "type": "string",
                        "description": "Function name to analyze"
                    },
                    "depth": {
                        "type": "integer",
                        "description": "Traversal depth",
                        "default": 1
                    }
                },
                "required": ["directory", "function_name"]
            }),
        },
        // get_dependents tool
        ToolDefinition {
            name: "get_dependents".to_string(),
            description: "Get functions that depend on the specified function".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "directory": {
                        "type": "string",
                        "description": "Project directory"
                    },
                    "function_name": {
                        "type": "string",
                        "description": "Function name to analyze"
                    },
                    "depth": {
                        "type": "integer",
                        "description": "Traversal depth",
                        "default": 1
                    }
                },
                "required": ["directory", "function_name"]
            }),
        },
        // validate_changes tool (Phase 6 Week 3)
        ToolDefinition {
            name: "validate_changes".to_string(),
            description: "Validate the correctness of proposed code changes".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "directory": {
                        "type": "string",
                        "description": "Project directory"
                    },
                    "changes": {
                        "type": "array",
                        "description": "Array of file changes to validate",
                        "items": {
                            "type": "object",
                            "properties": {
                                "file_path": {
                                    "type": "string",
                                    "description": "Relative path to the file"
                                },
                                "modified_content": {
                                    "type": "string",
                                    "description": "Modified file content"
                                },
                                "original_content": {
                                    "type": "string",
                                    "description": "Original file content (optional)"
                                },
                                "start_line": {
                                    "type": "integer",
                                    "description": "Start line for partial update (optional)"
                                },
                                "end_line": {
                                    "type": "integer",
                                    "description": "End line for partial update (optional)"
                                }
                            },
                            "required": ["file_path", "modified_content"]
                        }
                    }
                },
                "required": ["directory", "changes"]
            }),
        },
        // preview_changes tool
        ToolDefinition {
            name: "preview_changes".to_string(),
            description: "Preview changes with unified diff format (dry-run)".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "directory": {
                        "type": "string",
                        "description": "Project directory"
                    },
                    "changes": {
                        "type": "array",
                        "description": "Array of file changes to preview",
                        "items": {
                            "type": "object",
                            "properties": {
                                "file_path": {
                                    "type": "string",
                                    "description": "Relative path to the file"
                                },
                                "modified_content": {
                                    "type": "string",
                                    "description": "Modified file content"
                                },
                                "original_content": {
                                    "type": "string",
                                    "description": "Original file content (optional)"
                                }
                            },
                            "required": ["file_path", "modified_content"]
                        }
                    }
                },
                "required": ["directory", "changes"]
            }),
        },
        // apply_changes tool
        ToolDefinition {
            name: "apply_changes".to_string(),
            description: "Apply validated changes to files (creates backup by default)".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "directory": {
                        "type": "string",
                        "description": "Project directory"
                    },
                    "changes": {
                        "type": "array",
                        "description": "Array of file changes to apply",
                        "items": {
                            "type": "object",
                            "properties": {
                                "file_path": {
                                    "type": "string",
                                    "description": "Relative path to the file"
                                },
                                "modified_content": {
                                    "type": "string",
                                    "description": "Modified file content"
                                },
                                "start_line": {
                                    "type": "integer",
                                    "description": "Start line for partial update (optional)"
                                },
                                "end_line": {
                                    "type": "integer",
                                    "description": "End line for partial update (optional)"
                                }
                            },
                            "required": ["file_path", "modified_content"]
                        }
                    },
                    "create_backup": {
                        "type": "boolean",
                        "description": "Create backup before applying changes",
                        "default": true
                    }
                },
                "required": ["directory", "changes"]
            }),
        },
        // search_with_graph tool (Phase 7 GraphRAG)
        ToolDefinition {
            name: "search_with_graph".to_string(),
            description:
                "Search code with graph traversal - finds related files via dependency edges. Use semantic=true for AI-powered concept search."
                    .to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "directory": {
                        "type": "string",
                        "description": "Project directory"
                    },
                    "query": {
                        "type": "string",
                        "description": "Search query (function name, concept, or natural language)"
                    },
                    "top_k": {
                        "type": "integer",
                        "description": "Number of initial matches to find",
                        "default": 3
                    },
                    "graph_depth": {
                        "type": "integer",
                        "description": "How many dependency hops to traverse",
                        "default": 2
                    },
                    "semantic": {
                        "type": "boolean",
                        "description": "Use AI embeddings for semantic search (requires first run to build cache)",
                        "default": false
                    },
                    "filter_generic": {
                        "type": "boolean",
                        "description": "Filter out generic names like new, default, clone from results",
                        "default": true
                    }
                },
                "required": ["directory", "query"]
            }),
        },
        // parse_llm_edits tool - Parse LLM output back into file changes
        ToolDefinition {
            name: "parse_llm_edits".to_string(),
            description: "Parse LLM-edited content (from gather_context with format=llm_edit) into changes array for apply_changes".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "content": {
                        "type": "string",
                        "description": "LLM output containing <<<FILE>>> blocks"
                    }
                },
                "required": ["content"]
            }),
        },
        // get_call_chain tool - Find path between two functions
        ToolDefinition {
            name: "get_call_chain".to_string(),
            description: "Find the call chain (shortest path) from one function to another".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "directory": {
                        "type": "string",
                        "description": "Project directory"
                    },
                    "from": {
                        "type": "string",
                        "description": "Source function name"
                    },
                    "to": {
                        "type": "string",
                        "description": "Target function name"
                    },
                    "max_depth": {
                        "type": "integer",
                        "description": "Maximum search depth",
                        "default": 10
                    }
                },
                "required": ["directory", "from", "to"]
            }),
        },
        // get_call_tree tool - Visualize call hierarchy
        ToolDefinition {
            name: "get_call_tree".to_string(),
            description: "Get call tree showing all functions called by or calling a function".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "directory": {
                        "type": "string",
                        "description": "Project directory"
                    },
                    "function_name": {
                        "type": "string",
                        "description": "Function to analyze"
                    },
                    "depth": {
                        "type": "integer",
                        "description": "How many levels to traverse",
                        "default": 3
                    },
                    "direction": {
                        "type": "string",
                        "enum": ["callee", "caller"],
                        "description": "callee = functions this calls, caller = functions that call this",
                        "default": "callee"
                    }
                },
                "required": ["directory", "function_name"]
            }),
        },
    ]
}
