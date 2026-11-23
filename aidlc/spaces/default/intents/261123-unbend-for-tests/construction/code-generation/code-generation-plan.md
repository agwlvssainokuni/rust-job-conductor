# コードの計画(Code Generation Plan): テストの都合で曲げられた本番のコードと、残りの整理

上流: `construction/functional-design/functional-spec.md`(§2 の段ごとの設計、§4 のテストの変更 T1〜T31、§4.6 の消すテスト 5 件、§5 の足すテスト N1、§6 の件数の見込み、§7 の確かめ方、§8 のコミットの並び 1〜25、§10 の決定 D1〜D12 と承認済みの項目)、`rules.md`(BR)、`entities.md`、`inventory.md`(変える場所、§0 の基準、§8 の検索のコマンド、完了の判定)。

方針: テストのコードの都合で本番のコードを曲げない(ユーザーの判断)。本番から、テストのためだけの要素をなくし、テストの側で代わりを持つ。本番に、テストのための要素を新しく足さない。

方法: 小さなコミットで `main` に直接コミットし、各コミットでビルド、clippy、テストが通るようにする。push、タグ、リリースはしない。版は上げない。

## 守ること

- 利用者から見える動き(出力、終了コード、ステートファイルのバイト列、`--dry-run` の出力)を変えない。
- 既存のテストの変更は、承認済みの T1〜T31 と、消すテスト(§4.6 の 5 件)だけ。確かめる中身(利用者から見える動き)と期待する出力の文字列は変えない。名前の変更は ★ T1・T2・T30 だけ。
- 新しい依存を足さない(`Cargo.toml`・`Cargo.lock` の差分なし)。`unsafe` と src の `cfg(unix|windows)` を足さない。runtime の製品のコードに `.unwrap()`・`.expect(`・`panic!(`・`unreachable!(`・`todo!(`・`unimplemented!(` を書かない。cli の src の単体テストでも `panic!(` を使わない(`nfr_source.rs`)。converter は core に依存しない。誤りの表示に入力の値を含めない。
- 凍結した 5 つの仕様書(`docs/spec/{go,python,typescript,csharp,powershell}`)は変えない。
- ドキュメントコメントとテスト名は日本語。
- 想定外に、T1〜T31 と §4.6 の外の既存のテストを変えないと進めない項目が出たら、その項目は進めずに止めて報告する。

## 段 1: テストの都合で曲げられた所(コミット 1〜13)

- [x] Step 1: 言語と色の判定の確認用のメソッド(1a)、`reset --all` の数(1b)、`is_lane_drawable`(1c)、`LoadedEnvFiles::add`(1d)、`remove_stale_temps` の `now`(1e)を本番から消し、テストの側で代わりを持つ。コミット 1〜5。
- [x] Step 2: 誤りの種類の一覧 `ALL` を各クレートの `tests/common/kinds.rs` へ移し(網羅の `match` と、並びと重複のコンパイル時の確かめ。D1・D2)、本番の `ALL` 5 つを消す。消すテスト(§4.6 の core・converter の単体 2 件、cli の `args_grammar.rs` の 1 件)。コミット 6〜9。
- [x] Step 3: 種類の識別子 `code()` を消し、種類の説明に識別子の規則を書く(1f。★ T1・T2)。コミット 10。
- [x] Step 4: パニックのフックの差し替え口をなくし、本体を非公開の関数にして単体テストへ移す(1h)。「フックを設定すると 70 で終わる」の 1 件は消し、確かめられなくなることを仕様書に記録する。コミット 11。
- [x] Step 5: シグナルの `AlreadyRegistered` と登録済みの印をなくし、「登録はプロセスで 1 回だけ」を `Signals::start`・`run_cli_with`・`run_cli` の説明と仕様書に書く(1i、D5。★ T30。§4.6 の 1 件)。コミット 12。
- [x] Step 6: `node_state_of` を非公開に、`flow_edges` を `pub(crate)` に(1j、D8)。性能のテスト P4 の前後の時間を記録する。コミット 13。
- [x] Step 7: 段 1 の完了の判定(`inventory.md` の検索で、本番のコードからの参照が 0 件)と件数を記録する(見込み 1,156)。

## 段 2: ふつうの整理(コミット 15〜25)

- [x] Step 8: 古くなった説明(2a。仕様書の `.env` の重複キーの警告の 6 か所を含む)。コミット 15〜16。
- [x] Step 9: ステートの `definition_key` と `job_order` を型から消す(2b、D12)。ステートファイルのバイト列が前後で同じことを確かめて記録する。コミット 17。
- [x] Step 10: `CommandError` の理由を `CommandErrorKind::reason()` に(2c、D6。T31 を含む)、`ExitCodeClass`(2d)、小さい整理(2e)。コミット 18〜22。
- [x] Step 11: `decide_from_state`(2f、D7)と N1(D11)。コミット 23〜24。
- [x] Step 12: 仕様書 11 章のテストのファイルの表と件数。コミット 25。段 2 の後の件数を記録(見込み 1,157)。

## 最後

- [x] Step 13: `functional-spec.md` §7 の確かめ方(検索と、段の前後の `cargo test --locked --workspace -- --list` の比較)を走らせて結果を記録する。fmt、clippy、全テスト、カバレッジを通す。
- [x] Step 14: `code-summary.md`(コミットの一覧、件数の移り変わり、テストの変更・名前の変更・消したテストの一覧、検索の結果、ステートファイルのバイト列の比較、計画からのずれ)、`traceability.json`(SM と BR のすべてを、変えたファイルに結ぶ)、`memory.md`。

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
