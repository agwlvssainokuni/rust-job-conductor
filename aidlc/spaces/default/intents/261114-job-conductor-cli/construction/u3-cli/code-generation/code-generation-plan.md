# Code Generation Plan(u3-cli)

## Summary

- Builds: `conductor` の実行ファイル(パッケージ `conductor-cli`)。コマンド(run・pipeline・watch・reset・convert)、流れの組み立て、シグナル、表示(英語・日本語)、終了コード、利用者向けの README。あわせて、u1-core の定義ファイルのキーを `jobs` から `job` に直す
- Touches: `crates/u3-cli/`(新規。ソース、テスト、テスト専用の補助の実行ファイル)、`crates/u1-core/`(キー名の変更。`src/definition/walk.rs`、テスト、例)、ワークスペースの `Cargo.toml`(`[profile.release]`)と `Cargo.lock`、`README.md`(新規)
- Tests: 約 130 件(引数、終了コード、表示、流れ(偽の依存)、シグナル、binary を起動する少数のテスト、横断。Standard の方針と、行カバレッジ 80% の下限)

## 前提(設計から)

- 上流: functional-spec(W1〜W6)、rules(BR1.1〜BR7.3)、entities、nfr-requirements(security-requirements の差分表)、tech-stack-decisions(T1〜T6)、nfr-design(performance・security・scalability・reliability・observability の各設計、logical-components、**差分の表**)、infrastructure-design(cicd-pipeline)。**差分の表は、functional-design の関連する記述より優先する**。契約への反映(C2・C3・E1・E2・E3)は、functional-spec の「契約への反映が必要な点」のとおり。
- 設計の成果物は書き換えない。設計との差は、`code-summary.md` に記録する。
- テストの順序は、チームの実践(test-after)に従い、**層ごとに、実装してから、その層のテストを書いて実行する**。
- この Unit は、u1-core(`conductor-core`)、u2-runtime(`conductor-runtime`)、u4-converter(`conductor-converter`)に依存する(パス依存)。**u2-runtime と u4-converter は、変更しない**。u1-core は、キー名の変更(Step 2)だけを行う。足りない関数・型があれば、止めて報告する。
- クレートの作り(Q1: A): `lib.rs` に全部の処理を置き、`main.rs`(実行ファイル `conductor`)は、薄い入口。流れは、小さな trait(`Runtime`、`Sink`、`Env`、`Signals`。Q2: A)を受け取り、本番は実際の関数、テストは偽の実装を渡す。
- `#![forbid(unsafe_code)]`。OS 固有のコードは、シグナルの監視と端末の判定のモジュールの中だけ(`cfg(unix)`/`cfg(windows)`)。

## 実装の手順

### 層 0: 準備

- [x] **Step 1: クレートの準備と、依存の確認(テストの実行環境の準備を含む)**
  - `crates/u3-cli/Cargo.toml`(パッケージ `conductor-cli`、`publish = false`、`[[bin]] name = "conductor"` と、テスト専用の補助の実行ファイル `[[bin]] name = "conductor-cli-test-helper"`(`test = false`、`doc = false`))、`src/lib.rs`(`#![forbid(unsafe_code)]`)、`src/main.rs`(空の入口)を作る。依存: `conductor-core`、`conductor-runtime`、`conductor-converter`(パス依存)、`lexopt`、`signal-hook`(`cfg(unix)`)、`ctrlc`(`cfg(windows)`)。開発用に `tempfile`。Windows のテストだけ、開発用の依存に `windows-sys`(Ctrl+C の送信。`cfg(windows)`)。バージョンは、その時点の安定版で、`Cargo.lock` で固定する。ワークスペースの `Cargo.toml` に、`[profile.release]`(`opt-level = 3`、`lto = "thin"`、`codegen-units = 1`、`strip = true`)を足す。
  - **確認(tech-stack-decisions の仮定):** (a) `lexopt`、`signal-hook`、`ctrlc` が、最小の Rust のバージョン(1.85)の依存の解決(`rust-version`)で入る、(b) `signal-hook` の `Signals`(イテレータ)が、SIGHUP・SIGINT・SIGQUIT・SIGTERM の番号を返す、(c) `ctrlc` が、Windows の Ctrl+C を受け取る(コンパイルの確認まで。実行は CI)、(d) `cargo tree` に、C のコンパイルを要する依存(`cc` など)がない。**確かめられない場合は、止めて、人に確認する**(条件を緩めない)。結果を `code-summary.md` に記録する。
  - 実行できることを確かめる: `cargo test -p conductor-cli --locked`(テスト 0 件で成功)。
  - 関わる: NFR2.1、NFR3.1、NFR4.3、NFR5.1。
- [x] **Step 2: u1-core の定義ファイルのキーを `jobs` から `job` に直す(Q1: A。契約 E2)**
  - `crates/u1-core/src/definition/walk.rs` の、ルートのキー(`ROOT_KEYS`)、表の配列の読み取り(`root.get("jobs")`)、表示の文言(「`jobs` の要素」など)を `job` に直す。`transitions`(`[[job.transitions]]`)と、ほかのキーは変えない。u1-core の例(`tests/fixtures/*.toml`)、テストの定義の文字列、`lib.rs` などの doc を、`[[job]]` / `[[job.transitions]]` に直す。u4-converter と u2-runtime のテストの定義の例に、`jobs` があれば、直す(ソースの検索で探す。ただし、これらの Unit のソースの意味は変えない)。
  - 確認: `cargo test --workspace --locked` が、すべて通る(u1-core の 115 件、u2-runtime の 152 件、u4-converter の 112 件の、それぞれが、そのまま通る)。
  - 関わる: contract E2、AC1.1.1。

### 層 1: 型と純粋な部品

- [x] **Step 3: 型(`types.rs`)**: `CliCommand`(kind: run/pipeline/watch/reset/convert、job_id、pipeline_name、convert_input(`OsString`)、convert_target、config_path(`OsString`)、verbose)、`UsageError`(kind と固定の文)、`DisplayEvent`(kind、level: error/verbose/output、data)、`Language`(en/ja)。関わる: entities、BR1.1。
- [x] **Step 4: 引数の解釈(`args.rs`、ArgParser)**: `lexopt` で字句に分け、コマンドごとの文法の表で検査する(security-design の 1、1a): `--config path` と `--config=path`、`--to`、`--`(以降は位置引数)、`-v` はちょうど1回(`-vv` は誤り)、値の欠落(`missingValue`)、許されない場所のオプション(`optionNotAllowed`)、`--help`・`-h`・`--version`・`-V`(`--` より前でだけ、最初に処理)、引数がない(`missingArgument`)、`--to` の値が yaml・toml 以外(`invalidValue`)。パスは `OsString` のまま。使い方の文言(英語・日本語)。関わる: BR1.1〜BR1.6、NFR4.1。
- [x] **Step 5: 終了コード(`exit.rs`、ExitMapper)**: 純粋な関数。`reportedExitCode` を、どの OS でも 0〜255 に合わせる(範囲外は下位 8 ビット。エラーで 0 になるなら 1)。conductor 自身のエラーの定数: 64、65、66、67、68、70。関わる: BR5.1〜BR5.3、NFR2.2。
- [x] **Step 6: 表示(`report/`、Reporter)**: Messages(英語・日本語の定数の表。種類ごとの固定の文)、`escape_display`(制御文字、書式・双方向の制御文字を `\u{..}` に。64 文字で切る。パスは切らない)、Locale(`LC_ALL`、`LC_MESSAGES`、`LANG` の順。`ja` で始まれば日本語。**インスタンスごとの遅延のセル**。静的な `OnceLock` は使わない)、Color(UNIX 系で端末かつ `NO_COLOR` が未設定または空のときだけ。Windows は色なし)、WatchView(パイプライン名の1行と、JOB・STATUS・NOTE の3列の表)、各 DisplayEvent の文言の組み立て(エラー: `conductor: error: <場所>: <種類の文>: <原因>`、`-v`: `[conductor] ...`)。定義の誤りは、すべて1行ずつ。関わる: BR6.1〜BR6.5、BR7.1、NFR6.1〜NFR6.4、NFR4.2、NFR1.4。
- [x] **Step 7: 層 1 のテスト**: 引数の文法の表(全コマンドの正しい引数と誤り。`=` 形式、`--`、`-vv`、値の欠落、`--help` の順序、UTF-8 でないパス(UNIX 系))、終了コードの表(全項目。0、1、127、128 + 番号、64〜70、範囲外、エラーで 0 になる場合)、表示(全種類の文言を、英語・日本語で。エスケープ(ESC、改行、U+202E)、キー名の切り詰め、ロケール(`ja_JP.UTF-8`、`en_US.UTF-8`、未設定、`LC_ALL` の優先)、遅延(成功の経路で、ロケールの判定が行われない)、色(端末・非端末・`NO_COLOR`・Windows)、watch の表の列と空の表示)。`escape_display` の、同じ入力と期待値の表(u1-core・u2-runtime・u4-converter と揃える)。

### 層 2: 流れ(Orchestrator)

- [x] **Step 8: 差し替え口(`deps.rs`)と本番の実装**: trait `Runtime`(`load_definition`、`check_command`、`run_job`、`read_state`、`write_state`、`reset_state`、`convert`)、`Sink`(標準出力・標準エラーへの書き込み。失敗を値で返す)、`Env`(環境変数、端末かどうか)、`Signals`(`start(&InterruptSignal) -> Result<(), SignalError>`。すでに登録済みなら `AlreadyRegistered`)。本番の実装は、u1-core・u2-runtime・u4-converter の実際の関数と、標準出力・標準エラー、実際の環境。`platform/`(`cfg(unix)`: `signal-hook` の `Signals` を専用のスレッドで回し、最初の番号だけを `raise`。`cfg(windows)`: `ctrlc`)。関わる: BR4.1、NFR4.3、Q2: A。
- [x] **Step 9: 個別実行(`run.rs`、W2)**: BR2.1〜BR2.6 のとおり(定義の検査 64 → command の検査 64 → ジョブID 68 → ステートの読み込み 65 → 照合 65 → `decide_before_run`(Reject 66、Skip(`apply_skip`、変更があったときだけ書き込み、completed なら初期化、0)、Run)→ ステートが空なら初期ステート(`Fast`)→ `run_job` → `classify_and_decide`(Individual)→ `apply_outcome` → `write_state`(`Durable`)→ completed なら `reset_state` → 終了コード)。書き込み・初期化の失敗は 65(ジョブ実行済みの場合は、その旨を表示に含める)。成功コードに含まれない 0 の表示。シグナルの準備は、定義の検査より前(BR4.1、BR4.2a)。
- [x] **Step 10: 一括実行(`pipeline.rs`、W3)**: BR3.1〜BR3.4 のとおり(定義の記述順に、1つずつ。`decide_before_run` → Skip は `apply_skip` と書き込み、Run は実行と記録(Pipeline)。**記録のたびに completed を確かめ、true なら `reset_state` して 0**。エラーで停止。ジョブのあいだの合図で、`resume_point_on_interrupt` と `apply_interrupt`、128 + 番号。記述順のジョブがなくなったのに completed でない場合は、65)。
- [x] **Step 11: 補助のコマンド(`aux.rs`、W4〜W6)**: `watch`(ステートの読み込み、空の表示、表、65)、`reset`(`reset_state`、65)、`convert`(`convert` の結果を、全部作ってから標準出力に1回で書く。失敗は 67 で、標準出力は空。標準出力の書き込みの失敗は 67(`watch` は 65))。
- [x] **Step 12: 入口(`lib.rs` の `run_cli` / `run_cli_with`、`main.rs`、パニックのフック)**: `run_cli_with(args, deps) -> exit code`(trait を引数で受け取る)、`run_cli(args, env, io)`(本番の依存を組み立てる)。`main.rs`: パニックのフックを最初に設定し(固定の英語の文を標準エラーへ、終了コード 70。終了の呼び出しは差し替えられる)、`run_cli` の結果で、1回だけ `std::process::exit`。
- [x] **Step 13: 層 2 のテスト**(偽の `Runtime`・`Sink`・`Env`・`Signals` を使う `tests/flow_*.rs`)
  - 個別実行: 全部の失敗の経路(reliability-design の表の全行)と終了コード、`decide_before_run` の3つの結果、初期ステートを Run のあとに書くこと(Reject・Skip の前には書かない)、変更のない Skip で `write_state` を呼ばない、completed の初期化、at-least-once(ジョブのあとの書き込みの失敗で、ジョブ実行済みの表示と 65)、成功コードに含まれない 0、検査中の合図(Reject・Skip・各種エラーの終了コードが優先、Run なら起動しない)、`-v` の表示。AC1.3.1〜AC1.3.3、AC2.3.1〜AC2.3.3、AC2.4.1〜AC2.4.4。
  - 一括実行: AC3.1.1〜AC3.1.3(3ジョブ成功、警告の遷移で間をスキップ、`@end` で完了して初期化)、AC3.2.1〜AC3.2.3(エラーで停止、再試行なし、再開位置)、AC4.3.2(エラーからの再開)、スキップで完了した場合の初期化、ジョブのあいだの中断、完了の判定ができない場合の 65。
  - 補助のコマンド: AC5.1.1〜AC5.1.3(watch の表、空、ファイルへの出力で制御文字なし)、AC6.1.1・AC6.1.2(reset)、AC7.1.1〜AC7.1.3(convert の成功、失敗で標準出力が空、67)、閉じたパイプ(watch 65、convert 67)。
  - シグナルの準備の失敗は 70、2回目の `start` は `AlreadyRegistered`。

### 層 3: バイナリと横断

- [x] **Step 14: テスト専用の補助の実行ファイル**(`src/bin/conductor-cli-test-helper.rs`): 指定した終了コードで終わる、引数・環境変数を出す、眠り続ける、起動と終了の直前の時刻(単調な時計)を出す。`unsafe` と `libc` は使わない。
- [x] **Step 15: バイナリを起動するテスト(少数。`tests/binary_*.rs`)**: `env!("CARGO_BIN_EXE_conductor")` で、`--version`(0)、使い方の誤り(68)、`run` の終了コード(成功 0、警告 0、スキップ 0、エラー(ジョブの値)、起動できない 127、定義の誤り 64)、標準出力と標準エラーの分け方(`-v` なしの成功は両方空。ジョブの出力がそのまま)、`convert` の標準出力、`watch` をファイルに出力したときの制御文字の無さ、実際のシグナル(UNIX 系: ジョブの実行中に SIGTERM → 128 + 15 とステートの再開位置)。プロセスを起動するテストは、各 30 秒の上限(別のスレッド + `recv_timeout`)を持ち、超えたら、子プロセスを強制終了して回収してから失敗にする。Windows のテスト(`cfg(windows)`): Ctrl+C の経路(`GenerateConsoleCtrlEvent`。自動化できなければ、リリース前の手動確認のチェックリストに置き換え、`code-summary.md` に、その旨を記録する)。
- [x] **Step 16: 横断のテスト(`tests/nfr_*.rs`)**: ソースの検索(この Unit の `src/` に、`unsafe`、`low_level`、`std::net`、ネットワークのライブラリがない。`cfg(unix)`・`cfg(windows)` が `platform` の外にない)(NFR4.3、NFR4.5、NFR3.1)。パニックしない(ランダムな引数、壊れたステートを返す偽の `Runtime`)(NFR4.7)。起動のオーバーヘッドの測定(ジョブ10個、20 回。起動前 + ジョブのあとの p95、`pipeline` のジョブ1つあたり、ジョブ 1,000 個の定義の時間を、標準エラーと `GITHUB_STEP_SUMMARY` に出す。基準を超えても失敗にしない。`STARTUP_BENCH_ENFORCE` が `true` のときだけ失敗にする)(NFR1.1、NFR1.2、NFR1.5)。ジョブ実行中の CPU 時間(NFR1.3)。同じ定義に2つの `run` を同時に動かす(NFR1.6)。convert の出力が、全部1回で書かれる(NFR4.4)。
- [x] **Step 17: 利用者向けの README(`README.md`)**: 次の内容を含める(`docs-check` が確かめる見出しとキーワード): 概要、インストール(GitHub Releases、SHA-256 の確認の手順、macOS での警告の回避の手順(Gatekeeper。右クリックで開く、または `xattr -d com.apple.quarantine`)、PATH)、使い方(5つのコマンドと、オプション)、定義ファイルの書き方(`[[job]]`、`[[job.transitions]]`、`format_version`、成功コード・警告コード、`@end`)、終了コードの表(0、ジョブの値、1、64〜68、70、127、128 + 番号)、ステートファイルの場所(定義ファイルと同じディレクトリの `<定義ファイル名>.state`)、**保証の範囲(at-least-once)**、シグナルの扱い、convert の使い方、ロケールと言語。既存の `LICENSE` は変えない。
- [x] **Step 18: 公開の API の文書と確認**: `lib.rs` の `///` に、入口の関数、trait、モジュールの構成(logical-components との対応)、保証の範囲を書く。`cargo fmt --all -- --check`、`cargo clippy --workspace --all-targets --locked -- -D warnings`、`cargo test --workspace --locked` が通ること。`cargo llvm-cov` が使えれば、このクレートの行カバレッジを測って記録する(補助の実行ファイルのソースを除外。使えなければ「測っていない」と記録)。`cargo +1.85 build -p conductor-cli --locked` が、使える環境なら、通ること。`cargo tree -p conductor-cli -e normal,build` に `cc` などがない(`libc`、`windows-sys`、`linux-raw-sys` を除き、`-sys` もない)こと。

## テストの方針(Standard)

- 部品ごとに 5〜8 件を目安に、規則は規則ごとに確かめる。流れは偽の依存で、失敗の経路まで確かめ、バイナリを起動するテストは、少数に絞る。
- 行カバレッジ 80% 以上(下限。設定でしきい値を下げない)。`cfg(windows)` のコードは、Linux の測定に入らない(CI の Windows のテストの成功で確かめる)。
- ファイルを使うテストは `tempfile`。外部のサービスやネットワークは使わない。

## 設計からの追跡(ストーリー → 手順)

| ストーリー | 手順 |
|------------|------|
| US1.3(AC1.3.1〜AC1.3.3) | Step 2、4、9、13 |
| US2.3(AC2.3.1〜AC2.3.3) | Step 5、9、13、15 |
| US2.4(AC2.4.1〜AC2.4.4) | Step 6、9、13、15 |
| US2.5(AC2.5.3〜AC2.5.6) | Step 1、8〜10、15、16 |
| US3.1(AC3.1.1〜AC3.1.4) | Step 10、13、15 |
| US3.2(AC3.2.1〜AC3.2.3) | Step 10、13 |
| US5.1(AC5.1.1〜AC5.1.3) | Step 6、11、13、15 |
| US6.1(AC6.1.1、AC6.1.2) | Step 11、13 |
| US7.1(AC7.1.1〜AC7.1.3) | Step 11、13、15 |
| NFR1.x | Step 6、16 |
| NFR2.x(信頼性) | Step 9、10、13、15、17 |
| NFR4.x | Step 4、6、8、12、16 |
| NFR5.1 | Step 7、13、18 |
| NFR6.x | Step 6、13、15 |

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
