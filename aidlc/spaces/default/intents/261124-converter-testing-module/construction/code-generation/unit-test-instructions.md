# テストの手順(Unit Test Instructions): converter の testing モジュールの削除

Minimal の戦略。既存の Rust のテストの実行環境(`cargo test`)を使う。新しい設定は要らない。テスト名は日本語。この intent では新しいテストを足さない(移すだけ)。

## 実行のコマンド

```bash
# 全体(基準: 1,153 通過、1 無視、0 失敗。HEAD a91e7bd)
cargo test --locked --workspace

# converter だけ(各コミットで件数を見る。見込み: 単体 / 統合 = 80/36、93/23、95/21、97/19、96/19、96/19。統合のほかに 1 無視)
cargo test --locked -p conductor-converter

# テスト名の一覧(前後で比べる。移したテストはパスが変わるので、名前の最後の部分で比べる)
cargo test --locked --workspace -- --list 2>/dev/null | sort

# 本番の形(PartialEq なし)
cargo clippy -p conductor-converter --lib --locked -- -D warnings

# Windows の対象の clippy(macOS から)
cargo clippy --workspace --all-targets --locked --target x86_64-pc-windows-msvc -- -D warnings

# ドキュメントのリンクの直し漏れ(警告は基準の 5 件のまま)
cargo doc --workspace --no-deps --locked
```

## テストの内容

| 指標 | テスト | 確かめ方 |
|------|--------|----------|
| `testing` と `opposite` の削除 | T5、消すテスト 1 件 | `testing::`・`read_tree`・`opposite` の参照が 0 件(`functional-spec.md` §7) |
| テストを単体テストへ移す | T1〜T4、T6 | 移した 45 件が、名前と確かめの行と期待の値を変えずに `src/tests/` で通る。統合テストに残る 19 件も通る |
| `PartialEq` をテストのときだけに | T7 | `impl PartialEq for Value` が `#[cfg(test)]` の中だけにあり、本番の形(`--lib`)のビルドと clippy が通る |
| 合格の条件 | 全体 | fmt、clippy、全テスト 1,152 通過、行カバレッジ 80%、MSRV 1.100、`Cargo.toml`・`Cargo.lock` の差分なし |

## 期待するカバレッジ

- プロジェクト全体の行カバレッジ 80% 以上(Build and Test が確かめる。写しで 98.09%)。

## モックとスタブ

- 移すテストの補助(`yaml`・`toml`・`s`・`i`・`m`・`a`・`with_timeout`)は `src/tests/mod.rs` に置き、`yaml`・`toml` は非公開の `read_value` を、今の `read_tree` と同じ向きと同じファイル名で呼ぶ。

## テストデータ

- 新しいデータのファイルは足さない。`tests/fixtures/` は今の場所のまま、移したテストからは `include_str!` のパスを直して読む。
