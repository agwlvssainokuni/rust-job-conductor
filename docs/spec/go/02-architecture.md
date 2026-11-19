# 02 アーキテクチャ(パッケージ・依存方向・型の骨格・`Deps`・実行のシーケンス)

対象: `conductor` v0.8.0 と同じ挙動を実現する、Go の実装の構成。この文書は、パッケージの分け方を**固定**し、各パッケージの責務、依存の向き、公開する型・関数の骨格(シグネチャ)、テストのための差し替え口(`Deps`)、実行の流れ、並行性とエラー処理の方針を定める。
各機能の挙動は、[03-definition-format.md](03-definition-format.md)(定義)、[04-cli.md](04-cli.md)(引数)、[05-execution-flow.md](05-execution-flow.md)(実行の流れ)、[06-state-file.md](06-state-file.md)(ステート)、
[07-process-and-signals.md](07-process-and-signals.md)(プロセスとシグナル)、[08-output-and-i18n.md](08-output-and-i18n.md)(出力)、[09-converter.md](09-converter.md)(変換)、[10-errors-and-exit-codes.md](10-errors-and-exit-codes.md)(エラーと終了コード)、[11-quality-and-build.md](11-quality-and-build.md)(品質)で定める。この文書は、構成だけを定める。

骨格のコードは、型とシグネチャの例であり、実装ではない。フィールドの名前や型で、他の文書(特に 03・06)の定めと食い違う箇所があれば、その文書の定めを正とし、この文書の骨格を読み替える。

## 規定の強さの表記

| 表記 | 意味 |
|------|------|
| 【規定】 | 守る。パッケージの名前、依存の向き、責務の分け方、並行性の規則 |
| 【目安】 | 推奨。実装者が変えてよい(ファイルの分け方、補助の関数の名前など) |

---

## 1. 設計の前提

| 項目 | 内容 |
|------|------|
| 言語・版 | Go。`go.mod` の `go` 行は、実装時点の最新の安定版の 1 つ前以降(例: 「Go 1.25 以降」。実装時に、最新の 2 つの安定版を確認する) |
| モジュール名 | `conductor`(置き換えてよい。置き換えたときは、すべての import のパスを合わせる) |
| 構成 | 1 つのモジュールに、実行ファイル 2 つ(`cmd/conductor`、`cmd/conductor-testjob`)と、`internal/` 配下のパッケージ 9 つ |
| 外部のライブラリ | YAML の読み書きだけ(選定は [03-definition-format.md](03-definition-format.md) と [09-converter.md](09-converter.md))。それ以外は標準ライブラリ。引数の解析は自前(`flag` は使わない) |
| 公開の範囲 | すべて `internal/` に置く。他のモジュールから import できない(ライブラリとして公開しない) |
| 配布物 | `conductor` だけ。`conductor-testjob` は配布物に含めない |

---

## 2. パッケージ一覧と責務【規定】

| パッケージ | 責務 | 入出力 |
|------------|------|--------|
| `cmd/conductor` | 薄い入口。`os.Exit` を 1 回だけ呼ぶ。`defer`+`recover` で想定外の失敗を終了コード 70 にする。本番の `Deps` を作って `cli.Run` を呼ぶ | プロセスの引数・環境・標準入出力 |
| `internal/definition` | 定義ファイルの読み込み(場所の決定と読み取り、YAML/JSON の構文解析、入れ子の深さ、`format_version`、構造・型・遷移・終了コード・名前の検査)。誤りを位置(行)つきで返す。定義の型(`Pipeline`、`Job`、`Command`、`Retry`、`Transition`)と、誤りの種類(`ErrorKind`)を持つ。`run_id_env` だけのゆるい読み取りも持つ | ファイルを読む(`Load` だけ)。ほかは文字列に対する純粋な処理 |
| `internal/convert` | YAML ⇔ JSON の変換(`convert --to yaml\|json`)。定義の検証はしない。他の `internal/` に依存しない | 入力ファイル 1 つを読む(`Convert`)。`ConvertText` は文字列に対して純粋 |
| `internal/runid` | run ID の値の検査と、決定(`--run-id` と環境変数のどちらを使うか)に使う純粋な関数 | なし |
| `internal/flow` | 純粋な判断。実行結果の分類(成功・警告・エラー・分類なし)と報告する終了コード、遷移と次の動き、起動前の判断(実行・スキップ・拒否)、中断時の再開位置、リトライの判断と待ち時間、終了コードの範囲合わせ。**実行時の値の型**(`ExecResult`、`JobState`、`PipelineState`、`Outcome`、`NextStep` など)を持つ | なし(引数だけから結果を決める。ファイル・プロセス・環境変数・時刻に触れない) |
| `internal/state` | ステートの読み書き。JSON の表現(ファイル用の構造体と、固定の順序の書き出し)、記録関数(結果・スキップ・中断を、新しい `flow.PipelineState` として作る純粋な関数)、読み込み(検査と異常系)、書き込み(一時ファイル + `fsync` + `os.Rename` + ディレクトリの同期)、定義との整合性の検査、`reset` と `reset --all`、ステートファイルの場所と名前 | ステートのファイル(とそのディレクトリ)。記録関数と整合性の検査は純粋 |
| `internal/runner` | `command` の分割と検査、`os/exec` によるジョブの起動と待ち、シグナルの転送、リトライの待機、中断の合図(`Interrupt`)、シグナル監視、プラットフォーム層(`platform_unix.go`、`platform_windows.go`) | 子プロセス、時刻(待ち)、OS のシグナル。分割と検査は純粋 |
| `internal/report` | 出力。メッセージカタログ(英語・日本語)、ロケールと色の判定、`-v` の行、誤りの行、`watch` の表、使い方(`--help`)、入力由来の文字列のエスケープと切り詰め。**文を作るだけ**で、書き込みは `cli` が `Sink` へ行う | 環境変数・端末かどうかは、引数で受け取る(自分では読まない) |
| `internal/cli` | 引数解析(誤りの優先順位)、各サブコマンド(`run`・`pipeline`・`watch`・`reset`・`validate`・`convert`)の組み立て、`Deps`(差し替え口)とその本番の実装、終了コードの決定。上記のすべてを呼ぶ | `Deps` を通した、すべての入出力 |
| `internal/testsupport` | テスト用の偽実装(`Deps` の各インターフェースを満たす)、一時ディレクトリで定義やステートを作る補助。`cli` を import しない(2.3 節) | テストだけが使う |
| `internal/version` | 版の定数(`Version = "0.8.0"`)。`conductor --version` の表示と、リリースの検査([11-quality-and-build.md](11-quality-and-build.md))が使う | なし |
| `cmd/conductor-testjob` | 受け入れテスト用のテストジョブ(終了コード、スリープ、標準出力への書き込み、環境変数・引数の出力など)。配布物に含めない | テストだけが起動する |

### 2.1 各パッケージの公開(エクスポート)するものの骨格【規定】

各パッケージの、他のパッケージから使われる名前だけを示す(補助の名前は実装者が決める)。

#### `internal/definition`

```go
type Pipeline struct {
    FormatVersion  int
    Name           string
    Description    string // 空なら無し
    RunIDEnv       string // 空なら無し
    RemoveOnComplete bool
    Jobs           []Job   // 記述順
}
func (p *Pipeline) JobIndex(id string) (int, bool)

type Job struct {
    ID               string
    Command          Command
    WarningExitCodes []int
    SuccessExitCodes []int  // nil なら既定の [0]
    Transitions      []Transition
    Retry            *Retry
}
func (j *Job) EffectiveSuccessCodes() []int
func (j *Job) TransitionFor(on TransitionOn) (Transition, bool)

type Command struct {
    IsArray bool
    Line    string   // IsArray が偽のとき
    Argv    []string // IsArray が真のとき
}

type Retry struct {
    ExitCodes       []int
    Max             uint32
    IntervalSecs    uint64
    Multiplier      *uint32 // multiplier と max_interval は両方ある、または両方ない
    MaxIntervalSecs *uint64
}

type TransitionOn int // OnSuccess, OnWarning
type Transition struct {
    On     TransitionOn
    Target string // ジョブID。End のときは空
    IsEnd  bool   // 遷移先が @end
}

type ErrorKind int // FileMissing, Unreadable, TooLarge, Syntax, TypeMismatch, ... (03 章の全件)
type Error struct {
    Kind ErrorKind
    Path string // 定義ファイルのパス(表示用)
    Line int    // 0 なら位置なし
    // 追加のフィールド(キー名、ジョブID など)は 03 章に従う
}
func (e *Error) Error() string // 固定の文。入力の値や、ライブラリのエラー文を含めない

// Load は定義ファイルを読み、検証する。誤りは、すべて集めて返す(最初の 1 件で止めない)。
func Load(fsys fs.FS /* または path */, path string, expectedPipelineName string) (*Pipeline, []*Error)
// LooseRunIDEnv は、run_id_env だけをゆるく読む(読めない・キーがない・文字列でない・名前の形が違う、のどれでも "")。
func LooseRunIDEnv(path string) string
```

- `Load` の入力の取り方(`fs.FS` か、パスの文字列か)は、実装者が決めてよい。ただし、本番はファイルシステムを読み、テストは一時ディレクトリで動かす(`Deps.Defs` が包む。4 章)。
- 定義の書式は、拡張子で決める(`.yaml`/`.yml`/`.json`。それ以外は拒否)。[03-definition-format.md](03-definition-format.md)。

#### `internal/runid`

```go
type RunID string
func Validate(s string) error                 // 値の検査(文字集合・長さ。04・05 章)
func Resolve(flagValue *string, envName string, getenv func(string) (string, bool)) (*RunID, error)
```

#### `internal/flow`

```go
// --- 実行時の値の型(runner が作り、state が記録し、report が表示する) ---
type ExecResult struct {
    JobID              string
    Command            definition.Command
    Started            bool
    ExitCode           *int
    TerminatedBySignal *int
    StartFailure       bool
    Interrupted        bool
    SignalNumber       *int
}

type JobStatus string // "pending" | "success" | "warning" | "error" | "skipped"
type SkipReason string // "byTransition" | "byEnd"(ステート上の表記は 06 章)
type JobState struct {
    JobID      string
    Status     JobStatus
    ExitCode   *int
    SkipReason *SkipReason
}
type PipelineState struct {
    DefinitionKey  string
    PipelineName   string
    DefinitionPath string
    JobOrder       []string
    JobStates      []JobState
    ResumeJobID    *string
    EndReached     bool
    Completed      bool
    RunID          *string
    // 06 章の論理項目の全件
}

type Classification int // Success, Warning, Error
type Outcome struct {
    JobID             string
    Classification    *Classification // nil は「分類なし」
    ReportedExitCode  int
    ConductorError    bool
}
type RunMode int // Individual, Pipeline
type NextStep struct {
    JobID          string
    NextJobID      *string
    SkippedJobIDs  []string
    SkipReasonKind *SkipReason
    EndReached     bool
    Completed      bool
    ResumeJobID    *string
}
type DecisionKind int // Run, Skip, Reject
type DecisionReason int // SkipTarget, AfterEnd, AlreadyExecuted, OutOfOrder
type Decision struct { Kind DecisionKind; Reason DecisionReason }
type ResumePoint struct { ResumeJobID *string; EndReached bool; Completed bool }

// --- 純粋な関数 ---
func Classify(job *definition.Job, r ExecResult) Outcome
func ClassifyAndDecide(job *definition.Job, r ExecResult, states []JobState, def *definition.Pipeline, mode RunMode) (Outcome, NextStep)
func DecideBeforeRun(jobID string, def *definition.Pipeline, states []JobState, resumeJobID *string, endReached bool) Decision
func ResumePointOnInterrupt(def *definition.Pipeline, states []JobState) ResumePoint

type RetryDecision struct { Retry bool; WaitSecs uint64 }
func DecideRetry(job *definition.Job, r ExecResult, retriesDone uint32, lastWaitSecs uint64) RetryDecision
func ExitCodeFor(reported int, conductorError bool) int // 0〜255 に合わせる(10 章)

const ( // 終了コードの定数。10 章の全件
    ExitOK = 0
    // ...
)
```

#### `internal/state`

```go
type Durability int // Durable(同期してから置き換える), Fast(同期を省く。起動前の初期ステート)

type ErrorKind int // Unreadable, TooLarge, Corrupt, VersionMismatch, WriteFailed, RemoveFailed
type Error struct { Kind ErrorKind; Reason string /* 06 章の固定の理由文の識別子 */; Path string }
func (e *Error) Error() string // 固定の文

// 場所
type Location struct{ path string }
func LocationOf(definitionPath string, runID *runid.RunID) Location

// 記録関数(純粋。元を変更せず、新しい値を返す)
func InitialState(def *definition.Pipeline, definitionPath string, runID *runid.RunID) flow.PipelineState
func ApplyOutcome(s flow.PipelineState, o flow.Outcome, step flow.NextStep) flow.PipelineState
func ApplySkip(s flow.PipelineState, jobID string, reason flow.SkipReason /* ... */) flow.PipelineState
func ApplyInterrupt(s flow.PipelineState, rp flow.ResumePoint) flow.PipelineState

// 整合性の検査(純粋)
type Mismatch int
func CheckConsistency(s flow.PipelineState, def *definition.Pipeline, definitionPath string, runID *runid.RunID) (Mismatch, bool /* ok */)

// 入出力
type Writer struct { /* 時刻を返す関数 now func() time.Time を持つ(一時ファイル名用) */ }
func NewWriter(now func() time.Time) *Writer
func (w *Writer) Read(definitionPath string, runID *runid.RunID) (*flow.PipelineState, error)     // nil は「空」。完了のものも「空」
func (w *Writer) ReadRaw(definitionPath string, runID *runid.RunID) (*flow.PipelineState, error)  // 完了のものも返す(watch)
func (w *Writer) Write(definitionPath string, runID *runid.RunID, s flow.PipelineState, d Durability) error
func (w *Writer) Reset(definitionPath string, runID *runid.RunID) error
type ResetAllReport struct{ Removed int /* ... */ }
func (w *Writer) ResetAll(definitionPath string) (ResetAllReport, error)
```

- ステートの JSON の表現(ファイル用の構造体、キーの順、整数の幅の検査、深さの制限)は [06-state-file.md](06-state-file.md)。`flow.PipelineState` を直接 `json.Marshal` しない。`state` の内側の、ファイル用の構造体(非公開)を通す。
- `state` は、OS 別のディレクトリの同期を、`syncdir_unix.go`(`//go:build unix`)と `syncdir_windows.go`(`//go:build windows`)に置く。

#### `internal/runner`

骨格は [07-process-and-signals.md](07-process-and-signals.md)。公開するのは次のとおり。

```go
type CommandLine struct{ Program string; Args []string }
type CommandErrorKind int // UnclosedQuote, Empty
type CommandError struct{ Kind CommandErrorKind }
func (e *CommandError) Error() string // 固定の文(入力の値を含めない)

func Split(s string) (program string, args []string, err error)
func CheckCommand(c definition.Command) (CommandLine, error)

type Interrupt struct{ /* 非公開 */ }
func NewInterrupt() *Interrupt
func (i *Interrupt) Raise(n int)
func (i *Interrupt) Number() (n int, raised bool)
func (i *Interrupt) Done() <-chan struct{}

func RunJob(job *definition.Job, sig *Interrupt) flow.ExecResult
type WaitOutcome struct{ Interrupted bool; Number int }
func WaitFor(secs uint64, sig *Interrupt) WaitOutcome

var ErrAlreadyRegistered = errors.New("signal handling already registered")
func StartWatcher(sig *Interrupt) error // 本番: プロセスで 1 回だけ。2 回目は ErrAlreadyRegistered
```

#### `internal/report`

```go
type Locale int // English, Japanese
type Env interface { Getenv(name string) (string, bool); StdoutIsTerminal() bool } // cli.Env と同じ形(report は cli を import しない)
type Reporter struct{ /* ロケールと色を、最初に必要になったとき 1 回だけ判定する */ }
func NewReporter(env Env) *Reporter
// 文を作る関数群。戻り値は文字列(改行を含む)。書き込みはしない。
func (r *Reporter) VerboseResult(o flow.Outcome, er flow.ExecResult) string
func (r *Reporter) DefinitionErrors(errs []*definition.Error) string
func (r *Reporter) CommandError(path, jobID string, e *runner.CommandError) string
func (r *Reporter) StateError(e *state.Error) string
func (r *Reporter) Watch(s *flow.PipelineState, ...) string
func (r *Reporter) Usage() string
// ほか、08 章のメッセージの全件
```

- `report` は、`definition`・`flow`・`state`・`runner` の型を読むだけで、それらを変更しない。環境変数と端末の判定は、`Env` を通して受け取る(`os.Getenv` を直接呼ばない)。

#### `internal/cli`

2.2 節と 3 章(`Deps`)。

#### `cmd/conductor`

```go
func main() { os.Exit(realMain()) }
func realMain() (code int) // 最初に defer+recover を設定し、cli.Run(os.Args[1:], cli.SystemDeps()) の戻り値を返す
```

### 2.2 `cli` の公開するもの

```go
// Run は、引数を解析し、サブコマンドを実行して、終了コード(0〜255)を返す。os.Exit は呼ばない。
func Run(args []string, d Deps) int
func SystemDeps() Deps // 本番の実体の束
```

---

## 3. 依存の向き【規定】

矢印は「import する」。**循環させない**。`go build ./...` が通ることが、最低の検査である。

```text
cmd/conductor ─▶ cli
cmd/conductor-testjob ─▶ (標準ライブラリだけ)

cli ─▶ report, runner, state, flow, definition, convert, runid, version

report ─▶ state, runner, flow, definition        (cli を import しない)
runner ─▶ flow, definition
state  ─▶ flow, definition, runid
flow   ─▶ definition
convert ─▶ (外部の YAML ライブラリ、標準ライブラリだけ)
runid  ─▶ (標準ライブラリだけ)
definition ─▶ (外部の YAML ライブラリ、標準ライブラリだけ)
version ─▶ (なし)

testsupport ─▶ flow, definition, runner, state, runid, convert   (cli を import しない)
```

図(下へ向かって依存する):

```text
                 cmd/conductor
                       │
                      cli  ─────────┬────────────┐
        ┌──────┬──────┼──────┐      │            │
     report  runner  state  convert version     runid
        │ ╲      │ ╲    │ ╲
        │  ╲     ▼  ╲   ▼  ╲
        │   ╲──▶ flow ◀──── (state も runner も flow を使う)
        ▼          │
     (state, runner, flow, definition を読む)
                   ▼
              definition
```

規則:

1. **`cli` が最上位**。他のパッケージは `cli` を import しない。`report` も `cli` を import しない(環境の判定は `report.Env` の形で受け取る)。
2. **`definition` と `flow` が下層**。`definition` は、`internal/` のどれにも依存しない。`flow` は `definition` だけに依存する。入出力に触れない(`definition.Load` の読み取りを除く)。
3. **`state` と `runner` は、`flow` と `definition` の型を使うが、互いに依存しない**。`state` は「ステートを読み書きする」だけで、ジョブを起動しない。`runner` は「ジョブを起動して待つ」だけで、ステートに触れない。両者をつなぐのは `cli`。
4. **`convert` は、他の `internal/` に依存しない**。定義の検証(`definition`)を呼ばない。
5. **`report` は、データを受け取って文を作る**。判断をしない。何を表示するかは `cli` が決める。
6. **OS ごとの分岐は、`runner` の `platform_*.go`、`state` の `syncdir_*.go`、`cmd/conductor` の入口の OS 別のファイルだけ**に置く([07-process-and-signals.md](07-process-and-signals.md) の 8 章)。
7. **`testsupport` は `cli` を import しない**: `cli` のテスト(`package cli`)が `testsupport` を import しても、循環しないため。`testsupport` の偽実装は、`cli` のインターフェースを、メソッドの形が合うこと(構造的な一致)で満たす。インターフェースのメソッドのシグネチャは、`cli` の型ではなく、下層のパッケージの型だけで書く(4 章)。
8. 「`internal/` 配下の他のパッケージ」を、他のパッケージのテストが import する場合も、循環させない(外部テストパッケージ `xxx_test` を使えば、循環せずに import できる)。

---

## 4. `Deps`(差し替え口)とテスト用の偽実装【規定】

### 4.1 方針

`cli` の流れは、環境・時刻・ファイル(定義・ステート・変換)・プロセス・シグナル・標準出力/標準エラーを、**まとめて 1 つの `Deps` 構造体**で受け取る。各フィールドは interface。
本番の入口(`SystemDeps`)は本番の実体を渡し、テストは偽の実体を渡す。純粋な判断(`flow` の関数、`state` の記録関数、`report` の文の組み立て)は、interface を通さず、`cli` が直接呼ぶ(差し替え不要)。

### 4.2 interface の定義(`internal/cli/deps.go`)

```go
// 環境(環境変数、標準出力が端末か)。
type Env interface {
    Getenv(name string) (value string, ok bool) // 未設定は ok=false
    StdoutIsTerminal() bool
}

// 時刻。ステートの一時ファイル名などに使う。
type Clock interface {
    Now() time.Time
}

// 標準出力・標準エラーへの書き込み。書くたびに見えるようにする(バッファを持たない)。失敗は error で返す。
type Sink interface {
    WriteOut(p []byte) error
    WriteErr(p []byte) error
}

// 定義ファイルの読み込み。
type Definitions interface {
    Load(path string, expectedPipelineName string) (*definition.Pipeline, []*definition.Error)
    LooseRunIDEnv(path string) string
}

// ステートの読み書き(ファイルシステム)。
type States interface {
    Read(definitionPath string, runID *runid.RunID) (*flow.PipelineState, error)
    ReadRaw(definitionPath string, runID *runid.RunID) (*flow.PipelineState, error)
    Write(definitionPath string, runID *runid.RunID, s flow.PipelineState, d state.Durability) error
    Reset(definitionPath string, runID *runid.RunID) error
    ResetAll(definitionPath string) (state.ResetAllReport, error)
}

// 変換。
type Converter interface {
    Convert(inputPath string, to convert.Format) (string, error)
}

// プロセス(ジョブの起動と待ち)。
type Process interface {
    CheckCommand(c definition.Command) (runner.CommandLine, error)
    RunJob(job *definition.Job, sig *runner.Interrupt) flow.ExecResult
    Wait(secs uint64, sig *runner.Interrupt) runner.WaitOutcome // リトライの待機
}

// シグナルの監視。
type Signals interface {
    // 受け取った最初のシグナルの番号を sig に立てる監視を始める。
    // すでに登録済みなら runner.ErrAlreadyRegistered(流れは成功と同様に続ける)。
    // それ以外の error は、登録の失敗(終了コード 70)。
    Start(sig *runner.Interrupt) error
}

type Deps struct {
    Env       Env
    Clock     Clock
    Out       Sink
    Defs      Definitions
    States    States
    Converter Converter
    Process   Process
    Signals   Signals
}
```

- 引数 `args`、現在のカレントディレクトリ(`os.Getwd` が要るとき)も、環境の差し替えで扱えるよう、必要なら `Env` に `Getwd() (string, error)` を加える(実装者が決める)。
- **ファイルシステムの差し替え**は、`Definitions`・`States`・`Converter` の 3 つの interface が担う。本番の実装は、実ファイルを読み書きする。偽の実装は、メモリに持つ(ファイルなし)。ステートの書き込みの原子的な置換の失敗や、ディスクの異常の検査は、`state` パッケージの内側のテスト(一時ディレクトリの実ファイル)で行う。
- `Sink` は、書き込みの失敗を `error` で返す(`panic` にしない)。標準出力と標準エラーを混ぜない。**書き込みは `cli` の 1 か所の補助関数を通す**(順序を保つ)。
- `Process.Wait` は `runner.WaitFor` を包む。偽の実装は、待ち時間を記録するだけで実際には待たない。

### 4.3 本番の実装(`internal/cli/system.go`)【目安】

| 名前 | interface | 中身 |
|------|-----------|------|
| `systemEnv` | `Env` | `os.LookupEnv`。端末の判定は `os.Stdout` が文字デバイスか(`Stat().Mode()&os.ModeCharDevice`。UNIX 系と Windows の違いは、実装時に確かめる。**未確認**) |
| `systemClock` | `Clock` | `time.Now` |
| `stdSink` | `Sink` | `os.Stdout.Write` / `os.Stderr.Write`(`os.File` の `Write` は、バッファを持たない) |
| `systemDefs` | `Definitions` | `definition.Load`、`definition.LooseRunIDEnv` |
| `systemStates` | `States` | `state.NewWriter(clock.Now)` を持ち、メソッドを委譲 |
| `systemConverter` | `Converter` | `convert.Convert` |
| `systemProcess` | `Process` | `runner.CheckCommand`、`runner.RunJob`、`runner.WaitFor` |
| `systemSignals` | `Signals` | `runner.StartWatcher` |

### 4.4 偽の実装(`internal/testsupport`)【規定】

偽の実装は、次の振る舞いを持つ(名前は実装者が決める)。すべて `cli` を import せず、`cli` の interface を構造的に満たす。

| 偽の実装 | 振る舞い | 主な検査 |
|-----------|----------|----------|
| `FakeEnv` | 環境変数を表(`map`)で与える。`Getenv` を呼んだ回数と名前を記録する。`StdoutIsTerminal` の値を設定する | ロケール(`LC_ALL`・`LC_MESSAGES`・`LANG`)・`NO_COLOR`・run ID の環境変数。ロケールの判定を**遅延**して、最初に必要になったとき 1 回だけであること(成功・スキップで `-v` なしなら、読まない) |
| `FakeClock` | 固定の時刻を返す。進める操作を持つ | 一時ファイル名の時刻 |
| `BufSink` | `WriteOut`・`WriteErr` を、別々の `bytes.Buffer` に溜める。順序の記録も持つ。「n 回目の書き込みで失敗する」設定を持つ | 出力の中身、`cannot write to standard output`(`watch` の 65、`convert` の 67) |
| `FakeDefs` | パスごとに、決めた `*Pipeline` や誤りの並びを返す。または、実ファイル(一時ディレクトリ)を `definition.Load` で読む版 | 定義の誤り(64)の経路 |
| `MemStates` | ステートをメモリに持つ(ファイルなし)。読み込み・書き込み・削除の呼び出しを記録する。「読み込みで `state.Error` を返す」「書き込みで失敗する」設定を持つ | 大きなジョブ数(1,000 個)の流れの測定、ステートの異常系(65) |
| `FakeConverter` | 入力パスごとに、決めた文字列かエラーを返す | `convert` の 65・67 |
| `ScriptProcess` | ジョブIDごとに、台本の `flow.ExecResult`(終了コード・シグナル終了・起動失敗・中断)を順に返す。呼び出しの記録(何が、何回、どの順で実行されたか)を持つ。`CheckCommand` は、台本の誤り、または `runner.CheckCommand` を呼ぶ。`Wait` は、待ち時間(秒)を記録するだけ。「`Wait` が中断を返す」設定を持つ | 分類・遷移・再開・リトライ(待ち時間の列 1・2・4・…)・中断の経路。実際の待ちをしない |
| `FakeSignals` | `Start` の戻り値を設定する(成功、`ErrAlreadyRegistered`、その他の失敗)。「開始時に、すでに `sig.Raise(n)` を呼ぶ」設定を持つ | 登録の失敗(70)、登録済み、開始時にすでに合図が立っている |
| 補助 | 一時ディレクトリ(`t.TempDir()`)に、定義ファイルを書く。実行して、ステートファイルの内容を読み戻す | 実ファイルを使うテスト(受け入れテストの下位) |

- 偽の実装が、複数の goroutine から触れられる場合は、`sync.Mutex` で保護する(多くは、メインの goroutine からだけ)。
- 実プロセスを使うテスト(`runner` の内側と受け入れテスト)は、`cmd/conductor-testjob` をビルドして使う([11-quality-and-build.md](11-quality-and-build.md))。

---

## 5. 実行のシーケンス

### 5.1 入口

```text
main():
    os.Exit(realMain())                               # 終了は、ここで 1 回だけ

realMain() (code int):
    defer recover → 標準エラーに "conductor: error: internal error\n" を書く(失敗は無視)。code = 70     # 最初に設定
    return cli.Run(os.Args[1:], cli.SystemDeps())

cli.Run(args, d):
    parsed = 引数を解析する(04 章)
      誤り           → 使い方の誤りを標準エラーに表示、return 68
      help           → 使い方全文を標準出力へ、return 0
      version        → `conductor 0.8.0` を標準出力へ、return 0
      コマンド       → run / pipeline / watch / reset / validate / convert のどれかへ
```

### 5.2 `pipeline` の実行

`prepare`(前処理)の順序と、そこでの失敗の終了コードは、[05-execution-flow.md](05-execution-flow.md) と [10-errors-and-exit-codes.md](10-errors-and-exit-codes.md) に固定されている。

```text
cli: pipeline
 ├─ prepare
 │    ├─ sig := runner.NewInterrupt();  d.Signals.Start(sig)       → ErrAlreadyRegistered は続行 / その他の失敗 70(定義の検査より前)
 │    ├─ d.Defs.Load(...)                                        → 失敗 64(パイプライン名の照合を含む)
 │    ├─ d.Process.CheckCommand(全ジョブ)                          → 失敗 64(誤りのあるジョブすべてを 1 ジョブ 1 行で報告)
 │    ├─ run ID の決定(--run-id | 環境変数。runid パッケージ)      → 失敗 68
 │    ├─ d.States.Read(...)  (完了のものは「空」)                  → 失敗 65
 │    └─ state.CheckConsistency(...)                              → 食い違い 65
 └─ for job in 定義のジョブ(記述順):
      ├─ flow.DecideBeforeRun(...) → Run / Skip / Reject
      │    ├─ Reject → report: 表示、return 66
      │    ├─ Skip   → state.ApplySkip → 変化があれば d.States.Write(Durable)、続行(完了なら finish)
      │    └─ Run:
      │         ├─ ステートが空なら state.InitialState → d.States.Write(Fast)
      │         ├─ リトライ込みの実行
      │         │    loop { r := d.Process.RunJob(job, sig)
      │         │           dec := flow.DecideRetry(job, r, retriesDone, lastWait)
      │         │           Retry なら report: 知らせ(常に標準エラー) → w := d.Process.Wait(secs, sig)
      │         │                    w.Interrupted なら「起動しなかった中断」の ExecResult(Started=false, Interrupted=true, SignalNumber)を作って抜ける }
      │         ├─ flow.ClassifyAndDecide(..., RunMode=Pipeline)
      │         ├─ report: 表示(-v のときの結果行・遷移行。常時のエラー行)
      │         ├─ state.ApplyOutcome → d.States.Write(Durable)
      │         ├─ エラーまたは分類なし → return flow.ExitCodeFor(...)
      │         ├─ 完了 → finish(--remove-on-complete なら d.States.Reset)、return 0
      │         └─ sig に合図が立っている → flow.ResumePointOnInterrupt → state.ApplyInterrupt → d.States.Write(Durable)、return 128 + 番号
 └─ ループを抜けても完了していない → "cannot determine whether the pipeline is complete"、return 65
```

- `run` は、`prepare`(ジョブIDの存在の検査を含む)のあと、指定した 1 ジョブについて同じ「判断 → 実行 → 分類 → 記録」を行う。分類と次の動きは `RunMode = Individual` で求める。
- `watch`・`reset`・`validate`・`convert` は、シグナルを準備しない。それぞれ、ステートの読み込み(`ReadRaw`)・ステートの削除(`Reset`、`ResetAll`)・定義の検査(`Defs.Load` と `Process.CheckCommand`)・変換(`Converter.Convert`)を呼ぶ。
- `pipeline` は、次のジョブを「次の動き」の戻り値で決めない。記述順に全ジョブを見て、起動前の判断で決める(スキップ・再開は、ステートが運ぶ)。
- 合図の確認は、`cli` が `sig.Number()` で行う。`runner.RunJob` と `runner.WaitFor` が、待機中の合図を扱う([07-process-and-signals.md](07-process-and-signals.md))。

### 5.3 責務の流れ(データ)

```text
定義ファイル ──definition.Load──▶ *definition.Pipeline ─┐
ステートファイル ──state.Read──▶ flow.PipelineState ──────┤
                                                          ▼
                       flow.DecideBeforeRun ──▶ Run / Skip / Reject
runner.RunJob ──▶ flow.ExecResult ──flow.ClassifyAndDecide──▶ (Outcome, NextStep)
                                         │
        state.Apply*(結果・スキップ・中断) ──▶ 新しい PipelineState ──state.Write──▶ ステートファイル
Outcome.ReportedExitCode ──flow.ExitCodeFor──▶ conductor の終了コード
```

### 5.4 中断の伝わり方

- 中断の合図(`runner.Interrupt`)は、「立った/立っていない」と「最初のシグナル番号」を持つ。シグナル受信の goroutine が `Raise` で立て、待機ループ(`RunJob`、`WaitFor`)と `cli` の流れが読む(メインの goroutine)。
- 共有は `sync.Mutex` で保護する(読み書きの順序の乱れを起こさない)。詳細は [07-process-and-signals.md](07-process-and-signals.md) の 6 章と 9 章。

---

## 6. ファイル単位の責務表【目安】

実装者は、ファイルの分け方を変えてよい。ただし、パッケージの責務と依存の向き(2・3 章)と、OS 別のファイルの置き場所(`_unix.go`、`_windows.go`)は守る。

| パス | 責務 |
|------|------|
| `cmd/conductor/main.go` | `main` と `realMain`(`defer`+`recover`、`os.Exit` を 1 回) |
| `cmd/conductor/signal_unix.go`、`signal_windows.go` | 入口の OS 別の準備(必要な場合のみ。たとえば書き込み先が壊れたパイプのときの扱い。[07-process-and-signals.md](07-process-and-signals.md) の 12 章) |
| `cmd/conductor-testjob/main.go` | テストジョブ(サブコマンド: 終了コード、スリープ、出力、環境・引数・作業ディレクトリの表示、標準入力を読む、シグナルを無視する、など) |
| `internal/version/version.go` | 版の定数 |
| `internal/definition/types.go` | 定義の型 |
| `internal/definition/errors.go` | 誤りの種類(`ErrorKind`)と `Error` |
| `internal/definition/load.go` | `Load`(場所の決定、読み取り、書式の判定、段階の組み立て) |
| `internal/definition/intake.go` | ファイルの読み取り、大きさの上限、拡張子の判定 |
| `internal/definition/parse_yaml.go`、`parse_json.go` | YAML/JSON の構文解析(位置つき) |
| `internal/definition/depth.go` | 入れ子の深さの検査 |
| `internal/definition/walk.go` | 構造・型・遷移・終了コード・名前の検査 |
| `internal/definition/loose.go` | `run_id_env` のゆるい読み取り |
| `internal/convert/convert.go` | YAML ⇔ JSON の変換の入口(`Convert`、`ConvertText`) |
| `internal/convert/read_yaml.go`、`write_yaml.go`、`json.go`、`error.go` | 読み取り(エイリアスの展開と上限)、書き出し、JSON、誤り |
| `internal/runid/runid.go` | run ID の型、検査、決定 |
| `internal/flow/types.go` | 実行時の値の型 |
| `internal/flow/classify.go` | 分類 |
| `internal/flow/decide.go` | 起動前の判断 |
| `internal/flow/next_step.go` | 次の動き |
| `internal/flow/resume.go` | 中断時の再開位置 |
| `internal/flow/retry.go` | リトライの判断と待ち時間(飽和の計算) |
| `internal/flow/exit.go` | 終了コードの定数と範囲合わせ |
| `internal/state/types.go`、`location.go` | ファイル用の構造体、場所と名前 |
| `internal/state/record.go` | 記録関数(純粋) |
| `internal/state/consistency.go` | 整合性の検査(純粋) |
| `internal/state/reader.go` | 読み込み(14 段階の検査、異常系) |
| `internal/state/writer.go` | 書き込み(一時ファイル + `fsync` + `os.Rename`)、`Reset` |
| `internal/state/reset_all.go` | `ResetAll` |
| `internal/state/syncdir_unix.go`、`syncdir_windows.go` | ディレクトリの同期(OS 別) |
| `internal/runner/split.go` | 分割と検査 |
| `internal/runner/run.go` | ジョブの起動と結果の取り出し |
| `internal/runner/wait_job.go` | 待機ループ |
| `internal/runner/wait_retry.go` | リトライの待機 |
| `internal/runner/interrupt.go` | 中断の合図 |
| `internal/runner/watcher.go` | シグナル監視の共通の部分(登録は 1 回、受信の goroutine) |
| `internal/runner/platform_unix.go`、`platform_windows.go` | OS 別の部分 |
| `internal/report/locale.go` | ロケールと色の判定(遅延) |
| `internal/report/catalog.go` | メッセージカタログ(英語・日本語) |
| `internal/report/format.go` | 行の組み立て(`-v`、誤り) |
| `internal/report/escape.go` | エスケープと切り詰め |
| `internal/report/watch.go` | `watch` の表 |
| `internal/report/usage.go` | 使い方 |
| `internal/cli/args.go` | 引数の字句の解析と検査 |
| `internal/cli/deps.go`、`system.go` | `Deps` の interface と本番の実装 |
| `internal/cli/cli.go` | `Run`(入口の分岐、終了コード) |
| `internal/cli/prepare.go` | 前処理(`run`・`pipeline` に共通) |
| `internal/cli/run.go`、`pipeline.go` | `run`・`pipeline` の流れ |
| `internal/cli/exec_job.go` | リトライ込みの 1 ジョブの実行、記録、表示 |
| `internal/cli/aux.go` | `watch`・`reset`・`validate`・`convert` |
| `internal/testsupport/*.go` | 4.4 節の偽の実装と補助 |

`_test.go` は、各パッケージの内側(同じディレクトリ)に置く。実バイナリを起動する受け入れテストの置き場所は [11-quality-and-build.md](11-quality-and-build.md)。

---

## 7. 並行性の方針【規定】

| 項目 | 方針 |
|------|------|
| 基本 | 製品のコードは、ほぼ単一の goroutine(メイン)で動く。goroutine は、シグナル受信(0 または 1 つ)と、実行中のジョブの `Wait`(同時に最大 1 つ)だけ |
| 出力とステート | 標準出力・標準エラーへの書き込みと、ステートの読み書きは、メインの goroutine だけが行う |
| 共有する可変の値 | `runner.Interrupt`(`sync.Mutex` で保護)と、シグナル登録の印(`sync.Mutex` で保護)だけ |
| 待ち方 | `select` で、チャネルと `time.Timer` を待つ。ポーリングをしない |
| キャンセル | `context.Context` を使わない。`runner.Interrupt` で中断を伝える |
| goroutine の終わり | すべての goroutine に、終わる条件がある([07-process-and-signals.md](07-process-and-signals.md) の 9.1 節)。リークさせない |
| 検査 | `go test -race ./...` が通る。`go vet` の copylocks(`Interrupt` をコピーしない) |
| 排他制御 | 同じ定義への 2 つの `conductor` の同時の実行を、排他しない(両方が動き、ステートは、どちらも壊れた形では読めない。原子的な置換による) |

詳細(誰が所有し、どう終了するか、データ競合の避け方)は、[07-process-and-signals.md](07-process-and-signals.md) の 9 章。

---

## 8. エラー処理の方針【規定】

| 項目 | 方針 |
|------|------|
| 基本 | 失敗は、戻り値の `error`(または、検査の結果を表す値)で返す。外部の入力(引数・環境・ファイル・OS の結果)に対して、`panic` しない |
| `panic` を許す場所 | `main` の `recover` が捕まえる、実装の不具合(論理の矛盾)だけ。`recover` は `main` の 1 か所に置く(`cli` の内側では呼ばない) |
| 終了 | `os.Exit` は `cmd/conductor/main.go` の `main` だけ。`log.Fatal`、`panic` による終了を使わない。`cli.Run` は、終了コードを返す |
| 種類 | パッケージごとに、**種類(列挙)を持つエラー型**を公開する: `definition.Error`、`runner.CommandError`、`state.Error`、`convert` の誤り。呼び出し側は、`errors.As` で種類を取り出し、`switch` で分岐する。文字列を比べて分岐しない |
| 表示 | 利用者への表示の文は、`report` が、種類と引数(ジョブID、パスなど)から組み立てる(メッセージカタログ)。**`err.Error()` の文字列や、ライブラリ(YAML、`os`)のエラー文を、そのまま表示しない**(入力の値や、内部の情報が出るため。[10-errors-and-exit-codes.md](10-errors-and-exit-codes.md))。`Error()` が返す文は、固定の短い文だけにする |
| ラップ | 内部では、`fmt.Errorf("...: %w", err)` で原因を包んでよい。ただし、表示に使わない。原因は、デバッグ用に残してもよいが、利用者への出力に出さない |
| 複数の誤り | 定義の検査・`command` の検査は、誤りを**すべて集めて**返す(`[]*Error`)。1 件で止めない |
| 値で返す失敗 | ジョブの起動の失敗(`ExecResult.StartFailure`)、出力の書き込みの失敗(`Sink` の `error`)は、値として扱い、終了コードに結びつける |
| 終了コードの決定 | 「失敗の種類 → 終了コード」の対応は `cli` の 1 か所で行う([10-errors-and-exit-codes.md](10-errors-and-exit-codes.md)) |
| 想定外 | `recover` で捕まえ、固定の英語 1 行 `conductor: error: internal error` と終了コード 70([07-process-and-signals.md](07-process-and-signals.md) の 10 章) |
| 静的検査 | `errcheck` 相当(`golangci-lint`)で、戻り値の `error` を無視しない。意図して無視する場合(シグナルの送信の失敗など)は、`_ =` と、理由の注釈を付ける |

---

## 9. 設計判断のチェックリスト(確認)

| # | 判断 | この版の決定 |
|---|------|--------------|
| 1 | パッケージ構成 | 2 章のとおり固定 |
| 2 | 実行時の値の型の置き場所 | `internal/flow`(`state` と `runner` は、`flow` に依存する。`flow` は `state` と `runner` に依存しない) |
| 3 | ステートの書式 | JSON(`encoding/json`)。ファイル用の構造体を `state` の内側に持つ([06-state-file.md](06-state-file.md)) |
| 4 | 差し替え口 | `Deps` 構造体(interface のフィールド)。4 章 |
| 5 | シグナル | `os/signal`(`signal.Notify`)。受信の goroutine は 1 つ。合図は `runner.Interrupt` |
| 6 | 待機 | `select` と `time.Timer`。ポーリングなし |
| 7 | OS 別 | ビルドタグ(`//go:build unix`、`windows`)と `_unix.go` / `_windows.go` のファイル。`_unix.go` だけでは制約にならないので、1 行目付近に明示 |
| 8 | エラー | 種類を持つエラー型、`errors.As`、`report` が文を作る |
| 9 | 終了 | `os.Exit` は 1 か所 |
| 10 | 想定外の失敗 | `main` の `defer`+`recover` で 70 |

---

## 10. 未確認の事項

- 端末の判定(標準出力が端末か)に、標準ライブラリだけで使う方法(`os.File.Stat` の `ModeCharDevice`)が、UNIX 系と Windows の両方で、利用者が期待する結果になるか。実装時に 3 OS で確かめる。外部ライブラリ(端末の判定用)は、この仕様では採らない前提。
- `definition.Load` の引数を `fs.FS` にするかパスにするか(裁量とした)。
- `internal/testsupport` の偽実装が、`cli` の interface を構造的に満たすことは、コンパイル時に、`cli` のテストの側で `var _ Process = (*testsupport.ScriptProcess)(nil)` のように確かめる(この形の検査が、循環を起こさずに書けることは、実装時に確かめる。`cli` のテストは `package cli` か `package cli_test` のどちらでもよい)。
- Go の標準ライブラリ・`os/exec`・`os/signal` の挙動の記述は、公式のドキュメントの知識によるもので、この文書の作成時に、コードを実行して確かめていない([07-process-and-signals.md](07-process-and-signals.md) の 12 章)。
