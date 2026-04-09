# obsidian-clip

URLを渡すと、Webページを取得し Gemini API で要約・タグ付けして、Obsidian に自動保存する CLI ツール。

## 機能

- Webページのタイトル・本文を自動取得
- Gemini 2.5 Flash で日本語要約・ポイント抽出・タグ提案
- Obsidian Local REST API 経由で Bookmarks フォルダにノートを保存
- API エラー時の自動リトライ（最大3回）

## 保存されるノートの形式

```
Bookmarks/YYYY-MM-DD-タイトル.md
```

```markdown
## 概要
- **タイトル**: ページタイトル
- **URL**: https://example.com
- **保存日**: 2026-04-10

## 要約
（Gemini による要約）

## ポイント
- （箇条書き）

## タグ
#タグ1 #タグ2 #タグ3
```

## セットアップ

### 1. 必要なもの

- Rust ツールチェーン
- [Gemini API キー](https://aistudio.google.com/apikey)
- Obsidian + [Local REST API](https://github.com/coddingtonbear/obsidian-local-rest-api) プラグイン

### 2. Local REST API プラグインの設定

1. Obsidian → 設定 → コミュニティプラグイン → 「Local REST API」をインストール・有効化
2. プラグイン設定で **Enable Non-encrypted (HTTP) Server** を有効にする
3. 表示される API キーをコピー

### 3. 環境変数

シェルの設定ファイル（`~/.zshrc`, `~/.bashrc` など）に追加：

```bash
export GEMINI_API_KEY="your-gemini-api-key"
export OBSIDIAN_API_KEY="your-obsidian-api-key"
```

設定後、シェルを再起動するか `source` で反映してください。

### 4. インストール

#### ビルド済みバイナリ（推奨）

[Releases](https://github.com/kom3da/obsidian-clip/releases) からお使いの OS に合ったファイルをダウンロード：

| ファイル | 対象 |
|---|---|
| `obsidian-clip-aarch64-apple-darwin.tar.gz` | macOS (Apple Silicon) |
| `obsidian-clip-x86_64-apple-darwin.tar.gz` | macOS (Intel) |
| `obsidian-clip-x86_64-unknown-linux-gnu.tar.gz` | Linux (x86_64) |

```bash
tar xzf obsidian-clip-*.tar.gz
sudo mv obsidian-clip /usr/local/bin/
```

#### ソースからビルド

```bash
cargo install --path .
```

## 使い方

```bash
obsidian-clip <URL>
```

```
$ obsidian-clip https://www.rust-lang.org/
📄 ページを取得中: https://www.rust-lang.org/
✅ タイトル: Rust Programming Language
🤖 Geminiで要約中...
💾 Obsidianに保存中...
✅ 保存完了: Bookmarks/2026-04-10-Rust Programming Language.md
```

## 注意事項

- Obsidian が起動中でないと保存できません（Local REST API はローカルサーバーとして動作）
- Vault 内に `Bookmarks` フォルダを事前に作成しておく必要があります
- `GEMINI_API_KEY` 未設定でも動作しますが、要約はスキップされます
