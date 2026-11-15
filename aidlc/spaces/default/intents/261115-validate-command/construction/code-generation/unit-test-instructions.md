# Unit Test Instructions(validate-command)

## 実行の環境

- 言語とツール: Rust(edition 2024、最小 1.85)、`cargo`。外部のサービスやネットワークは要らない。
- テストの枠組み: 標準の `cargo test`(単体は `#[cfg(test)]`、統合は各クレートの `tests/`)。バイナリを起動するテストは、`env!("CARGO_BIN_EXE_conductor")` を使う。
- 設定ファイル: 変更なし。

## この作業のテストの実行

このスコープは、Unit に分けない(zero-Unit)。層ごとの確認は、そのクレートだけを動かし、最後に全体を 1 回動かす。

```bash
cargo test -p conductor-cli --locked       # validate の層の確認(Step 5〜6)
cargo test --workspace --locked            # 改名の確認(Step 4)と、全体の確認(Step 9)
```

## 整形と静的検査・最小の Rust

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo +1.85 build --workspace --locked --all-targets
cargo check --workspace --tests --target x86_64-pc-windows-msvc --locked
```

## 期待する網羅(Minimal の方針)

| 範囲 | 件数の目安 | 内容 |
|------|------------|------|
| cli: 引数 | 約 3 | `validate`・名前あり・`-v`・`--config` が解釈される。名前 2 個と `--to` は 68。使い方の文言と `CommandKind` の列挙の期待値の更新 |
| cli: 流れ(偽の `Runtime`) | 約 6 | ステート・実行・シグナルを呼ばない、定義の誤りと command の誤りは 64、名前の一致・不一致・省略、既定は無出力で `-v` は 1 行、存在しないプログラムは 0 |
| cli: 実際のバイナリ | 約 3 | 正しい定義 0、誤りの定義 64、`.state` が作られず、壊れた `.state` があっても 0 |

合計: 約 12 件。改名(FR2)の確認は、既存の全テスト(585 件)が成功することで行う。

## カバレッジ

- 目標: 行カバレッジ **80% 以上**(チームの下限。しきい値は下げない)。全体の下限は、Build and Test が、全テストで判定する(`cargo llvm-cov --workspace --locked --ignore-filename-regex 'bin/conductor-(cli-)?test-helper' --fail-under-lines 80`)。この作業のテストだけで 80% を下回っても、テストの失敗としては扱わず、結果に記録する。この測定は、ほかの命令とは別の命令として実行する。

## モックとスタブ

- 流れのテストは、既存の偽の `Runtime`・`Sink`・`Env`・`Signals` を使い、ステート系と `run_job` が呼ばれないことを、呼び出しの記録で確かめる。バイナリのテストは、実際のファイルを使う(モックにしない)。

## テストデータ

- 定義の例は、テストの中の文字列(`[[job]]` の形)と、一時ディレクトリ(`tempfile`)のファイル。
