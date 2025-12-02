# Design Document: Code Dependency Graph Search System

English | [日本語](DESIGN.ja.md)

## 1. System Architecture

### 1.1 Overall Structure

```
┌─────────────────────────────────────────────────┐
│              LLM Interface Layer                │
│  (Natural Language Query / Context Provider)    │
└─────────────────────────────────────────────────┘
                        ↓
┌─────────────────────────────────────────────────┐
│         Search Orchestration Layer              │
│  (Vector Search + Graph Traversal Integration)  │
└─────────────────────────────────────────────────┘
         ↓                              ↓
┌──────────────────┐          ┌──────────────────┐
│ Vector Search    │          │ Graph Traversal  │
│ Engine           │          │ Engine           │
└──────────────────┘          └──────────────────┘
         ↓                              ↓
┌─────────────────────────────────────────────────┐
│                Index Layer                      │
│  (Vector DB / Dependency Graph DB)              │
└─────────────────────────────────────────────────┘
                        ↓
┌─────────────────────────────────────────────────┐
│           Analysis & Build Layer                │
│  (tree-sitter / AST Analysis / Graph Build)     │
└─────────────────────────────────────────────────┘
                        ↓
┌─────────────────────────────────────────────────┐
│                Codebase                         │
└─────────────────────────────────────────────────┘
```

### 1.2 Technology Stack

**Core Implementation**
- Language: Rust
- Rationale: High performance, memory safety, easy parallelization

**Analysis Foundation**
- tree-sitter: Multi-language parser
- Language-specific tree-sitter grammars

**Vector Search**
- Qdrant or Milvus: Vector database
- sentence-transformers: Embedding models (CodeBERT, GraphCodeBERT)

**Graph Processing**
- petgraph: Rust graph library
- Persistence: RocksDB or SQLite

**API/CLI**
- axum: Web framework (REST API)
- clap: CLI framework

## 2. Data Model

### 2.1 Code Entity

```rust
enum NodeType {
    Function,
    Class,
    Method,
    Interface,
    Type,
    Module,
    File,
}

struct CodeNode {
    id: String,
    node_type: NodeType,
    name: String,
    file_path: String,
    start_line: u32,
    end_line: u32,
    signature: String,
    docstring: Option<String>,
    code_snippet: String,
    embedding: Vec<f32>,
}
```

### 2.2 Dependency Edge

```rust
enum EdgeType {
    Calls,
    CalledBy,
    References,
    Implements,
    Extends,
    Imports,
    TypeOf,
}

struct DependencyEdge {
    from_node: String,
    to_node: String,
    edge_type: EdgeType,
    weight: f32,
}
```

### 2.3 Dependency Graph

```rust
struct DependencyGraph {
    nodes: HashMap<String, CodeNode>,
    edges: Vec<DependencyEdge>,
    adjacency_list: HashMap<String, Vec<String>>,
}
```

## 3. Core Features

### 3.1 Analysis & Index Building

#### Code Analysis Flow

```
1. File reading
   ↓
2. Parse with tree-sitter → Generate AST
   ↓
3. AST traversal
   - Extract function/class definitions
   - Extract call relationships
   - Extract import/export
   ↓
4. Generate CodeNode
   ↓
5. Generate embedding vectors
   - Combine function name + signature + docstring
   - Encode with CodeBERT
   ↓
6. Save to Vector DB
```

#### Dependency Graph Construction

```rust
fn build_dependency_graph(files: Vec<FilePath>) -> DependencyGraph {
    let mut graph = DependencyGraph::new();
    
    // Phase 1: Extract nodes
    for file in files {
        let ast = parse_file(file);
        let nodes = extract_nodes(ast);
        graph.add_nodes(nodes);
    }
    
    // Phase 2: Build edges
    for file in files {
        let ast = parse_file(file);
        let edges = extract_dependencies(ast, &graph.nodes);
        graph.add_edges(edges);
    }
    
    // Phase 3: Build index
    graph.build_adjacency_list();
    
    graph
}
```

### 3.2 Hybrid Search

#### Search Flow

```rust
struct SearchRequest {
    query: String,
    top_k: usize,              // Default: 3
    max_hops_forward: usize,   // Default: 2
    max_hops_backward: usize,  // Default: 1
    max_tokens: usize,         // Default: 8000
}

struct SearchResult {
    contexts: Vec<ContextGroup>,
    total_tokens: usize,
}

struct ContextGroup {
    anchor_node: CodeNode,
    related_nodes: Vec<CodeNode>,
    relevance_score: f32,
}
```

#### Vector Search for Anchor Points

```rust
fn find_anchor_nodes(query: &str, top_k: usize) -> Vec<(CodeNode, f32)> {
    let query_embedding = encode_query(query);
    let results = vector_db.search(query_embedding, top_k);
    
    results.into_iter()
        .map(|(node_id, score)| {
            let node = graph.get_node(node_id);
            (node, score)
        })
        .collect()
}
```

#### Dependency Graph Traversal

```rust
fn explore_dependencies(
    graph: &DependencyGraph,
    anchor: &CodeNode,
    max_hops_forward: usize,
    max_hops_backward: usize,
) -> Vec<CodeNode> {
    let mut visited = HashSet::new();
    let mut result = Vec::new();
    
    result.push(anchor.clone());
    visited.insert(anchor.id.clone());
    
    // Forward traversal (callees)
    let forward_nodes = bfs_forward(
        graph,
        &anchor.id,
        max_hops_forward,
        &mut visited
    );
    result.extend(forward_nodes);
    
    // Backward traversal (callers)
    let backward_nodes = bfs_backward(
        graph,
        &anchor.id,
        max_hops_backward,
        &mut visited
    );
    result.extend(backward_nodes);
    
    // Add type definitions
    let type_nodes = find_related_types(graph, &result);
    result.extend(type_nodes);
    
    result
}
```

### 3.3 LLM Interface

#### Context Generation

```rust
fn format_context_for_llm(result: SearchResult) -> String {
    let mut output = String::new();
    
    output.push_str("# Search Results\n\n");
    
    for (i, group) in result.contexts.iter().enumerate() {
        output.push_str(&format!("## Anchor {}: {}\n", i + 1, group.anchor_node.name));
        output.push_str(&format!("Relevance Score: {:.2}\n\n", group.relevance_score));
        
        output.push_str("### Anchor Code\n");
        output.push_str(&format_code_block(&group.anchor_node));
        output.push_str("\n");
        
        output.push_str("### Related Code\n");
        for node in &group.related_nodes {
            output.push_str(&format_code_block(node));
            output.push_str("\n");
        }
    }
    
    output
}
```

## 4. Dead Code Detection

### 4.1 Detection Types

```rust
enum DeadCodeType {
    Orphaned,       // Completely unused
    Zombie,         // Does nothing
    UnusedImport,   // Unused imports
    Unreachable,    // Unreachable code
    Deprecated,     // Old version remnants
    Untested,       // No tests
}

struct CodeHealthReport {
    dead_code: Vec<DeadCodeItem>,
    total_waste_lines: usize,
    cleanup_tasks: Vec<CleanupTask>,
    technical_debt_score: f32,
}
```

### 4.2 Detection Algorithm

```rust
fn find_orphaned_nodes(graph: &DependencyGraph) -> Vec<CodeNode> {
    graph.nodes
        .values()
        .filter(|node| {
            graph.get_callers(&node.id).is_empty() &&
            !is_entry_point(node) &&
            !is_exported(node)
        })
        .cloned()
        .collect()
}
```

## 5. API Design

### 5.1 REST API

```
POST /api/search
Request:
{
    "query": "user authentication process",
    "top_k": 3,
    "max_hops_forward": 2,
    "max_hops_backward": 1,
    "max_tokens": 8000
}

Response:
{
    "contexts": [...],
    "total_tokens": 6543
}
```

### 5.2 CLI

```bash
# Build index
code-graph index build --path ./my-project --languages typescript,python

# Search
code-graph search "user authentication" --top-k 3

# Clean dead code
code-graph clean --dry-run
code-graph clean --auto --safe-only
```

## 6. Performance Optimization

### 6.1 Parallelization

```rust
fn parse_files_parallel(files: Vec<FilePath>) -> Vec<CodeNode> {
    files
        .par_iter()
        .flat_map(|file| {
            let ast = parse_file(file);
            extract_nodes(ast)
        })
        .collect()
}
```

### 6.2 Caching

```rust
struct SearchCache {
    anchor_cache: LruCache<String, Vec<(CodeNode, f32)>>,
    exploration_cache: LruCache<String, Vec<CodeNode>>,
}
```

## 7. Development Phases

### Phase 1: MVP (Current)
- Single language support (TypeScript)
- Basic dead code detection
- CLI tool
- Small projects (<1000 files)

### Phase 2: Multi-language Support
- Support for 5+ languages
- Advanced dependency analysis
- Medium projects (<5000 files)

### Phase 3: Enterprise Scale
- Large projects (10000+ files)
- Incremental updates
- Advanced optimization
- IDE plugins

## 8. Future Vision

### Short-term (1 year)
- Major language support (TypeScript, Python, Rust, Go, Java)
- IDE plugins (VSCode, IntelliJ)
- Automated cleanup features

### Mid-term (2-3 years)
- Real-time monitoring
- Advanced refactoring support
- Team collaboration features

### Long-term (3+ years)
- LLM-optimized code structure transformation
- New development paradigm: "LLM-optimized programming"

## References

- [tree-sitter](https://tree-sitter.github.io/)
- [Rust petgraph](https://docs.rs/petgraph/)
- [Qdrant Vector Database](https://qdrant.tech/)
