# Code Generation Plan — u3-retry-display-docs(表示の文言の確定と文書)

## Summary

- Builds: リトライの前の知らせの文言(英語と日本語)の確定と、README へのリトライの説明(項目の書き方、動き、シグナル、at-least-once との関係)の追加。README の例が実際に通ることを確かめるテストを 1 つ足す
- Touches: `README.md`(目次、定義ファイルの書き方の表と新しい小節、ステートファイルの場所の「自動の再試行はしません」の一文、終了コードの表の 64、保証の範囲、シグナルの扱い)、`crates/cli/src/report/messages.rs`(文言を直す必要が出たときだけ)、`crates/cli/tests/report_messages.rs`(文言を直したときだけ。u2 が足したテストの更新)、`crates/cli/tests/docs_retry.rs`(新規)
- Tests: 新規の統合テスト 1 ファイル(約 4 件)。既存のテストは変更なしで通す。CI の `docs-check`(README の必須の語の検査)と同じ検査を、ローカルで実行する

## 根拠

- 要件: FR6.2(英語と日本語の文言の確定)、FR8.1(README)(`requirements.md`)。意図の整理 Q7: 利用者への周知は README の更新だけ
- 前提: u1(設定の読み取りと検証)、u2(実行、シグナル、知らせ)が完了している。README に書く動きは、実装とテストで確かめた事実だけにする(`unit-of-work.md`、`code-summary.md` の u1、u2、各 `rules.md`)
- この Unit は、もともとあった `README.md` を変更する。要件 FR8.1 と、あなたが選んだ周知の方法(README の更新)が求めるもので、この計画に名前を挙げている
- チームの方針: test-after、`cargo fmt` と `cargo clippy -D warnings`、行カバレッジ 80%、変更は小さく、こまめにコミットする

## 名前を付けて追加するデータの要素

- なし(データの型、誤りの種類、表示の種類は、追加しない)。文言(固定の文)を、確定するだけ

## 実装ステップ(test-after: 各層を実装してから、その層のテストを書いて実行する)

- [ ] Step 1: 変更前の基準を取る。`cargo test --workspace --locked` が全件通ることを記録する。CI の `docs-check` と同じ検査(README に、`SHA-256`、`Gatekeeper`、`終了コードの表`、`ステートファイルの場所`、`at-least-once`、`シグナル`、`convert` の 7 語があること)を、ローカルで実行して通ることを確かめる
- [ ] Step 2: 文言の確定。u2 が書いた最初の文言(英語: `conductor: retry: job 'a' exited with code 75; retry 2 of 3 in 40 seconds`、日本語: `conductor: retry: ジョブ 'a' が終了コード 75 で終わりました。40 秒後にリトライします(2/3 回目)`)を、既存の文言(`messages.rs` の他の文、README の用語)と照らして確認する。直す必要があれば、`messages.rs` の文言と、u2 が足した `report_messages.rs` のテストを直す(既存のアサーションは変えない)。直さない場合は、その確認の結果を `code-summary.md` に記録する。表示のレベル(`-v` なしで、常に標準エラー)は、設計(Q3: A)のとおりで、変更しない。(FR6.2)
- [ ] Step 3: README の実装(文書の層)。次を書く。(FR8.1)
  - 目次に、リトライの小節を足す
  - 「定義ファイルの書き方」の表に `[job.retry]` の行を足し、直後に小節「リトライの設定(`[job.retry]`)」を作る。例の定義、5 つのキー(`exit_codes`、`max`、`interval`、`multiplier`、`max_interval`)の意味と範囲(終了コードは 1〜255 の整数の一覧、`max` は 1〜100、間隔は `"30s"`・`"2m"`・`"1h"` の形で 1 秒〜24 時間、倍率は 2〜10、`max_interval` は `interval` 以上)、省略の規則(`exit_codes`、`max`、`interval` は必須、`multiplier` と `max_interval` は両方か両方なし)、重なりの禁止(成功・警告の終了コードと重ならない)、定義の誤り(64)になる場合
  - 動き: どんな結果のときにリトライするか(ジョブ自身の終了コードが一覧にあるとき。起動できなかった場合、シグナルで終了した場合、中断は対象外)、回数(最初の実行に加えて最大 `max` 回、合計で `max + 1` 回)、待ち時間(固定と、倍率と上限の式と例)、使い切ったときの扱い(通常のエラーと同じ)、リトライの前の知らせ(標準エラー、`-v` なし、英語と日本語の例)
  - 「ステートファイルの場所」の「**自動の再試行はしません**」を、「リトライの設定を書かなければ、自動の再試行はしません」に直し、リトライの回数はステートに記録しないこと、途中で止まったあとの再開は、そのジョブを最初の実行から数え直すことを書く
  - 「終了コードの表」の 64 に、リトライの設定の誤りを足す。必要なら、128 + 番号の行に、リトライの待機中の中断を足す
  - 「保証の範囲(at-least-once)」に、リトライするジョブは、同じ処理が複数回走るため、再実行されても困らない作りにすること、conductor が落ちたあとの再開で、回数は数え直されることを足す
  - 「シグナルの扱い」に、リトライの待機中にシグナルを受けたときの動き(すぐ待機をやめ、再開位置をそのジョブとして記録し、128 + 番号で終わる。エラーとしては記録しない)を足す
  - 版の表記は、既存の書き方(`(v0.7.0 以降)` など)に合わせて、`(v0.8.0 以降)` とする(次の版が 0.8.0 になるという前提。リリースの版は、あなたが決める)
  - 既存の 7 つの必須の語を、消さない
- [ ] Step 4: 文書のテストを書いて実行する。`crates/cli/tests/docs_retry.rs`(新規)。(a) README に、`[job.retry]` と 5 つのキーの名前が書かれていること、(b) 「自動の再試行はしません」という古い一文が、そのままの形では残っていないこと、(c) README の「リトライの設定」の例の定義(TOML)を取り出して、実際の読み取り(`conductor-core` の `load_definition`)に通すと、誤りなく読めて、リトライの設定が入っていること、(d) CI の `docs-check` と同じ 7 つの必須の語が、README にあること。`cargo test -p conductor-cli --test docs_retry --locked`
- [ ] Step 5: `demo/` は、この Unit では変更しない(デモのスクリプトは、`unix_demo`・`windows_demo` のテストが動きを確かめており、リトライの追加の必要がないため)。変更が必要と分かったら、計画に追記して承認を取る
- [ ] Step 6: 全体の確認。`cargo fmt --all -- --check`、`cargo clippy --workspace --all-targets --locked -- -D warnings`、`cargo test --workspace --locked`、検査コマンド(`cargo llvm-cov ... --fail-under-lines 80`)、`docs-check` の 7 語、`msrv_consistency`(README の MSRV の記載が変わっていないこと)が通ること。新しい外部クレートが増えていないこと。既存のアサーションが変わっていないこと
- [ ] Step 7: 記録。`code-summary.md`、`source-manifest.json`、`traceability.json` を作る

## 触れないもの(この Unit の外)

- 定義の読み取りと検証(u1)、実行・待機・シグナル・知らせの仕組み(u2)
- `demo/`(上の Step 5)、`Cargo.toml`、リリースの版の更新
- 新しい外部クレートは、追加しない

## 要件との対応

| 要件 | ステップ |
|---|---|
| FR6.2 | 2(文言の確定)、4(README の例と文言) |
| FR8.1 | 3、4、6 |

## Assumptions

- README に書く版の表記は `v0.8.0 以降` とする。次のリリースの版は、あなたが決めるため、違っていれば、タグを付ける前に直す
- 文言は、u2 の最初の文言で足りる見込み(直す必要が出たら、Step 2 で直す)
- README の「リトライの設定」の例は、`[job.retry]` を含む、正しい TOML の定義 1 つにする(Step 4 の (c) で、実際に読めることを確かめる)

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
  "input_sha256": "sha256:d5b3fba95d247c0e12f7b2d508688f21b918b16d18e551864079baef0ef0ea0b",
  "contract_sha256": "sha256:d612e34b5f9d41c4f21a42502a2a8d564d0dfa60fd65fe4f577e5714d8511806"
}
```
