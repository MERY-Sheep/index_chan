pub const SCHEMA: &str = r#"
-- プロジェクト情報
CREATE TABLE IF NOT EXISTS projects (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    path TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- ファイル情報
CREATE TABLE IF NOT EXISTS files (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    project_id INTEGER NOT NULL,
    path TEXT NOT NULL,
    language TEXT NOT NULL,
    hash TEXT NOT NULL,
    last_modified TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE,
    UNIQUE(project_id, path)
);

-- 関数定義
CREATE TABLE IF NOT EXISTS functions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    file_id INTEGER NOT NULL,
    name TEXT NOT NULL,
    line_start INTEGER NOT NULL,
    line_end INTEGER NOT NULL,
    is_exported BOOLEAN NOT NULL DEFAULT 0,
    is_used BOOLEAN NOT NULL DEFAULT 0,
    signature TEXT,
    FOREIGN KEY (file_id) REFERENCES files(id) ON DELETE CASCADE
);

-- 関数呼び出し
CREATE TABLE IF NOT EXISTS calls (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    caller_id INTEGER NOT NULL,
    callee_id INTEGER NOT NULL,
    line INTEGER NOT NULL,
    FOREIGN KEY (caller_id) REFERENCES functions(id) ON DELETE CASCADE,
    FOREIGN KEY (callee_id) REFERENCES functions(id) ON DELETE CASCADE
);

-- 依存関係（集約）
CREATE TABLE IF NOT EXISTS dependencies (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    from_function_id INTEGER NOT NULL,
    to_function_id INTEGER NOT NULL,
    edge_type TEXT NOT NULL,
    FOREIGN KEY (from_function_id) REFERENCES functions(id) ON DELETE CASCADE,
    FOREIGN KEY (to_function_id) REFERENCES functions(id) ON DELETE CASCADE,
    UNIQUE(from_function_id, to_function_id, edge_type)
);

-- インデックス
CREATE INDEX IF NOT EXISTS idx_files_path ON files(path);
CREATE INDEX IF NOT EXISTS idx_files_hash ON files(hash);
CREATE INDEX IF NOT EXISTS idx_functions_name ON functions(name);
CREATE INDEX IF NOT EXISTS idx_functions_file ON functions(file_id);
CREATE INDEX IF NOT EXISTS idx_calls_caller ON calls(caller_id);
CREATE INDEX IF NOT EXISTS idx_calls_callee ON calls(callee_id);
CREATE INDEX IF NOT EXISTS idx_deps_from ON dependencies(from_function_id);
CREATE INDEX IF NOT EXISTS idx_deps_to ON dependencies(to_function_id);
"#;
