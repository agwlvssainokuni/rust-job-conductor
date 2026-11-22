# テストの手順(Unit Test Instructions): 動きを変えない小さな整理とテストの補助

Minimal の戦略。既存の Rust のテストの実行環境(`cargo test`)を使う。新しい設定は要らない。テスト名は日本語。

## 実行のコマンド

```bash
# 全体(基準: 1,142 通過、1 無視、0 失敗。HEAD af018bc)
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
| 本体の整理(段 1〜6) | 既存のテスト(無変更で通る)、N1、N4 | 出力と終了コードの期待の文字列が変わらず通る。`functional-spec.md` §7 の検索で、重複・ずれ・到達しない分岐・文字列での比較が残っていない |
| テストの補助(段 7) | T1〜T46、N2、N3、N5 | テストの名前と件数が変わらない(T38 の 1 件の削除と、移動の道の違いを除く)。§7 の検索で写しが残っていない |
| 件数 | 全体 | 1,142 − 1 + 5 = 1,146 通過、1 無視、0 失敗。段ごとの見込み: 段 1 の後 1,141、段 3 の後 1,142、段 5 の後 1,143、段 7 の後 1,146 |
| 合格の条件 | 全体 | fmt、clippy、全テスト、行カバレッジ 80%、MSRV 1.100、`Cargo.lock` の差分なし |

## 期待するカバレッジ

- プロジェクト全体の行カバレッジ 80% 以上(Build and Test が確かめる)。到達しない分岐が消えるので、下がらない見込み。

## モックとスタブ

- cli の流れのテストは、既存の偽物(`crates/cli/tests/common/mod.rs`)を使う。T15〜T19 で中身を直し、`ScriptedRuntime` に `run_lines` の記録を足す。

## テストデータ

- 新しいデータのファイルは足さない。N4 は一時ディレクトリにファイルを作る。
