# Code Generation Plan(demo-run-id)

## Summary

- Builds: デモ(UNIX 版 `demo/run-demo.sh`、Windows 版 `demo/windows/run-demo.cmd`)に、場面 7(validate)、8(run ID でステートを分ける)、9(環境変数と後始末)を足す。定義に `run_id_env = "DEMO_RUN_ID"`。利用者の環境の影響を受けないようにする。案内と、自動テスト(Windows 版を拡張、UNIX 版を新設)。
- Touches: `demo/run-demo.sh`、`demo/windows/run-demo.cmd`、`demo/conductor.toml`、`demo/windows/conductor.toml`、`demo/README.md`、`crates/cli/tests/windows_demo.rs`、新しい `crates/cli/tests/unix_demo.rs`。`conductor` 本体、ルートの `README.md`、版、`.github/` は、変えない。
- Tests: 新しいテスト 約 10 件(Windows 版の検証の追加 約 6 件、UNIX 版 約 3 件)。既存のテストは、そのまま成功。行カバレッジ 80% 以上を保つ(本体は変えないので、実質変わらない)。

## 前提(設計から)

- 要件: `inception/requirements-analysis/requirements.md`(FR1〜FR6、NFR1〜NFR4)。調査と、デモの構造・実際の表示(英・日)・落とし穴: `inception/reverse-engineering/developer-scan.md` の Handoff Summary((1)〜(7))。行番号は目安で、スクリプトが正。zero-Unit(express)のため、この計画が設計を兼ねる。
- **場面(FR1、FR2):** 既存の場面 0〜6 の後ろに 7〜9 を足す(`scene` の見出しの形を既存に揃える)。順に 1 つずつ見せる(並行実行は使わない)。
  - **場面 7「validate」:** `validate -v`(正しい定義。0 と 1 行)→ 一時ディレクトリに作った誤りの定義(`command` の引用符が閉じていない。例 `bad.toml`)に `validate --config bad.toml`(64)→ 実行していないこと(ジョブの出力が出ない)と、ステートファイルがないこと(一覧が空)を見せる。誤りの定義は、デモの中で、スクリプトが作る(UNIX は `printf`、Windows は `echo` の並び。ファイルはデモの一時ディレクトリの中だけ)。
  - **場面 8「run ID でステートを分ける」:** 見せる順: `--run-id build-1` で `DEMO_LOAD_EXIT=2`(2 で止まる)→ 環境変数 `DEMO_RUN_ID=build-2` で、`pipeline`(0 で完走)→ ステートのファイル名の一覧(`conductor.toml.build-1.state` だけ)→ `watch --run-id build-1`(`Run ID: build-1`、止まった位置)→ 同じ `--run-id build-1` で `pipeline -v`(`load` から再開。`extract`・`transform` は `skipped (already executed)`。0)→ 一覧が空。
  - **場面 9「環境変数と後始末」:** `DEMO_RUN_ID=env-1` と `--run-id opt-1` の両方で `DEMO_LOAD_EXIT=2` の `pipeline`(オプションが勝つ: `conductor.toml.opt-1.state` ができる)→ `--run-id ""` で `DEMO_RUN_ID=env-1` のまま(run ID なしに固定: `conductor.toml.state` ができる)→ 止まった run をもう 1 つ(`--run-id opt-2`)→ 一覧(3〜4 個のステート)→ `reset --run-id opt-1` で 1 つ消す → `reset --all` で残りをすべて消す(0)→ 一覧が空、`watch` が `no state`。
  - 環境変数を使うコマンドの直前に、`DEMO_RUN_ID=…` の指定を 1 行で表示する(`printf`/`echo`)。環境変数の限定は、既存の流儀(UNIX: サブシェル `( VAR=…; export VAR; cmd … )`。Windows: `setlocal` / `set "VAR=…"` / `call :runcmd …` / `endlocal`)。ステートファイルの一覧の表示は、UNIX は `ls conductor.toml*.state`(無いときは何も出ない場合の見せ方を決める。例: 一覧の関数 `states()` で、無ければ `(なし)`)、Windows は `dir /b conductor.toml*.state`(同様)。
- **定義(FR3):** 両デモの `conductor.toml` の `[pipeline]` に `run_id_env = "DEMO_RUN_ID"` を足す。場面 0〜6 の動作は変わらない。
- **環境の影響(FR4):** 両デモの冒頭で `DEMO_RUN_ID` を消す(UNIX `unset DEMO_RUN_ID`、Windows `set "DEMO_RUN_ID="`)。UNIX 版は `LC_ALL` と `LC_MESSAGES` を空にして、`DEMO_LANG`(`LANG`)が効くようにする(Windows 版と同じ)。
- **案内(FR5):** `demo/README.md` の場面の表に 7〜9、`DEMO_RUN_ID` と `run_id_env` の説明(箇条書き)、Windows 版の節の「場面 0〜6」の文言を直す。
- **自動テスト(FR6):**
  - Windows 版 `windows_demo.rs`(`#![cfg(windows)]`): `Demo::run` が `CONDUCTOR_RUN_ID` と `DEMO_RUN_ID` を環境から外す(`env_remove`)。場面 7〜9 の検証を足す(`demo.run(&[...], &[(環境変数, 値)])` の形。ステートの有無は、一時ディレクトリのファイルの有無と、`watch` の標準出力で確かめる)。`run-demo.cmd` は実行しない(従来どおり)。
  - UNIX 版 `unix_demo.rs`(`#![cfg(unix)]`、新しいファイル。Apache ヘッダ付き): `sh demo/run-demo.sh` を、`CONDUCTOR`(`env!("CARGO_BIN_EXE_conductor")`)、`DEMO_SLEEP=0`、`DEMO_LANG=en_US.UTF-8` で実行する。`CONDUCTOR_RUN_ID`・`DEMO_RUN_ID`・`LC_ALL`・`LC_MESSAGES`・`NO_COLOR` は、テストが明示的に設定する(利用者の環境に左右されない)。終了コード 0、見出し(場面 0〜9)、要点の行(`Run ID: build-1`、`definition is valid`、`→ 終了コード 64`)が出ること、一時ディレクトリがなくなること(リポジトリに何も残らない)を確かめる。時間: `DEMO_SLEEP=0` で、数秒以内。`sh` がない環境のために、`sh` を起動できなければ、テストを失敗にする(UNIX の CI にはある)。
- 1.85 でビルドできる書き方(let チェーンを使わない)。`unsafe` を増やさない。

## 実装の手順(test-after: 実装してから、その層のテストを書いて実行する)

- [x] **Step 1. 実行環境の確認:** `cargo test --workspace --locked` が成功する(663 件)ことを確かめ、`unit-test-instructions.md` のコマンドを確定する。`demo/` の現状(`DEMO_SLEEP=0.1 sh demo/run-demo.sh` が 0)を確かめる。
- [x] **Step 2. 定義と UNIX 版デモを実装する(FR1、FR3、FR4):** `demo/conductor.toml` の `run_id_env`。`demo/run-demo.sh` の冒頭の整理(`DEMO_RUN_ID` を消す、`LC_ALL`・`LC_MESSAGES` を空に)、場面 7〜9、補助(ステートの一覧の関数、環境変数の指定の表示)。`DEMO_SLEEP=0.1 sh demo/run-demo.sh` を実行して、出力と終了コードを、目で確かめる(日本語と、`DEMO_LANG=en_US.UTF-8` の英語)。`CONDUCTOR_RUN_ID=zzz DEMO_RUN_ID=zzz LC_ALL=C` を付けても、完走して、日本語のまま、既存の場面が run ID なしで動くことも確かめる。
- [x] **Step 3. UNIX 版のテスト(FR6.2):** `crates/cli/tests/unix_demo.rs` を実装し、`cargo test -p conductor-cli --test unix_demo --locked` を実行する。
- [ ] **Step 4. Windows 版デモを実装する(FR2、FR3):** `demo/windows/conductor.toml` の `run_id_env`。`demo/windows/run-demo.cmd` の冒頭(`DEMO_RUN_ID` を消す)と、場面 7〜9。`.cmd` は UTF-8・CRLF のまま(ファイルの書き込みは LF だけを出すので、書いた後に、CRLF になっていることを、`file` や `grep -c $'\r'` などの読み取りで確かめる。LF のままの行があれば直す)。この環境(macOS)では実行できないので、既存の場面の書き方(`call :runcmd`、`setlocal`/`endlocal`、`set "X=1"`、`%errorlevel%` の扱い、`ping` の待ち、`dir /b`)を、そのまま写し、書いた内容を、読み直して確かめる。
- [ ] **Step 5. Windows 版のテスト(FR6.1):** `windows_demo.rs` に、`env_remove` と場面 7〜9 の検証を足す。`cargo check --workspace --tests --target x86_64-pc-windows-msvc --locked` で、型検査を通す(実行は CI の `windows-2022`)。macOS では、`windows_demo` は 0 件として動くことを確かめる。
- [ ] **Step 6. 案内(FR5):** `demo/README.md` を更新する。
- [ ] **Step 7. 全体の確認:** `cargo fmt --all -- --check`、`cargo clippy --workspace --all-targets --locked -- -D warnings`、`cargo test --workspace --locked`、`cargo +1.85 build --workspace --locked --all-targets`、`cargo check --workspace --tests --target x86_64-pc-windows-msvc --locked`、`cargo llvm-cov` で行カバレッジ 80% 以上。ルートの README の語句検索(CI の `docs-check` と同じ)が、変わらず通る。`git status` で、`crates/*/src`、ルートの `README.md`、`Cargo.toml`、`Cargo.lock`、`.github/` に変更がないことを確かめる。

## テストの方針(Minimal)

要件ごとに最も狭い層で 1 件を目安にする。デモそのもの(`run-demo.sh`)の確認は、UNIX 版のテストと、手での実行で行う。Windows 版の `run-demo.cmd` は、CI でも実行されないため、書き方を既存に揃え、Windows 版のテスト(`conductor` を直接呼ぶ)で、場面の中身が正しいことを確かめる。`run-demo.cmd` 自体の Windows の実機での実行は、あなたの確認の項目になる(Build and Test で記録する)。

## 設計からの追跡(要件 → 手順)

| 要件 | 手順 |
|------|------|
| FR1.1〜FR1.4 | Step 2、3 |
| FR2 | Step 4、5 |
| FR3 | Step 2、4 |
| FR4 | Step 2、4 |
| FR5 | Step 6 |
| FR6.1 | Step 5 |
| FR6.2 | Step 3 |
| NFR1 | Step 7(変更の範囲の確認) |
| NFR2 | Step 1、2、7(既存の場面とテストが成功のまま) |
| NFR3 | Step 7 |
| NFR4 | Step 2、4(POSIX `sh`、`cmd.exe` だけ、ライセンスヘッダ) |

## Assumptions

- 新しいファイル `crates/cli/tests/unix_demo.rs` を足す(UNIX 版のデモの検証)。ほかは、既存のファイルの編集。

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
