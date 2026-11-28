# コード生成の計画: v0.10.4 の大きなファイルの分割

## Summary

- Builds:
  - 3 つの大きなファイル(`walk/mod.rs`、`args.rs`、`report/mod.rs`)を、子のファイルに分ける
  - `build_command` と `render` を関数に分ける
  - 文書の直し
  - 小さな直し 2 つ
  - 版を 0.10.4 にする
- Touches:
  - `crates/core/src/definition/walk/`(`mod.rs` と、子の 3 ファイルを新たに作る)
  - `crates/cli/src/args/`(`args.rs` を移す。子の 3 ファイルを新たに作る)
  - `crates/cli/src/report/`(`mod.rs` と、子の 4 ファイルを新たに作る)
  - `crates/cli/src/lib.rs`、`crates/core/src/lib.rs`、`crates/core/src/error.rs` の文書コメント
  - `crates/cli/src/tests/report_messages.rs` の `with_all_events`
  - `docs/spec/rust/` の 02・07・08・12 章
  - `aidlc/spaces/default/memory/team.md` の 1 行
  - 版の 22 ファイル
- Tests:
  - 新しいテストは無い。
  - 既存のテストがすべて、名前と期待の値を変えずに通ることで、動きが変わらないことを確かめる。
  - `with_all_events` の一覧の直し(FR4.4)だけは、テストの中身を直す。

出典:
- 機能の設計: `functional-spec.md` の 0〜7 章、`rules.md` の BR1.1〜BR6.3、`entities.md`
- 要件: `requirements.md` の FR1.1〜FR5.1、NFR1〜NFR6
- 設計の承認のときに直すと決めたレビューの指摘: R-01・R-02。下の「設計のレビューからの持ち越し」に反映した。

本番のコードの動きは変えない(NFR1)。公開の範囲も変えない(NFR2)。依存は足さない(NFR4)。

## Testing Contract

```json
{
  "version": 1,
  "methodology": "test-after",
  "source": "team",
  "ordering": "各テスト可能な層を実装してから、その層のテストを書いて実行する。",
  "scope": "refactor",
  "test_strategy": "minimal",
  "project_type": "brownfield",
  "applicable_notes": [
    {
      "layer": "org",
      "text": "We treat tests as a first-class deliverable in every Bolt. The specific\nmethodology (TDD, BDD, ATDD, or classic test-after) is affirmed at\npractices-discovery and recorded in `team.md` under this heading with explicit\n`Methodology` and `Ordering` fields; Code Generation resolves those fields\nindependently from coverage, tooling, and scope notes.\n\nWhen no posture has been affirmed, our default per scope is:\n- **Methodology**: test-after\n- **Ordering**: implement each applicable testable layer, then write and run\n  that layer's tests.\n- `mvp`, `enterprise`, `feature`, `infra` add an 80% line-coverage\n  floor and CI execution before merge.\n- `bugfix`, `security-patch` add a targeted regression for the specific\n  bug/vulnerability and require the existing suite to remain green.\n- `express` uses the Minimal strategy: requirement-driven unit tests (one per\n  requirement, with a happy-path floor per component); existing tests remain\n  green.\n- `poc`, `refactor`, `workshop`, `classic` add no extra new-test floor and require the\n  existing suite to remain green.\n\nThe active `Test Strategy` still applies in every scope and determines test\nvolume/types. Scope floors are additive; they never reduce or replace the\nselected strategy.\n\nBuild and Test verifies defined coverage floors and affirmed quality targets;\nthey may not be weakened to make a step pass.\n\nAffirm a stricter posture in `team.md` if the team commits to one."
    },
    {
      "layer": "team",
      "text": "- **Methodology**: test-after\n- **Methodology evidence**: チームは test-after を続けると決めた(インタビュー Q3)。履歴も整合する。v0.9.0 の U4 では、`create_file` の追加(602d15c)の後にそのテスト(90a27fa)、ひな形の生成(0fbd466)の後にひな形のテスト(5b750f8)が続く。U3 も流れ(a9eb237)の後に流れのテスト(365f45c)と実バイナリのテスト(062c627)が続く。`code-quality-assessment.md` も「チームの方針(test-after、行カバレッジ 80%)と一致する」と記す。\n- **Ordering**: 各テスト可能な層を実装してから、その層のテストを書いて実行する。\n- テストの種類: 各クレートの `tests/` に統合テスト(core 36、runtime 16、cli 27、converter 2 のファイル。`tests/` の直下の `.rs` の数)。単体テストは各モジュールの `mod tests` と、`crates/cli/src/tests/`(12 ファイル)、`crates/converter/src/tests/`(4 ファイル)(どちらも `mod.rs` と補助を含むすべての `.rs` の数)。`#[test]` は合計 1,472(`grep -rE '^\\s*#\\[test\\]' crates --include='*.rs'` の行の数)。\n- 流れのテストは偽の `Runtime`/`Sink`/`Env`/`Signals` を注入する。実バイナリのテストは `conductor` とテスト専用の補助バイナリを動かす。非機能のテスト(`nfr_*.rs`、`*_perf.rs`)、網羅のテスト(`ALL` 配列)、参照出力との一致(`graph_layout_oracle.rs`)、文書との一致(`src/tests/docs.rs`、`msrv_consistency.rs`)、デモのテスト(`unix_demo.rs`、`windows_demo.rs`)を含む。テスト名は日本語。\n- テストの都合で本番のコードを曲げない。テスト専用の引数・`pub`・関数を本番に残さず、流れのテストは公開の入口と差し替え口で書く(インタビュー Q6)。\n- 行カバレッジ 80% を下限とする。CI の `coverage` ジョブが `cargo llvm-cov --fail-under-lines 80` で強制する(テスト専用の補助バイナリは除外)。\n- 主な開発環境の OS は macOS で、テストはローカルの macOS で実行する。他の OS のテストは GitHub Actions の CI(`test`: ubuntu、macos、windows)で確認する。マージ前に CI を通す(インタビュー Q3)。\n- 最小の Rust(1.100.0)でもテストを実行する(`msrv-test`)。\n- 起動の性能(`nfr_perf.rs`)は `startup-bench` で測るが、警告のみで失敗にしない(`STARTUP_BENCH_ENFORCE=false`、`continue-on-error`)。\n- 機能を足す intent でも、test-after と 80% の下限、単体テストと実バイナリを動かす統合テストを続ける(インタビュー Q3)。\n- ステートの中身を調べるテスト(`nfr_content.rs`)は、開始と終了の時刻だけを許す(v0.10.0 の B6 で直した)。それ以外(コマンド・環境変数・出力)は検査する(インタビュー Q7)。"
    }
  ],
  "obligations": {
    "strategy": "minimal",
    "strategy_volume": [
      "One verifiable test per requirement at the narrowest effective level.",
      "At least one happy-path unit test per component.",
      "Unit tests are the default; a bugfix/security scope floor may require an integration or E2E regression when that is the narrowest level that reproduces the defect."
    ],
    "scope_floor": [
      "Keep the existing test suite green.",
      "This scope adds no extra new-test floor beyond the selected test strategy."
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
  "input_sha256": "sha256:84a6164ed16cd4376e081f73cdeeadb6cc3cfabd12040daa45aa36f88beb8bbd",
  "contract_sha256": "sha256:c9b2f53228841856ea483cb5b2aecdb683253246a1719a035daae2d423a83405"
}
```

テストの進め方は test-after に従う。

- この作業でテストできる層は、既存のテストがそのまま覆っている。対象は次の 3 つ。
  - core の定義の統合テスト
  - `src/tests/args_grammar.rs`
  - `src/tests/report_messages.rs` と cli の統合テスト
- 移すコミットと分けるコミットのたびに、関わるテスト(`unit-test-instructions.md`)を走らせる。
- 層の終わり(walk、args、report)ごとに、全体のテストを走らせる。
- plan_profile にある層(データベース、API、画面)は、この作業には無いので扱わない。

## データの形の変更

- 無い。型、ステート、定義の書式、表示の文は変えない。

## 設計のレビューからの持ち越し(この段階で扱う)

- **R-01**: 分け終わりの主な基準を、「`functional-spec.md` の 2.1・3.1・4.1 の移す項目が `mod.rs` に無いこと」にする。
  - 行数は参考値にする。目安は、`walk/mod.rs` が 420 行、`args/mod.rs` が 340 行、`report/mod.rs` が 250 行。行数が超えても止めず、`code-summary.md` に実測を書く。
  - `render` の基準は「各枝はメソッドを 1 つ呼ぶだけ」にする。`cargo fmt` で行が折れてもよい。
  - `build_command` の基準は「3.2 の関数を順に呼ぶだけ」にする。40 行を参考値にする。
- **R-02**: 3.2 の関数の間の受け渡しを、次のとおりに決める(Step 7)。
  - `collect_tokens` は `(Given, Vec<OsString>)` を返す。
  - `command_kind` は、位置引数のイテレータ(`vec.into_iter()`)を `&mut` で受け取り、先頭を取る。
  - `check_options` は `given.count(Opt::RunId)` で見るだけにし、値は取り出さない。
  - `fill_options` は、最初に `given.take(Opt::RunId)` で値を取り出し、今と同じ順(`--assume` → `--format` → `--interval` → `--run-id`)で解釈する。`--to` の値も一緒に返す。
  - `check_operands` は、同じイテレータの残りから、1 つ目の位置引数と余りの引数を見る。
  - `fill_operand` は、`kind`、`&mut CliCommand`、1 つ目の位置引数、`--to` を受け取る。
  - 分けるコミットでは、`src/tests/args_grammar.rs` の誤りの順のテストを名指しで走らせる(`unit-test-instructions.md`)。

## ステップ

### Step 1: 準備(テストの道具と出発点)

- [x] `unit-test-instructions.md` のコマンドを変更の前に走らせ、結果を `code-summary.md` に記録する。記録するのは、テストの数、`#[test]` の数(1,472)、3 つのファイルの行数(746・695・679)。

### Step 2〜4: `walk/mod.rs` を分ける(移すだけ。FR2、BR2.1〜BR2.4)

各ステップでは、移した後に `cargo build -p conductor-core --locked` と、core のテスト(`unit-test-instructions.md`)を走らせてからコミットする。`env.rs`・`retry.rs` には差分を出さない。

- [x] Step 2: `string`、`boolean`、`exit_codes`、`command`、`exit_code_of`、`integer_of` を `walk/values.rs` に移す。
  - `mod.rs` に `mod values;` と `use values::{exit_code_of, integer_of};` を置く。
  - コミット: `core: walk の値の読み取りを values.rs に移す`
- [x] Step 3: `RawTransition`、`transitions`、`transition_on` を `walk/transition.rs` に移す。
  - `mod.rs` に `use transition::RawTransition;` を置く。
  - コミット: `core: walk の遷移を transition.rs に移す`
- [x] Step 4: `cross_check`、`check_exit_codes` を `walk/cross_check.rs` に移す。
  - `mod.rs` の `//!` にある子モジュールの説明と、`crates/core/src/lib.rs:57` の説明を、分けた後の形に直す。
  - `const PIPELINE_KEYS`・`const JOB_KEYS` を含む行を、今の定義より前に置かない(BR2.2)。
  - コミット: `core: walk のジョブをまたぐ検査を cross_check.rs に移す`
- [x] `cargo test -p conductor-core --locked` と `cargo test -p conductor-cli --test spec_numbers --locked` を走らせる。

### Step 5〜8: `args.rs` を分ける(FR3、BR3.1〜BR3.4)

- [x] Step 5: `git mv crates/cli/src/args.rs crates/cli/src/args/mod.rs` を行う。中身は変えない。
  - コミット: `cli: args.rs を args/mod.rs にする`
- [x] Step 6: `functional-spec.md` の 3.1 のとおり、オプションの表と検査を `args/options.rs` に、字句を `args/tokens.rs` に、値の解釈を `args/values.rs` に移す。子ごとに 1 コミット(3 コミット)にし、各コミットの前に args のテストを走らせる。
  - コミット: `cli: args のオプションの表を options.rs に移す`
  - コミット: `cli: args の字句を tokens.rs に移す`
  - コミット: `cli: args の値の解釈を values.rs に移す`
- [x] Step 7: `build_command` を、3.2 の 6 つの関数と、`convert_operand`・`init_operand` に分ける(R-02 の受け渡しで)。
  - 返す誤りの種類、対象、順は変えない。
  - `args_grammar.rs` の 69 のテストと、cli の統合テストを走らせる。
  - コミット: `cli: build_command を手順ごとの関数に分ける`
- [x] Step 8: `crates/cli/src/lib.rs:47-59` のモジュールの表の `args` の行を直す。
  - コミット: `cli: lib.rs のモジュールの表を args の分割に合わせる`
- [x] `cargo test -p conductor-cli --locked` を走らせる。

### Step 9〜12: `report/mod.rs` を分ける(FR1、BR1.1〜BR1.5)

- [x] Step 9: `StateContext`、`ResultDetail`、`DisplayEvent` と `level`、`FramePrefix`、`CLEAR_SCREEN` を `report/event.rs` に移す。
  - `mod.rs` に `pub use event::{DisplayEvent, FramePrefix, ResultDetail, StateContext};` を置く。
  - コミット: `cli: report の出来事の定義を event.rs に移す`
- [x] Step 10: `MAX_DISPLAY_CHARS`、`escape_name`、`escape_full`、`push_line` を `report/escape.rs` に移す。
  - `mod.rs` に `pub(crate) use escape::{escape_full, escape_name, push_line};` を置く。
  - コミット: `cli: report のエスケープを escape.rs に移す`
- [x] Step 11: 1 行の組み立ての 12 の関数を `report/lines.rs` に移す。
  - コミット: `cli: report の 1 行の組み立てを lines.rs に移す`
- [x] Step 12: `report/render.rs` を作る。`render` の 35 の枝の中身を、`functional-spec.md` の 4.2 の 9 つの節の、変種ごとのメソッドに写す。`render` は網羅の `match` で、各枝はメソッドを 1 つ呼ぶだけにする。
  - `mod.rs` の `//!` と、`crates/cli/src/lib.rs` の `report` の行を直す。
  - `src/tests/report_messages.rs` の全テスト(特に 694 行と 726 行)と、cli の統合テストを走らせる。
  - コミット: `cli: render の枝を表示の形ごとのメソッドに分ける`
- [x] `cargo test -p conductor-cli --locked` と `cargo test -p conductor-cli --test nfr_source --locked` を走らせる。

### Step 13: 小さな直し(FR4.3、FR4.4、BR4.3、BR4.4)

- [x] `crates/core/src/error.rs:176` の `report::escape` を、`report::escape_full`(`report/escape.rs`)に直す。
  - コミット: `core: error.rs の文書コメントの古い名前を直す`
- [x] `src/tests/report_messages.rs` の `with_all_events` に `DefinitionValid` と `ResetAllKept` を足し、そのテストを走らせる。
  - コミット: `cli: with_all_events に足りない 2 つの変種を足す`

### Step 14: 仕様書(FR4.1、BR4.1)

- [x] `functional-spec.md` の 6.1 の箇所を、分けた後の形に直す(02・07・08・12 章)。12 章の、段の時点の記述は残す。
- [x] `grep -rn 'args\.rs\|report/mod\.rs の\|walk/mod\.rs' docs/spec/rust/` で残りを確かめ、今の形として書いた行が無いことを `code-summary.md` に記録する。
- [x] コミット: `docs: rust 版の仕様書のモジュールの記述を分けた後の形にする`

### Step 15: team.md(FR4.2、BR4.2)

付録 A の 1 行だけを書き換える。今の文の一部で場所を確かめ、見つからなければ止めて報告する。

- [x] `git diff aidlc/spaces/default/memory/team.md` で、その 1 行だけが変わったことを確かめる。
- [x] コミット: `aidlc: team.md の使い方の表の場所を args の分割に合わせる`

### Step 16: 版を 0.10.4 にする(FR5.1、BR5.1)

- [x] `git show --stat ad775bc` で範囲を確かめる。同じ 22 ファイルの `0.10.3` を `0.10.4` にする。`Cargo.lock` は 4 つのクレートの版の行だけを直す。
- [x] 次の 4 つを通す。
  - `cargo build --locked`
  - `cargo test --workspace --locked`
  - `cargo fmt --all -- --check`
  - `cargo clippy --workspace --all-targets --locked -- -D warnings`
- [x] コミット: `版を 0.10.4 にする`

### Step 17: まとめと記録

- [x] `code-summary.md` に次のことを書く。
  - 変えたファイル
  - 分け終わりの判定(R-01 の主な基準と、行数の実測)
  - テストの数(前後で同じであること)
  - 残した事項
- [x] `traceability.json` に、FR1.1〜FR5.1 と、それを満たすファイルを書く。v0.10.3 と同じく、`upstream_ids` には BR と NFR も含める。

## 付録 A: team.md の書き換え(FR4.2。行は 43cc050 の時点の目安)

| 節 | 行 | 今の文 | 書き換えた後の文 |
|---|---|---|---|
| Code Style | 67 | `- README は必須語句の検査(`docs-check`)つきで保つ。使い方の表は `args.rs`、`report/messages.rs`、README の 3 か所を揃える。` | `- README は必須語句の検査(`docs-check`)つきで保つ。使い方の表は `args/`(`mod.rs` の文法の表と `options.rs` のオプションの表)、`report/messages.rs`、README の 3 か所を揃える。` |

- 決まりの中身(3 か所を揃える)は変えない。
