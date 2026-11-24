# テストの手順(Unit Test Instructions): cli の本番の小さなモジュールの統合

Minimal の戦略。既存の Rust のテストの実行環境(`cargo test`)を使う。新しい設定は要らない。この intent では新しいテストを足さず、消さず、名前を変えない。

## 実行のコマンド

```bash
# 全体(基準: 1,149 通過、1 無視、0 失敗。HEAD 6284aaf)
cargo test --locked --workspace

# cli(各コミットで)
cargo test --locked -p conductor-cli

# テスト名の一覧(前後で比べる。差 0)
cargo test --locked --workspace -- --list 2>/dev/null | sort

# 本番の形
cargo clippy -p conductor-cli --lib --bins --locked -- -D warnings

# Windows の対象の clippy(platform の OS ごとのコードを変えるので、各コミットで)
cargo clippy --workspace --all-targets --locked --target x86_64-pc-windows-msvc -- -D warnings

# ドキュメントのリンクの直し漏れ(警告は基準の 5 件のまま)
cargo doc --workspace --no-deps --locked
```

## テストの内容

| 指標 | テスト | 確かめ方 |
|------|--------|----------|
| モジュールの統合 | cli の全テスト | 処理は変えないので、すべてのテストが期待を変えずに通る。消したモジュールのパスの参照が 0 件(`functional-spec.md` §7) |
| エスケープの関数の置き場所 | T1(`src/tests/escape_table.rs`) | `use` の 1 行を書き換え、確かめの行は変えずに通る |
| `platform` の中継 | シグナルのテスト(`signals_real.rs`、`signals_order.rs`、流れのテスト)、`nfr_source.rs` | macOS で通り、Windows の対象の clippy が通る。Windows の実行は CI で確かめる |
| 合格の条件 | 全体 | fmt、clippy、全テスト 1,149 通過、行カバレッジ 80%、MSRV 1.100、`Cargo.toml`・`Cargo.lock` の差分なし |

## 期待するカバレッジ

- プロジェクト全体の行カバレッジ 80% 以上(Build and Test が確かめる。写しで 98.06%)。

## モックとスタブ

- 新しい偽物は使わない。

## テストデータ

- 新しいデータのファイルは足さない。
