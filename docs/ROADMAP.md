# index-chan Improvement Roadmap

**Version**: v0.3.0 (Post-MVP)  
**Last Updated**: 2025-12-03

## Current Status

MVP achieved with 9 MCP tools:
- Dead code detection
- Context generation with dependencies
- Bulk changes with validation and backup

## Proposed Improvements

### High Priority (User Experience)

#### 1. Undo Command
Restore from the most recent backup.

```bash
index-chan undo
# Restores from .index-chan/backups/20251203_143022
```

**Implementation**: Read latest backup directory, restore files.

#### 2. `.indexchanignore` File
Exclude specific files/directories from scanning.

```
# .indexchanignore
node_modules/
dist/
*.test.ts
__mocks__/
```

**Implementation**: Parse ignore file, filter during scan.

#### 3. Error Message Improvements
- Clearer error messages with recovery suggestions
- Localized messages (English/Japanese)

### Medium Priority (Feature Expansion)

#### 4. JavaScript Support
Extend TypeScript support to plain JavaScript files.

```bash
index-chan scan ./src  # Now handles .js files too
```

**Implementation**: Add tree-sitter-javascript, share analysis logic.

#### 5. Change History Tracking
Track all apply_changes operations.

```bash
index-chan history
# Shows:
# 2025-12-03 14:30:22 - Modified src/auth.ts (backup: 20251203_143022)
# 2025-12-03 10:15:00 - Modified src/utils.ts (backup: 20251203_101500)

index-chan diff --backup 20251203_143022
# Shows diff between current and backup
```

**Implementation**: Store metadata in SQLite, add history commands.

#### 6. CI/CD Integration
Exit codes and machine-readable output for CI pipelines.

```bash
index-chan scan --ci ./src
# Exit code = number of dead code items (0 = clean)

index-chan scan --ci --threshold 5 ./src
# Exit 1 if dead code count > 5
```

**GitHub Actions Example:**
```yaml
- name: Dead Code Check
  run: |
    index-chan scan --ci --threshold 10 ./src
```

### Low Priority (Future Consideration)

#### 7. Python Support
Extend to Python projects using tree-sitter-python.

```bash
index-chan scan ./src --language python
```

**Challenges**: Different import system, dynamic typing.

#### 8. VSCode Extension
Standalone extension for non-Kiro users.

**Features:**
- Inline dead code highlighting
- Quick fix actions
- Dependency graph view

#### 9. Git History Analysis
Analyze why code became dead.

```bash
index-chan analyze-history ./src
# Output:
# unusedHelper: Became unused after commit abc123 (refactor: remove legacy auth)
# oldFunction: Last used 6 months ago
```

#### 10. Multi-Language Project Support
Handle projects with mixed languages.

```bash
index-chan scan ./src --languages ts,js,py
```

## Implementation Priority Matrix

```
                    Impact
                    High    │ Medium  │ Low
              ──────────────┼─────────┼──────────
Effort  Low   │ undo        │ .ignore │ --ci
              │             │         │
        Med   │ JS support  │ history │ VSCode
              │             │         │
        High  │ Python      │ Git     │ Multi-lang
              │             │ analysis│
```

## Recommended Order

### Phase 7.1: Quick Wins (1-2 weeks)
1. `undo` command
2. `.indexchanignore` support
3. `--ci` flag for CI/CD

### Phase 7.2: JavaScript (2-3 weeks)
4. JavaScript file support
5. Shared analysis logic refactoring

### Phase 7.3: History (2-3 weeks)
6. Change history tracking
7. `history` and `diff` commands

### Phase 7.4: Future (TBD)
8. Python support
9. VSCode extension
10. Git history analysis

## Success Metrics

| Feature | Metric |
|---------|--------|
| undo | Recovery time < 5 seconds |
| .indexchanignore | Scan time reduction for large projects |
| --ci | Zero false positives in CI |
| JS support | Same accuracy as TypeScript |
| history | Complete audit trail |

## Feedback Channels

- GitHub Issues
- User surveys after each release
- Usage analytics (opt-in)
