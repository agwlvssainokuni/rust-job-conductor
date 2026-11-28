# コード生成の計画: v0.10.3 の仕様書の粗の直しと数の検査

## Summary

- Builds:
  - 仕様書の小さな粗の直し(A-1)
  - `team.md` の古い記述の書き換え(A-2)
  - 仕様書の数を実装と突き合わせるテスト `crates/cli/tests/spec_numbers.rs`(A-3)
  - 版を 0.10.3 にする
- Touches:
  - `docs/spec/language-agnostic/11-quality-and-build.md`
  - `crates/core/tests/env_types.rs` のテスト名 1 つ
  - `crates/cli/tests/spec_numbers.rs`(新規)
  - `aidlc/spaces/default/memory/team.md`
  - テストで見つかった仕様書のずれがあれば、その行
  - 版の 22 ファイル
- Tests:
  - `spec_numbers.rs` の数ごとのテスト(12 個)
  - 効くことの確かめ(一時的にずらして落ちることを見る。コミットしない)
  - 既存のテストすべて

出典:
- 機能の設計: `functional-spec.md` の 1〜6 章、`rules.md` の BR1.1〜BR4.1、`entities.md`
- 要件: `requirements.md` の FR1.1〜FR4.1、NFR1〜NFR5
- 設計の承認で直すと決めたレビューの指摘(R-01〜R-04): 下の「設計のレビューからの持ち越し」に反映した。R-05 は当てはまらない(2026-11-28 に人が「署名は見送ります。効果がさほど見込めないので。」と答えた)ので、4 章 #8 の文は変えない
- Units とユーザーストーリーは無い。そのため、各ステップは要件と規則の ID に対応させる

本番のコード(`crates/*/src/`)は、版の外を変えない(NFR1、NFR2)。依存は足さない(NFR4)。

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
      "text": "- **Methodology**: test-after\n- **Methodology evidence**: チームは test-after を続けると決めた(インタビュー Q3)。履歴も整合する。v0.9.0 の U4 では、`create_file` の追加(602d15c)の後にそのテスト(90a27fa)、ひな形の生成(0fbd466)の後にひな形のテスト(5b750f8)が続く。U3 も流れ(a9eb237)の後に流れのテスト(365f45c)と実バイナリのテスト(062c627)が続く。`code-quality-assessment.md` も「チームの方針(test-after、行カバレッジ 80%)と一致する」と記す。\n- **Ordering**: 各テスト可能な層を実装してから、その層のテストを書いて実行する。\n- テストの種類: 各クレートの `tests/` に統合テスト(core 36、runtime 16、cli 26、converter 2 のファイル。`tests/` の直下の `.rs` の数)。単体テストは各モジュールの `mod tests` と、`crates/cli/src/tests/`(12 ファイル)、`crates/converter/src/tests/`(4 ファイル)(どちらも `mod.rs` と補助を含むすべての `.rs` の数)。`#[test]` は合計 1,460(`grep -rE '^\\s*#\\[test\\]' crates --include='*.rs'` の行の数)。\n- 流れのテストは偽の `Runtime`/`Sink`/`Env`/`Signals` を注入する。実バイナリのテストは `conductor` とテスト専用の補助バイナリを動かす。非機能のテスト(`nfr_*.rs`、`*_perf.rs`)、網羅のテスト(`ALL` 配列)、参照出力との一致(`graph_layout_oracle.rs`)、文書との一致(`src/tests/docs.rs`、`msrv_consistency.rs`)、デモのテスト(`unix_demo.rs`、`windows_demo.rs`)を含む。テスト名は日本語。\n- テストの都合で本番のコードを曲げない。テスト専用の引数・`pub`・関数を本番に残さず、流れのテストは公開の入口と差し替え口で書く(インタビュー Q6)。\n- 行カバレッジ 80% を下限とする。CI の `coverage` ジョブが `cargo llvm-cov --fail-under-lines 80` で強制する(テスト専用の補助バイナリは除外)。\n- 主な開発環境の OS は macOS で、テストはローカルの macOS で実行する。他の OS のテストは GitHub Actions の CI(`test`: ubuntu、macos、windows)で確認する。マージ前に CI を通す(インタビュー Q3)。\n- 最小の Rust(1.100.0)でもテストを実行する(`msrv-test`)。\n- 起動の性能(`nfr_perf.rs`)は `startup-bench` で測るが、警告のみで失敗にしない(`STARTUP_BENCH_ENFORCE=false`、`continue-on-error`)。\n- v0.10.0 の機能にも、test-after と 80% の下限、単体テストと実バイナリを動かす統合テストを続ける(インタビュー Q3)。\n- ステートの中身を調べるテスト(`nfr_content.rs`)は、B6 のために開始と終了の時刻だけを許すように直す。それ以外(コマンド・環境変数・出力)は今までどおり検査する(インタビュー Q7)。"
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
  "input_sha256": "sha256:c9387614783c345a74417a345a583c5a6ae6f79a28c5c31daf5acd2fe2b58a52",
  "contract_sha256": "sha256:400acbdde5df13dffc33dd51dd2e149814f5bcde0642c5c3fc5e831c8563d84f"
}
```

テストの進め方は test-after です。この作業でテスト可能な層は、数の検査のテストそのものだけです。Step 6 でテストを書き、すぐに走らせ、Step 7 で効くことを確かめます。plan_profile の層(データ、API、画面など)は、この作業には無いので扱いません。

## データの形の変更

- 無い。コードの型・ステートの形・定義の書式は変えない(NFR1)。

## 設計のレビューからの持ち越し(この段階で扱う)

- **R-01**: 表 3.1 の #5 の L06 の決まった場所は `ステート全体の項目`(language-agnostic の 06 の 2.1)、R06 は `最上位のキー`(2.2)とする。テストを書く前に、#1〜#12 の決まった場所の見出しと文の一部を、仕様書の今の本文と 1 つずつ突き合わせる(Step 5)。結果は付録 A の表として `code-summary.md` に書く。
- **R-02**: 計画の模擬の行(L11 の 2.1 の表の最初の「目安」の行)の根拠の列にはすでに `AC-573` があるので、変えない。書き添えるのは 7 行の強制の列だけ(Step 4)。
- **R-03**: 版の注記で除く行の一覧は、注記が実際にある行だけにする。注記が無く手がかりの語も無い行(R12 の段 10 の「(33 種)」、段 5 の「(22 種)」)は「全体の探しの対象外」として書き分ける(Step 5 の付録 A)。
- **R-04**: 全体の探しの、手がかりの語と数の並び(数が語の前か後ろか、同じ文の中か)を、README(R00・L00)と 12 章の実際の行に当てて決める。その上で、検査される行と検査されない行を `code-summary.md` に表で書く(Step 5、Step 6)。仕様書の言い回しは変えない。

## ステップ

### Step 1: 準備(テストの道具の確かめ)

- [x] `unit-test-instructions.md` のコマンドを変更の前に走らせ、結果を `code-summary.md` に記録する(既存のテストが通る出発点)。
- [x] 機能の設計の 2.1〜2.4 の確かめの `grep` を変更の前に走らせ、出発点を記録する。

### Step 2: テスト名の書き方をそろえる(FR1.1、BR1.1)

- [x] L11 の根拠の列で、関数名を `_` のまま書いた 14 個(AC-097a、AC-206〜234 の 13 個)の関数名の部分の `_` を空白にする。
- [x] `::` の後ろに `_` を含む根拠の書き方が残らないことを `grep` で確かめる(識別子だけの例外があれば `code-summary.md` に列挙する)。
- [x] コミット: `docs: テスト名の書き方をそろえる`

### Step 3: env_types.rs のテスト名と AC-430(FR1.2、FR1.3、BR1.2、BR1.3)

- [x] `crates/core/tests/env_types.rs` の名前に「33 種」を含むテスト関数の名前を「34 種」にする。仕様書に同じ名前があれば同じに直す。
- [x] `cargo test -p conductor-core --test env_types --locked` を走らせる。
- [x] コミット: `core: env_types のテスト名を 34 種にする`
- [x] L11 の AC-430 の前提を `LC_ALL=C`、`LC_MESSAGES=C` にし、期待の列に「この前提では、デモが `LC_ALL` と `LC_MESSAGES` を消したかは、表示からは見分けられない」を書き足す。R11 は変えない。
- [x] コミット: `docs: AC-430 の前提を実際のテストに合わせる`

### Step 4: 性能の表の目安(FR1.4、BR1.4、R-02)

- [x] L11 の 2.1 の表で、強制の列が「目安」の 7 行を「目安(参照実装は強制として検査する)」にする。根拠の列は変えない。AC-573 の行(4.27)も変えない。
- [x] コミット: `docs: 性能の表の目安に参照実装の強制を書き添える`

### Step 5: 決まった場所と手がかりの突き合わせ(R-01、R-03、R-04)

- [x] `functional-spec.md` の 3.1 の #1〜#12 について、次のことを仕様書の今の本文で確かめる。
  - 決まった場所の見出し・文の一部(R-01 の直しを含む)
  - 全体の探しの手がかりの語と数の並び
  - 版の注記で除く行
  - 対象外の行
- [x] 確かめた結果を付録 A の形の表として `code-summary.md` に書く。テストの中の決まった場所と手がかりは、この表と同じにする。
- [x] 実装と違う数(注記の無いずれ)が見つかったら、そのずれを `code-summary.md` に挙げる。仕様書を実装に合わせて直し、別のコミット `docs: 仕様書の数を実装に合わせる` にする。仕様書の書き方を検査に合わせて変えることはしない。

### Step 6: 数の検査のテストを書いて走らせる(FR3.1〜FR3.3、BR3.1〜BR3.5)

- [x] `crates/cli/tests/spec_numbers.rs` を作る。
  - 根は `Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")` とする。読んだ文字列の `\r` を除いてから行に分ける。
  - 標準ライブラリだけを使う。
  - 補助の関数として次の 5 つを置く。
    - 列挙の変種を数える
    - 配列の `"…"` を数える
    - 見出しの下の表の本体の行を数える
    - 「N 種」などの数を取り出す
    - 版の注記の有無を判断する
  - 数ごとのテスト関数を 12 個置く(名前は日本語)。
  - 実装の値が取れない・0 のとき、決まった場所が見つからないとき、全体の探しが 1 件も当たらないときは失敗にする。
  - 失敗の文には次のものを入れる。
    - 数の名前
    - 相対のパスと行番号
    - 見出しか行の一部
    - 仕様書の値と実装の値
- [x] `cargo test -p conductor-cli --test spec_numbers --locked` を走らせ、通ることを確かめる。
- [x] `cargo fmt --all -- --check` と `cargo clippy --workspace --all-targets --locked -- -D warnings` を通す。
- [x] コミット: `cli: 仕様書の数を実装と突き合わせるテストを足す`

### Step 7: 効くことの確かめ(FR3.4、BR3.6)

`functional-spec.md` の 3.3 の 3 つを 1 つずつ行い、毎回テストが落ちることを確かめる。

- 仕様書の「場面 0〜15」を「0〜16」にする。
- 注記の無い「34 種」の 1 か所を「35 種」にする。
- 見出しの「種類の一覧」を変える。

それぞれ、ファイルの道具で書き換えてテストを走らせ、同じ道具で元の文に書き戻す。

- [x] 3 つのそれぞれで、落ちたテストの名前と失敗の文の 1 行目を `code-summary.md` に記録する。
- [x] 書き戻した後に `git diff --stat docs/` が空であることを確かめる。ずらした変更はコミットしない。

### Step 8: team.md の古い記述(FR2.1〜FR2.3、BR2.1)

付録 B の #1〜#10 を書き換える。各行は「今の文の一部」で場所を確かめ、見つからなければ止めて報告する。表の外は変えない。

- [x] 書き換えた後、`git diff aidlc/spaces/default/memory/team.md` の差分が付録 B の行だけであることを確かめる。
- [x] コミット: `aidlc: team.md の古い記述を今に合わせる`

### Step 9: team.md のテストの数を数え直す(付録 B の #11)

- [x] 次の数を数え直す。
  - `tests/` の直下の `.rs` の数(cli は 27 になる見込み)
  - `src/tests/` の `.rs` の数
  - `grep -rE '^\s*#\[test\]' crates --include='*.rs'` の行の数
- [x] 32 行目の数を、数え直した値に直す。
- [x] コミット: `aidlc: team.md のテストの数を数え直す`

### Step 10: 版を 0.10.3 にする(FR4.1、BR4.1)

- [x] 10ce84e と同じ 22 ファイルの `0.10.2` を `0.10.3` にする(`git show --stat 10ce84e` で範囲を確かめる)。
- [x] `cargo build --locked` が通ることを確かめる。
- [x] `cargo test --workspace --locked`、`cargo fmt --all -- --check`、`cargo clippy --workspace --all-targets --locked -- -D warnings` を通す。
- [x] コミット: `版を 0.10.3 にする`

### Step 11: まとめと記録

- [x] `code-summary.md` に次のことを書く。
  - 変えたファイル
  - 付録 A の表
  - Step 7 の結果
  - テストの数
  - 残した事項(60 行目の `#[allow]` の記述は後の候補)
- [x] `traceability.json` に、FR1.1〜FR4.1 と、それを満たすステップ・ファイルを書く。

## 付録 A: 数の検査の表(Step 5 で確かめて `code-summary.md` に写す)

`functional-spec.md` の 3.1 の表に、R-01 の直しを入れたものを出発点にする。各行に次の 4 つを書く。

- 決まった場所(ファイル、見出しか文の一部、取り方)
- 全体の探しの手がかりと数の並び
- 注記で除く行
- 対象外の行

#5 は次のとおりとする。

- R06: `### 2.2 最上位のキー` の下の表(table_rows)
- L06: `### 2.1 ステート全体の項目` の下の表(table_rows)

## 付録 B: team.md の書き換え(FR2.3。行は 9133eda の時点の目安)

| # | 節 | 行 | 今の文の一部 | 書き換えた後の文 |
|---|---|---|---|---|
| 1 | Way of Working | 11 | `(867 コミットすべてが非マージ、作者は 1 人)` | `(すべてのコミットが非マージで、作者は 1 人)` |
| 2 | Way of Working | 14 | ``タグは `v0.1.0` から `v0.9.9` まで付いている。`` | ``タグは `v0.1.0` から付いている。`` |
| 3 | Way of Working | 15 | 行全体(`機能を足す intent は版の 2 桁目を上げ(v0.8.0、v0.9.0)` で始まる) | `- 機能を足す intent は版の 2 桁目を上げ(v0.9.0、v0.10.0 など)、整理・修正の intent は 3 桁目を上げる(v0.9.1、v0.10.1 など)。機能を足す intent は、すべての機能を作り終えてから版を上げる(インタビュー Q2)。` |
| 4 | Walking Skeleton | 24 | 行全体(`v0.10.0 では B1〜B7 を 1 つずつ作る。` で始まる) | `- 機能を足す intent では、機能を 1 つずつ作る。順序は関わる部分ごとにまとめる(v0.10.0 の例: 定義の書き方 B1 → B2 → B3、グラフ B4、出力とステート B5 → B6 → B7)(スコープ定義 Q4、インタビュー Q2)。` |
| 5 | Walking Skeleton | 25 | 行全体(`7 つを作り終えてから版を 0.10.0 にする` を含む) | `- 各機能の最後に README とデモを更新し、すべてを作り終えてから版を上げる(スコープ定義 Q6、インタビュー Q2)。` |
| 6 | Testing Posture | 39 | 行全体(`v0.10.0 の機能にも、test-after と 80% の下限` で始まる) | `- 機能を足す intent でも、test-after と 80% の下限、単体テストと実バイナリを動かす統合テストを続ける(インタビュー Q3)。` |
| 7 | Testing Posture | 40 | 行全体(`B6 のために開始と終了の時刻だけを許すように直す` を含む) | ``- ステートの中身を調べるテスト(`nfr_content.rs`)は、開始と終了の時刻だけを許す(v0.10.0 の B6 で直した)。それ以外(コマンド・環境変数・出力)は検査する(インタビュー Q7)。`` |
| 8 | Deployment | 55 | 行全体(`v0.10.0 でも見送る(インタビュー Q4)` を含む) | ``- 署名・公証、Intel macOS の動作確認、`audit` の強制、`startup-bench` の強制は見送る(インタビュー Q4。署名の見送りは 2026-11-28 にも確かめた)。`` |
| 9 | Code Style | 68 | 行全体(`v0.10.0 の機能でも、新しい外部ライブラリ(依存)を追加しない。` で始まる) | ``- 機能を足す intent でも、新しい外部ライブラリ(依存)を追加しない。必要な処理(時刻の書式、DOT の整形、`watch` の表示し直しなど)は自前で書く。既存の規則(`project.md` の NEVER)をそのまま使う(インタビュー Q5)。`` |
| 10 | Code Style | 68 の後ろ | 68 行目と `## Forbidden` の間に空行が無い | 空行を 1 つ入れる |
| 11 | Testing Posture | 32 | `core 36、runtime 16、cli 26、converter 2` と `` `#[test]` は合計 1,460 `` | Step 9 で数え直した値(他の語は変えない) |

- 決まりの中身は変えない。#3〜#9 は、済んだ intent の言い方を、どの版にも当てはまる言い方に直すだけ。
- #7 と #6 は Testing Posture の行なので、Testing Contract の入力が変わる。この piece of work の Guard Policy は off なので、計画の承認は開き直さない。
