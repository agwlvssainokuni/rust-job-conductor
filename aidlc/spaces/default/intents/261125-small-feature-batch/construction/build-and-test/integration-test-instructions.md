# 結合テストの手順: v0.10.0

出典: 各ユニットの単体テストの手順(`construction/*/code-generation/unit-test-instructions.md`)とコードのまとめ(`code-summary.md`)、契約(`inception/contract-design/contract-summary.md` の C1〜C6)、team.md の Testing Posture。

## テストの仕組み

- Rust の標準のテスト(`cargo test`)。結合テストは各クレートの `tests/`。
- 流れのテスト(`crates/cli/tests/flow_*.rs`)は偽の `Runtime`・`Sink`・`Env`・`Signals` を差し込んで、core・runtime・cli の境目を通す。
- 実バイナリのテスト(`crates/cli/tests/binary_*.rs`、`signals_real.rs`、`unix_demo.rs`)は、ビルドした `conductor` とテスト専用の補助バイナリを動かす。

## ユニットをまたぐ境目と、それを確かめるテスト

| 境目 | ユニット | 確かめるテスト |
|---|---|---|
| 環境の統合(C1): 表の `env_file`(必須かどうか)と `unset_env` の `removals` を同じ統合の出力で扱う | u2、u3 | `crates/core/tests/env_merge.rs`、`crates/cli/tests/flow_env.rs`、`binary_env.rs` |
| 試行ごとの時刻(C2・C4): `log_dir` のファイル名の時刻とステートの `started_at` が同じ値 | u5、u6 | `crates/cli/tests/flow_run.rs`、`flow_retry.rs`、`binary_log.rs` |
| リトライ(C3)と試行の記録: パイプラインの既定のリトライの各試行がステートの試行になる。待ちの間は試行が開いている | u1、u6、u7 | `crates/cli/tests/flow_retry.rs`、`crates/core/tests/pipeline_activity.rs` |
| `watch` の表(C5): 所要時間の列と継続表示 | u6、u7 | `crates/cli/tests/flow_aux.rs`、`binary_basic.rs` |
| グラフ(C5): `--format dot` と `--state` の古い形のステートで 65 | u4、u6 | `crates/cli/tests/flow_graph.rs`、`binary_graph.rs` |
| `--dry-run`: 消す名前の行と古い形のステートで 65 | u3、u6 | `crates/cli/tests/flow_dry_run.rs` |
| 使い方の表と README の一致 | 全部 | `crates/cli/src/tests/docs.rs` |
| デモ | 全部 | `crates/cli/tests/unix_demo.rs`、`windows_demo.rs`(Windows) |

## 走らせ方

```bash
cargo test --workspace --locked
```

- 各ユニットの単体テストのコマンドは、それぞれの `unit-test-instructions.md` にある。Build and Test では重複を除いて 1 回ずつ走らせる。
- Windows だけのテスト(`windows_demo.rs`、Windows の大文字小文字を区別しない `unset_env`、Windows の追記の表示)は、CI の windows で走らせる(後の CI Pipeline の段階)。

## 期待するカバレッジ

- 行カバレッジ 80% 以上(プロジェクト全体。CI の `coverage` と同じコマンド)。

```bash
cargo llvm-cov --workspace --locked --ignore-filename-regex 'bin/conductor-(cli-)?test-helper' --fail-under-lines 80 --summary-only
```

## テストのデータ

- 定義、`.env`、ステートは各テストの中で一時ディレクトリに作る。外のサービスは使わない。
