# Vision Document: Code Dependency Graph Search System

English | [日本語](VISION.ja.md)

## 1. Overview

This project proposes a new code search system called "**Index-chan**" that combines vector search with dependency graph traversal.

Traditional text search or vector search alone suffers from "context pollution" where irrelevant code fragments are mixed in, degrading LLM response quality. Our system identifies anchor points through vector search and extracts logically related code through dependency graphs, providing minimal yet coherent context to LLMs.

### System Flow

```
Natural Language Query
  ↓
Vector Search (Get top-k functions/files)
  ↓
Anchor Candidates [func_a, func_b, func_c]
  ↓
Dependency Graph Traversal from each anchor
  ↓
Merge & Prioritize Results
  ↓
Provide Optimized Context to LLM
```

## 2. Background & Challenges

### 2.1 Challenges LLMs Face in Code Search

- Complex inter-file dependencies scatter necessary information
- Vector search mixes lexically similar but logically unrelated code
- Dependency direction (call relationships, type relationships) not reflected in results
- LLMs infer from incorrect context, leading to wrong answers

### 2.2 Comparison with Existing Tools

**Vector Search Only (e.g., GitHub Copilot)**
- Pros: Flexible natural language search
- Cons: Ignores logical dependencies, irrelevant code mixed in

**Graph Traversal Only (e.g., LSP definition jump)**
- Pros: Accurate dependency tracking
- Cons: Requires manual anchor specification, no natural language support

**Our System's Differentiation**
- Vector search automatically identifies "approximately here"
- Graph traversal collects "logically necessary" items comprehensively
- Combines strengths of both approaches

## 3. System Concept

### 3.1 Hybrid Search Architecture

**Step 1: Anchor Point Identification via Vector Search**
- Generate embedding vectors per function/class (including docstrings/comments)
- Retrieve top-k items (k=3~5) for natural language queries
- Secure multiple anchor candidates to reduce search failure risk

**Step 2: Dependency Graph Traversal**
- From each anchor, explore:
  - Callees (2 hops)
  - Callers (1 hop)
  - Type definitions/interfaces
  - Related functions in same file
- Assign priority to traversal results (direct > indirect dependencies)

**Step 3: Context Optimization**
- Deduplication (when same code reached from multiple anchors)
- Remove low-priority items based on token limits
- Group by anchor and present to LLM

### 3.2 Dependency Graph Structure

**Nodes**
- Functions
- Classes/Types
- Modules/Files

**Edges**
- Call relationships
- Reference relationships
- Inheritance relationships (extends/implements)
- Import/export relationships

**Analysis Foundation**
- Multi-language parsing with tree-sitter
- Dependency extraction via AST analysis

### 3.3 Unified Context for Batch Editing

**Traditional Problems**
- Slow due to individual file read/edit
- Difficult to maintain inter-file consistency
- Easy to miss changes
- Waste tokens by sending unnecessary code to LLM

**Our Approach**
```
Identify related code via dependency graph
  ↓
Extract only relevant functions/classes from multiple files
  ↓
Present as single unified context file to LLM
  ↓
LLM edits the unified context
  ↓
index-chan parses diffs and merges back to original files
```

**Example:**

```
Query: "Rename authenticateUser to verifyUser"

1. Dependency graph traversal
   src/auth.ts: authenticateUser()
   src/api.ts: login() (calls authenticateUser)
   src/types.ts: User type definition

2. Generate unified context
   ┌─────────────────────────────────────┐
   │ // === src/auth.ts ===              │
   │ function authenticateUser(u: User) {│
   │   // ...                            │
   │ }                                   │
   │                                     │
   │ // === src/api.ts ===               │
   │ function login() {                  │
   │   authenticateUser(user);           │
   │ }                                   │
   │                                     │
   │ // === src/types.ts ===             │
   │ interface User { ... }              │
   └─────────────────────────────────────┘

3. LLM edits
   ┌─────────────────────────────────────┐
   │ // === src/auth.ts ===              │
   │ function verifyUser(u: User) {      │
   │   // ...                            │
   │ }                                   │
   │                                     │
   │ // === src/api.ts ===               │
   │ function login() {                  │
   │   verifyUser(user);                 │
   │ }                                   │
   │                                     │
   │ // === src/types.ts ===             │
   │ interface User { ... }              │
   └─────────────────────────────────────┘

4. index-chan merges diffs
   src/auth.ts: authenticateUser → verifyUser
   src/api.ts: authenticateUser(user) → verifyUser(user)
   src/types.ts: no changes
```

**Benefits**
- Token efficiency: Extract only related code, exclude unnecessary parts
- Consistency: LLM sees all related code, no missed changes
- Speed: Single LLM request edits multiple files
- Atomicity: All succeed or all fail (avoid partial states)
- Reviewability: Review all changes at once
- Structure preservation: Original file structure maintained

## 4. Target Users

- Engineers dealing with complex codebases daily
- Teams performing LLM-assisted code review, modification, refactoring
- Companies with large monorepos
- Technical departments seeking advanced static analysis and code search

## 5. System Value

### 5.1 Technical Value

- Accurate context based on dependencies
- Reduced LLM error rate and hallucinations
- Optimized context size (exclude unnecessary code)
- Wide applicability through multi-language support
- Continuous code quality maintenance
- Technical debt reduction through automated dead code detection

### 5.2 Business Value

- Significantly reduce engineer search/investigation time
- Lower code understanding and maintenance costs
- Improve knowledge sharing efficiency in large teams
- Shorten onboarding time
- Prevent codebase bloat through automated cleanup
- Visualize and quantify technical debt

### 5.3 Competitive Advantage

- Unique combination of vector search and graph traversal
- Realize "next-generation code search" based on structural understanding
- High-consistency refactoring through unified context batch editing
- Create new tool category optimized for LLM era

## 6. Success Metrics (KPIs)

### Quantitative Metrics

- LLM answer accuracy: 80%+ correct rate (+20% vs baseline)
- Context size reduction: 40% average reduction
- Search time: <1 second (1000-file projects)
- Hallucination rate: 30% reduction
- Dead code detection rate: 95%+
- Codebase size reduction: 10-15% average

### Qualitative Metrics

- User satisfaction: NPS score 50+
- Developer search efficiency: 85%+ respond "found needed code"

## 7. Development Phases

### Phase 1: MVP (3 months)

**Goal**: Implement and validate basic features

- Single language support (TypeScript or Python)
- Vector search (using existing libraries)
- Basic dependency graph (function calls only)
- Dead code detection (rule-based)
- Validation on small projects (<1000 files)

**Deliverables**:
- Prototype
- Accuracy evaluation report
- Dead code detection accuracy report

### Phase 2: Accuracy Improvement (3 months)

**Goal**: Raise to practical level

- Add type information
- Import/export analysis
- Weighting and priority system
- Automated cleanup features
- Health report generation
- Validation on medium projects (<5000 files)

**Deliverables**:
- Beta version
- Benchmark results
- Cleanup effectiveness report

### Phase 3: Scale Support (6 months)

**Goal**: Commercial-grade completion

- Multi-language support (5+ languages)
- Incremental updates
- Parallelization and optimization
- Large project support (10000+ files)

**Deliverables**:
- Official release
- Documentation and API
- Specialized LLM model

## 8. Use Cases

### 8.1 Code Understanding

```
Query: "Understand user authentication process"
  ↓
System presents related code as unified context
  ↓
LLM generates explanation viewing entire context
```

### 8.2 Refactoring

```
Query: "Rename authenticateUser to verifyUser"
  ↓
Identify impact scope via dependency graph (10 files)
  ↓
Extract only related functions/classes
  ↓
Present as single unified context file to LLM
  ↓
LLM edits the unified context
  ↓
index-chan parses diffs and merges back to original 10 files
  ↓
Batch apply (consistency guaranteed)
```

### 8.3 Bug Fixing

```
Query: "Fix missing null checks"
  ↓
Retrieve related function groups via dependency graph
  ↓
Extract related code and create unified context
  ↓
LLM analyzes and fixes unified context
  ↓
index-chan merges diffs back to original files
  ↓
Batch apply fixes across multiple files
```

### 8.4 Code Cleanup

```
Periodic or manual execution
  ↓
Scan entire codebase
  ↓
Detect dead code, zombie code, unused imports
  ↓
Generate prioritized cleanup list
  ↓
After user confirmation, auto-delete safe items
  ↓
Result report: "548 lines reduced, 12% technical debt improved"
```

## 9. Summary

This project dramatically improves accuracy and efficiency of LLM-oriented code search by combining vector search with graph traversal.

Furthermore, unified context batch editing enables "high-consistency large-scale refactoring" that was difficult with traditional file-by-file editing. This is not just a search tool, but a next-generation development support system leveraging LLMs.

Going beyond traditional full-text or vector search, it realizes "next-generation code search and editing" based on structural understanding. It has high value both technically and commercially, with great potential as a development tool for the LLM era.

Starting with MVP for small-scale validation, we'll confirm accuracy and performance, then scale up gradually—a realistic strategy.
