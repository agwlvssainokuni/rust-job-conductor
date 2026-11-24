# 意図の記述(Intent Statement): cli の本番の小さなモジュールの統合

## Sources

- [desc] Initial description: "7 回目の調査の intent C: cli の本番の小さなモジュールの統合(動きを変えない。版は上げない)。(1) src/retry.rs(93 行。run_job_with_retry 1 つ。使うのは step.rs だけ)を src/step.rs へ。(2) src/report/escape.rs(34 行。包むだけの関数 2 つ)を src/report/mod.rs へ(使うのは report の 4 ファイル)。(3) src/graph.rs(78 行)と src/init.rs(67 行)を、シグナルの準備をしない補助のコマンドをまとめた src/aux_commands.rs(118 行。watch・reset・validate・convert)へ。(4) src/platform/mod.rs の start_signal_watcher → spawn_watcher → unix::spawn_watcher の二重の中継を、runtime と同じ再公開の形に。platform のディレクトリとファイル名は変えない(cli の nfr_source.rs がパスの要素 platform で判定する)。判断が要るもの: どれをまとめるか。テストと仕様書 rust の該当箇所を直す。範囲の外: core の小さなモジュール(duration.rs、edges.rs)、テストの統合、platform のファイルの統合"
- [scope] Workflow-selected scope: `refactor`.
- [Q1] `intent-capture-questions.md` Q1: A(設計とコードの計画は 1 回で承認し、小さなコミットで進め、結果は最後にまとめて見せる。テストは `use` のパスの書き換えだけで、名前・確かめの行・期待の値は変えない)
- [Q2] `intent-capture-questions.md` Q2: A(`retry.rs` を `step.rs` へまとめる)
- [Q3] `intent-capture-questions.md` Q3: A(`report/escape.rs` を `report/mod.rs` へまとめる)
- [Q4] `intent-capture-questions.md` Q4: A(`graph.rs` と `init.rs` を `aux_commands.rs` へまとめる)
- [Q5] `intent-capture-questions.md` Q5: A(`platform/mod.rs` の中継を、runtime と同じ再公開の形にする)
- [memory:M1] `aidlc/spaces/default/memory/project.md#Mandated`: "ALWAYS 変更は小さな単位に保ち、小さな単位でレビューする(チームの慣行)。 (affirmed 2026-11-20)"
- [memory:M2] `aidlc/spaces/default/memory/project.md#Mandated`: "ALWAYS 行カバレッジ 80% 以上を保つ(CI の `coverage` ジョブ。`--fail-under-lines 80`)。 (affirmed 2026-11-20)"
- [memory:M3] `aidlc/spaces/default/memory/project.md#Mandated`: "ALWAYS 最小の Rust(MSRV 1.100)でビルドとテストが通ることを保ち、引き上げるときは README の記載と一致させる(`build-check`、`msrv-test`、`msrv_consistency.rs`)。 (affirmed 2026-11-20)"
- [memory:M4] `aidlc/spaces/default/memory/project.md#Mandated`: "ALWAYS `cargo fmt` と `cargo clippy -D warnings` を通してからマージする(CI の `lint`)。 (affirmed 2026-11-20)"
- [memory:M5] `aidlc/spaces/default/memory/project.md#Forbidden`: "NEVER 今回の作業で新しい外部ライブラリ(依存)を追加しない。`.env` の読み込みとグラフの描画は自前で書く(インタビュー Q4)。 (affirmed 2026-11-20)"

## Problem Statement

- cli の本番のコードに、使う側が 1 か所の小さなモジュール(`retry.rs`)、包むだけの小さなファイル(`report/escape.rs`)、同じ性質のコマンドなのに別のファイルに分かれたもの(`graph.rs`、`init.rs`)、二重の中継(`platform/mod.rs`)がある。[desc]
- 解決したいのは、利用者から見える動きを変えずに、これらを使う側や同じ性質の所へまとめることである。[desc] [Q2] [Q3] [Q4] [Q5]

## Target Customer

- このコマンドラインツールのコードとテストを保守し、機能を足す開発者。仕様書を読む実装者。[desc]

## Success Metrics

- `retry.rs` の中身は `step.rs` にあり、`retry.rs` は無くなる。[Q2]
- `report/escape.rs` の中身は `report/mod.rs` にあり、`report/escape.rs` は無くなる。[Q3]
- `graph` と `init` のコマンドは `aux_commands.rs` にあり、`graph.rs` と `init.rs` は無くなる。シグナルの準備をしないコマンドは `aux_commands.rs`、準備をするコマンド(`run`、`pipeline`、`dry_run`)は 1 コマンド 1 ファイル、という分け方になる。[Q4]
- `platform/mod.rs` の中継は 1 段になり、runtime の `platform` と同じ再公開の形になる。`platform` のディレクトリと、その中のファイル(`mod.rs`、`unix.rs`、`windows.rs`)の名前は変わらない。[Q5] [desc]
- 確かめる手段: 無くなるファイルが無いこと、無くなったモジュールのパス(`crate::retry`、`report::escape`、`crate::graph`、`crate::init`)の参照が 0 件であることを検索で確かめる。[desc]
- テストは `use` のパスの書き換えだけで、名前・確かめの行・期待の値は変えない。テスト名の差は 0。件数は変わらない(基準は実装を始める前の HEAD の `cargo test --locked --workspace` の通過の件数)。[Q1]
- cli の製品のコードの検査(`nfr_source.rs`)が、テストの数と中身を変えずに通る。[desc]
- 仕様書 rust のモジュールの表と説明が実装と合い、無くなったファイルの名前が残らないことを検索で確かめる。[desc]
- 利用者から見える動き(出力、終了コード、ステートファイル、`--dry-run` の出力、シグナルの扱い)は変わらない。版は上げない。[desc]
- 合格の条件: `cargo fmt --all -- --check` と `cargo clippy --workspace --all-targets --locked -- -D warnings` が通る [memory:M4]。`platform` の OS ごとのコードを変えるので、Windows の対象の clippy も通す(CI の `test` は Windows でも動く)[desc]。行カバレッジ 80% 以上 [memory:M2]。MSRV 1.100 でのテストが通る [memory:M3]。新しい依存を足さない [memory:M5]。

## Initiative Trigger

- 7 回目のリファクタリングの調査(ファイルの統合の観点を加えた洗い出し)の候補を 3 つの intent に分けた、その 3 つ目(任意)。[desc]

## Initial Scope Signal

- Workflow-selected scope: `refactor`(workflow-selected)[scope]
- 範囲: 初期の記述の (1)〜(4)。[desc] [Q2] [Q3] [Q4] [Q5]
- 進め方: 設計とコードの計画は 1 回で承認し、実装は小さなコミットで進める。結果は最後にまとめて見せる。[Q1] [memory:M1]
- 範囲の外: core の小さなモジュール、テストの統合、`platform` のファイルの統合。[desc]

## Assumptions & Open Questions

None.
