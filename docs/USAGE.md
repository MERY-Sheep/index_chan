# index-chan Usage Guide

**Version**: v0.3.0 (Phase 6 - MVP Complete)

## Overview

index-chan is a code analysis and modification tool designed for LLM agents (Kiro, Cursor, etc.). It provides 9 MCP (Model Context Protocol) tools that enable LLMs to safely understand and modify code.

## Installation

```bash
# Build from source
cargo build --release

# Install globally
cargo install --path .
```

## Quick Start

### For LLM Agents (Kiro/Cursor)

**1. Configure MCP in Kiro:**

Edit `~/.kiro/settings/mcp.json`:
```json
{
  "mcpServers": {
    "index-chan": {
      "command": "/path/to/index-chan",
      "args": ["mcp-server"],
      "disabled": false,
      "autoApprove": [
        "scan", "stats", "search",
        "gather_context", "get_dependencies", "get_dependents"
      ]
    }
  }
}
```

**2. Use from LLM:**
```
User: "Add rate limiting to the authentication feature"

LLM: index-chan.gather_context({
       directory: ".",
       entry_point: "authenticateUser",
       depth: 2
     })
     → Retrieves related code

LLM: Modifies code

LLM: index-chan.validate_changes({...})
     → Validates changes

LLM: index-chan.preview_changes({...})
     → Shows diff

User: "Apply it"

LLM: index-chan.apply_changes({...})
     → Applied with backup
```

## CLI Commands

### Dead Code Detection

```bash
# Basic scan
index-chan scan <directory>

# JSON output
index-chan scan <directory> --output report.json

# With LLM analysis
index-chan scan <directory> --llm

# Using database (faster)
index-chan scan <directory> --use-db
```

### Cleaning

```bash
# Interactive cleaning
index-chan clean <directory>

# Dry run (preview only)
index-chan clean <directory> --dry-run

# Auto-clean safe code only
index-chan clean <directory> --auto --safe-only
```

### Annotation

```bash
# Add annotations to preserve code
index-chan annotate <directory>

# Dry run
index-chan annotate <directory> --dry-run

# With LLM analysis
index-chan annotate <directory> --llm
```

### Graph Export

```bash
# GraphML (for Gephi, yEd, Cytoscape)
index-chan export <directory> -o graph.graphml -f graphml

# DOT (for Graphviz)
index-chan export <directory> -o graph.dot -f dot

# JSON (for custom visualization)
index-chan export <directory> -o graph.json -f json
```

### 3D Web Visualization

```bash
# Build with web feature
cargo build --features web --release

# Start web server
index-chan visualize <directory> --port 8080

# Auto-open browser
index-chan visualize <directory> --port 8080 --open
```

### Database Management

```bash
# Initialize project (once)
index-chan init <directory>

# Show statistics
index-chan stats <directory>

# Watch for changes (background)
index-chan watch <directory>
```

### Code Search

```bash
# Create index
index-chan index <directory>

# Search code
index-chan search "authentication" --context -k 5
```

### Chat History Analysis

```bash
# Extract topics
index-chan topics chat_history.json

# Find related messages
index-chan related chat_history.json "error" -k 3 --context
```

## MCP Tools

### Basic Tools

| Tool | Description |
|------|-------------|
| `scan` | Detect dead code |
| `search` | Search code (requires index) |
| `stats` | Get project statistics |

### Context Generation

| Tool | Description |
|------|-------------|
| `gather_context` | Generate code context with dependencies |
| `get_dependencies` | Get functions that a function depends on |
| `get_dependents` | Get functions that depend on a function |

### Bulk Changes

| Tool | Description |
|------|-------------|
| `validate_changes` | Validate code changes |
| `preview_changes` | Preview changes as diff |
| `apply_changes` | Apply validated changes with backup |

## Typical Workflows

### Workflow 1: Dead Code Cleanup

```bash
# 1. Scan for dead code
index-chan scan ./src

# 2. Review results
# 3. Clean safely
index-chan clean ./src --auto --safe-only
```

### Workflow 2: Feature Modification (via LLM)

```
1. LLM gathers context with gather_context
2. LLM modifies code
3. LLM validates with validate_changes
4. LLM previews with preview_changes
5. User approves
6. LLM applies with apply_changes
```

### Workflow 3: Database-Powered Development

```bash
# 1. Initialize once
index-chan init ./my_project

# 2. Start watching (background)
index-chan watch ./my_project

# 3. Fast scans from database
index-chan scan ./my_project --use-db
```

## Feature Flags

| Flag | Description |
|------|-------------|
| `web` | Enable 3D web visualization |
| `db` | Enable database features |
| `llm` | Enable LLM analysis |

Build with features:
```bash
cargo build --features "web,db" --release
```

## System Requirements

**Standard Mode:**
- Memory: ~50MB
- Disk: ~10MB

**LLM Mode:**
- Memory: 3GB+
- Disk: 3GB+ (model cache)

## Related Documentation

- [VISION.md](VISION.md) - Long-term vision
- [DESIGN.md](DESIGN.md) - Technical design
