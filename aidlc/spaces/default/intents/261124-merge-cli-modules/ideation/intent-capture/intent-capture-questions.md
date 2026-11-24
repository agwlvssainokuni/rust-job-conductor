# 意図の把握の質問(Intent Capture Questions): cli の本番の小さなモジュールの統合

## Sources

- [desc] Initial description: "7 回目の調査の intent C: cli の本番の小さなモジュールの統合(動きを変えない。版は上げない)。(1) src/retry.rs(93 行。run_job_with_retry 1 つ。使うのは step.rs だけ)を src/step.rs へ。(2) src/report/escape.rs(34 行。包むだけの関数 2 つ)を src/report/mod.rs へ(使うのは report の 4 ファイル)。(3) src/graph.rs(78 行)と src/init.rs(67 行)を、シグナルの準備をしない補助のコマンドをまとめた src/aux_commands.rs(118 行。watch・reset・validate・convert)へ。(4) src/platform/mod.rs の start_signal_watcher → spawn_watcher → unix::spawn_watcher の二重の中継を、runtime と同じ再公開の形に。platform のディレクトリとファイル名は変えない(cli の nfr_source.rs がパスの要素 platform で判定する)。判断が要るもの: どれをまとめるか。テストと仕様書 rust の該当箇所を直す。範囲の外: core の小さなモジュール(duration.rs、edges.rs)、テストの統合、platform のファイルの統合"
- [scope] Workflow-selected scope: `refactor`.
- [memory:M1] `aidlc/spaces/default/memory/project.md#Mandated`: "ALWAYS 変更は小さな単位に保ち、小さな単位でレビューする(チームの慣行)。 (affirmed 2026-11-20)"
- [memory:M2] `aidlc/spaces/default/memory/project.md#Mandated`: "ALWAYS 行カバレッジ 80% 以上を保つ(CI の `coverage` ジョブ。`--fail-under-lines 80`)。 (affirmed 2026-11-20)"
- [memory:M3] `aidlc/spaces/default/memory/project.md#Mandated`: "ALWAYS 最小の Rust(MSRV 1.100)でビルドとテストが通ることを保ち、引き上げるときは README の記載と一致させる(`build-check`、`msrv-test`、`msrv_consistency.rs`)。 (affirmed 2026-11-20)"
- [memory:M4] `aidlc/spaces/default/memory/project.md#Mandated`: "ALWAYS `cargo fmt` と `cargo clippy -D warnings` を通してからマージする(CI の `lint`)。 (affirmed 2026-11-20)"
- [memory:M5] `aidlc/spaces/default/memory/project.md#Forbidden`: "NEVER 今回の作業で新しい外部ライブラリ(依存)を追加しない。`.env` の読み込みとグラフの描画は自前で書く(インタビュー Q4)。 (affirmed 2026-11-20)"

## Q1. [プロダクト] 進め方と既存のテストの扱い

背景: 4 つとも本番のコードの置き場所を変えるだけで、処理は変えません。単体テストの `use` のパス(`crate::report::escape::…` など)は、置き場所に合わせて直す必要があります。

- A. 前回と同じ。設計とコードの計画は 1 回で承認し、実装は小さなコミットで進め、結果は最後にまとめて見せる。テストは `use` のパスの書き換えだけで、名前・確かめの行・期待の値は変えない
- B. 段ごとに止まり、結果を見せて、続けるかを確かめる
- X. Other (please specify)

[Answer]: A

## Q2. [プロダクト] `retry.rs` を `step.rs` へまとめるか

背景: `retry.rs`(93 行)の関数 `run_job_with_retry` を使うのは `step.rs`(251 行。1 つのジョブを実行する手順)だけです。まとめると約 330 行になります。リトライは 1 つのジョブの実行の一部です。

- A. まとめる
- B. まとめない(リトライを独立した関心ごととして残す)
- X. Other (please specify)

[Answer]: A

## Q3. [プロダクト] `report/escape.rs` を `report/mod.rs` へまとめるか

背景: `report/escape.rs`(34 行)は、core の `escape_display` を包む 1 行の関数 2 つ(`escape_full`、`escape_name`)だけです。使うのは report の 4 ファイル(`mod.rs`、`plan_text.rs`、`watch.rs`、`graph_text.rs`)で、どれも report の中です。まとめると `report/mod.rs` は 643 行から約 660 行になり、`push_line` など他の共有の補助と同じ所に並びます。

- A. まとめる
- B. まとめない(エスケープの補助を独立したファイルとして残す)
- X. Other (please specify)

[Answer]: A

## Q4. [プロダクト] `graph.rs` と `init.rs` を `aux_commands.rs` へまとめるか

背景: `aux_commands.rs`(118 行)は、シグナルの準備をしない補助のコマンド(`watch`、`reset`、`validate`、`convert`)をまとめたファイルです。`graph`(78 行)と `init`(67 行)も、シグナルの準備をしないコマンドで、関数 1 つずつです。一方、シグナルの準備をするコマンド(`run`、`pipeline`、`dry_run`)は 1 コマンド 1 ファイルです。まとめると `aux_commands.rs` は約 240 行になります。

- A. まとめる(シグナルの準備をしないコマンドは `aux_commands.rs`、準備をするコマンドは 1 コマンド 1 ファイル、という分け方にそろう)
- B. まとめない(`graph` と `init` は v0.9.0 で足した機能として、1 コマンド 1 ファイルに残す)
- X. Other (please specify)

[Answer]: A

## Q5. [プロダクト] `platform/mod.rs` の二重の中継

背景: cli の `platform/mod.rs` は、`start_signal_watcher` が OS ごとの `spawn_watcher` を呼び、それが `unix::spawn_watcher`(または `windows::…`)を呼ぶ、二重の中継になっています。runtime の `platform` は、OS ごとの関数を `#[cfg(unix)] pub(crate) use unix::… as …;` の形で直接見せています。ファイルとディレクトリは変えません(cli のソースの検査が `platform` の名前で判定するため)。

- A. runtime と同じ再公開の形にする(中継を 1 段にし、約 15 行減る)
- B. 今のまま
- X. Other (please specify)

[Answer]: A
