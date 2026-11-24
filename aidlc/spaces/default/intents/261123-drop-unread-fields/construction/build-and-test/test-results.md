# テストの結果(Test Results)

macOS(主な開発環境)、rustc 1.100.0、`39fd6bc` の時点。

| 検査 | 結果 |
|------|------|
| `cargo fmt --all -- --check` | 差分なし |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | 警告なし |
| `cargo clippy ... --target x86_64-pc-windows-msvc -- -D warnings` | 警告なし |
| `cargo test --locked --workspace` | 1,153 通過、0 失敗、1 無視 |
| `cargo llvm-cov`(CI と同じ引数) | 行 98.08%、関数 98.20%、領域 97.76%(下限 80%) |
| `cargo doc --workspace --no-deps --locked` | 警告 5 件(基準と同じ。この intent と無関係) |
| `conductor --version` | `conductor 0.9.8` |

## 件数

| 時点 | core | runtime | converter | cli | 全体 |
|---|---|---|---|---|---|
| 基準(`172d8c6`) | 382 | 197 | 116 | 458 | 1,153 |
| 後(`b660947`) | 382 | 197 | 116 | 458 | 1,153 |

見込み(1,153 − 0 + 0)と一致した。無視は converter の 1 件のまま。テスト名の差は、名前を変えた ★ 2 件だけ。

## 確かめたこと

- 完了の判定の検索(`functional-spec.md` §7): 消した項目の参照はすべて 0 件。仕様書の消す語句は 0 件で、残す記述(重複キーの警告の `file`・`line`・`name`、00-README の層の用語「出どころ」)は残る。
- `git diff 172d8c6 -- crates/cli/src crates/runtime crates/converter` は空。

## テストの変更

- 既存のテストの変更は T1〜T20 と、§4.6 の確かめの行 13 行の削除だけ。期待の文字列は変えていない。名前の変更は ★ 2 件だけ。テストは消していない。
- Linux と Windows でのテストの実行は、push の後の CI で確かめる。

上流: `construction/code-generation/unit-test-instructions.md`(指標の確かめ方)、`construction/code-generation/code-generation-plan.md`。
