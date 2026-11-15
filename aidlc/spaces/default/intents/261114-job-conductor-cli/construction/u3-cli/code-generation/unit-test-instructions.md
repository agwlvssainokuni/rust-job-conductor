# Unit Test Instructions(u3-cli)

## 実行の環境

- 言語とツール: Rust(edition 2024)、`cargo`。外部のサービスやネットワークは要らない(依存は、最初の取得だけ公開のレジストリから)。
- テストの枠組み: 標準の `cargo test`(単体は `#[cfg(test)]`、統合は `crates/u3-cli/tests/`)。開発用の依存は `tempfile`(Windows のテストだけ `windows-sys`)。流れのテストは、偽の `Runtime`・`Sink`・`Env`・`Signals`(trait)を、`run_cli_with` に渡して行う。バイナリを起動するテストは、`env!("CARGO_BIN_EXE_conductor")` と、テスト専用の補助の実行ファイル(`conductor-cli-test-helper`)で行い、`cargo test` が自動でビルドする。
- 設定ファイル: ワークスペースの根の `Cargo.toml`(`crates/*` と `[profile.release]`)と、`crates/u3-cli/Cargo.toml`(Step 1 で作る)。この Unit は、`conductor-core`・`conductor-runtime`・`conductor-converter` に依存する。

## この Unit のテストの実行(この Unit に限る)

```bash
cargo test -p conductor-cli --locked
```

- 最初のテストを書く前に、この命令が、テスト 0 件で成功することを確かめる(Step 1)。
- 特定のテストだけを動かすときは、フィルタを足す(例: `cargo test -p conductor-cli --locked args::`、`cargo test -p conductor-cli --locked --test flow_run`)。
- Step 2(u1-core のキー名の変更)の確認だけは、u1-core のテストを動かす: `cargo test -p conductor-core --locked`。全体の命令(`-p` なしの `cargo test`)は、この Unit の確認には使わない(Step 18 の最後の確認を除く)。

## 整形と静的検査(Unit の確認)

```bash
cargo fmt --all -- --check
cargo clippy -p conductor-cli --all-targets --locked -- -D warnings
```

## 期待する網羅(Standard の方針)

| 範囲 | 件数の目安 | 内容 |
|------|------------|------|
| 引数の解釈 | 20〜25 | 5つのコマンドの正しい引数と誤り、`=` 形式、`--`、`-vv`、値の欠落、`-v`・`--config` の範囲、`--help`・`--version`、UTF-8 でないパス |
| 終了コード | 10〜12 | 表の全項目、範囲外、エラーで 0 になる場合 |
| 表示 | 18〜22 | 全種類の文言(英語・日本語)、エスケープ、ロケール、遅延、色、watch の表 |
| 流れ(個別実行) | 20〜25 | 失敗の経路の全行、3つの判断、書き込みの順序、at-least-once、検査中の合図、-v |
| 流れ(一括実行) | 10〜12 | AC3.1.x、AC3.2.x、AC4.3.2、スキップで完了、ジョブのあいだの中断、65 |
| 補助のコマンド | 10〜12 | watch、reset、convert(成功・失敗・閉じたパイプ) |
| 入口・シグナル・パニック | 6〜8 | `run_cli_with`、シグナルの準備の失敗(70)、2回目の `start`、パニックのフック(70。終了を差し替え) |
| バイナリを起動 | 8〜10 | `--version`、使い方の誤り、終了コード、標準出力と標準エラーの分け方、convert、watch、実際のシグナル(UNIX 系)、Windows の Ctrl+C |
| 横断(NFR) | 8〜10 | ソースの検索、パニックしない、起動の測定(失敗にしない)、CPU、同時の実行、convert の1回の書き込み |

合計: 約 130 件。

## 時間の上限

- プロセスを起動するテストは、各 30 秒の時間の上限(別のスレッド + `recv_timeout`)を持つ。超えたら、テストが持つ子プロセスを強制終了して回収してから、失敗にする。この上限が検知するのは、ハングだけである。

## カバレッジ

- 目標: 行カバレッジ **80% 以上**(チームの下限。しきい値は下げない)。
- 全体の下限は、Build and Test が、組み立てたプロジェクトの全テストで判定する。この Unit だけのテストで 80% を下回ったときは、**この Unit のテストの失敗としては扱わず**、結果に記録する。測定は、`cargo llvm-cov -p conductor-cli --locked`(導入されていれば)を、**ほかの命令とは別の命令として**実行する。補助の実行ファイルのソースは除外する(`--ignore-filename-regex`)。`cfg(windows)` のコードは、Linux の測定に入らず、Windows の CI のテストの成功で確かめる。
- この Unit の範囲(`crates/u3-cli/src/` のうち、補助の実行ファイルを除く)で、行カバレッジ 80% 以上を、Build and Test が確かめる。

## モックとスタブ

- 流れは、偽の `Runtime`・`Sink`・`Env`・`Signals` で、失敗を起こす(ジョブのあとの書き込みの失敗、初期化の失敗、閉じたパイプ、シグナルの準備の失敗、起動前・ジョブのあいだの合図)。純粋な部品(引数、終了コード、表示、watch の表、エスケープ)は、モックを使わず、引数の値を直接作って渡す。
- バイナリを起動するテストは、実際の `conductor` と補助の実行ファイルを使う(モックにしない)。ファイルを使うテストは、`tempfile` の一時ディレクトリ。
- 時間の測定(NFR1)は、標準のライブラリの単調な時計を使う。基準を超えても失敗にしない(`STARTUP_BENCH_ENFORCE` が `true` のときだけ失敗)。

## テストデータ

- 定義ファイルの例は、テストの中の文字列(`[[job]]` の形)と、`tests/fixtures/*.toml`。大きな定義(ジョブ 1,000 個)は、テストの中で生成する(ファイルをコミットしない)。
- 実際の機密や、利用者の環境のパスは、テストデータに含めない。
