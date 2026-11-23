# テストの手順(Unit Test Instructions): テストだけが使う本番のコードの削除

Minimal の戦略。既存の Rust のテストの実行環境(`cargo test`)を使う。新しい設定は要らない。テスト名は日本語。この intent では新しいテストを足さない。

## 実行のコマンド

```bash
# 全体(基準: 1,157 通過、1 無視、0 失敗。HEAD 9e91512)
cargo test --locked --workspace

# 各コミットでの全テストのコンパイル
cargo test --workspace --locked --no-run

# テスト名の一覧(前後で比べる)
cargo test --locked --workspace -- --list 2>/dev/null | sort

# Windows でだけコンパイルされるテストの確かめ(macOS から)
cargo clippy --workspace --all-targets --locked --target x86_64-pc-windows-msvc -- -D warnings

# ドキュメントのリンクの直し漏れ(警告は基準の 5 件のまま)
cargo doc --workspace --no-deps --locked
```

## テストの内容

| 指標 | テスト | 確かめ方 |
|------|--------|----------|
| 削除と公開範囲の縮小 | T1〜T32、§4.6 の消すテスト 4 件 | 消す・狭める名前の参照が 0 件(`functional-spec.md` §7 の検索)。テスト名の差は消した 4 件だけ。期待の文字列は変わらず通る |
| `RunIdError` を 1 つに | T4 | 4 つの場合の入力が通らないことを確かめる |
| 合格の条件 | 全体 | fmt、clippy、全テスト 1,153 通過、行カバレッジ 80%、MSRV 1.100、`Cargo.toml`・`Cargo.lock` の差分なし |

## 期待するカバレッジ

- プロジェクト全体の行カバレッジ 80% 以上(Build and Test が確かめる)。テストだけが使う実装が消えるので、下がらない見込み。

## モックとスタブ

- 既存の偽物(各クレートの `tests/common/mod.rs`)を使う。runtime のテストの補助は、文字列から `Command` を作る形にする(D5。`tests/` の中だけ)。

## テストデータ

- 新しいデータのファイルは足さない。
