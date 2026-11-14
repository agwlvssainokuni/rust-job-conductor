# Code Generation Plan(u1-core)

## Summary

- Builds: 定義ファイルの読み込みと検証、結果の分類・次の動き・完了・再開位置の判断(Rust のライブラリ 1 つ)
- Touches: `Cargo.toml`(ワークスペース)、`crates/u1-core/`(ソースとテスト)、`.gitignore`(`target/` の 1 行)
- Tests: 約 55 件(単体と、定義ファイルを使う統合テスト。Standard の方針と、行カバレッジ 80% の下限)

## 前提(設計から)

- 上流: functional-spec(W1〜W4)、rules(BR1.1〜BR5.1)、entities、contract-summary(C1・C2・E2)、security-design(読み取りの流れ、D0〜D5、既存の成果物との差分)、logical-components、cicd-pipeline、tech-stack-decisions。
- 設計の成果物どうしで食い違う点は、security-design の「既存の成果物との差分」を優先する。設計の成果物(entities.md など)は書き換えず、差分は `code-summary.md` に記録する。
- テストの順序は、チームの実践(test-after)に従い、**層ごとに、実装してから、その層のテストを書いて実行する**。
- Rust のコードは、ワークスペースの根(`Cargo.toml`)と、`crates/u1-core/`(パッケージ名 `conductor-core`)に置く。他の Unit のクレートは、`crates/` に並べる。
- 公開する関数(contract C2): `load_definition`、`decide_before_run`、`classify_and_decide`、`resume_point_on_interrupt`。引数と戻り値は、entities.md の型。
- 起動前の判断で「スキップのあとに完了を判定する」のは、Cli がスキップをステートに記録したあと、`resume_point_on_interrupt` を呼ぶことで得る(最後のジョブがスキップなら `completed` が true)。新しい関数は足さない。この呼び方は、u3-cli の設計で確かめる。

## 実装の手順

### 層 0: 準備

- [x] **Step 1: ワークスペースとテストの実行環境(test runner readiness)**
  - ワークスペースの根の `Cargo.toml`(`members = ["crates/*"]`、resolver 3)、`crates/u1-core/Cargo.toml`(edition 2024、`rust-version` は edition 2024 を使える最低)、`crates/u1-core/src/lib.rs`(空のクレート)を作る。
  - `.gitignore` の末尾に `target/` を 1 行足す(Rust のビルド成果物を除くため。これがないと `cargo` の実行で、成果物がコミットの対象になる。ほかの行は変えない)。
  - 依存: 読み取りのライブラリ(位置の情報を持つ文書の木を提供する、`toml` の仲間。`toml` の `toml::de::DeTable`、または `toml_edit` の `ImDocument` のどちらか。security-design の「Assumptions」)、`thiserror`。開発用に `tempfile`。**バージョンは、その時点の安定版を選び、`Cargo.lock` で固定する。**
  - ライブラリの確認(security-design の確認項目): (a) 要素ごとのバイトの範囲が取れること、(b) 入れ子の深さ 1000 の入力で、異常終了せず失敗が返ること。確認の結果と選んだライブラリを `code-summary.md` に記録する。(b) が満たされなければ、読み取りを別スレッドで行う代替(security-design の D4)を実装する。
  - 実行できることを確かめる: `cargo test -p conductor-core --locked`(テスト 0 件で成功)。
  - 関わる: NFR2.1、NFR3.1、NFR5.1、NFR4.7。

### 層 1: 共有する型とエラー(Types、Describe、Locate)

- [x] **Step 2: 共有する型(`types.rs`)**
  - entities.md の型: `PipelineDefinition`(name、description、format_version、jobs)、`JobDefinition`(id、command、warning_exit_codes、success_exit_codes、transitions)、`Transition`(on: `Success`/`Warning`、next: `JobId(text)` または `End`)、`JobExecution`、`JobState`(status: `Pending`/`Success`/`Warning`/`Error`/`Skipped`)、`SkipReason`(`ByTransition`/`ByEnd`)、`PipelineState`、`Decision`(`Run`/`Skip`/`Reject`)と `DecisionReason`(`SkipTarget`/`AfterEnd`/`AlreadyExecuted`/`OutOfOrder`)、`JobOutcome`、`Classification`、`NextStep`、`RunMode`(`Individual`/`Pipeline`。functional-spec の W3、BR3.6 による)、`ResumePoint`(resume_job_id、end_reached、completed。契約 C2 の `resume_point_on_interrupt` の出力)。
  - `JobDefinition::effective_success_codes()`(省略時は `[0]`)を持つ。入出力に依存しない(NFR2.1、NFR3.1)。
  - 関わる: contract C1、BR1.11、BR2.1。
- [x] **Step 3: エラーの型と表示(`error.rs`、Describe、Locate)**
  - `DefinitionErrorKind`: fileMissing、unreadable、tooLarge(追加)、syntax(追加)、typeMismatch、unknownKey、versionMissing、versionUnsupported、unknownTarget、errorTransition、backwardOrSelfTarget、overlappingExitCodes、zeroAsWarning、pipelineNameMismatch、duplicateJobId、invalidJobId、noJobs、emptyField、duplicateTransition。`DefinitionError`(file、line、kind、reason)。`thiserror` で表示の文を付ける(固定の文だけ。ライブラリのエラー文、ソースの抜粋、誤った値は含めない)。
  - キー名など入力に由来する文字列は、制御文字をエスケープし、64 文字までに切って表示する。`Locate`: バイトの位置から行番号を求める(改行は LF と CRLF の両方)。
  - 関わる: NFR4.6、NFR4.1、NFR4.2、BR1.1〜BR1.13(種類)。
- [x] **Step 4: 層 1 のテスト**: 型の既定値(成功コードの省略時)、表示の文に入力の値が入らないこと、キー名のエスケープと切り詰め、行番号(LF、CRLF、先頭、末尾)、`DefinitionErrorKind` の表示。

### 層 2: 定義の読み込みと検証(DefinitionLoader)

- [x] **Step 5: 取り込み(`definition/intake.rs`、Intake)**
  - ファイルの場所を決める(BR4.1: 指定がなければ、実行ディレクトリの `conductor.toml`)。大きさの確認と、「上限(1 MiB)+ 1 バイト」までの読み取り。超えたら `tooLarge`。存在しなければ `fileMissing`。権限がない・通常のファイルでない・UTF-8 でないものは、`unreadable`(理由で区別)。
  - 関わる: BR1.9、BR4.1、NFR4.4、AC1.3.1、AC1.3.2、AC1.3.3。
- [x] **Step 6: 読み取りと深さと版(`definition/parse.rs`、`depth.rs`、`version.rs`)**
  - 位置を保つ文書の木として読み取る。文法の誤りは `syntax`(行つき)。木を走査して入れ子の深さを数え、32 段を超えたら `syntax`(テーブル、テーブルの配列、インラインのテーブル、配列を 1 段ずつ数える)。版のキー(`format_version`)を確認し、無ければ `versionMissing`、1 でなければ `versionUnsupported` を返して、そこで終える(他の検査はしない)。
  - 関わる: BR1.7、NFR4.3、NFR4.7、AC1.2.4。
- [x] **Step 7: 構造の検査と定義の組み立て(`definition/walk.rs`、Walk)**
  - 木を自分で走査し、許されたキーの集合(`format_version`、`pipeline.{name,description}`、`jobs[].{id,command,warning_exit_codes,success_exit_codes,transitions}`、`transitions[].{on,next}`)と照らして `unknownKey`、期待する型と照らして `typeMismatch`(必須の値の欠落を含む。パイプラインが 2 つ以上ある `[[pipeline]]` も `typeMismatch`。BR1.12)を返す。
  - 構造の検査: ジョブIDの重複・空・`@` の接頭(BR1.13)、ジョブが 0(BR1.13)、空の name・command(BR1.13)、同じ `on` の遷移の重複(BR1.13)、遷移先の存在(BR1.1)、エラーへの遷移(BR1.2)、自分自身・前のジョブへの遷移(BR1.3)、終了コードの重複と 0 の警告指定(BR1.4)、`expected_pipeline_name` との一致(BR1.8)。
  - 誤りは、**すべて集めて**返す(各誤りに、親の要素の位置を使った行を付ける)。型の誤りで検査できなくなった子の、後続の検査は、その子だけ省略し、ほかの検査は続ける。誤りがなければ、`PipelineDefinition`(ジョブの順は書いた順。BR1.11)を返す。
  - 関わる: BR1.1〜BR1.6、BR1.8、BR1.11〜BR1.13、NFR4.1、NFR4.2、NFR4.5、AC1.1.1〜AC1.1.4、AC1.2.1〜AC1.2.5。
- [x] **Step 8: 公開の入口(`load_definition`、`parse_definition`)**
  - `load_definition(path: Option<&Path>, expected_pipeline_name: Option<&str>) -> Result<PipelineDefinition, Vec<DefinitionError>>`。取り込みから結果までの流れ(Intake → Parse → Depth → VersionCheck → Walk)を組み立てる。入出力に触れるのは Intake だけ。ファイルに触れずに試せるよう、`parse_definition(text, file_label, expected_pipeline_name)` を内部の入口として公開する。
  - 関わる: contract C2(`load_definition`)、NFR4.5。
- [x] **Step 9: 層 2 のテスト**(`tests/definition_*.rs`、`tests/fixtures/*.toml`、各モジュールの単体テスト)
  - 取り込み: 存在しない、大きさの上限(ちょうど 1 MiB は受理、1 MiB + 1 は `tooLarge`)、UTF-8 でない、ディレクトリを指定。
  - 読み取り・深さ・版: 文法の誤り、深さ 32 は受理、33 は `syntax`、1000 でも異常終了しない、版なし、版 2、版の誤りのとき他の誤りは出ない。
  - 構造: BR1.1〜BR1.13 の各規則に、違反の例と、受理の例。複数の誤りが、1 回で、すべて返ること。行番号が正しいこと(CRLF を含む)。
  - 統合: 正しい定義(mockups の例に近い 3〜4 ジョブ)を、ファイルから読み、ジョブ・遷移・既定の成功コードが期待どおりであること。

### 層 3: 判断(Flow)

- [x] **Step 10: 起動前の判断(`flow/decide.rs`、`decide_before_run`)**: BR3.5 の 7 つの順序どおり(skipTarget、afterEnd、alreadyExecuted、run、最初のジョブ、順番どおりの run、outOfOrder の reject)。ジョブの記録がない場合は pending と同じ扱い。純粋な関数(入出力なし)。関わる: BR3.5、BR3.8、AC2.2.2、AC2.3.3、AC4.3.1。
- [x] **Step 11: 分類と返す終了コード(`flow/classify.rs`)**: BR2.1(成功・警告・エラー。成功コードを書いたら、その値だけ)、BR2.2(警告は 0、エラーは終了コードのまま、成功コードに含まれない 0 は 1 と `conductor_error`)、BR2.3(起動しなかった中断 → 中断 → 起動失敗 127 → シグナル 128 + 番号 → 通常、の優先順位)、BR2.4(起動しなかった中断は分類せず、再開位置にし、128 + 番号)。関わる: AC2.1.3〜AC2.1.6、AC2.3.1、AC2.3.2。
- [x] **Step 12: 次の動きと完了(`flow/next_step.rs`、`classify_and_decide`)**: BR3.1(遷移先、`@end`、既定の遷移 BR1.10)、BR3.2(間のジョブのスキップ)、BR3.3(`@end` 以降のスキップ)、BR3.4(エラーで停止、再開位置、再実行の成功・警告で解除)、BR3.6(完了。`RunMode` による `@end` の扱いの違い)。Step 11 の分類と合わせて、`classify_and_decide(job, execution, states, definition, run_mode) -> (JobOutcome, NextStep)` を公開する。関わる: BR1.10、BR3.1〜BR3.4、BR3.6、AC1.1.2、AC1.1.3、AC2.1.6、AC2.2.1、AC2.2.3、AC2.2.4、AC3.1.1〜AC3.1.3、AC3.2.1〜AC3.2.3、AC4.2.1〜AC4.2.4、AC4.3.2、AC4.3.4。
- [x] **Step 13: ジョブのあいだの中断の再開位置(`flow/resume.rs`、`resume_point_on_interrupt`)**: BR3.7(まだ起動していない最初のジョブ。最後の直後なら再開位置なしで `completed`。`endReached` を引き継ぐ。エラーとしては記録しない)。スキップの記録のあとの完了の判定にも使う(前提を参照)。関わる: BR3.7、BR3.6、AC4.3.1。
- [x] **Step 14: 層 3 のテスト**(`flow/` の単体テストと `tests/flow_*.rs`): 判断の各分岐を、表形式のテストで、境界まで網羅する。Flow は、ファイルとプロセスに触れずに試せることを、テストの書き方(入出力なしの引数だけ)で示す。BR2.3 の優先順位(中断と終了コード、起動失敗とシグナルが同時のとき)、`successExitCodes` を書いたときの 0、`individual` と `pipeline` の `@end`、再開の一連の流れ(エラー → 再実行 → 解除 → 完了)。

### 層 4: 横断

- [x] **Step 15: 公開の API の文書(`lib.rs`、`///` の説明)**: 公開する型と 4 つの関数に、入出力・失敗・規則の ID を書く。モジュールの構成(Intake など、logical-components の呼び名との対応)を `lib.rs` の説明に書く。関わる: contract C1・C2。
- [x] **Step 16: 横断のテスト(`tests/nfr_*.rs`)**:
  - NFR1.1: ジョブ 10 個の定義を 100 回読み、p95 を標準のライブラリの時計で求めて、標準エラーに出力する(基準 20ms を超えても、失敗にしない。Q1: A)。`GITHUB_STEP_SUMMARY` があれば、そこにも書き出す。
  - NFR4.5: `parse_definition` が、ファイルを作らず、環境変数も読まず、コマンドを解釈しないこと(コマンドの文字列が、そのまま保持されること)。
  - NFR4.6: すべての `DefinitionErrorKind` で、表示の文に、入力の値が現れないこと。
- [x] **Step 17: 確認(Build and Test の前の、このユニットの自己確認)**: `cargo fmt --all -- --check`、`cargo clippy -p conductor-core --all-targets --locked -- -D warnings`、`cargo test -p conductor-core --locked` が通ること。`cargo llvm-cov` が使えれば、このクレートの行カバレッジを測って記録する(使えなければ、「測っていない」と `code-summary.md` に書く。80% の判定は Build and Test が、全体のテストで行う)。

## テストの方針(Standard)

- 部品ごとに 5〜8 件を目安に、判断の規則は規則ごとに網羅する。単体テストは `#[cfg(test)]` のモジュールと `tests/`、定義ファイルの例は `tests/fixtures/`。
- 行カバレッジ 80% 以上(下限。設定でしきい値を下げない)。
- ファイルを使うテストは `tempfile` で一時ディレクトリを作る。外部のサービスやネットワークは使わない。

## 設計からの追跡(ストーリー → 手順)

| ストーリー | 手順 |
|------------|------|
| US1.1(AC1.1.1〜AC1.1.4) | Step 7、8、9、12 |
| US1.2(AC1.2.1〜AC1.2.5) | Step 6、7、9 |
| US1.3(AC1.3.1〜AC1.3.3) | Step 5、8、9 |
| US2.1(AC2.1.3〜AC2.1.6。AC2.1.1・AC2.1.2 は u2-runtime) | Step 11、12、14 |
| US2.2(AC2.2.1〜AC2.2.4) | Step 10、12、14 |
| US2.3(AC2.3.1〜AC2.3.3) | Step 10、11、14 |
| US3.1(AC3.1.1〜AC3.1.3。AC3.1.4 は u3-cli) | Step 12、14 |
| US3.2(AC3.2.1〜AC3.2.3) | Step 12、14 |
| US4.2(AC4.2.1〜AC4.2.4) | Step 12、13、14 |
| US4.3(AC4.3.1、AC4.3.2、AC4.3.4。AC4.3.3 は u2-runtime) | Step 10、12、13、14 |
| NFR1.1 / NFR2.1 / NFR3.1 | Step 1、16 |
| NFR4.1〜NFR4.7 | Step 3、5、6、7、9、16 |
| NFR5.1 | Step 4、9、14、17 |

## Testing Contract

```json
{
  "version": 1,
  "methodology": "test-after",
  "source": "team",
  "ordering": "各テスト可能な層を実装してから、その層のテストを書いて実行する。",
  "scope": "mvp",
  "test_strategy": "standard",
  "project_type": "greenfield",
  "applicable_notes": [
    {
      "layer": "org",
      "text": "We treat tests as a first-class deliverable in every Bolt. The specific\nmethodology (TDD, BDD, ATDD, or classic test-after) is affirmed at\npractices-discovery and recorded in `team.md` under this heading with explicit\n`Methodology` and `Ordering` fields; Code Generation resolves those fields\nindependently from coverage, tooling, and scope notes.\n\nWhen no posture has been affirmed, our default per scope is:\n- **Methodology**: test-after\n- **Ordering**: implement each applicable testable layer, then write and run\n  that layer's tests.\n- `mvp`, `enterprise`, `feature`, `infra` add an 80% line-coverage\n  floor and CI execution before merge.\n- `bugfix`, `security-patch` add a targeted regression for the specific\n  bug/vulnerability and require the existing suite to remain green.\n- `express` uses the Minimal strategy: requirement-driven unit tests (one per\n  requirement, with a happy-path floor per component); existing tests remain\n  green.\n- `poc`, `refactor`, `workshop`, `classic` add no extra new-test floor and require the\n  existing suite to remain green.\n\nThe active `Test Strategy` still applies in every scope and determines test\nvolume/types. Scope floors are additive; they never reduce or replace the\nselected strategy.\n\nBuild and Test verifies defined coverage floors and affirmed quality targets;\nthey may not be weakened to make a step pass.\n\nAffirm a stricter posture in `team.md` if the team commits to one."
    },
    {
      "layer": "team",
      "text": "- **Methodology**: test-after\n- **Methodology evidence**: インタビュー Q3 で、チームは test-after(実装してからテストを書く)を選んだ。\n- **Ordering**: 各テスト可能な層を実装してから、その層のテストを書いて実行する。\n- `cargo test` による単体テストと、バイナリを実行する統合テストを含める。\n- 行カバレッジ 80% を下限とする(インタビュー Q4: A)。\n- 主な開発環境の OS では、ローカルでテストを実行する。それ以外の OS(Linux・macOS・Windows のうち該当するもの。制約 C3)のテストは、GitHub Actions の CI で実行する。マージ前に CI を通す。"
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
    "runner_step": "Bootstrap the minimal test runner/configuration and record the exact unit-scoped command.",
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
      "Bootstrap the minimal test runner/configuration and record the exact unit-scoped command.",
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
  "input_sha256": "sha256:ce0f69628897190511501dd3c515b27d7bab3d0e15b2a2103f574b66ace49bcb",
  "contract_sha256": "sha256:6bd9ef79a3010e02afd33449a0c5b558cbba318b21e4e7e1f917ad9a9138e14f"
}
```
