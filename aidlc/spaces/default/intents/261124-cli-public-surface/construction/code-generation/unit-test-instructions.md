# テストの手順(Unit Test Instructions): cli のライブラリの公開範囲を狭める

Minimal の戦略。既存の Rust のテストの実行環境(`cargo test`)を使う。新しい設定は要らない。テスト名は日本語。この intent では新しいテストを足さず、消さない(移すだけ)。

## 実行のコマンド

```bash
# 全体(基準: 1,152 通過、1 無視、0 失敗。HEAD 4f93936)
cargo test --locked --workspace

# cli だけ(各コミットで件数を見る。基準は単体 2 / 統合 456、最後は 164 / 294)
cargo test --locked -p conductor-cli

# テスト名の一覧(前後で比べる。移したテストはパスが変わるので、名前の最後の部分で比べる)
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
| 公開範囲を狭める | 全体、`nfr_source.rs` | `lib.rs` の公開の項目が設計の一覧(残す 13)と一致。`tests/` から狭めた名前の参照が 0 件(`functional-spec.md` §7) |
| テストを単体テストへ移す | T1〜T7 | 移した 162 件が、名前と確かめの行と期待の値を変えずに `src/tests/` で通る。名前の最後の部分で比べた差は 0 |
| 書き方を変えて残すテスト | T8〜T10 | `flow_init.rs` の期待のひな形が元と同じバイト列。`flow_validate.rs` と `tests/common` が通る |
| ソースの検査(D2) | `nfr_source.rs` | `src/tests/` の外の src は今と同じ検査。`lib.rs` の末尾の宣言と、`#[path` が無いことを確かめる。テストの数は 6 のまま |
| 合格の条件 | 全体 | fmt、clippy、全テスト 1,152 通過、行カバレッジ 80%、MSRV 1.100、`Cargo.toml`・`Cargo.lock` の差分なし |

## 期待するカバレッジ

- プロジェクト全体の行カバレッジ 80% 以上(Build and Test が確かめる。写しで 98.06%)。

## モックとスタブ

- 移すテストの補助(`FakeEnv`、`FakeSink`、`state_error`、`readme` など)は `src/tests/` に写す(D7)。`tests/` に残る流れのテストは、今の `tests/common` の偽物を使う。

## テストデータ

- 新しいデータのファイルは足さない。
