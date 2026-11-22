# 意図の記述(Intent Statement): 動きを変えない整理の第 2 弾

## Sources

- [desc] Initial description: "動きを変えない整理の第 2 弾(調査の候補のうち B3・F・JobExecution の列挙型化を除く全部): (1) 説明のずれ(apply_skip の説明、Plain、escape、lib.rs の表、config_path・65・read_state_raw、definition_key の仮、仕様書 11 章のテストの件数)と制約(core・converter の forbid(unsafe_code)、上限の数と定数の const assert)、(2) 小さな重複と計算量(status_at の共有と pending_ids の O(n²)、merge_env の O(E²)、既定の定義ファイル名、@end と Transition の target の型、<file>:<line>、graph の text と Mermaid の共通、使い方の誤りと出力の失敗の補助、終了コードの補助、ステートの削除、escape を core の sanitize に)、(3) cli の ctx の整理(report_execution の移動、Checked と Session、prepare_run と prepare_checked、resolve_run_id の置き場所)、(4) 型とテスト(run ID の型、スキップと拒否の理由の型、PlanMode/RunMode・EdgeKind/TransitionOn の重複、flow_with_state.rs を今の流れに)。利用者から見える動きは変えない"
- [scope] Workflow-selected scope: `refactor`.
- [Q1] `intent-capture-questions.md` Q1: A(承認は 1 回、段の順に進め、結果は最後にまとめる)
- [Q2] `intent-capture-questions.md` Q2: A(公開の型の変更に伴う既存のテストの変更は、組み立ての値と import だけ認める。設計で一覧を示して承認を取る)
- [Q3] `intent-capture-questions.md` Q3: A(`flow_with_state.rs` を今の cli の流れに合わせる)
- [Q4] `intent-capture-questions.md` Q4: A(上限に近い入力の性能のテストを足し、前後を測る)
- [memory:M1] `aidlc/spaces/default/memory/project.md#Mandated`: "ALWAYS 変更は小さな単位に保ち、小さな単位でレビューする(チームの慣行)。 (affirmed 2026-11-20)"
- [memory:M2] `aidlc/spaces/default/memory/project.md#Mandated`: "ALWAYS 行カバレッジ 80% 以上を保つ(CI の `coverage` ジョブ。`--fail-under-lines 80`)。 (affirmed 2026-11-20)"
- [memory:M3] `aidlc/spaces/default/memory/project.md#Mandated`: "ALWAYS 最小の Rust(MSRV 1.100)でビルドとテストが通ることを保ち、引き上げるときは README の記載と一致させる(`build-check`、`msrv-test`、`msrv_consistency.rs`)。 (affirmed 2026-11-20)"
- [memory:M4] `aidlc/spaces/default/memory/project.md#Mandated`: "ALWAYS `cargo fmt` と `cargo clippy -D warnings` を通してからマージする(CI の `lint`)。 (affirmed 2026-11-20)"
- [memory:M5] `aidlc/spaces/default/memory/project.md#Forbidden`: "NEVER 今回の作業で新しい外部ライブラリ(依存)を追加しない。`.env` の読み込みとグラフの描画は自前で書く(インタビュー Q4)。 (affirmed 2026-11-20)"

## Problem Statement

- ドキュメントコメントと仕様書の一部が、今の実装とずれている(初期の記述の `apply_skip` の説明、`Plain`、escape、`lib.rs` の表、`config_path`・65・`read_state_raw`、`definition_key` の仮、仕様書 11 章のテストの件数)。読む人を誤らせる。[desc]
- unsafe を使わないことや、理由の文に書いた上限の数が、コンパイラや検査で守られていない。[desc]
- 小さな重複が残り、2 か所で上限に近い入力のときに二乗の時間がかかる。[desc] [Q4]
- cli の `ctx.rs` に表示の組み立てがあり、同じ形の型と手順が重なっている。[desc]
- 文字列で表している種類や、あり得ない組み合わせを許す型がある。runtime のテストの 1 つが、今と違う古い流れを再現している。[desc] [Q3]
- 解決したいのは、利用者から見える動きを変えずに、これらを直すことである。[desc]

## Target Customer

- このコマンドラインツールのコードとテストを保守し、機能を足す開発者。仕様書を読む実装者。[desc]

## Success Metrics

- 段 1(説明と制約): 挙げたドキュメントコメントと仕様書の記述が、実装と一致する。core と converter に `#![forbid(unsafe_code)]` がある。理由の文に書いた上限の数が、定数とコンパイル時の検査で結ばれる。[desc]
- 段 2(小さな重複と計算量): 挙げた重複が 1 か所にまとまる。`@end` に着いたときの残りのジョブの列挙と、`.env` の合成が、入力の大きさに比例する時間で終わる。上限に近い入力の性能のテストを足し、直す前と後の時間を記録する。[desc] [Q4]
  - 入力の大きさ: `.env` は 1 ファイルの上限の 10,000 項目(`crates/core/src/envfile/mod.rs:40` の `MAX_ENV_ENTRIES`)。ジョブは既存の性能のテストと同じ 1,000 個(`crates/core/tests/plan_perf.rs:25` の `N`)。 [Q4]
  - 合格の時間の上限と、直す前に対する余裕の割合は、設計の段で直す前の測定をもとに数で決める。 [Q4]
- 段 3(cli の ctx): 表示の組み立てが `ctx.rs` の外にあり、`Checked` と `Session`、`prepare_run` と `prepare_checked` の重複がなく、run ID の解決が引数の解釈の外にある。[desc]
- 段 4(型とテスト): run ID の指定、スキップの理由と拒否の理由、同じ意味の列挙型が、型で表され、到達しない分岐と文字列での比較がなくなる。`flow_with_state.rs` が今の cli の流れ、名前、期待になる。[desc] [Q3]
- 利用者から見える動き(出力、終了コード、ステートファイル、`--dry-run` の出力)は変わらない。既存のテストは変えずに通る。例外として、公開の型の変更で組み立ての値と import を直す必要があるテストと、`flow_with_state.rs` は変えてよい。前者は期待する出力の文字列を変えず、設計の段で一覧と理由を示して承認を取る。[desc] [Q2] [Q3]
  - 確かめる手段: 段の前と後でテストの件数を記録する。合格は、失敗 0 で、件数が「基準 + 足したテスト」に一致すること。`flow_with_state.rs` は残して今の流れに合わせる(Q3: A)ので、原則として件数は変えない。合わせる中で件数が変わるときは、その増減と理由を記録する。[Q1] [Q3] [Q4]
  - 段 1〜4 のすべての項目について、設計の段で今の場所(ファイルと行)と目標の形を一覧にし、実装の後に、その場所に重複、ずれ、到達しない分岐、文字列での比較が残っていないことを確かめる。[Q1]
- 合格の条件: `cargo fmt --all -- --check` と `cargo clippy --workspace --all-targets --locked -- -D warnings` が通る [memory:M4]。行カバレッジ 80% 以上 [memory:M2]。MSRV 1.100 でのテストが通る [memory:M3]。新しい依存を足さない [memory:M5]。

## Initiative Trigger

- v0.9.5 のあとの調査で挙がった候補のうち、動きを変えないものをまとめて片付ける。動きが変わる B3(core の定義ファイルの読み込みの順序)は別の intent にする。[desc]

## Initial Scope Signal

- Workflow-selected scope: `refactor`(workflow-selected)[scope]
- 範囲: 段 1〜4。[desc]
- 範囲の外: B3、F、`JobExecution` の列挙型化。[desc]
  - 補足(調査の結果の記録): B3 は core の定義ファイルの読み込みの順序(`crates/core/src/definition/intake.rs` の `read_definition`)で、利用者から見える変更になるため別の intent にする。F は `run_job`(`crates/runtime/src/runner.rs`)に分割済みのコマンドを渡すもので、Runtime トレイトと仕様書の 4 つの章に触れる。 [desc]
  - 辺の一覧(`EdgeList`)は定義からは到達しないが、仕様書(`docs/spec/rust/05`、`08`)と README が防御として残すと明記しているので、触らない。 初期の記述の「利用者から見える動きは変えない」に従う。[desc]
  - 凍結した 5 つの仕様書(`docs/spec/{go,python,typescript,csharp,powershell}`)は、前の intent `261122-internal-cleanup` で中身を変えないと決めている。段 1 で直す仕様書 11 章は `docs/spec/rust/11-quality-and-build.md` で、凍結した 5 つには含まれない。直す仕様書は `docs/spec/rust` と `docs/spec/language-agnostic` だけ。 初期の記述の「仕様書 11 章」はこの 11 章を指す。[desc]
- 進め方: 設計とコードの計画は 1 回で承認し、実装は段の順に、小さなコミットで進める。結果は最後にまとめて見せる。[Q1] [memory:M1]

## Assumptions & Open Questions

None.
