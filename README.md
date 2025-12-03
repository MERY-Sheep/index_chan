<div align="center">
  <img src="ascii_image.png" alt="index-chan" width="600">
  
  # index-chan
  
  [日本語](README.ja.md) | English
  
  [![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
  [![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org/)
  
  Code analysis and modification tool for LLM agents (Phase 6 Complete - MVP Achieved!)
</div>

## Overview

**🎉 MVP Achieved! (Phase 6 Complete - v0.3.0)**

index-chan is a code analysis and modification tool designed for LLM agents (Kiro, Cursor, etc.). It provides 9 MCP (Model Context Protocol) tools that enable LLMs to understand and safely modify code.

**Key Features:**
- **Dead Code Detection**: Automatically detect unused code
- **Context Generation**: Gather functions with dependencies
- **Batch Changes**: Validate, preview, and apply code changes safely
- **Import Validation**: Prevent LLM hallucinations using dependency graphs
- **Automatic Backup**: Timestamped backups for safety

**Architecture:**
```
LLM Agent (Kiro/Cursor)
    ↓ MCP Protocol
index-chan MCP Server
    ↓ Dependency Graph
TypeScript Project
```

## Features

### MCP Tools (Phase 6 ✅ Complete)

**9 MCP tools for LLM agents:**

**Basic Features:**
1. **scan**: Dead code detection
2. **search**: Code search (requires indexing)
3. **stats**: Project statistics

**Context Generation:**
4. **gather_context**: Gather functions with dependencies
5. **get_dependencies**: Get function dependencies
6. **get_dependents**: Get function dependents

**Batch Changes:**
7. **validate_changes**: Validate code changes
8. **preview_changes**: Preview changes with diff
9. **apply_changes**: Apply validated changes safely

### Core Features

- **TypeScript AST parsing** with tree-sitter
- **Dependency graph** construction and analysis
- **Dead code detection** (unused functions, classes)
- **Safety level evaluation** (definitely safe / probably safe / needs review)
- **Interactive and automatic** deletion modes
- **Annotation feature** (auto-add suppression comments)
- **Import validation** using dependency graphs (prevents LLM hallucinations)
- **Automatic backup** with timestamps

## Installation

```bash
cargo install --path .
```

## Quick Start

### For LLM Agents (Kiro/Cursor)

**1. Build index-chan:**
```bash
cargo build --release
```

**2. Configure MCP in Kiro:**

Edit `~/.kiro/settings/mcp.json`:
```json
{
  "mcpServers": {
    "index-chan": {
      "command": "/path/to/index-chan/target/release/index-chan",
      "args": ["mcp-server"],
      "disabled": false,
      "autoApprove": [
        "scan",
        "stats",
        "search",
        "gather_context",
        "get_dependencies",
        "get_dependents"
      ]
    }
  }
}
```

**3. Use from LLM:**
```
User: "Add rate limiting to the authentication function"

LLM: index-chan.gather_context({
       directory: ".",
       entry_point: "authenticateUser",
       depth: 2
     })
     → Gets related code

LLM: Modifies code

LLM: index-chan.validate_changes({...})
     → Validates changes

LLM: index-chan.preview_changes({...})
     → Shows diff

User: "Apply it"

LLM: index-chan.apply_changes({...})
     → Applied with backup
```

## CLI Usage

### Scan (Detection Only)

```bash
# Basic scan
index-chan scan <directory>

# JSON output
index-chan scan <directory> --output report.json

# LLM analysis mode (Phase 1.5 ✅)
index-chan scan <directory> --llm
```

### Clean (Interactive)

```bash
# Interactive deletion with confirmation
index-chan clean <directory>
```

### Clean (Automatic, Safe Only)

```bash
# Auto-delete only definitely safe code
index-chan clean <directory> --auto --safe-only
```

### Dry Run

```bash
# Preview without actual deletion
index-chan clean <directory> --dry-run
```

### Annotation

```bash
# Add suppression annotations to "future use" code
index-chan annotate <directory>

# Dry run
index-chan annotate <directory> --dry-run

# LLM analysis mode (high precision)
index-chan annotate <directory> --llm
```

### Export Dependency Graph (Phase 3.1 ✅)

```bash
# GraphML format (for Gephi, yEd, Cytoscape)
index-chan export <directory> -o graph.graphml -f graphml

# DOT format (for Graphviz)
index-chan export <directory> -o graph.dot -f dot

# JSON format (for custom visualization)
index-chan export <directory> -o graph.json -f json
```

**Visualize with Graphviz:**
```bash
# SVG output
dot -Tsvg graph.dot -o graph.svg

# PNG output (3D layout)
neato -Tpng graph.dot -o graph.png
```

### 3D Web Visualization (Phase 3.2 ✅)

```bash
# Build with web feature
cargo build --features web --release

# Start web server
cargo run --features web --release -- visualize <directory> --port 8080

# Auto-open browser
cargo run --features web --release -- visualize <directory> --port 8080 --open
```

**Features:**
- Interactive 3D graph with Three.js + force-graph-3d
- Real-time statistics (nodes, edges, unused count)
- Node details on click
- Camera controls (rotate, zoom, pan)
- Dark theme UI

**Open in browser:** http://localhost:8080

### Code Search (Phase 2 🚧)

```bash
# Create search index
index-chan index <directory>

# Search for code
index-chan search "authentication"

# Search with context
index-chan search "file upload" --context

# Specify number of results
index-chan search "unused" -k 5
```

### Conversation Analysis (Phase 2 🚧)

```bash
# Analyze chat history
index-chan analyze-chat chat_history.json --output graph.json

# Extract topics
index-chan topics chat_history.json
```

## LLM Mode (Phase 1.5)

### Overview

LLM mode uses Qwen2.5-Coder-1.5B for high-precision semantic analysis.

**Features:**
- Fully local execution (no code sent externally)
- Auto-detection of "planned for future", "experimental", "WIP" code
- Git history-aware decisions
- Confidence scores

### System Requirements

**LLM Mode:**
- Memory: 3GB+ recommended
- Disk: 3GB+ (model cache)
- First run: ~3GB download
- Inference speed: ~2s/function (CPU)

**Normal Mode:**
- Memory: tens of MB
- Disk: few MB

## Development Status and Roadmap

### 🎉 MVP Achieved! (Phase 6 Complete)

**Phase 1: Dead Code Detection CLI** ✅ Complete
- TypeScript analysis and dependency graph construction
- Unused code detection and removal

**Phase 1.5: LLM Integration** ✅ Complete
- High-precision analysis with local LLM
- Identification of "planned for future use" code

**Phase 2: Search + Conversation Graph** ✅ Complete
- Vector-based code search
- Conversation graph for chat history
- Token reduction (39.5-60% achieved)

**Phase 3: Graph Visualization** ✅ Complete
- GraphML/DOT/JSON export
- 3D web visualization

**Phase 4: Database Layer** ✅ Complete
- SQLite persistence
- File watching and auto-update

**Phase 6: MCP Integration** ✅ Complete (MVP!)
- 9 MCP tools for LLM agents
- Context generation with dependencies
- Batch changes with validation
- Import validation (prevents hallucinations)
- Automatic backup

**Phase 5: Tauri Desktop App** ❄️ Frozen
- Postponed to focus on CLI/MCP

See [docs/VISION.md](docs/VISION.md) for detailed vision and [Doc/MVP/MVP_ロードマップ.md](Doc/MVP/MVP_ロードマップ.md) for Japanese roadmap.

### Completed Phases ✅

**Phase 1: Dead Code Detection**
- [x] TypeScript analysis (tree-sitter)
- [x] Dependency graph construction
- [x] Dead code detection
- [x] Deletion features (interactive/auto)
- [x] Annotation features

**Phase 1.5: LLM Integration**
- [x] LLM integration (Qwen2.5-Coder-1.5B)
- [x] Local inference
- [x] Context collection (Git history)
- [x] High-precision analysis

**Phase 2: Search + Conversation Graph**
- [x] Vector search foundation
- [x] Conversation graph foundation
- [x] CLI integration
- [x] Embedding model integration (BERT with Candle)
- [x] Topic detection
- [x] Related message search
- [x] Token reduction (39.5-60% achieved)

**Phase 3: Graph Visualization**
- [x] GraphML/DOT/JSON export
- [x] 3D web visualization (Three.js + force-graph-3d)

**Phase 4: Database Layer**
- [x] SQLite persistence
- [x] File hash-based change detection
- [x] File watching and auto-update
- [x] Database integration for existing commands

**Phase 6: MCP Integration (MVP!)**
- [x] MCP server implementation (JSON-RPC 2.0, stdio)
- [x] 9 MCP tools (scan, search, stats, gather_context, etc.)
- [x] Context generation with dependencies
- [x] Batch changes (validate, preview, apply)
- [x] Import validation using dependency graphs
- [x] Automatic backup with timestamps
- [x] Integration testing

### Next Steps

**Short-term:**
- Real-world usage and feedback collection
- Error handling improvements
- Performance optimization

**Mid-term:**
- TypeScript type checking integration
- ESLint integration
- Automatic test execution

**Long-term:**
- Multi-language support (JavaScript, Python, Rust)
- Web UI for change history
- Support for other LLM agents (Claude, ChatGPT)

## Testing

```bash
# Test with sample project
cargo run -- scan test_project

# JSON output
cargo run -- scan test_project --output report.json

# LLM inference test
cargo run --release -- test-llm
```

## Disclaimer

**Please read [DISCLAIMER.md](DISCLAIMER.md) before using this project.**

This is a personal experimental project. The author is not a professional programmer and cannot provide professional support.

## License

MIT License - See [LICENSE](LICENSE) file for details

## Documentation

- [docs/](docs/): Design and vision documents (English)
- [Doc/](Doc/): Development notes (Japanese, not published)

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for contribution guidelines.

## Security

See [SECURITY.md](SECURITY.md) for security policy.

## Disclaimer

⚠️ **Important Disclaimer**

**This is a personal experimental project.**

- The author is not a professional programmer
- Phase 1.5 (LLM Integration) just completed and still unstable
- Not recommended for production use
- May contain bugs and issues
- Support is limited (questions may not be answered)
- Use at your own risk

**About Contributions:**
- Bug reports are welcome, but immediate response is not guaranteed
- This project is created for learning and experimentation purposes
