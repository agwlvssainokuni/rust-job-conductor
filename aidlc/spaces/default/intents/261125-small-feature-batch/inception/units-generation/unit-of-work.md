# ユニットの定義: v0.10.0 の小さな機能のまとまり

出典: 要件(`requirements.md`)、コンポーネントの一覧(`components.md`)、設計の決定(`decisions.md`)、ユニットの分割の質問(`units-generation-questions.md`。以下 Q1〜Q4)。

機能ごとに 1 つのユニットにする(Q1)。どのユニットも、配布する単一のバイナリ `conductor` の一部を変えるため、種類は `service`、配置は既存のバイナリに組み込む(embedded)。各ユニットは、その機能の README とデモの更新まで含める(FR8.1)。

## ユニットの一覧

| Unit ID | Directory | 機能 | 種類 | 配置 | 大きさ |
|---|---|---|---|---|---|
| U1 | u1-pipeline-retry | B1 パイプライン共通のリトライの既定と `retry = false` | service | embedded | M |
| U2 | u2-optional-env-file | B2 無くてもよい `.env`(表の `env_file`) | service | embedded | S |
| U3 | u3-unset-env | B3 `unset_env` | service | embedded | M |
| U4 | u4-graph-dot | B4 `graph --format dot` | service | embedded | M |
| U5 | u5-log-dir | B5 `log_dir`(出力の複製)と、時刻の整形(CoreTime)・現在の時刻(RuntimeClock) | service | embedded | L |
| U6 | u6-attempt-times | B6 ステートの試行ごとの時刻と `watch` の所要時間 | service | embedded | L |
| U7 | u7-watch-follow | B7 `watch` の継続表示と、版を 0.10.0 にする作業 | service | embedded | M |

## 各ユニット

### U1 u1-pipeline-retry

- **説明**: `[pipeline.retry]` を全ジョブの既定にし、ジョブで `retry = false` と書けば打ち消せるようにする。
- **境界と責務**: CoreTypes(実効の retry)、CoreDefinition(`[pipeline.retry]` の検証、`retry` の表か `false`、実効の retry の決定、成功・警告との重なりの検査)。リトライの判断と流れは変えない(ADR-003、ADR-008)。
- **要件**: FR1.1〜FR1.6、FR8.1。
- **注意**: 既定を書かない定義の動きを変えない(NFR1)。誤りの表示に値を含めない(NFR6)。

### U2 u2-optional-env-file

- **説明**: `env_file` の項目に `{ path, required }` の表を許し、`required = false` のファイルが無ければ読まずに進む。
- **境界と責務**: CoreTypes(EnvFileRef)、CoreDefinition(表の検証)、CoreEnvFile(読む対象に required を添える)、RuntimeEnvFile(無いファイルの扱い)、CliFlows(`--dry-run` と `graph` での扱い)。
- **要件**: FR2.1〜FR2.5、FR8.1。
- **注意**: あるが読めないファイルは今までどおり 64(FR2.3)。

### U3 u3-unset-env

- **説明**: `[pipeline]` と `[job]` の `unset_env` に書いた名前を、親から引き継いだ環境から消してジョブに渡す。
- **境界と責務**: CoreTypes、CoreDefinition(名前の規則、`env` との重なり)、CoreEnvFile(統合の結果に消す名前を足す。`.env` の中身との重なり)、RuntimeRunner(親の環境から除く)、CliDeps(`run_job` に消す名前を渡す)(ADR-007)。
- **要件**: FR3.1〜FR3.5、FR8.1。
- **注意**: U2 の `.env` の読み取りの変更の上に積む(Q4)。

### U4 u4-graph-dot

- **説明**: `graph --format dot` を足し、`-v` と `--state` に対応する。
- **境界と責務**: CliArgs(`dot` の値)、CliReport(DOT の描画と escape)(ADR-006)。
- **要件**: FR4.1〜FR4.4、FR8.1。
- **注意**: 状態の表し方(属性)は機能の設計で決める(要件のレビューの指摘 R-04)。

### U5 u5-log-dir

- **説明**: `[pipeline] log_dir` があるとき、ジョブの出力を端末にも出しつつ、試行ごとに 1 つのファイルへまとめて残す。
- **境界と責務**: CoreTypes(log_dir)、CoreDefinition(検証)、CoreTime(新規。時刻の整形と読み取り。Q3 によりここで作る)、RuntimeClock(新規。現在の時刻)、RuntimeLog(新規。ディレクトリとファイル)、RuntimeRunner(出力の複製)、CliDeps(現在の時刻、ログのファイルの指定)、CliFlows(試行ごとにログを指定)(ADR-002、ADR-005)。
- **要件**: FR5.1〜FR5.5、FR8.1。
- **注意**: 作れない・書けないときの扱い、名前の形、権限、衝突の避け方は機能の設計で決める。ファイル名の時刻とステートの時刻は、流れが取った同じ値を使う方針(部品の設計のレビューの指摘 R-03)。

### U6 u6-attempt-times

- **説明**: ステートの各ジョブに試行ごとの開始と終了の時刻を書き、`watch` の表に所要時間を出す。
- **境界と責務**: CoreTypes(JobState の attempts、Attempt)、RuntimeState(読み書き、時刻の無いステートは 65)、CliFlows(開始を書いてからジョブを動かす)、CliReport(所要時間の列、65 のときの `reset` の案内)。U5 の CoreTime と RuntimeClock を使う(ADR-004)。
- **要件**: FR6.1〜FR6.9、FR8.1。
- **注意**: `format_version` は 1 のまま。`nfr_content.rs` を時刻だけ許すように直す(team-practices.md)。

### U7 u7-watch-follow

- **説明**: `watch` にオプションで継続表示を足す。最後に、版を 0.10.0 にする。
- **境界と責務**: CliArgs(オプションと間隔)、CliFlows(繰り返し、中断のシグナル、パイプラインの終わり)、CliReport(UNIX 系の端末での描き直し)。版の更新(`crates/cli/Cargo.toml`)(Q2)。
- **要件**: FR7.1〜FR7.5、FR8.1、FR8.2。
- **注意**: 所要時間の表示(U6)を使う。版を上げるため、他のすべてのユニットの後に作る(Q2)。

## 共通の制約

- 新しい依存を加えない。`unsafe` は runtime の `platform/unix.rs` の外に書かない。`cfg` は `platform` の外に置かない(NFR4、NFR5)。
- test-after、行カバレッジ 80% 以上、3 つの OS の CI と MSRV の CI(NFR2、NFR3)。
- テストの都合で本番のコードを曲げない(project.md の Forbidden)。
