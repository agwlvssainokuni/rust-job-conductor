# 契約のまとめ: v0.10.0 の小さな機能のまとまり

出典: ユニットの定義(`unit-of-work.md`)、ユニットの依存(`unit-of-work-dependency.md`)、コンポーネントの一覧(`components.md`)、要件(`requirements.md`)、契約の設計の質問(`contract-design-questions.md`。以下 Q1〜Q3)。

すべての契約は、単一のバイナリ `conductor` の中の関数の呼び出しか、利用者から見えるファイル・コマンドラインの形である。ネットワークの API は無い。

## 契約の表

| # | Provider Unit | Consumer | Mechanism | Owner |
|---|---|---|---|---|
| C1 | u2-optional-env-file | u3-unset-env | 同じプログラムの中の関数の呼び出し(環境の統合) | u2 が作り、u3 が広げる |
| C2 | u5-log-dir | u6-attempt-times | 同じプログラムの中の関数の呼び出し(時刻の整形と現在の時刻) | u5 |
| C3 | u1、u2、u3、u5 | External: 定義ファイルを書く利用者 | 定義ファイル(TOML)の新しいキー | 各ユニット |
| C4 | u6-attempt-times | External: ステートを読む利用者と、古い版の `conductor` | ステートファイル(TOML)の試行の時刻 | u6 |
| C5 | u4、u7 | External: コマンドラインの利用者 | コマンドラインの引数と出力 | 各ユニット |
| C6 | u5-log-dir | External: ログを読む利用者 | ログのファイルの置き方と中身 | u5 |

## C1: 環境の統合(u2 → u3)

```yaml
# 共有の型と関数の形(Rust の型の概略。名前は機能の設計で確定する)
EnvFileRef:
  path: string          # 定義ファイルのディレクトリが基準
  required: bool        # 文字列で書いたときと、表で省いたときは true
load_env_files:
  input: [definition_path, list of EnvFileRef]
  output: LoadedEnvFiles | list of EnvFileError
  rule: required が false で、ファイルが無いものは読まずに飛ばす(誤りにしない)
merge_env:                # u2 では「足す変数」だけ。u3 で「消す名前」を足す
  output_u2: { add: [(name, value)] }
  output_u3: { add: [(name, value)], remove: [name] }
  error_u3: unset_env の名前が、そのジョブに当てはまる env か .env の変数と重なれば EnvFileError(64)
```

- u2 は今の統合の出力(足す変数)を変えない。u3 が出力に「消す名前」を足し、`run_job` に渡す。

## C2: 時刻(u5 → u6)

```yaml
Timestamp:                  # CoreTime。値は UNIX 時刻の秒(整数)
  format_rfc3339: Timestamp -> string     # 例 "2026-11-25T09:12:48Z"(UTC、秒まで、末尾 Z)
  parse_rfc3339: string -> Timestamp | 誤り # 上の形だけを受け付ける
  format_compact: Timestamp -> string     # 例 "20261125T091248Z"(ログのファイル名用)
  duration: (Timestamp, Timestamp) -> 秒
Runtime.now:                # cli の差し替え口。本番は runtime の RuntimeClock(std の SystemTime)
  output: Timestamp
```

- 流れ(CliFlows)が `now` で取った同じ値を、ステートの `started_at` とログのファイル名の両方に使う。

## C3: 定義ファイルの新しいキー(利用者)

```toml
[pipeline]
env_file = [".env", { path = ".env.local", required = false }]  # u2: 文字列と表を混ぜて書ける
unset_env = ["HTTP_PROXY"]                                        # u3
log_dir = "logs"                                                  # u5: 相対パスは定義ファイルのディレクトリが基準

[pipeline.retry]                                                  # u1: [job.retry] と同じ項目と検査
exit_codes = [75]
max = 3
interval = "10s"

[[job]]
id = "build"
command = ["make"]
retry = false                    # u1: 既定を打ち消す。retry は表か false
unset_env = ["SECRET_TOKEN"]     # u3

[[job]]
id = "test"
command = ["make", "test"]
[job.retry]                      # u1: ジョブの表があれば既定と混ぜずに丸ごと使う
exit_codes = [1]
max = 2
interval = "5s"
```

- 新しいキーは足すだけで、今あるキーの意味は変えない(NFR1)。`format_version` は 1 のまま。
- `retry = true`、表の `env_file` の知らないキー、空の `log_dir` は定義の誤り(64)。

## C4: ステートファイルの試行(利用者、古い版)

```toml
format_version = 1
definition_key = "..."
# (ほかの最上位のキーは今のまま)

[[job]]
id = "build"
status = "success"
exit_code = 0

[[job.attempt]]
started_at = "2026-11-25T09:12:48Z"
ended_at = "2026-11-25T09:13:05Z"

[[job]]
id = "test"
status = "pending"     # 状態の値は今の 5 つ(pending、success、warning、error、skipped)のまま。running は足さない

[[job.attempt]]
started_at = "2026-11-25T09:13:05Z"
# ended_at が無い: 動いている最中か、中断で止まった(FR6.9)
```

- 状態の値は増やさない。`pending` のジョブで、最後の試行に `ended_at` が無いものを「動いている最中か、中断で止まった」と読む。この 2 つの見分け方は、u6 の機能の設計で決める(契約のレビューの指摘 R-01)。
- `pending` と `skipped` 以外のジョブ(`success`、`warning`、`error`)に `[[job.attempt]]` が 1 つも無いステートは、壊れたステートの誤り(65)。`pending` で試行が無いジョブは未着手、`skipped` のジョブは動いていないので、試行は無くてよい(契約のレビューの指摘 R-02)。標準エラーに `conductor reset` で直せる旨を出す(FR6.4、FR6.8)。
- `ended_at` が無くてよいのは、各ジョブの最後の試行だけ(FR6.9)。
- 古い版の `conductor` は、`attempt` を知らないキーとして拒否する(65)。この向きの互換は求めない(要件の範囲の外)。
- コマンド、環境変数、ジョブの出力は書かない(NFR6)。

## C5: コマンドラインと出力(利用者)

```text
conductor graph [-v] [--config <パス>] [--format <text|mermaid|dot>] [--state [--run-id <ID>]] [<パイプライン名>]
conductor watch [--config <パス>] [--run-id <ID>] [--follow [--interval <秒>]]
```

- `--format dot`: Graphviz の DOT の `digraph` を標準出力に 1 回で書く。値は DOT の文字列として escape する(FR4.3)。状態の属性は機能の設計で決める。
- `--follow`: 一定の間隔(`--interval`、既定 2 秒)で表示し直す。Ctrl-C か、パイプラインの終わりで終わる。`--interval` は `--follow` と一緒のときだけ使える。範囲と終了コードは機能の設計で決める。
- `watch` の表に所要時間の列を足す。
- 使い方の表は `args.rs`、`report/messages.rs`、README の 3 か所で揃える。

## C6: ログのファイル(利用者)

```text
<log_dir>/<ジョブの ID>.<開始の時刻>.<試行の番号>.log
例: logs/build.20261125T091248Z.1.log
```

- 試行ごとに 1 つのファイル。標準出力と標準エラーをまとめて、子が書いた順に書く(Q3)。
- `conductor` は、ジョブの出力のほかに何も書き足さない(NFR6)。
- 開始の時刻は、ステートの `started_at` と同じ値(C2)。
- 同じ名前のファイルがあれば上書きしない。避け方(番号を足すか、誤りにするか)は機能の設計で決める。

## 契約の持ち主の規則

- 各契約は、表の Owner のユニットが持つ。後のユニットが広げるときは、前の形を壊さない(C1 の u2 → u3)。
- 定義ファイルとコマンドラインは、足すだけの変更にする。今あるキーとオプションの意味は変えない。
- ステートファイルは、古い版との互換を求めない(要件で決めた)。`format_version` は上げない。
- ステートとログの名前の時刻は、同じ値・同じ時計から取る(C2)。

## Open questions

| Contract | Question | Blocks |
|---|---|---|
| C4 | `pending` で最後の試行に `ended_at` が無いとき、「動いている最中」と「中断で止まった」を `watch` がどう見分けるか(状態の値は増やさない) | u6-attempt-times |
| C6 | ジョブの ID に使える文字の規則を確かめ、ファイル名に使えない文字や `.` が入りうるなら扱いを決める | u5-log-dir |
| C5 | DOT で状態をどの属性で表すか | u4-graph-dot |
| C5 | `--interval` の範囲と、`--follow` の終了コード | u7-watch-follow |
| C6 | 同じ名前のファイルがあるときの避け方と、ログのファイルの権限 | u5-log-dir |
| C6 | ログのディレクトリやファイルが作れないときの扱い | u5-log-dir |
