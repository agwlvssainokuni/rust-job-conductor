# テストの手順(Unit Test Instructions): テストの都合で曲げられた本番のコードと、残りの整理

Minimal の戦略。既存の Rust のテストの実行環境(`cargo test`)を使う。新しい設定は要らない。テスト名は日本語。

## 実行のコマンド

```bash
# 全体(基準: 1,161 通過、1 無視、0 失敗。HEAD c5f1a3a)
cargo test --locked --workspace

# テスト名の一覧(段の前後で比べる)
cargo test --locked --workspace -- --list 2>/dev/null | sort

# クレートごと
cargo test --locked -p conductor-core
cargo test --locked -p conductor-runtime
cargo test --locked -p conductor-converter
cargo test --locked -p conductor-cli
```

## テストの内容

| 指標 | テスト | 確かめ方 |
|------|--------|----------|
| 段 1(テストの都合で曲げられた所) | T1〜T11・T14・T16・T18・T20〜T24・T27〜T30、消すテスト 5 件、各クレートの `tests/common/kinds.rs` | 本番のコードから、消した名前の参照が 0 件(`inventory.md` の検索)。テストは公開の入口・偽物の記録・テストの補助・単体テストで代わりを持つ。テスト名の差は、消した 5 件と、名前の変わった 3 件(★ T1・T2・T30)と、置き場所の変わったテストだけ。件数 1,156 |
| 段 2(ふつうの整理) | T12・T13・T15・T17・T19・T25・T26・T31、N1 | 期待する出力の文字列が変わらず通る。ステートファイルのバイト列が前後で同じ。件数 1,157 |
| 合格の条件 | 全体 | fmt、clippy、全テスト 1,157 通過、行カバレッジ 80%、MSRV 1.100、`Cargo.toml`・`Cargo.lock` の差分なし |

## 期待するカバレッジ

- プロジェクト全体の行カバレッジ 80% 以上(Build and Test が確かめる)。テストのためだけの関数が消えるので、下がらない見込み。

## モックとスタブ

- cli の流れのテストは、既存の偽物(`crates/cli/tests/common/mod.rs`)を使う。偽の `Env` に色の判定の記録(`color_read`)を足す。

## テストデータ

- 新しいデータのファイルは足さない。誤りの種類の一覧の写しは、各クレートの `tests/common/kinds.rs` に置く。
