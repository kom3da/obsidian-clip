# obsidian-clip

URLからWebページを取得し、Gemini APIで要約してObsidianに保存するCLIツール。

## コマンド

- `cargo test` — テスト実行
- `cargo clippy -- -D warnings` — lint
- `cargo fmt -- --check` — フォーマットチェック
- `cargo build --release` — リリースビルド
- `cargo install --path .` — ローカルインストール

## 構成

- `src/main.rs` — 引数解析・オーケストレーション
- `src/config.rs` — 設定ファイル・環境変数読み込み
- `src/fetch.rs` — Webページ取得・HTML解析
- `src/gemini.rs` — Gemini API呼び出し・リトライ
- `src/obsidian.rs` — Obsidian Local REST API保存
- `src/note.rs` — ファイル名生成・テンプレート展開

## ルール

- テストは各モジュール内に `#[cfg(test)] mod tests` で書く
- 外部APIに依存するテストは書かない（fetch/gemini/obsidianのネットワーク部分）
- `cargo fmt` と `cargo clippy -- -D warnings` をコミット前に通すこと
- リリースは `git tag vX.Y.Z && git push origin vX.Y.Z` で自動ビルド
