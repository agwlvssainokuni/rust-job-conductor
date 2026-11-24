# テストの手順(Unit Test Instructions): 小さな直し

Minimal の戦略。既存の Rust のテストの実行環境(`cargo test`)を使う。新しい設定は要らない。テスト名は日本語。この intent では新しいテストを足さない。

## 実行のコマンド

```bash
# 全体(基準: 1,152 通過、1 無視、0 失敗。HEAD 4f249ca)
cargo test --locked --workspace

# クレートごと(各コミットで件数を見る)
cargo test --locked -p conductor-core
cargo test --locked -p conductor-cli

# テスト名の一覧(前後で比べる。差は消した 3 件だけ。移した 1 件は名前が同じ)
cargo test --locked --workspace -- --list 2>/dev/null | sort

# 本番の形
cargo clippy -p conductor-cli --lib --bins --locked -- -D warnings

# Windows の対象の clippy(macOS から)
cargo clippy --workspace --all-targets --locked --target x86_64-pc-windows-msvc -- -D warnings

# ドキュメントのリンクの直し漏れ(警告は基準の 5 件のまま)
cargo doc --workspace --no-deps --locked
```

## テストの内容

| 指標 | テスト | 確かめ方 |
|------|--------|----------|
| テストの重複の削除 | T1、T3 | 消した 3 件と同じことを、`plan_simulate_perf` の P2 と `args_grammar` の init の既定のテストが確かめ続ける |
| 性能のテストの移動 | T2 | 移した 1 件が、名前と本体を変えずに `plan_simulate_perf.rs` で通る。`plan_perf.rs` が無い |
| ソースの検査の強化 | T4 | `nfr_source.rs` の 6 件が通る。製品のコードに include のマクロがあれば失敗する |
| 説明と仕様書 | (テストなし) | `functional-spec.md` §7 の検索 |
| 合格の条件 | 全体 | fmt、clippy、全テスト 1,149 通過、行カバレッジ 80%、MSRV 1.100、`Cargo.toml`・`Cargo.lock` の差分なし |

## 期待するカバレッジ

- プロジェクト全体の行カバレッジ 80% 以上(Build and Test が確かめる。写しで 98.06%)。

## モックとスタブ

- 新しい偽物は使わない。

## テストデータ

- 新しいデータのファイルは足さない。
