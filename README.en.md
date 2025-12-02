# index-chan

Dead code detection CLI for TypeScript projects (Phase 1)

[日本語版 README](README.md)

## Overview

**Current Features (Phase 1):**
index-chan is a CLI tool that detects and safely removes unused code (dead code) in TypeScript projects using AST analysis and LLM-powered semantic understanding.

**Future Vision:**
The ultimate goal is to build a "Code Dependency Graph Search System" that combines dependency graphs with vector search. This will be a next-generation development support tool that enables LLMs to understand and edit code with accurate context. See [docs/VISION.md](docs/VISION.md) for details.

**Currently at Phase 1 (Dead Code Detection) stage.**

## Features

- TypeScript AST parsing with tree-sitter
- Dependency graph construction
- Dead code detection (unused functions, classes)
- Safety level evaluation (definitely safe / probably safe / needs review)
- Interactive and automatic deletion modes
- Annotation feature (auto-add suppression comments)
- **🆕 LLM Integration** (Phase 1.5 ✅ Complete)
- High-precision analysis with Qwen2.5-Coder-1.5B
- Automatic detection of "planned for future use" code
- Identification of experimental features and WIP
- Fully local execution (privacy-preserving)
- Meaningful responses in Japanese

## Installation

```bash
cargo install --path .
```

## Usage

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

### Current Position: Phase 1.5 (LLM Integration) Complete ✅

This project is being developed in stages:

**Phase 1: Dead Code Detection CLI** ✅ Complete
- TypeScript analysis and dependency graph construction
- Unused code detection and removal

**Phase 1.5: LLM Integration** ✅ Complete
- High-precision analysis with local LLM
- Identification of "planned for future use" code

**Phase 2: Multi-language Support** (Planned)
- Support for Rust, Python, Go, Java, etc.
- Advanced dependency analysis

**Phase 3: Code Dependency Graph Search System** (Future)
- Vector search + graph traversal
- Optimized context provision for LLMs
- Unified context editing

See [docs/VISION.md](docs/VISION.md) for detailed vision.

### Phase 1 Completed ✅
- [x] TypeScript analysis (tree-sitter)
- [x] Dependency graph construction
- [x] Dead code detection
- [x] Deletion features (interactive/auto)
- [x] Annotation features

### Phase 1.5 Completed ✅
- [x] LLM integration (Qwen2.5-Coder-1.5B)
- [x] Local inference
- [x] Context collection (Git history)
- [x] High-precision analysis

### Phase 1.5 Improvements Planned
- [ ] Accuracy validation on real projects
- [ ] Prompt optimization
- [ ] Enhanced error handling

### Phase 2 Planned (Multi-language Support)
- [ ] Rust, Python, Go, Java support
- [ ] Advanced dependency analysis
- [ ] Incremental updates

### Phase 3 Planned (Search System)
- [ ] Vector search integration
- [ ] Hybrid search (vector + graph)
- [ ] Context optimization for LLMs
- [ ] Unified context editing

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
