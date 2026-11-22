# 意図の把握の質問(Intent Capture Questions): `--dry-run` の模擬の重複をなくすリファクタリング

## Sources

- [desc] Initial description: "--dry-run の模擬の重複をなくすリファクタリング: crates/core/src/plan/simulate.rs の apply_result が、runtime の state/record.rs の apply_outcome とほぼ同じ文で、ステートの更新を写している(コメントに「本番と同じ結果になるよう書く」とある)。更新の関数を core に移し、本番と模擬の両方から呼ぶ。本番は終点の後のスキップなどを apply_skip で記録するが、模擬はスキップで状態を変えない違いがあり、その扱いを決める。"
- [scope] Workflow-selected scope: `refactor`.
- [memory:M1] `aidlc/spaces/default/memory/project.md#Mandated`: "ALWAYS 変更は小さな単位に保ち、小さな単位でレビューする(チームの慣行)。 (affirmed 2026-11-20)"
- [memory:M2] `aidlc/spaces/default/memory/project.md#Mandated`: "ALWAYS 行カバレッジ 80% 以上を保つ(CI の `coverage` ジョブ。`--fail-under-lines 80`)。 (affirmed 2026-11-20)"
- [memory:M3] `aidlc/spaces/default/memory/project.md#Mandated`: "ALWAYS 最小の Rust(MSRV 1.100)でビルドとテストが通ることを保ち、引き上げるときは README の記載と一致させる(`build-check`、`msrv-test`、`msrv_consistency.rs`)。 (affirmed 2026-11-20)"
- [memory:M4] `aidlc/spaces/default/memory/project.md#Mandated`: "ALWAYS `cargo fmt` と `cargo clippy -D warnings` を通してからマージする(CI の `lint`)。 (affirmed 2026-11-20)"
- [memory:M5] `aidlc/spaces/default/memory/project.md#Forbidden`: "NEVER 今回の作業で新しい外部ライブラリ(依存)を追加しない。`.env` の読み込みとグラフの描画は自前で書く(インタビュー Q4)。 (affirmed 2026-11-20)"

## Q1. [プロダクト] スキップのときの模擬の扱い

背景: 本番は、スキップのとき `apply_skip` で、そのジョブを `skipped` にし、再開位置と完了を直します。模擬は、スキップでは何も変えません(`simulate.rs` の「模擬の状態は変えない(BR1.8)」)。読んだ限りでは、`--dry-run` の出力はたいてい同じですが、模擬に古い再開位置が残るため、まれな形で、後のジョブの判断が本番と違う恐れがあります。

- A. 模擬も、本番と同じ `apply_skip` を使う(模擬と本番の更新を完全に 1 つにする)。設計の段階で、今の模擬と出力を比べ、`--dry-run` の出力が変わる形があれば、それを示して確かめてから進める
- B. 結果の記録(`apply_outcome`)だけを共通にし、スキップで状態を変えない今の模擬の扱いは、そのまま残す(出力は確実に変わらない)
- X. Other (please specify)

[Answer]: A

## Q2. [プロダクト] 何が出来上がれば「できた」と言えるか

- A. ステートの更新の関数が core の 1 か所にあり、runtime(本番)と plan(模擬)がそれを呼ぶ。模擬の写し(`apply_result`)はなくなる。runtime の公開の名前(`apply_outcome` など)は、再エクスポートで残す
- B. A に加えて、仕様書(`docs/spec/rust` の実行の流れと `--dry-run` の章)の記述も、共通の更新として書き直す
- X. Other (please specify)

[Answer]: B

## Q3. [プロダクト] 確かめ方

背景: 本番の更新は runtime の state のテスト、模擬は `plan_pipeline.rs`、`plan_run.rs`、`flow_sequences.rs` などが確かめています。

- A. 既存のテストを変えずに、すべて通ることで確かめる(Q1 で A を選び、出力が変わる形を承認したときは、その形のテストだけを直す)。加えて、ランダムな定義とステートで、今の模擬と新しい模擬の出力を比べる確認を、設計の段階で 1 回行う
- B. 既存のテストが通ることだけで確かめる
- X. Other (please specify)

[Answer]: A
