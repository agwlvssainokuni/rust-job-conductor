# テストの手順(Unit Test Instructions): 読まれない項目の削除

Minimal の戦略。既存の Rust のテストの実行環境(`cargo test`)を使う。新しい設定は要らない。テスト名は日本語。この intent では新しいテストを足さず、テストも消さない。

## 実行のコマンド

```bash
# 全体(基準: 1,153 通過、1 無視、0 失敗。HEAD 933f6de)
cargo test --locked --workspace

# 各コミットでの全テストのコンパイル
cargo test --workspace --locked --no-run

# テスト名の一覧(前後で比べる。差は ★ 2 件の名前の変更だけ)
cargo test --locked --workspace -- --list 2>/dev/null | sort

# Windows の対象の clippy(macOS から)
cargo clippy --workspace --all-targets --locked --target x86_64-pc-windows-msvc -- -D warnings

# ドキュメントのリンクの直し漏れ(警告は基準の 5 件のまま)
cargo doc --workspace --no-deps --locked
```

## テストの内容

| 指標 | テスト | 確かめ方 |
|------|--------|----------|
| 読まれない項目の削除 | T1〜T20、§4.6 の 13 行 | 消す項目の参照が 0 件(`functional-spec.md` §7 の検索)。残る項目(名前の並び、値、行き先、警告の `file`)で同じ意味を確かめる。期待の文字列は変わらず通る |
| 仕様書 | S1〜S21 | 直す語句が該当する節に残らず、残す記述(計画の表示が名前だけ、重複キーの警告の行、図の描き方)が残る |
| 合格の条件 | 全体 | fmt、clippy、全テスト 1,153 通過、行カバレッジ 80%、MSRV 1.100、`Cargo.toml`・`Cargo.lock` の差分なし |

## 期待するカバレッジ

- プロジェクト全体の行カバレッジ 80% 以上(Build and Test が確かめる)。

## モックとスタブ

- 既存の偽物とテストの補助(各クレートの `tests/common/`、core の `graph_common`)を使う。

## テストデータ

- 新しいデータのファイルは足さない。
