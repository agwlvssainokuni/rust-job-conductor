# コード生成の計画(Code Generation Plan): args.rs のオプションの表化

## Summary

- Builds: `args.rs` のオプションの表 `OPTIONS` と検査の順の表 `CHECKS`、字句と組み立ての段階の書き換え、`convert --to` の誤りの対象を `--to` に、`enter_flow` の doc コメントの修正、仕様書の更新
- Touches: `crates/cli/src/args.rs`、`crates/cli/src/dry_run.rs`(コメントだけ)、`crates/cli/tests/args_grammar.rs`(テストを足すだけ。既存のテストは変えない)、`docs/spec/rust/{04-cli,10-errors-and-exit-codes}.md`、`docs/spec/language-agnostic/{04-cli,10-errors-and-exit-codes,11-quality-and-build}.md`、必要なら `docs/spec/rust` の `args.rs` の内部の形を書いた箇所
- Tests: 既存のテストは 1 行も変えない。表化の前に、入り組んだ報告の順を固定するテストを足す。`--to` の誤りの対象のテストを足す

設計の出典: `construction/functional-design/functional-spec.md`(§1〜§7)、`rules.md`(BR1.1〜BR5.1)、`entities.md`。追跡の ID は成功の指標 `SM1`〜`SM5`。テストは Minimal、test-after、行カバレッジ 80% の下限(Build and Test が全体で確かめる)。新しい依存、`unsafe` は足さず、`src/` に `cfg` を足さない。

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
      "text": "- **Methodology**: test-after\n- **Methodology evidence**: 基準(前回のインタビュー Q3)でチームは test-after を選んだ。履歴も整合する。たとえば `[job.retry]` では、型・読み取りの実装(d554ece、445d478)の後に、テストを追加するコミット(8ec98b5)が続き、実行の流れ(19d7b1a)の後に README のテスト(de4e3ad)が続く。`code-quality-assessment.md` も「チームの方針(test-after、行カバレッジ 80%)と一致する」と記す。\n- **Ordering**: 各テスト可能な層を実装してから、その層のテストを書いて実行する。\n- テストの種類: 各クレートの `tests/` に統合テスト(core 10、runtime 13、cli 21、converter 5 のファイル)。流れのテストは偽の `Runtime`/`Sink`/`Env`/`Signals` を注入する。実バイナリのテストは `conductor-cli-test-helper` を使う。非機能のテスト(`nfr_*.rs`)、網羅のテスト(`ALL` 配列)、デモのテスト(`unix_demo.rs`、`windows_demo.rs`)を含む。テスト名は日本語。\n- 行カバレッジ 80% を下限とする。CI の `coverage` ジョブが `cargo llvm-cov --fail-under-lines 80` で強制する(テスト専用の補助バイナリは除外)。\n- ローカルでは、主な開発環境の OS でテストを実行する。それ以外の OS のテストは GitHub Actions の CI(`test`: ubuntu、macos、windows)で実行する。マージ前に CI を通す。\n- 最小の Rust(1.100.0)でもテストを実行する(`msrv-test`)。\n- 起動の性能(`nfr_perf.rs`)は `startup-bench` で測るが、警告のみで失敗にしない(`STARTUP_BENCH_ENFORCE=false`、`continue-on-error`)。\n- 今回の機能にも、test-after と 80% の下限、単体テストと実バイナリを動かす統合テストを続ける(インタビュー Q3)。\n- 主な開発環境の OS は macOS で、テストはローカルの macOS で実行する。他の OS は CI で確認する(インタビュー Q6)。"
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
  "input_sha256": "sha256:c5ec90c86cfde4bc3f3a63db01255b93f6cdb74000081f6d363ccea531e3e68a",
  "contract_sha256": "sha256:f4f95e97f799def5c5b22a16d75ab141601b6d1a9476cc0f7b322bd0d1aafcd6"
}
```

この intent の層は「引数の読み取り(Business logic にあたる)」の 1 つだけ。データモデル、データアクセス、API、画面の層は無い。

## 手順

- [x] Step 1: 既存のテストの実行環境を確かめ、基準の件数を記録する(`cargo test --locked --workspace`。基準: 1,117 通過、1 無視。引数だけなら `cargo test --locked -p conductor-cli --test args_grammar`)。
- [x] Step 2: 報告の順を固定するテストを `args_grammar.rs` の末尾に足す(今の動きのまま通ること)。`--force` の重複が `--assume` の不可より先、`--assume` の不可が `--dry-run` の重複より先、「使えない」がすべての「重複」より先(例: `watch -v -v` は `-v` の不可、`graph --format a --format b -v -v` は `--format` の重複)。今のテストがすでに確かめている組は足さない。テスト名は日本語。コミット。(BR2.1。SM2)
- [x] Step 3: `OPTIONS`(`Opt`、`OptSpec`、`Allowed`)を足し、字句の段階を表から行う形にする(`functional-spec.md` 3.1、3.3)。`Token` を `Opt(Opt, Option<OsString>)`、`Unknown`、`Positional` にする。`-h`/`--help`/`-V`/`--version` は表より先。`map_lex_error` の集合を表から作る。コミット。(BR1.1、BR1.2。SM1)
- [x] Step 4: `CHECKS` を足し、組み立ての段階の「使えない」と「重複」を表から行う形にする(3.2。並びは仕様書の 04 章 4.2 の 6・7 の順)。オプションごとの値を `Opt` で引く 1 つの並びにする。`CHECKS` のあとの検査は今の順のまま。コミット。(BR1.3。SM1)
- [x] Step 5: 引数の層のテストを実行する: 既存のテストを変えずに全体が通り、件数が基準 + Step 2 で足した分であること。`git diff` で既存のテストの行が変わっていないこと。`functional-spec.md` 2 の表の「今」の場所に、オプションごとの `match` の腕と `if` が残っていないこと。(BR2.1、BR5.1。SM1、SM2)
- [ ] Step 6: `--to` の誤りの対象を `--to` にする(値を含めない。UTF-8 でない値も)。テスト `to_の値の誤りの対象は_to_で値を含まない`(`json`、`YAML`、空文字)と、UTF-8 でない値のテスト(既存の `#[cfg(unix)]` のテストと同じ置き方)を足す。仕様書を直す: `docs/spec/rust/04-cli.md` の 73・90・111・291 行、`docs/spec/rust/10-errors-and-exit-codes.md` の 233 行、`docs/spec/language-agnostic/04-cli.md` の 79・134・327 行、`docs/spec/language-agnostic/10-errors-and-exit-codes.md` の 241 行、`docs/spec/language-agnostic/11-quality-and-build.md` の 611 行(AC-237)と 752 行(AC-337)の期待とテストの列。凍結した 5 つの仕様書は直さない。全体のテストを実行する。コミット。(BR3.1、BR3.2。SM3)
- [ ] Step 7: `dry_run.rs:32` の `enter_flow` の doc コメントを「位置引数が無ければ、使い方の誤り(68)。コマンドからは `parse_args` が先に止めるが、公開の型 `CliCommand` を直接組み立てて呼ばれたときの守り」の趣旨に直す。分岐は残す。コミット。(BR4.1。SM4)
- [ ] Step 8: `docs/spec/rust` のうち `args.rs` の内部の形(`Token` の変種、`build_command` の `if` の並びなど)を書いた箇所があれば、表の形に合わせる。無ければ何もしない。コミット。(SM1)
- [ ] Step 9: 全体の確認: `cargo fmt --all -- --check`、`cargo clippy --workspace --all-targets --locked -- -D warnings`、`cargo test --locked --workspace` が通ること(行カバレッジは Build and Test が確かめる)。(BR5.1。SM5)
- [ ] Step 10: 文書と追跡: `code-summary.md`(件数、足したテストの一覧、表の形、仕様書の変更の一覧、版の更新の候補であること)、`traceability.json` を作る。

## 守ること

- 利用者から見える変更は `--to` の誤りの対象だけ。ほかの誤りの種類、対象、報告の順、出力、終了コードは変えない。
- 既存のテストを 1 行も変えない(足すだけ)。
- 新しい外部ライブラリを足さない(`Cargo.toml`、`Cargo.lock` に差分を出さない)。`unsafe` を足さない。`src/` の `platform` の外に `cfg` を足さない。
- 公開の型(`CliCommand`、`ParsedArgs`、`UsageError`、`UsageErrorKind`)の形を変えない。
- コミットは手順ごとの小さな単位。各コミットでビルド、clippy、全体のテストが通ること。メッセージは日本語。
- ファイルは、ファイルの作成と編集のツールで書く(シェルの sed、perl、python で書き換えない)。
- 範囲の外: `CliCommand` の型の形、`resolve_run_id`、`--to` 以外の誤りの対象、凍結した 5 つの仕様書。

## Assumptions

なし(新しい内部の型 `Opt`、`OptSpec`、`Check` は、設計の `entities.md` に書いた)。
