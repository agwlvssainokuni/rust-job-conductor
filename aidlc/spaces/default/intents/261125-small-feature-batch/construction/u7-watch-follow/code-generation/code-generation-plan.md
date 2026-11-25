# コード生成の計画: u7-watch-follow(`watch --follow` と版の 0.10.0)

## Summary

- Builds: `conductor watch --follow [--interval <秒>]` で、ステートを一定の間隔(既定 2 秒、1〜3600)で描き直す。完了・消えた・止まったら最後に描いて 0、Ctrl-C でも 0。UNIX 系の端末は画面を消して描き直し、ほかは追記。リトライの待ちの間は試行を開いたままにする(U6 の流れの直し)。最後に版を 0.10.0 にする
- Touches: core の `flow/activity.rs`(新規)・`flow/mod.rs`・`lib.rs`、cli の `args.rs`・`types.rs`・`aux_commands.rs`・`step.rs`・`platform/`・`report/mod.rs`・`report/messages.rs`、テスト、`README.md`、デモ、版の更新(4 つの `Cargo.toml`、`Cargo.lock`、`binary_basic.rs`、`docs/spec/` の版の記載)
- Tests: 単体テスト約 15(判定の 4 つの値、引数)、流れのテスト約 12(継続表示の読み込みの並び、合図、リトライの待ち)、実バイナリのテスト 3、docs のテスト、デモ

出典: 機能の設計(`functional-spec.md`、`rules.md` の BR1.1〜BR5.2、`entities.md`)、非機能の設計(`reliability-design.md`、`security-design.md`、`performance-design.md`、`scalability-design.md`、`observability-design.md`、`logical-components.md`)、ユニットの定義(`unit-of-work.md` の U7)、要件(`requirements.md` の FR7.1〜FR7.5、FR8.1、FR8.2)、契約(`contract-summary.md` の C5)。ユーザーストーリーは作っていないため、各ステップは要件と規則の ID に対応させる。

テストの進め方は Testing Contract(test-after)に従う。core → cli の順に、層を作ってからその層のテストを書いて走らせる。

## データの形の変更

- `CliCommand.follow: bool`、`CliCommand.interval: Option<u32>`(`entities.md`)。
- core の `PipelineActivity`(`Completed`・`Running`・`Stopped`・`Between`)と `pipeline_activity(state)`(`entities.md`)。
- cli の中の `FollowEnd`(終わった理由。`entities.md`)。
- ステートの形は変えない(U6 のまま)。

## 設計のレビューからの持ち越し(このユニットで扱う)

- 継続表示の繰り返しで持ち越す値の戻し方(非機能の設計のレビュー): ステートを読めた(有る・無い)ときは `read_failures` を 0 に戻す。有る・読めないときは `absent_once` を偽に戻す。消えたと判定するのは「無い」が 2 回続いたときだけ、65 は「読めない」が 3 回続いたときだけ。
- 読めない(`Unreadable`)ときの読み直しは、利用者の間隔ではなく 1 秒後に行う(非機能の設計のレビュー。権限の誤りなどで誤りの表示が長く遅れないように)。3 回続けば 65。
- 画面を消せるかの判定は `platform` の 1 つの関数で、端末かどうかと合わせて流れの初めに 1 回だけ決め、描く関数に真偽で渡す。流れのテストの期待は、今の `flow_aux.rs` と同じく `cfg!(unix)` で分ける(テストの中の `cfg!` は規則で認められている)(非機能の設計のレビュー)。
- リトライの待ちの間は試行を開いたままにし、終わりの時刻をメモリに持つ(`reliability-design.md` の 2、BR3.4)。U6 の `flow_retry.rs` などの書く回数と待ちの間のステートの期待を合わせて直す。
- README に、エラーのジョブの後に続くジョブがある定義では間の短い時間に止まったと判定して終わることがある点を書き、その節の語を `docs.rs` の必須の語句に足す(機能の設計のレビュー、非機能の要件のレビュー)。
- 使い方の誤りの終了コードは 68(質問の Q1 の「2」は誤記。`functional-spec.md` の 4)。

## ステップ

### Step 1: テストの準備を確かめる

- [x] 変更の前に、`unit-test-instructions.md` の 3 つのコマンドを走らせ、結果を記録する(新しいファイルは除く)。

### Step 2: 終わりの判定(core)とその層のテスト

- [x] `flow/activity.rs` に `PipelineActivity` と `pipeline_activity` を足し、`lib.rs` から公開する(BR3.1)。
- [x] 新しい `crates/core/tests/pipeline_activity.rs` に、4 つの値、リトライの待ちの間の形(最後の試行が開いていて `pending` または `error`)が `Running`、エラーのジョブの後の間が `Stopped`、を足す。core のコマンドを走らせる。

### Step 3: リトライの待ちの間の試行(cli の `step.rs`)とその層のテスト

- [x] `run_job_with_retry` で、リトライするときは終わりの時刻を持って書かずに待ち、次の試行の始めに `end_attempt` → `start_attempt` → Fast で 1 回書く。待ちの途中の中断では `end_attempt` をしてから抜ける(BR3.4)。
- [x] `flow_retry.rs` ほかの期待を合わせ、`reliability-design.md` の 2 の (1)〜(4) を確かめるテストを足す。cli の流れのテストを走らせる。

### Step 4: 引数と使い方(cli)

- [x] `types.rs` に `follow`・`interval`、`args.rs` に `--follow`・`--interval <秒>` の文法(1〜3600 の整数、`--follow` と一緒のときだけ、`watch` だけ。誤りは今の使い方の誤りで 68、値は表示しない)と使い方の表(BR1.1、BR1.2)。`messages.rs` の使い方の文(英日)を揃える。
- [x] `args_grammar.rs` に、正しい場合と誤りの場合(0、3601、小数、負、文字、`--follow` の無い `--interval`、`watch` 以外)を足す。

### Step 5: 継続表示の流れと描き方(cli)

- [x] `platform/` に「画面を消せるか」の関数(UNIX 系は真、Windows は偽)。`report/` に、画面を消す文字列と、追記の区切りを前に置く描き方(BR2.2)。
- [x] `aux_commands.rs` の `run_watch` で、`--follow` のときの繰り返し(合図の準備、読む、描く、判定、待つ。上の持ち越しの戻し方と読み直し)を足す。`--follow` が無ければ今のまま(BR1.1、BR2.1、BR3.1〜BR3.3、BR4.1、BR4.2)。

### Step 6: cli の層のテストを書いて走らせる

- [x] `flow_aux.rs` に、継続表示の読み込みの並び(有る → 無い → 有る で続く、無いが 2 回で 0、読めない 2 回 → 有る で続く、読めないが 3 回で 65、壊れたで 65、無い → 有る(始まる前から見る))、完了と止まったで 0、待ちの途中の合図で 0、合図の準備の失敗で 70、書き込みと削除が 0 回、1,000 回の繰り返しを足す。
- [x] 実バイナリのテスト(`binary_basic.rs` か新しいファイル)に、完了のステートで 1 回描いて 0、`--interval 0` で 68 で値が出ない、パイプの出力に `ESC` が無いことを足す。
- [x] cli のコマンドを走らせる。

### Step 7: デモと README

- [x] デモに `watch --follow` の場面(完了したステートを見て終わる)を足し、`demo/README.md`、`unix_demo.rs`・`windows_demo.rs` を合わせる。`unix_demo` を走らせる。
- [x] `README.md` の使い方に `--follow`・`--interval` を足し、継続表示の動き(間隔、描き直しと追記、終わる条件、終了コード、ステートが無いときに待つこと、エラーの後に続くジョブがある定義での注意)を書く。`docs.rs` の必須の語句を足し、`cargo test -p conductor-cli --lib --locked -- tests::docs` を走らせる。

### Step 8: 書式と静的検査

- [x] `cargo fmt --all -- --check` と `cargo clippy --workspace --all-targets --locked -- -D warnings` が通ることを確かめる(NFR7.1)。

### Step 9: 版を 0.10.0 にする

- [x] 「版を 0.10.0 にする」の 1 コミットで、今の版の更新(1f88e59 の「版を 0.9.9 にする」)と同じ範囲を直す。4 つのクレートの `Cargo.toml`、`Cargo.lock` の自分のクレートの版、`crates/cli/tests/binary_basic.rs` の版の期待、`docs/spec/` の版の記載。`cargo test --workspace --locked` が通ることを確かめる(BR5.2、FR8.2)。

## 規則と要件の対応

| ステップ | 規則 | 要件 |
|---|---|---|
| Step 2 | BR3.1 | FR7.3 |
| Step 3 | BR3.4 | FR7.3 |
| Step 4 | BR1.1、BR1.2 | FR7.1、FR7.2 |
| Step 5 | BR2.1、BR2.2、BR3.1〜BR3.3、BR4.1、BR4.2 | FR7.1〜FR7.5 |
| Step 2、3、6 | すべて | FR7.1〜FR7.5、NFR1〜NFR6 |
| Step 7 | BR5.1 | FR8.1 |
| Step 8 | | NFR7 |
| Step 9 | BR5.2 | FR8.2 |

## コミットの分け方

小さな単位でコミットする(件名は日本語、先頭に `core:`・`cli:`・`docs:`)。各コミットで fmt と clippy が通るようにし、テストの期待の直しは振る舞いを変えるコミットと同じコミットに入れる(途中のコミットでテストが落ちないように)。例: Step 2 は core の 2 コミット、Step 3 は 1〜2 コミット、Step 4・5・6 は cli の 3〜5 コミット、Step 7 は `docs:`、Step 9 は「版を 0.10.0 にする」(先頭の語を付けない。今までの版の更新と同じ)。

## Testing Contract

```json
{
  "version": 1,
  "methodology": "test-after",
  "source": "team",
  "ordering": "各テスト可能な層を実装してから、その層のテストを書いて実行する。",
  "scope": "feature",
  "test_strategy": "standard",
  "project_type": "brownfield",
  "applicable_notes": [
    {
      "layer": "org",
      "text": "We treat tests as a first-class deliverable in every Bolt. The specific\nmethodology (TDD, BDD, ATDD, or classic test-after) is affirmed at\npractices-discovery and recorded in `team.md` under this heading with explicit\n`Methodology` and `Ordering` fields; Code Generation resolves those fields\nindependently from coverage, tooling, and scope notes.\n\nWhen no posture has been affirmed, our default per scope is:\n- **Methodology**: test-after\n- **Ordering**: implement each applicable testable layer, then write and run\n  that layer's tests.\n- `mvp`, `enterprise`, `feature`, `infra` add an 80% line-coverage\n  floor and CI execution before merge.\n- `bugfix`, `security-patch` add a targeted regression for the specific\n  bug/vulnerability and require the existing suite to remain green.\n- `express` uses the Minimal strategy: requirement-driven unit tests (one per\n  requirement, with a happy-path floor per component); existing tests remain\n  green.\n- `poc`, `refactor`, `workshop`, `classic` add no extra new-test floor and require the\n  existing suite to remain green.\n\nThe active `Test Strategy` still applies in every scope and determines test\nvolume/types. Scope floors are additive; they never reduce or replace the\nselected strategy.\n\nBuild and Test verifies defined coverage floors and affirmed quality targets;\nthey may not be weakened to make a step pass.\n\nAffirm a stricter posture in `team.md` if the team commits to one."
    },
    {
      "layer": "team",
      "text": "- **Methodology**: test-after\n- **Methodology evidence**: チームは test-after を続けると決めた(インタビュー Q3)。履歴も整合する。v0.9.0 の U4 では、`create_file` の追加(602d15c)の後にそのテスト(90a27fa)、ひな形の生成(0fbd466)の後にひな形のテスト(5b750f8)が続く。U3 も流れ(a9eb237)の後に流れのテスト(365f45c)と実バイナリのテスト(062c627)が続く。`code-quality-assessment.md` も「チームの方針(test-after、行カバレッジ 80%)と一致する」と記す。\n- **Ordering**: 各テスト可能な層を実装してから、その層のテストを書いて実行する。\n- テストの種類: 各クレートの `tests/` に統合テスト(core 33、runtime 14、cli 25、converter 2 のファイル)。単体テストは各モジュールの `mod tests` と、`crates/cli/src/tests/`(11 ファイル)、`crates/converter/src/tests/`(4 ファイル)。`#[test]` は合計 1,170。\n- 流れのテストは偽の `Runtime`/`Sink`/`Env`/`Signals` を注入する。実バイナリのテストは `conductor` とテスト専用の補助バイナリを動かす。非機能のテスト(`nfr_*.rs`、`*_perf.rs`)、網羅のテスト(`ALL` 配列)、参照出力との一致(`graph_layout_oracle.rs`)、文書との一致(`src/tests/docs.rs`、`msrv_consistency.rs`)、デモのテスト(`unix_demo.rs`、`windows_demo.rs`)を含む。テスト名は日本語。\n- テストの都合で本番のコードを曲げない。テスト専用の引数・`pub`・関数を本番に残さず、流れのテストは公開の入口と差し替え口で書く(インタビュー Q6)。\n- 行カバレッジ 80% を下限とする。CI の `coverage` ジョブが `cargo llvm-cov --fail-under-lines 80` で強制する(テスト専用の補助バイナリは除外)。\n- 主な開発環境の OS は macOS で、テストはローカルの macOS で実行する。他の OS のテストは GitHub Actions の CI(`test`: ubuntu、macos、windows)で確認する。マージ前に CI を通す(インタビュー Q3)。\n- 最小の Rust(1.100.0)でもテストを実行する(`msrv-test`)。\n- 起動の性能(`nfr_perf.rs`)は `startup-bench` で測るが、警告のみで失敗にしない(`STARTUP_BENCH_ENFORCE=false`、`continue-on-error`)。\n- v0.10.0 の機能にも、test-after と 80% の下限、単体テストと実バイナリを動かす統合テストを続ける(インタビュー Q3)。\n- ステートの中身を調べるテスト(`nfr_content.rs`)は、B6 のために開始と終了の時刻だけを許すように直す。それ以外(コマンド・環境変数・出力)は今までどおり検査する(インタビュー Q7)。"
    }
  ],
  "obligations": {
    "strategy": "standard",
    "strategy_volume": [
      "Five to eight tests per component.",
      "Unit tests plus integration tests for key boundaries.",
      "Add E2E, performance, or security tests when requirements demand them."
    ],
    "scope_floor": [
      "Meet an 80% line-coverage floor.",
      "Run the selected tests in CI before merge."
    ],
    "combination_rule": "Apply every selected-strategy obligation and every scope-floor obligation; neither replaces the other, and a targeted scope regression may add the narrowest necessary test type beyond the strategy default."
  },
  "plan_profile": {
    "methodology": "test-after",
    "runner_step": "Verify the existing test runner/configuration and record the exact unit-scoped command.",
    "runner_ready_before_first_test": true,
    "testable_layers": [
      "Data model / database behavior",
      "Repository / data access",
      "Business logic",
      "API / endpoint",
      "Frontend behavior"
    ],
    "steps": [
      "Project structure and production configuration skeleton.",
      "Verify the existing test runner/configuration and record the exact unit-scoped command.",
      "Data model / database behavior - implement.",
      "Data model / database behavior - write and run its tests after implementation.",
      "Repository / data access - implement.",
      "Repository / data access - write and run its tests after implementation.",
      "Business logic - implement.",
      "Business logic - write and run its tests after implementation.",
      "API / endpoint - implement.",
      "API / endpoint - write and run its tests after implementation.",
      "Frontend behavior - implement.",
      "Frontend behavior - write and run its tests after implementation.",
      "Environment/build configuration.",
      "Documentation and traceability."
    ]
  },
  "input_sha256": "sha256:ed0086a4c967568436c54943aebacd308a988af279eee16ed426cf4f39a154fa",
  "contract_sha256": "sha256:3789717991ddc007f6cbca5d56d6b2d0b248de4ce1061637170c6787b5bf88e9"
}
```
