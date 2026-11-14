# Code Generation Plan(u2-runtime)

## Summary

- Builds: ジョブの直接起動とシグナルの転送(JobRunner)、ステートの保持・照合・初期化・記録(StateStore)を受け持つライブラリ
- Touches: `crates/u2-runtime/`(パッケージ `conductor-runtime`。ソース、テスト、テスト専用の補助の実行ファイル)。ワークスペースの `Cargo.lock`(`libc` の追加)
- Tests: 約 80 件(単体、実際のプロセスを使う統合、ステートの原子性、OS 固有、ジョブの一連の流れ、横断。Standard の方針と、行カバレッジ 80% の下限)

## 前提(設計から)

- 上流: functional-spec(W1〜W6)、rules(BR1.1〜BR2.10)、entities、contract-summary(C3、E3)、nfr-requirements の security-requirements(NFR4.1〜NFR4.9、**既存の成果物との差分**)、tech-stack-decisions(T1〜T3)、nfr-design の security-design(**WaitLoop、StateReader、StateWriter、Recorder、D1〜D4、既存の成果物との差分**)、logical-components、cicd-pipeline。**差分の表は、functional-design の関連する記述より優先する**。
- 設計の成果物は書き換えない。設計との差は、`code-summary.md` に記録する。
- テストの順序は、チームの実践(test-after)に従い、**層ごとに、実装してから、その層のテストを書いて実行する**。
- コードは、`crates/u2-runtime/`(パッケージ名 `conductor-runtime`)に置く。ワークスペースは `crates/*` を含む。**この Unit は、u1-core(パッケージ `conductor-core`)に依存する**(共有する型 `JobExecution`、`JobState`、`PipelineState`、`JobOutcome`、`NextStep`、`PipelineDefinition`、`ResumePoint`、`DecisionReason` などを、そのまま使う)。u1-core のソースは、変更しない。u1-core の型で足りないものがあれば、止めて報告する。
- 公開する関数(契約 C3、functional-spec の「契約への反映」): `run_job`、`check_command`、`read_state`、`write_state`(耐久性の指定つき)、`check_consistency`、`reset_state`、`initial_state`、`apply_outcome`、`apply_skip`、`apply_interrupt`。ファイルに触れずに試せる入口も、必要なものは公開する。
- OS 固有のコードは、`platform` モジュール(`cfg(unix)` と `cfg(windows)`)の中だけに置く。`unsafe` は、UNIX 系のシグナル送信の1か所だけ(`#![deny(unsafe_code)]` を置き、そのモジュールだけ `#[allow(unsafe_code)]`)。

## 実装の手順

### 層 0: 準備

- [x] **Step 1: クレートの準備と、依存の確認(テストの実行環境の準備を含む)**
  - `crates/u2-runtime/Cargo.toml`(edition・rust-version・license は、ワークスペースから継ぐ。`publish = false`)、`src/lib.rs`(`#![deny(unsafe_code)]`、空のクレート)を作る。依存: `conductor-core`(パス依存)、`toml`(**u4-converter と同じバージョンと機能の集合**(`parse`、`std`、`display`、`serde`)。`preserve_order` は有効にしない)、`thiserror`(同じ)、`libc`(`[target.'cfg(unix)'.dependencies]`)。開発用に `tempfile`(同じ)。バージョンは、その時点の安定版を選び、`Cargo.lock` で固定する。
  - テスト専用の補助の実行ファイルの `[[bin]]`(名前 `conductor-test-helper`、`src/bin/conductor-test-helper.rs`。`test = false`、`doc = false`)を足す。
  - 確認: (a) `std::os::unix::process::CommandExt::process_group` が、最小のバージョン(1.85)で使える、(b) `libc` が、最小のバージョンでビルドできる、(c) `toml` が、状態の読み書きに必要な機能(読み取り、書き出し、`i64`、真偽値、配列、テーブル)を満たし、深さ 1000 と 100,000 の入力で、異常終了せず失敗が返る(u1-core と同じ確認)。**確かめられない場合は、止めて、人に確認する**(条件を緩めない)。結果を `code-summary.md` に記録する。
  - 実行できることを確かめる: `cargo test -p conductor-runtime --locked`(テスト 0 件で成功)。
  - 関わる: NFR2.1、NFR3.1、NFR4.7、NFR4.8、NFR5.1。

### 層 1: 型とエラー

- [x] **Step 2: 型とエラー(`types.rs`、`error.rs`、Describe)**
  - entities.md の型: `CommandLine`(program、args)、`CommandError`(kind: `UnclosedQuote`/`Empty`、reason)、`InterruptSignal`(読み取り専用の合図。立ったら下りない。立っているあいだ、最初のシグナルの番号を運ぶ。スレッド間で安全に共有できる作り)、`StateLocation`、`StateError`(path、kind: `Unreadable`/`TooLarge`/`Corrupt`/`VersionMismatch`/`WriteFailed`/`RemoveFailed`、reason)、`ConsistencyResult`(`Ok` / `Mismatch(PipelineName|JobIds|JobOrder)`)、`Durability`(`Durable`/`Fast`)。`thiserror` で、kind ごとに固定の表示の文を付ける(入力の値と、ライブラリのエラー文は含めない。パスは含めてよい)。
  - 関わる: NFR4.5、BR1.1、BR2.3、BR2.5。
- [x] **Step 3: 層 1 のテスト**: 全 kind の表示に入力の値が入らない、`InterruptSignal` が立つ・下りない・最初の番号を保つ、等しさ。

### 層 2: ジョブの起動(JobRunner)

- [x] **Step 4: command の分け方(`split.rs`、Split と `check_command`)**
  - BR1.1 の規則を、文字を1つずつ読む状態機械で実装する(再帰なし)。空白(スペース・タブ・改行)で区切る。`'...'` はそのまま。`"..."` の中は、左から順に読み、バックスラッシュの次が二重引用符のときだけ、その2文字を二重引用符1つとして解釈し、ほかのバックスラッシュはそのまま。引用符の外のバックスラッシュもそのまま。引用符で囲んだ部分と囲んでいない部分は、つなげて1つの引数。`''` と `""` は空の引数。変数・`*`・`|` などは解釈しない。閉じていない引用符は `UnclosedQuote`、コマンド名がなければ `Empty`。
  - 関わる: BR1.1、NFR4.2、AC2.1.1。
- [x] **Step 5: プラットフォーム(`platform/`)**
  - `platform/unix.rs`(`cfg(unix)`): 新しいプロセスグループでの起動の指定と、シグナルの送信 `send_signal(pid: u32, signal: i32)`(唯一の `unsafe`。pid が `i32` に収まり 0 より大きいこと、シグナルが `{ SIGHUP, SIGINT, SIGQUIT, SIGTERM }`(`libc` の定数)のどれかであることを確かめてから、`libc::kill` を1回呼ぶ。失敗は無視)。`platform/windows.rs`(`cfg(windows)`): 何もしない(Ctrl+C は OS が配る)。ほかの場所に、`cfg(unix)`・`cfg(windows)`・`unsafe` を置かない。
  - 関わる: NFR3.1、NFR4.8、BR1.7。
- [x] **Step 6: ジョブの起動と待ち(`runner.rs`、Spawn と WaitLoop、`run_job`)**
  - `run_job(job: &JobDefinition, signal: &InterruptSignal) -> JobExecution`。流れは security-design の「ジョブの起動と待ち」のとおり: 分け方 → 起動前の合図の確認(立っていれば `started = false`・`interrupted = true`)→ 起動(シェルを介さない。UNIX 系は新しいプロセスグループ。作業ディレクトリ・環境変数・標準入出力を引き継ぐ。失敗は `startFailure`)→ 待ちのくり返し(`try_wait` が回収する唯一の場所。間隔は 1ms から 10ms。直前の `try_wait` が「まだ」を返した回にだけ、合図を見て、1回だけ送る。`try_wait` の失敗は `startFailure`(理由は固定の文))→ 終了の観測(`exitCode`、UNIX 系の `terminatedBySignal`)→ 観測の直後に合図が立っていれば `interrupted = true` と `signalNumber`。
  - 関わる: BR1.2〜BR1.7、NFR4.1、NFR4.6、NFR4.7、NFR4.8、NFR4.9、AC2.1.1、AC2.1.2、AC2.1.5、AC2.5.1、AC2.5.6。
- [x] **Step 7: 補助の実行ファイル(`src/bin/conductor-test-helper.rs`)**: 引数、環境変数、作業ディレクトリを標準出力に出し、指定された終了コードで終わる。「眠り続ける」モードを持つ(シグナルの受信は、ハンドラなしで、既定の動作による終了で確かめる。`unsafe` と `libc` は使わない)。
- [x] **Step 8: 層 2 のテスト**(`split.rs` の単体、`platform` の単体、`tests/runner_*.rs`)
  - 分け方: 規則の表(引用符、バックスラッシュ、`\\"` の左から読む規則、Windows のパス、空の引数、タブ・改行、連結、`;` `&&` `|` を含む語、閉じていない引用符、空)。
  - 送信: pid が 0・負・`i32` を超える値、許可されないシグナルのとき、送らずに拒否される(送信の判断を、純粋な関数に分けて試す)。
  - 起動(補助の実行ファイル): 引数のメタ文字がそのまま渡る、環境変数・作業ディレクトリの引き継ぎ、存在しないコマンドの `startFailure`、終了コードの返り方(0、非0、大きな値)、シグナルで終了(UNIX 系。眠り続けるところへ SIGTERM が届いて `terminatedBySignal = 15`)、起動前の合図(起動しない)、実行中の合図(1回だけ転送され、`interrupted = true`、`signalNumber`)、2回目の合図は追加の動作をしない、Windows の `.cmd` への危険な引数(`cfg(windows)`。`startFailure`)。**プロセスを起動するテストは、各 30 秒の時間の上限(別のスレッド + `recv_timeout`)を持ち、超えたら、テストが持つ子プロセスを強制終了して回収してから失敗にする**。

### 層 3: ステート(StateStore)

- [x] **Step 9: ステートの読み込み(`state/reader.rs`、StateReader、`read_state`)**
  - 場所の決定(定義ファイルと同じディレクトリの `<定義ファイル名>.state`)。開いたハンドルの大きさ確認、上限(1 MiB)+ 1 バイトまでの読み取り(`impl Read` を取る関数に分ける)、長さ 0 は空、UTF-8 でなければ `Corrupt`、TOML として読む、`format_version` を先に確認(無ければ `Corrupt`、1 でなければ `VersionMismatch`)、型付きの構造への走査と BR2.3 の内容の矛盾の検査(未知のキー、型の誤り、必須のキーの欠落、status の不正、skip_reason・exit_code の矛盾、id の重複、job_order との不一致、resume_job_id が job_order にない、completed と resume_job_id の両立)、`completed` が true なら空として返す。
  - 関わる: BR2.1〜BR2.3、NFR4.3、NFR4.5、NFR4.7、AC4.1.3、AC5.1.2。
- [x] **Step 10: ステートの書き込みと初期化(`state/writer.rs`、StateWriter と Reset)**
  - `write_state(path, state, durability)`: 一時ファイル(`.<ステートファイル名>.tmp.<プロセス番号>.<時刻>.<連番>`)を、新規作成で作り(既存なら連番を進めて最大 16 回)、全内容(TOML、`format_version = 1`、E3 のキー、`definition_key` と `definition_path` は同じ文字列)を書き、`Durable` のときだけ同期し、置き換え(`rename`)。`Durable` のとき、UNIX 系でディレクトリを同期(失敗は無視)。成功した `Durable` の書き込みのあと、同じ名前の型の、更新から 24 時間以上たった一時ファイルを消す(失敗は無視)。失敗したら、自分が作った一時ファイルを消し、`WriteFailed`。`reset_state(path)`: 読まずに削除。存在しなければ成功。削除の失敗は `RemoveFailed`。
  - 関わる: BR2.2、BR2.4、BR2.6、NFR4.4、AC4.1.2、AC4.3.3、AC6.1.1、AC6.1.2。
- [x] **Step 11: 照合と記録(`state/consistency.rs`、`state/record.rs`、Consistency と Recorder)**
  - `check_consistency(state, definition)`: パイプライン名 → ジョブIDの集合 → 並び、の順に比べる。`initial_state(definition, definition_path)`。`apply_outcome(state, job_id, execution, outcome, next)`: status(分類のとおり。分類がなければ変えない)と exit_code(JobExecution の `exitCode` そのまま。無ければ以前の値を消す)、`skippedJobIds` を `skipped`(理由は `skipReasonKind`)、`resume_job_id` と `completed` は NextStep のとおり、`end_reached` は一度 true になったら戻さない。`apply_skip(state, job_id, reason, definition)`: `AlreadyExecuted` は何も変えない。`SkipTarget`・`AfterEnd` は `skipped` にしたあと、error のジョブがあれば `resume_job_id` と `completed` を変えず、なければ `completed`(いま skipped にしたジョブが、定義の順で最後であり、かつ pending が無い)と `resume_job_id`(定義の順で最初の pending。無ければ無し)を書き、`AfterEnd` なら `end_reached` を true にする。`apply_interrupt(state, resume_point)`。
  - 関わる: BR2.5、BR2.8、AC2.1.6、AC2.2.1〜AC2.2.4、AC3.2.1〜AC3.2.3、AC4.1.1、AC4.2.1〜AC4.2.4、AC4.3.1、AC4.3.2、AC4.4.1〜AC4.4.3。
- [x] **Step 12: 層 3 のテスト**(`state/` の単体と `tests/state_*.rs`)
  - 読み込み: 存在しない(空)、長さ 0(空)、`completed` のステート(空)、1 MiB ちょうど(受理)・+1(`TooLarge`)・終わりのない入力(上限 + 1 で止まる)、不正な UTF-8、BR2.3 の各項目(未知のキー、型の誤り、必須のキーの欠落、status の不正、skip_reason と exit_code の矛盾、id の重複、並びの不一致、resume_job_id が job_order にない、completed と resume_job_id の両立)、将来の版(未知のキーを含む)が `VersionMismatch`。
  - 書き込み: 往復(書いたステートを読み戻すと等しい)、置き換え先がディレクトリのとき `WriteFailed` でステートが変わらない、一時ファイルの名前が重なったとき連番を進める、残った一時ファイルが読み込みに影響しない、古い一時ファイルの後始末、`Fast` と `Durable` の指定、権限による失敗(UNIX 系、root でないときだけ。スキップしたら出力に明記する)、初期化(存在しない・ある)。
  - 照合: `Ok`、パイプライン名の違い、ID の改名・追加・削除、並びだけの違い。
  - 記録: 初期ステート、`apply_outcome`(成功・警告・エラー・起動しなかった中断・スキップの記録・`end_reached` の単調性・古い exit_code の消去)、`apply_skip`(`AlreadyExecuted` は不変、error があるとき不変、最後のジョブのスキップで `completed`、`@end` のあとの個別実行(A → `@end`、B・C が pending のまま、B のスキップで完了しない、最後の C のスキップで完了))、`apply_interrupt`。

### 層 4: 流れ・横断

- [x] **Step 13: ジョブの一連の流れのテスト(`tests/flow_with_state.rs`)**: u1-core の `decide_before_run`・`classify_and_decide` と、この Unit の `run_job`(補助の実行ファイル)・`apply_*`・`write_state`・`read_state`・`reset_state` を組み合わせて、Cli の流れを小さな試験用の関数で再現する。AC4.2.1(3ジョブを個別実行で最後まで → ステートが初期化される)、AC4.2.2・AC4.2.3(最後のジョブのスキップ。`@end`)、AC4.2.4(一連の実行の `@end`)、AC4.3.1・AC4.3.2(エラーからの再開)、AC4.3.3(reset のあと先頭から)、AC4.3.4(自動の再実行がない)、AC4.4.1〜AC4.4.3(食い違い)、AC2.1.6(成功コードに含まれない 0 → エラー、ステートにエラーの位置)、AC2.5.2(排他制御がない: 2つの実行がそれぞれ動く)、at-least-once(ジョブの終了後・書き込みの前に止まった場合の再実行)。
- [x] **Step 14: 横断のテスト(`tests/nfr_*.rs`)**
  - ソースの検索(`src/` だけ): `unsafe` が `platform` の UNIX 系の1か所だけ、`cfg(unix)`・`cfg(windows)` が `platform` の外にない、`"sh"`・`"cmd"` を呼ばない(NFR4.1、NFR4.8、NFR3.1)。
  - パニックしない: 壊れた TOML、ランダムな入力、巨大な値の状態、ランダムな command の文字列で、パニックしない(NFR4.7)。`128 + signalNumber` などの桁あふれは飽和。
  - エラーの表示に入力の値が現れない(全 kind)(NFR4.5)。ステートファイルに command と環境変数の値が現れない(NFR4.5)。
  - NFR1.1: ジョブ10個の定義で、全ジョブの `check_command` + ステートの読み込み + 照合 + 初期ステートの書き込み(`Fast`)を 100 回測り、p95 を標準エラーに出力する(基準 20ms を超えても、失敗にしない)。`GITHUB_STEP_SUMMARY` があれば、そこにも書き出す。結果の書き込み(`Durable`)の時間も、あわせて記録する(基準なし)。
- [x] **Step 15: 公開の API の文書(`lib.rs`、`///` の説明)**: 公開する関数と型に、入出力・失敗・規則の ID を書く。モジュールの構成(logical-components の呼び名との対応)、呼び出しの順序(契約 C2 の順序の続き)、保証の範囲(at-least-once)、公開しないものを、`lib.rs` に書く。関わる: contract C3。
- [x] **Step 16: 確認(Build and Test の前の、この Unit の自己確認)**: `cargo fmt --all -- --check`、`cargo clippy -p conductor-runtime --all-targets --locked -- -D warnings`、`cargo test -p conductor-runtime --locked` が通ること。`cargo llvm-cov` が使えれば、このクレートの行カバレッジを測って記録する(補助の実行ファイルのソースは除外。使えなければ「測っていない」と記録する。80% の判定は Build and Test が、全体のテストで行う)。`cargo +1.85 build -p conductor-runtime --locked` が、使える環境なら、通ること(使えなければ、記録する)。`cargo tree -p conductor-runtime --edges normal` に `-sys` がない(`libc` を除く)こと。

## テストの方針(Standard)

- 部品ごとに 5〜8 件を目安に、規則は規則ごとに確かめる。単体テストは `#[cfg(test)]` のモジュールと `tests/`。実際のプロセスを使うテストは、補助の実行ファイルで行う。
- 行カバレッジ 80% 以上(下限。設定でしきい値を下げない)。`cfg(windows)` のコードは、Linux の測定に入らない(CI の Windows のテストの成功で確かめる)。
- ファイルを使うテストは `tempfile` で一時ディレクトリを作る。外部のサービスやネットワークは使わない。

## 設計からの追跡(ストーリー → 手順)

| ストーリー | 手順 |
|------------|------|
| US2.1(AC2.1.1、AC2.1.2、AC2.1.5、AC2.1.6) | Step 4〜8、11、13 |
| US2.2(状態の記録。AC2.2.1〜AC2.2.4) | Step 11〜13 |
| US2.5(AC2.5.1、AC2.5.2、AC2.5.6) | Step 6、8、13 |
| US3.2(AC3.2.1〜AC3.2.3) | Step 11、12、13 |
| US4.1(AC4.1.1〜AC4.1.3) | Step 9〜12 |
| US4.2(AC4.2.1〜AC4.2.4) | Step 10〜13 |
| US4.3(AC4.3.1〜AC4.3.4) | Step 10〜13 |
| US4.4(AC4.4.1〜AC4.4.3) | Step 11〜13 |
| US5.1(AC5.1.1、AC5.1.2。状態の読み取り) | Step 9、12 |
| US6.1(AC6.1.1、AC6.1.2) | Step 10、12、13 |
| NFR1.1 / NFR2.1 / NFR3.1 | Step 1、5、14 |
| NFR4.1〜NFR4.9 | Step 2、4〜14 |
| NFR5.1 | Step 3、8、12、13、16 |

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
