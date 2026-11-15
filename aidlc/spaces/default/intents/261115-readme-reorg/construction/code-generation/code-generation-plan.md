# Code Generation Plan(readme-reorg)

## Summary

- Builds: `README.md` の構成の整理(節の順番、目次、クイックスタート、run ID の説明の集約、表のセルの短縮、ソースからのビルドの手順、版の注記)と、見つかった誤り・書き漏れ 6 項目の修正。`demo/README.md` の見直し。
- Touches: `README.md` と `demo/README.md` だけ。`conductor` 本体、デモのスクリプトとテスト、`.github/`、`Cargo.toml`・`Cargo.lock`、版は、変えない。
- Tests: 文書の変更なので、新しいテストコードは書かない(変更を 2 つの Markdown に限る NFR2 のため)。確認は、コマンドで行い、結果を Build and Test に記録する: CI の `docs-check` と同じ検索、目次のアンカーの確認、クイックスタートの実行、ソースからのビルドの手順の実行、整理前後の事実の突き合わせ、`demo/README.md` とスクリプトの突き合わせ。既存の 666 件のテストは、そのまま成功する(コードは変えない)。

## 前提(設計から)

- 要件: `inception/requirements-analysis/requirements.md`(FR1〜FR8、NFR1〜NFR4)。調査と照合表: `inception/reverse-engineering/developer-scan.md` の Handoff Summary((1) 節と run ID の所在、(2) 主張とコードの照合表と、不一致 ①〜⑥、(3) `docs-check`、(4) ソースビルドの事実、(5) インストールの事実)。zero-Unit(express)のため、この計画が設計を兼ねる。
- **新しい README の節の順序(FR1.1、Q2: A):** `#`(題名) → 概要(題名の下の文と箇条書き) → `## 目次` → `## インストール`(`### 1. ダウンロード` / `### 2. SHA-256 の確認` / `### 3. 展開と PATH` / `### macOS での警告の回避の手順(Gatekeeper)` / `### ソースからのビルド`) → `## クイックスタート` → `## 使い方`(構文、コマンド表、オプション表、注記、例、デモへのリンク) → `## 定義ファイルの書き方`(例、キー表、注記、`### command の書き方`、`#### 配列で書く(v0.2.0 以降)`) → `## ステートファイルの場所`(概要、`### run ID でステートを分ける(v0.5.0 以降)`) → `## 終了コードの表` → `## 保証の範囲(at-least-once)` → `## シグナルの扱い` → `## convert の使い方` → `## ロケールと言語` → `## ライセンス`。`終了コードの表` は、今の位置(定義ファイルの後ろ)から、ステートの後ろへ動く(`ステートファイルの場所` が run ID の小節を含み、終了コードの 65・68 の説明と相互に参照するため)。見出しの語(`SHA-256`、`終了コードの表`、`ステートファイルの場所`、`at-least-once`、`Gatekeeper` を含む箇所)は、変えない。
- **run ID の集約(FR2):** 規則の本体は `### run ID でステートを分ける(v0.5.0 以降)` にだけ置く(与え方、優先順位、`--run-id ""`、値の規則、ファイル名、中身の `run_id`、`watch`・`reset` の動き、`reset --all`、大文字小文字、同時起動)。ほかの場所は、(a) 使い方の構文のコードブロック(オプションの形)、(b) コマンド表の `watch`・`reset` の行(1 文)、(c) オプション表の `--run-id`・`--all`(1 文 + 小節への参照)、(d) 例のコメント、(e) 定義の例とキー表の `run_id_env`(1 文 + 小節への参照)、(f) 終了コードの表の 64・65・68(意味の列挙は短く)、だけにする。
- **表の短縮(FR3):** コマンド表(`watch`・`reset`・`validate` の行)とオプション表(`--run-id`・`-v`・`--all` の行)の各セルを、1 文(目安 100 文字以内)に。移した詳細は、`## 使い方` の下の小節(`### コマンドの詳しい動き`)か、run ID の小節に置く。`validate` の詳細(実行しない、ステートに触れない、名前の照合、`-v` の 1 行)は、小節に置く。
- **クイックスタート(FR1.3):** 作業ディレクトリ(例: `mkdir demo-conductor && cd demo-conductor`)で、最小の `conductor.toml`(`[pipeline] name = "hello"`、ジョブ 3 つ `step1`・`step2`・`step3`。`command = ["sh", "-c", "echo step1"]` の形。`step2` は、環境変数 `FAIL` で失敗させられる(`exit ${FAIL:-0}`))を書き、`conductor validate`(終了コード 0)→ `FAIL=3 conductor pipeline -v hello`(`step2` で終了コード 3)→ `conductor watch`(`step2` が `error`)→ `conductor pipeline -v hello`(`step2` から再開して完了)→ `conductor watch`(ステートなし)。出力と終了コードは、**実際に実行して得たものを**書く。Windows では `command` を `cmd` のものに読み替える旨を 1 行。
- **誤りの修正(FR4):** 4.1 インストールの展開(`tar xzf` のあと、`conductor-<版>-<対象>/` のディレクトリの中の `conductor` を `install`。zip も同じ構造と書く)、4.2 シグナルの節に `validate`、4.3 `reset --all` の run ID なしのステートの扱いと削除失敗の 65、4.4 OS の表の Intel macOS の注記(クロスビルド・動作の確認なし)、4.5 終了コードの書き方の統一、4.6 `demo/README.md` の場面 0。
- **ソースからのビルド(FR5):** `### ソースからのビルド` に、必要なもの(Rust 1.85 以降、`git`)、`git clone https://github.com/agwlvssainokuni/rust-job-conductor.git` と `cd rust-job-conductor`、ビルド、できる場所、テスト、`PATH` への置き方(`install -m 755 target/release/conductor /usr/local/bin/conductor`、または `cargo install --path crates/cli --locked`)。
- **版の注記(FR6):** `(vX.Y.Z 以降)` の形を、配列の `command`(v0.2.0)、`validate`(v0.4.0)、run ID と `reset --all`(v0.5.0)に付ける。互換の注意(配列は v0.1.x では読めない、`[[jobs]]` の旧キー)は、その節の中に残す。
- **`demo/README.md`(FR7):** `demo/run-demo.sh`・`demo/windows/run-demo.cmd`・各スクリプトを読んで、場面 0〜9 の表(UNIX 版と Windows 版で同じ)、環境変数の名前と既定値(`DEMO_PAUSE`、`DEMO_LANG`、`DEMO_SLEEP`(UNIX 0.5 秒、Windows 1 秒。スクリプトで確認する)、`DEMO_RUN_ID`、`DEMO_<ジョブ名>_EXIT`、`CONDUCTOR`)、コマンド例を、突き合わせて直す。構成(UNIX 版と Windows 版を並べる)は保つ。ルートの README の用語と揃える。
- 事実を変えない(FR8)。整理の前の README を、調査の照合表と突き合わせ、整理の後も、各行の根拠の記述が残っていることを確かめる。

## 実装の手順(文書の整理: 書く → 確かめる)

- [x] **Step 1. 基準の確認:** 整理前の `README.md` と `demo/README.md` の内容を、`git show HEAD:README.md` で、確認用に保存せずに読む。CI の `docs-check` と同じ検索(7 語句)が、整理前に通ることを確かめる。`cargo test --workspace --locked` が成功する(666 件)ことを確かめる。
- [x] **Step 2. README の構成を組み直す(FR1.1、FR1.2、FR3):** 節の順序を直し、目次を足し、コマンド表とオプション表のセルを短くして、詳細を小節に移す。内容(事実)は変えない。
- [x] **Step 3. run ID の説明を 1 か所に(FR2):** run ID の小節に、規則の本体を集め、ほかの箇所は、1 文と参照にする。
- [x] **Step 4. 誤りの修正と版の注記(FR4.1〜FR4.5、FR6):** 6 項目のうち、README の 5 項目(`demo/README.md` の場面 0 は Step 7)と、版の注記を直す。
- [x] **Step 5. ソースからのビルドの手順(FR5):** 小節を書く。書いたコマンドを、実際に実行して確かめる(`cargo build --release --locked --bin conductor`、`cargo test --workspace --locked`、`cargo install --path crates/cli --locked --root <target/ の下の一時ディレクトリ>` が成功し、できた `conductor --version` が表示される)。
- [x] **Step 6. クイックスタート(FR1.3):** 書いた定義と手順を、`target/` の下の一時ディレクトリで、リリースビルドの `conductor` を使って、実際に実行する。出力と終了コードを、README に反映する(英語と日本語のどちらで書くかは、`LANG=C` の英語の出力を、日本語の説明の中に引用する)。
- [x] **Step 7. `demo/README.md` の見直し(FR7、FR4.6):** スクリプトと突き合わせて直す。
- [x] **Step 8. 確認:** (a) CI の `docs-check` と同じ `grep` が 7 語句とも見つかる、(b) 目次と、README 内の `](#...)` のリンクが、実在する見出しに対応する(GitHub のアンカーの規則で、スクリプトで確かめる)、`demo/README.md`・`LICENSE` へのリンクが実在する、(c) run ID の値の規則などが README の中で 1 か所だけにある(検索で確かめる)、(d) 整理前の README の、照合表の各行の根拠となる語句が、整理後にも残っている(整理前後で、規則・終了コード・表示の語句を検索して突き合わせる)、(e) `git diff --stat` が、`README.md` と `demo/README.md` と、この intent の記録だけである、(f) `cargo fmt --all -- --check`、`cargo clippy --workspace --all-targets --locked -- -D warnings`、`cargo test --workspace --locked`(666 件)。

## テストの方針(Minimal)

文書の変更なので、テストコードは書かない(NFR2)。Step 8 の確認を、テストの代わりに、実行して結果を記録する(Build and Test の `test-results.md` に、コマンドと結果を書く)。

## 設計からの追跡(要件 → 手順)

| 要件 | 手順 |
|------|------|
| FR1.1、FR1.2 | Step 2、8(b) |
| FR1.3 | Step 6 |
| FR2 | Step 3、8(c) |
| FR3 | Step 2 |
| FR4.1〜FR4.5 | Step 4 |
| FR4.6 | Step 7 |
| FR5 | Step 5 |
| FR6 | Step 4 |
| FR7 | Step 7、8(b) |
| FR8 | Step 8(d) |
| NFR1 | Step 8(a) |
| NFR2 | Step 8(e) |
| NFR3 | Step 8(b) |
| NFR4 | Step 8(f) |

## Assumptions

- 新しいファイルは作らない。変更は `README.md` と `demo/README.md` の編集だけ。
- クイックスタートの出力は、`LANG=C` の英語の出力を使い、`--help` の使い方の文などの変わりうる文を、載せない(`conductor` の表示の文言は、日本語の説明の中の引用にとどめる)。

## Testing Contract

```json
{
  "version": 1,
  "methodology": "test-after",
  "source": "team",
  "ordering": "各テスト可能な層を実装してから、その層のテストを書いて実行する。",
  "scope": "express",
  "test_strategy": "minimal",
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
  "input_sha256": "sha256:47c2e6e6f234580fd227d246415c56f693b43f2a7953fee56ef7217b54cbd097",
  "contract_sha256": "sha256:3e75c1c022f49e57c8d1612ebaf8b3e7016754454c19abb2dc7f313d6288a4e3"
}
```
