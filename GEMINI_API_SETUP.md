# Gemini API セットアップガイド

## 1. APIキーの取得

1. https://aistudio.google.com/app/apikey にアクセス
2. 「Create API Key」をクリック
3. プロジェクトを選択（または新規作成）
4. APIキーをコピー（例: `AIzaSyA...`で始まる39文字）

## 2. APIキーの設定

### Windows (PowerShell)

```powershell
# 環境変数を設定
$env:GEMINI_API_KEY="your-api-key-here"

# 確認
echo $env:GEMINI_API_KEY

# アプリ起動
cargo tauri dev
```

### 永続的に設定（推奨）

```powershell
# ユーザー環境変数に設定
[System.Environment]::SetEnvironmentVariable("GEMINI_API_KEY", "your-api-key-here", "User")

# PowerShellを再起動してから確認
echo $env:GEMINI_API_KEY
```

## 3. トラブルシューティング

### エラー: "API Key not found"

**原因**
- APIキーが設定されていない
- APIキーに空白や改行が含まれている
- APIキーが無効

**解決方法**
```powershell
# 1. APIキーを再確認
echo $env:GEMINI_API_KEY

# 2. 長さを確認（39文字程度）
$env:GEMINI_API_KEY.Length

# 3. 正しいキーを再設定
$env:GEMINI_API_KEY="AIzaSy..."

# 4. アプリを完全に再起動
```

### エラー: "API_KEY_INVALID"

**原因**
- APIキーが間違っている
- APIキーが無効化されている
- APIが有効化されていない

**解決方法**
1. https://aistudio.google.com/app/apikey で新しいキーを作成
2. Gemini API が有効になっているか確認
3. 新しいキーで再設定

## 4. 動作確認

アプリでチャットを送信すると、以下のログが表示されます：

```
🔍 環境変数を確認中...
✅ APIキー取得成功: AIzaSyA... (長さ: 39文字)
🌟 Gemini APIを使用
📡 Gemini APIにリクエスト送信中...
⏱️  リクエスト送信: 0.5秒
✅ Gemini APIから応答を受信
```

## 5. セキュリティ

### ❌ 避けるべき

```bash
# コードにハードコード
let api_key = "AIzaSy..."; // NG!

# Gitにコミット
git add .env  # NG!
```

### ✅ 推奨

```bash
# 環境変数で管理
$env:GEMINI_API_KEY="..."

# .gitignoreに追加
echo ".env" >> .gitignore
echo "GEMINI_API_KEY.txt" >> .gitignore
```

## 6. 参考リンク

- API Key 取得: https://aistudio.google.com/app/apikey
- Gemini API ドキュメント: https://ai.google.dev/
- 料金: https://ai.google.dev/pricing
