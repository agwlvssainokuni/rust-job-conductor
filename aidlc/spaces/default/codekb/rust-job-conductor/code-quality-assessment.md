# コードの品質の評価

## テスト

| 項目 | 状態 |
|------|------|
| 枠組み | Rust 標準の `#[test]` だけ。テスト名は日本語 |
| 数 | `#[test]` の数: core 381、runtime 200、cli 473、converter 116(合計 1,170) |
| 統合テスト | core 33、runtime 14、cli 25、converter 2 のファイル(`tests/`)。単体テストは各モジュールの `mod tests` と、`crates/cli/src/tests/`、`crates/converter/src/tests/` |
| 流れのテスト | `cli/tests/flow_*.rs`(10 ファイル)が、偽の `Runtime`/`Sink`/`Env`/`Signals` を注入して、終了コード・表示・ステートの書き込みの順序を確かめる |
| 実バイナリのテスト | `cli/tests/binary_*.rs`(7 ファイル)が、`conductor` とテスト専用の補助バイナリを動かす。シグナルは `binary_signal.rs`、`signals_real.rs`、`signals_order.rs` |
| 非機能のテスト | 性能(`*_perf.rs`、`nfr_performance.rs`、`nfr_perf.rs`)、安全性(`nfr_security.rs`)、頑健さ(`nfr_robust.rs`)、ソースの検査(`nfr_source.rs`: `unsafe` の位置、`cfg` の位置、ネットワークの不在)、内容(`nfr_content.rs`: エラーの表示とステートの内容に入力の値が現れない) |
| 網羅のテスト | `CommandKind::ALL` などの配列で、種類の追加漏れを捕まえる |
| 参照出力との一致 | `core/tests/graph_layout_oracle.rs` が、`fixtures/graph_layout_oracle*.md`(参照実装の出力。最大 48,529 行)と配置を比べる |
| 文書との一致 | `cli/src/tests/docs.rs`(README と使い方の表)、`cli/tests/msrv_consistency.rs`(MSRV)、デモのテスト(`unix_demo.rs`、`windows_demo.rs`) |
| カバレッジ | CI の `coverage` で行 80% 以上を強制(テスト専用の補助バイナリは除外)。チームの方針(test-after、行カバレッジ 80%)と一致する |
| OS | CI の `test` で ubuntu・macos・windows。`msrv-test` で 1.100.0 |

評価: 良い。純粋な中核と差し替え口の分離で、流れの失敗の経路(書き込みの失敗、シグナル、出力の失敗)まで偽の実装で試せる。

## 静的検査と書式

- `cargo fmt --all -- --check` と `cargo clippy --workspace --all-targets --locked -- -D warnings` が CI の `lint` で関門になる。設定ファイルは無く既定。
- `#[allow]` は `runtime/src/platform/unix.rs` の `#![allow(unsafe_code)]` 1 つだけ。lint の抑止は無い。

## CI/CD

- マージの関門(`ci.yml`): `lint`、`build-check`(MSRV、C のビルドの不在)、`test`(3 OS)、`msrv-test`、`coverage`、`docs-check`。`startup-bench` は警告のみ。
- リリース(`release.yml`): タグが `main` の上にあり `crates/cli/Cargo.toml` の版と一致すること → `ci.yml` の再利用 → 4 対象のビルドと smoke test → 下書きのリリース。
- `audit.yml`: `cargo audit`(PR と毎週。警告のみ)。
- 変更は PR を使わず `main` に直接コミットする運用(`team.md`)のため、関門は `push` の CI で事後に効く。

## 文書

- `README.md`(772 行)は、インストール、クイックスタート、使い方、定義の書き方(retry、env)、ステート、終了コード、保証、シグナル、graph、dry-run、init、convert、ロケールを書く。必須の語句は `docs-check` が検査。
- 各モジュールの冒頭の `//!` が、責務、契約、規則の番号(BR/NFR/FR)、守る性質を日本語で書く。
- `docs/spec/` に 7 言語版の再構築用の仕様(各 13 文書)。

## 簡易評価(良い / 普通 / 弱い)

| 観点 | 評価 | 根拠 |
|------|------|------|
| 命名の明瞭さ | 良い | Rust 標準の命名。型・関数の名前が責務を表し、文書が規則の番号と結ぶ |
| 関数の大きさ | 普通 | 多くは小さい。`walk`、`args` の解釈、文言の表は長いが、表の形で単一の責務 |
| エラーの扱い | 良い | 失敗はすべて戻り値(パニックしない)。固定の文。表示の失敗も値で扱う |
| テストの存在 | 良い | 主要な経路にすべてテストがある |
| 重複 | 普通 | 使い方の表が 3 か所(`args.rs`、`messages.rs`、README)。escape の規則が core と converter に写しで 2 つ(同じ期待値の表で揃える) |
| 不要なコード | 良い | `TODO`/`FIXME`/`HACK` は無い。古い引数を除いた経緯は文書に残す(例: `run_job` の旧い引数) |

## 守る性質(コードで確認したもの)

- `unsafe` は runtime の `platform/unix.rs` の `libc::kill` 1 か所。runtime は `#![deny(unsafe_code)]` で、`unix.rs` だけ `#![allow(unsafe_code)]`。core・converter・cli(`main.rs`、補助バイナリを含む)は `#![forbid(unsafe_code)]`。
- 本体のコードの `cfg(unix)`/`cfg(windows)` は `runtime/src/platform/` と `cli/src/platform/` の中だけ。ほかの出現は `#[cfg(test)]` のテストの中(`core/src/definition/intake.rs`、`converter/src/intake.rs`、`cli/src/tests/`)と、`Cargo.toml` の対象別の依存。
- ネットワークのコードとライブラリは無い。
- エラーの表示に入力の値・ライブラリのエラー文を含めない。キー名・ジョブID・パスは escape して切り詰める。
- ステートに command・環境変数・出力・時刻を書かない。環境変数の値を持つ型(`EnvVar`、`EnvEntry`、`EffectiveEntry`)の `Debug` は伏せ字。計画と図は環境変数の値を持たない。
- 入力の大きさの上限(定義・`.env`・ステート・変換の入力は 1 MiB、入れ子 32 段、`.env` は 10,000 項目、変換の展開は 100,000 ノードと 16 MiB)。

## 技術的負債と、この intent への影響

| 項目 | 場所 | 影響 |
|------|------|------|
| ステートの読み取りが知らないキーを拒否する | `runtime/src/state/reader.rs` の `TOP_KEYS`・`JOB_KEYS` | 開始・終了の時刻を足すと、旧版の `conductor` が新しいステートを 65 で拒否する。`format_version` の扱い(上げる/据え置く)と、読み取りの互換を設計で決める必要がある |
| 時刻を扱う仕組みが無い | core は純粋、runtime・cli にも時刻の型が無い | 時刻の取得(cli か runtime)と、ステートへの書式(UNIX 時刻か RFC 3339 か。暦の計算を自前で書くか)を決める必要がある。依存を加えない方針 |
| リトライはジョブだけ | `core/src/types.rs` の `JobDefinition.retry`、`walk/mod.rs` の `PIPELINE_KEYS` | `[pipeline.retry]` は、読み取りで各ジョブへ合成するか、型に既定を持たせて `retry_decision` で選ぶかの選択。`--dry-run` の `retryable`、`graph -v` のリトライの表示も同じ値を使う必要がある |
| 起動の口が「足す」環境だけを受け取る | `runtime/src/runner.rs` の `run_job`、`core/src/envfile/merge.rs` の `EffectiveEnv` | `unset_env` には、消す名前を統合の結果と起動の口に通す変更が要る。自動の変数(`CONDUCTOR_*`)や `env` との優先の規則、`--dry-run` の名前の要約も揃える必要がある |
| `env_file` は必須のファイルだけ | `runtime/src/envfile.rs` の `read_one`、`core/src/types.rs` の `env_file: Vec<String>` | 任意の `.env` には、定義の書式(印の付け方)と、型(必須かどうかの情報)、読み取りの分岐が要る。`validate`・`graph`・`--dry-run` も同じ読み取りを通る |
| 標準入出力は引き継ぐだけ | `runtime/src/runner.rs` | `log_dir` には、ファイルの作成(runtime の責務)と、パスの解決(定義ファイルの基準)、試行ごとの扱い、`Runtime` trait の拡張が要る |
| `watch` は 1 回だけ | `cli/src/aux_commands.rs` の `run_watch` | 連続表示には、待機と中断(今はシグナルの準備をしない)、画面の書き換え(Windows では色を使わない方針)が要る |
| `graph` の形式は 2 つ | `cli/src/types.rs` の `GraphFormat`、`args.rs` の値の検査、`report/graph_mermaid.rs` | DOT は Mermaid と同じく `FlowGraph` からの純粋な整形で足せる。ラベルの escape の規則を DOT 用に持つ必要がある |
| 公開の構造体のフィールドが公開 | `core/src/types.rs` | フィールドの追加で、テストのリテラルの組み立てが広く変わる |
| 使い方の表の重複 | `args.rs`、`report/messages.rs`、`README.md` | 新しいキー・オプションのたびに 3 か所の更新が要る(`src/tests/docs.rs` が一部を検査) |
| `team.md` のテストファイルの数が古い | `aidlc/spaces/default/memory/team.md` の Testing Posture | 現状は core 33、runtime 14、cli 25、converter 2。記述の更新の候補 |
