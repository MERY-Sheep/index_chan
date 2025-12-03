use colored::*;
use std::path::Path;

/// エラーメッセージと解決方法を表示
pub fn print_error_with_help(error_type: ErrorType, context: &str) {
    println!("\n{}", "❌ エラーが発生しました".red().bold());
    println!();
    
    match error_type {
        ErrorType::FilePermission(path) => {
            println!("原因: {} の書き込み権限がありません", path.display());
            println!();
            println!("{}", "💡 解決方法:".yellow().bold());
            println!("  1. ファイルの権限を確認:");
            println!("     {}", format!("dir {}", path.display()).cyan());
            println!("  2. 読み取り専用属性を解除:");
            println!("     {}", format!("attrib -r {}", path.display()).cyan());
            println!("  3. 管理者権限で実行:");
            println!("     {}", "管理者としてコマンドプロンプトを開く".cyan());
        }
        ErrorType::FileNotFound(path) => {
            println!("原因: {} が見つかりません", path.display());
            println!();
            println!("{}", "💡 解決方法:".yellow().bold());
            println!("  1. パスが正しいか確認:");
            println!("     {}", format!("dir {}", path.parent().unwrap_or(Path::new(".")).display()).cyan());
            println!("  2. ファイルが削除されていないか確認");
            println!("  3. 相対パスではなく絶対パスを使用");
        }
        ErrorType::InvalidDirectory(path) => {
            println!("原因: {} は有効なディレクトリではありません", path.display());
            println!();
            println!("{}", "💡 解決方法:".yellow().bold());
            println!("  1. ディレクトリが存在するか確認:");
            println!("     {}", format!("dir {}", path.display()).cyan());
            println!("  2. TypeScriptファイルが含まれているか確認:");
            println!("     {}", format!("dir {}\\*.ts /s", path.display()).cyan());
        }
        ErrorType::BackupNotFound => {
            println!("原因: バックアップが見つかりません");
            println!();
            println!("{}", "💡 解決方法:".yellow().bold());
            println!("  1. まだ変更操作を実行していない可能性があります");
            println!("  2. 利用可能なバックアップを確認:");
            println!("     {}", "index-chan undo <directory> --list".cyan());
            println!("  3. バックアップディレクトリを確認:");
            println!("     {}", "dir .index-chan\\backups".cyan());
        }
        ErrorType::DatabaseNotFound(path) => {
            println!("原因: データベースが見つかりません: {}", path.display());
            println!();
            println!("{}", "💡 解決方法:".yellow().bold());
            println!("  1. プロジェクトを初期化:");
            println!("     {}", format!("index-chan init {}", context).cyan());
            println!("  2. データベースパスを確認:");
            println!("     {}", format!("dir {}\\*.db", path.parent().unwrap_or(Path::new(".")).display()).cyan());
        }
        ErrorType::ParseError(file) => {
            println!("原因: {} の解析に失敗しました", file.display());
            println!();
            println!("{}", "💡 解決方法:".yellow().bold());
            println!("  1. ファイルの構文エラーを確認:");
            println!("     {}", "tsc --noEmit".cyan());
            println!("  2. ファイルが破損していないか確認");
            println!("  3. .indexchanignoreで除外:");
            println!("     {}", format!("echo {} >> .indexchanignore", file.display()).cyan());
        }
        ErrorType::NoTypeScriptFiles => {
            println!("原因: TypeScriptファイルが見つかりません");
            println!();
            println!("{}", "💡 解決方法:".yellow().bold());
            println!("  1. 正しいディレクトリを指定しているか確認");
            println!("  2. .ts または .tsx ファイルが存在するか確認:");
            println!("     {}", format!("dir {}\\*.ts /s", context).cyan());
            println!("  3. .indexchanignoreで除外されていないか確認");
        }
        ErrorType::BackupRestoreFailed(files) => {
            println!("原因: 一部のファイルの復元に失敗しました");
            println!();
            println!("失敗したファイル:");
            for file in files {
                println!("  - {}", file.display());
            }
            println!();
            println!("{}", "💡 解決方法:".yellow().bold());
            println!("  1. ファイルが他のプログラムで開かれていないか確認");
            println!("  2. 書き込み権限を確認");
            println!("  3. 手動でバックアップから復元:");
            println!("     {}", format!("copy .index-chan\\backups\\<timestamp>\\*.bak <destination>").cyan());
        }
        ErrorType::LLMApiError(message) => {
            println!("原因: LLM APIエラー: {}", message);
            println!();
            println!("{}", "💡 解決方法:".yellow().bold());
            println!("  1. APIキーが設定されているか確認:");
            println!("     {}", "echo %GEMINI_API_KEY%".cyan());
            println!("  2. APIキーを設定:");
            println!("     {}", "set GEMINI_API_KEY=your-api-key".cyan());
            println!("  3. ネットワーク接続を確認");
            println!("  4. LLMなしで実行:");
            println!("     {}", format!("index-chan {} (--llmフラグを外す)", context).cyan());
        }
    }
    
    println!();
    println!("{}", "📁 バックアップについて:".blue().bold());
    println!("  変更操作を実行すると、自動的にバックアップが作成されます");
    println!("  バックアップは .index-chan/backups/ に保存されます");
    println!("  undoコマンドで元に戻すことができます");
}

/// エラーの種類
pub enum ErrorType {
    FilePermission(std::path::PathBuf),
    FileNotFound(std::path::PathBuf),
    InvalidDirectory(std::path::PathBuf),
    BackupNotFound,
    DatabaseNotFound(std::path::PathBuf),
    ParseError(std::path::PathBuf),
    NoTypeScriptFiles,
    BackupRestoreFailed(Vec<std::path::PathBuf>),
    LLMApiError(String),
}
