# 02 アーキテクチャ(ディレクトリ・依存方向・型の骨格・`Deps`・実行のシーケンス)

対象: `conductor` v0.8.0 と同じ挙動を実現する、TypeScript(Node.js)の実装の構成。この文書は、ディレクトリの分け方を**固定**し、各ディレクトリの責務、依存の向き、公開する型・インターフェース・関数の骨格(シグネチャ)、テストのための差し替え口(`Deps`)、実行の流れ、非同期とエラー処理の方針、`package.json` と `tsconfig.json` の規定を定める。
各機能の挙動は、[03-definition-format.md](03-definition-format.md)(定義)、[04-cli.md](04-cli.md)(引数)、[05-execution-flow.md](05-execution-flow.md)(実行の流れ)、[06-state-file.md](06-state-file.md)(ステート)、
[07-process-and-signals.md](07-process-and-signals.md)(プロセスとシグナル)、[08-output-and-i18n.md](08-output-and-i18n.md)(出力)、[09-converter.md](09-converter.md)(変換)、[10-errors-and-exit-codes.md](10-errors-and-exit-codes.md)(エラーと終了コード)、[11-quality-and-build.md](11-quality-and-build.md)(品質)で定める。この文書は、構成だけを定める。

骨格のコードは、型とシグネチャの例であり、実装ではない。フィールドの名前や型で、他の文書(特に 03・05・06)の定めと食い違う箇所があれば、その文書の定めを正とし、この文書の骨格を読み替える。

## 規定の強さの表記

| 表記 | 意味 |
|------|------|
| 【規定】 | 守る。ディレクトリの名前、依存の向き、責務の分け方、非同期の規則 |
| 【目安】 | 推奨。実装者が変えてよい(ファイルの分け方、補助の関数の名前など) |
| 未確認 | 確かめていない。実装時に確かめる(10 章) |

---

## 1. 設計の前提

| 項目 | 内容 |
|------|------|
| 言語・実行環境 | TypeScript と Node.js。サポートする Node は、実装時点の最新の LTS 系列と、その 1 つ前の LTS 系列(具体的な版は実装時に確認。この文書の作成時点では、26 系と 24 系が該当すると思われるが、未確認)。`package.json` の `engines` に最小版を書く |
| モジュール方式 | ESM。`"type": "module"`、`module` と `moduleResolution` は `NodeNext`。相対 import には、拡張子 `.js` を付ける(TypeScript のソースでも `import ... from './x.js'`) |
| 型検査 | `strict`(8.2 節の `tsconfig.json`)。`any` を使わない(`unknown` を使い、絞り込む) |
| パッケージ名 | `conductor`(置き換えてよい。置き換えたときは、`package.json` の `name` と、`bin` のキーだけを変える。ソースの import には影響しない) |
| 構成 | 1 つのパッケージに、実行ファイル 1 つ(`src/bin/conductor.ts`)と、`src/` 配下のディレクトリ 9 つ(2 章)。テストの補助(`test/support/`、`test/jobs/`)は配布物に含めない |
| 実行時の依存 | YAML の読み取りだけ(1 つのライブラリ。書き出しは自前。選定は [03-definition-format.md](03-definition-format.md) と [09-converter.md](09-converter.md))。JSON は自前の字句・構文解析で読む。引数の解析は自前(`node:util` の `parseArgs` は使わない。理由は [04-cli.md](04-cli.md))。ほかは `node:` の標準モジュールだけ(`node:child_process`、`node:fs`、`node:path`、`node:os`、`node:process`、`node:readline` など) |
| 開発時の依存 | TypeScript、ESLint と typescript-eslint、Prettier、Vitest と `@vitest/coverage-v8`、`@types/node`(選定と版は実装時に確認。[11-quality-and-build.md](11-quality-and-build.md)) |
| 公開の範囲 | ライブラリとして公開しない。`exports` は `package.json` だけ(8.1 節) |
| 配布物 | `conductor` だけ(`dist/` と `package.json`)。テスト用ジョブは含めない |

---

## 2. ディレクトリ一覧と責務【規定】

| ディレクトリ | 責務 | 入出力 |
|--------------|------|--------|
| `src/bin/` | 薄い入口(`conductor.ts`)。先頭に `#!/usr/bin/env node`。想定外の例外と `Promise` の拒否を捕まえて、`conductor: error: internal error` を標準エラーに 1 行出し、終了コード 70 にする。本番の `Deps` を作って `cli` の `run` を呼ぶ。終了(`process.exit`)は、ここだけ | プロセスの引数・環境・標準入出力 |
| `src/definition/` | 定義ファイルの読み込み(場所の決定と読み取り、YAML/JSON の構文解析、入れ子の深さ、`format_version`、構造・型・遷移・終了コード・名前の検査)。誤りを位置(行)つきで返す。**定義の型**(`Definition`、`Job`、`Command`、`RetryPolicy`、`Backoff`、`Transition`)と、誤りの種類(`DefinitionErrorKind`。03 章)を持つ。`run_id_env` だけのゆるい読み取りも持つ。木の読み取り(`readTree`)は `convert` が共有する | ファイルを読む(`load` だけ)。ほかは文字列に対する純粋な処理 |
| `src/convert/` | YAML ⇔ JSON の変換(`convert --to yaml\|json`)。定義の検証はしない。木の読み取りは `definition` と共有する | 入力ファイル 1 つを読む(`convertFile`)。`convertText` は文字列に対して純粋 |
| `src/runid/` | run ID の値の検査と、決定(`--run-id` と環境変数のどちらを使うか)に使う純粋な関数。**`RunId` の型**を持つ | なし |
| `src/flow/` | 純粋な判断。実行結果の分類(成功・警告・エラー・分類なし)と報告する終了コード、遷移と次の動き、起動前の判断(実行・スキップ・拒否)、中断時の再開位置、リトライの判断と待ち時間(`bigint` の飽和演算)、終了コードの範囲合わせ。**実行時の値の型**(`JobExecution`、`JobState`、`PipelineState`、`JobOutcome`、`NextStepResult` など)と、終了コードの定数を持つ | なし(引数だけから結果を決める。ファイル・プロセス・環境変数・時刻に触れない) |
| `src/state/` | ステートの読み書き。JSON の表現(ファイル用の型と、固定の順序の書き出し)、記録関数(結果・スキップ・中断を、新しい `PipelineState` として作る純粋な関数)、読み込み(14 段階の検査と異常系)、書き込み(一時ファイル + 同期 + 名前の付け替え + ディレクトリの同期)、定義との整合性の検査、`reset` と `reset --all`、ステートファイルの場所と名前。**ステートの誤りの型**(`StateError`)を持つ | ステートのファイル(とそのディレクトリ)。記録関数と整合性の検査は純粋 |
| `src/runner/` | `command` の分割と検査、`node:child_process` によるジョブの起動と待ち、シグナルの転送、リトライの待機、中断の合図(`Interrupt`)、シグナル監視、プラットフォーム層(`platform.ts`。`process.platform` を読む唯一の場所)。**`CommandLine` と `CommandError` の型**を持つ | 子プロセス、時刻(待ち)、OS のシグナル。分割と検査は純粋 |
| `src/report/` | 出力。メッセージカタログ(英語・日本語)、ロケールと色の判定、`-v` の行、誤りの行、`watch` の表、使い方(`--help`)、入力由来の文字列のエスケープと切り詰め。**文を作るだけ**で、書き込みは `cli` が `Sink` へ行う | 環境変数・端末かどうか・OS の種別は、引数(`ReportEnv`)で受け取る(自分では読まない) |
| `src/cli/` | 引数解析(誤りの優先順位)、各サブコマンド(`run`・`pipeline`・`watch`・`reset`・`validate`・`convert`)の組み立て、`Deps`(差し替え口のインターフェース)とその本番の実装、終了コードの決定。上記のすべてを呼ぶ | `Deps` を通した、すべての入出力 |
| `src/version.ts` | 版の定数(`export const VERSION = "0.8.0"`)。`conductor --version` の表示と、リリースの検査([11-quality-and-build.md](11-quality-and-build.md))が使う。`package.json` の `version` と一致することを、テストで検査する | なし |
| `test/` | 単体テスト(`src/` と同じ構成)と受け入れテスト(`test/acceptance/`。実際の `dist/bin/conductor.js` を起動する)。配布物に含めない | テストだけが使う |
| `test/support/` | テスト用の偽実装(`Deps` の各インターフェースを満たす)、一時ディレクトリで定義やステートを作る補助(4.4 節) | テストだけが使う |
| `test/jobs/` | 受け入れテスト用のテストジョブ(終了コード、スリープ、標準出力への書き込み、環境変数・引数の出力など。[11-quality-and-build.md](11-quality-and-build.md))。配布物に含めない | テストだけが起動する |

### 2.1 各ディレクトリの公開(`export`)するものの骨格【規定】

各ディレクトリの `index.ts` が、他のディレクトリから使われる名前だけを `export` する(バレル)。他のディレクトリからの import は、**必ず相手の `index.ts` を通す**(`import ... from '../definition/index.js'`)。`index.ts` に出さない補助の名前は、ディレクトリの内側だけで使う(補助の名前は実装者が決める)。型だけの import は `import type` を使う(`verbatimModuleSyntax`。8.2 節)。

型の置き場所の決定(依存の最下層に置く。循環を避けるため。3 章):

| 型 | 置き場所 |
|----|----------|
| 定義の型(`Definition`、`Job`、`Command`、`RetryPolicy`、`Backoff`、`Transition`)、定義の誤り(`DefinitionError`) | `src/definition/` |
| `RunId` | `src/runid/` |
| 実行時の値の型(`JobExecution`、`JobState`、`PipelineState`、`JobOutcome`、`NextStepResult`、`ResumePoint` など)、終了コードの定数 | `src/flow/` |
| ステートの誤り(`StateError`)、`Durability`、`StateStore` | `src/state/` |
| `CommandLine`、`CommandError`、`CommandResult`、`Interrupt`、`WaitOutcome`、`Platform`、`PlatformKind` | `src/runner/` |
| `Locale`、`ReportEnv`、`Reporter` | `src/report/` |
| `Deps` と、その各インターフェース(`Env`、`Clock`、`Sink`、`Definitions`、`States`、`Converter`、`Process`、`Signals`) | `src/cli/` |

#### `src/definition/`

```ts
// 03 章の型が正。ここでは他のディレクトリが使う名前だけを示す。
export interface Definition {
  readonly formatVersion: number;
  readonly name: string;                 // pipeline の名前
  readonly description: string | null;
  readonly runIdEnv: string | null;
  readonly removeOnComplete: boolean;
  readonly jobs: readonly Job[];         // 記述順
}
export function jobIndex(d: Definition, id: string): number | null;

export interface Job {
  readonly id: string;
  readonly command: Command;
  readonly warningExitCodes: readonly number[];
  readonly successExitCodes: readonly number[] | null; // null なら既定の [0]
  readonly transitions: readonly Transition[];
  readonly retry: RetryPolicy | null;
}
export function effectiveSuccessCodes(j: Job): readonly number[];
export function transitionFor(j: Job, on: 'success' | 'warning'): Transition | null;

export type Command =
  | { readonly kind: 'line'; readonly line: string }
  | { readonly kind: 'argv'; readonly argv: readonly string[] };

export interface RetryPolicy {
  readonly exitCodes: readonly number[];
  readonly max: number;                  // 32 ビット符号なしの範囲(number で足りる)
  readonly intervalSecs: bigint;         // 64 ビット符号なし。bigint
  readonly backoff: Backoff | null;      // multiplier と maxInterval は両方ある、または両方ない
}
export interface Backoff {
  readonly multiplier: number;           // 32 ビット符号なし
  readonly maxIntervalSecs: bigint;      // 64 ビット符号なし。bigint
}

export interface Transition {
  readonly on: 'success' | 'warning';
  readonly next: string;                 // ジョブID。isEnd のときは ""
  readonly isEnd: boolean;               // 遷移先が @end
}

export type DefinitionErrorKind = /* 03 章の全件(文字列リテラルの和集合) */ string;
export class DefinitionError extends Error {
  readonly file: string;
  readonly line: number;                 // 0 なら行なし
  readonly column: number;               // 順序づけにだけ使う
  readonly kind: DefinitionErrorKind;
  readonly reason: string;               // 固定の日本語の文。message も同じ。入力の値や、ライブラリのエラー文を含めない
}

export type LoadResult =
  | { readonly ok: true; readonly definition: Definition }
  | { readonly ok: false; readonly errors: readonly DefinitionError[] };

// load は定義ファイルを読み、検証する。誤りは、見つけたものをまとめて返す(早い段階の誤りは 1 件で打ち切る。03 章)。拒否されない Promise を返す。
export function load(path: string, expectedPipelineName: string | null): Promise<LoadResult>;
// parse は、ファイルに触れず、文字列に対して 03 章の段階 2〜5 を行う(純粋)。
export function parse(text: string, format: 'yaml' | 'json', fileLabel: string, expectedPipelineName: string | null): LoadResult;
// looseRunIdEnv は、run_id_env だけをゆるく読む(どの失敗でも "")。
export function looseRunIdEnv(path: string): Promise<string>;
// readTree は、YAML/JSON を 03 章の規定で木に読む。convert が共有する。
export function readTree(text: string, format: 'yaml' | 'json', fileLabel: string): TreeResult;
```

- 定義の書式は、拡張子で決める(`.yaml`/`.yml`/`.json`、大文字小文字を区別しない。それ以外は `unsupportedFormat`)。[03-definition-format.md](03-definition-format.md)。
- 終了コードの配列(`warningExitCodes` など)は、整数の字句から `bigint` で範囲を検査し(32 ビット符号つきの範囲)、通ったものだけを `number` に直す(範囲外の値を `number` に落とさない。03 章)。

#### `src/runid/`

```ts
export type RunId = string & { readonly __brand: 'RunId' };  // 検査を通った値だけ
export function validateRunId(s: string): RunIdError | null; // 値の検査(文字集合・長さ。04・05 章)
export function resolveRunId(
  flagValue: string | null,
  envName: string | null,
  getenv: (name: string) => string | undefined,
): ResolveResult; // { ok: true; runId: RunId | null } | { ok: false; error: RunIdError }
```

#### `src/flow/`

```ts
// 型と関数の骨格の全体は 05 章が正(JobStatus、SkipReason、JobState、JobOutcome、NextStepResult、
// Decision、ResumePoint、RetryDecision、classify、classifyAndDecide、decideBeforeRun、
// decideRetry、nextWaitSecs、resumePointOnInterrupt、exitCodeFor、signalExitCode)。
// ここには、runner、state、report が使う型だけを示す。

// ジョブの実行結果(07 章 4.4 節)
export interface JobExecution {
  readonly jobId: string;
  readonly command: Command;                    // 表示とステートには使わない(jobId だけ)
  readonly started: boolean;                    // false は、起動の前に合図が立っていたときだけ
  readonly exitCode: number | null;
  readonly terminatedBySignal: number | null;
  readonly startFailure: boolean;
  readonly interrupted: boolean;
  readonly signalNumber: number | null;
}

export interface PipelineState {
  readonly definitionKey: string;
  readonly definitionPath: string;
  readonly pipelineName: string;
  readonly runId: string | null;
  readonly jobOrder: readonly string[];
  readonly jobStates: readonly JobState[];
  readonly resumeJobId: string | null;
  readonly endReached: boolean;
  readonly completed: boolean;
  // 06 章の論理項目の全件
}

// 終了コードの定数。10 章の全件
export const EXIT_OK = 0;
// ...

// リトライの待ち時間(秒)は bigint。64 ビット符号なしの上限で飽和する(05 章)。
export function nextWaitSecs(policy: RetryPolicy, retriesDone: number, lastWait: bigint | null): bigint;
```

#### `src/state/`

```ts
export type Durability = 'durable' | 'fast'; // durable: 同期してから置き換える。fast: 同期を省く(起動前の初期ステート)

export type StateErrorKind = 'unreadable' | 'tooLarge' | 'corrupt' | 'versionMismatch' | 'writeFailed' | 'removeFailed';
export class StateError extends Error {
  readonly kind: StateErrorKind;
  readonly reason: string;  // 06 章の固定の理由文の識別子。message は固定の文
  readonly path: string;
}

// 記録関数(純粋。元を変更せず、新しい値を返す)
export function initialState(def: Definition, definitionPath: string, runId: RunId | null): PipelineState;
export function applyOutcome(s: PipelineState, o: JobOutcome, step: NextStepResult): PipelineState;
export function applySkip(s: PipelineState, jobId: string, reason: SkipReason /* ... */): PipelineState;
export function applyInterrupt(s: PipelineState, rp: ResumePoint): PipelineState;

// 整合性の検査(純粋)
export type Mismatch = /* 06 章の食い違いの種類 */ string;
export function checkConsistency(s: PipelineState, def: Definition, definitionPath: string, runId: RunId | null): Mismatch | null;

// 入出力
export interface StateStoreOptions {
  readonly now: () => Date;            // 一時ファイル名用
  readonly syncDirectory: boolean;     // UNIX 系 true、Windows false(cli が PlatformKind から決める)
}
export type StateReadResult =
  | { readonly ok: true; readonly state: PipelineState | null } // null は「空」
  | { readonly ok: false; readonly error: StateError };
export interface StateStore {
  read(definitionPath: string, runId: RunId | null): Promise<StateReadResult>;     // 完了のものも「空」
  readRaw(definitionPath: string, runId: RunId | null): Promise<StateReadResult>;  // 完了のものも返す(watch)
  write(definitionPath: string, runId: RunId | null, s: PipelineState, d: Durability): Promise<StateError | null>; // null は成功
  reset(definitionPath: string, runId: RunId | null): Promise<StateError | null>;
  resetAll(definitionPath: string): Promise<ResetAllResult>;
}
export function createStateStore(opts: StateStoreOptions): StateStore;
```

- ステートの JSON の表現(ファイル用の型、キーの順、整数の幅の検査、深さの制限)は [06-state-file.md](06-state-file.md)。`PipelineState` を直接 `JSON.stringify` しない。`state` の内側の、ファイル用の型(`index.ts` に出さない)を通し、キーの順を固定して書き出す。
- 入出力は `node:fs/promises` の非同期の API を使う(7 章)。

#### `src/runner/`

骨格は [07-process-and-signals.md](07-process-and-signals.md)。公開するのは次のとおり。

```ts
export interface CommandLine { readonly program: string; readonly args: readonly string[] }
export type CommandErrorKind = 'unclosedQuote' | 'empty';
export class CommandError extends Error { readonly kind: CommandErrorKind } // message は固定の文(入力の値を含めない)
export type CommandResult =
  | { readonly ok: true; readonly line: CommandLine }
  | { readonly ok: false; readonly error: CommandError };

export function splitCommand(s: string): CommandResult;
export function checkCommand(c: Command): CommandResult;

export class Interrupt {
  raise(n: number): void;
  number(): number | null;
  whenRaised(): Promise<number>;
}

export function runJob(job: Job, sig: Interrupt, opts?: RunJobOptions): Promise<JobExecution>; // 拒否されない
export type WaitOutcome =
  | { readonly interrupted: false }
  | { readonly interrupted: true; readonly number: number };
export function waitRetry(secs: bigint, sig: Interrupt): Promise<WaitOutcome>;                  // 拒否されない

export type SignalStart = 'started' | 'alreadyRegistered' | 'failed';
export function startWatcher(sig: Interrupt): { readonly result: SignalStart; stop(): void };
// 本番: プロセスで 1 回だけ。2 回目は 'alreadyRegistered'。stop は、テストでリスナーを外すため

export type PlatformKind = 'posix' | 'windows';
export function detectPlatform(): PlatformKind; // process.platform を読む唯一の関数
```

#### `src/report/`

```ts
export type Locale = 'en' | 'ja';
export interface ReportEnv {
  getenv(name: string): string | undefined;
  stdoutIsTerminal(): boolean;
  platform(): PlatformKind; // 色を使えるかの判定に使う(Windows は色なし)
}
export interface Reporter {
  // 文を作る関数群。戻り値は文字列(改行を含む)。書き込みはしない。
  verboseResult(o: JobOutcome, er: JobExecution): string;
  definitionErrors(errs: readonly DefinitionError[]): string;
  commandError(path: string, jobId: string, e: CommandError): string;
  stateError(e: StateError): string;
  watch(s: PipelineState /* ... */): string;
  usage(): string;
  // ほか、08 章のメッセージの全件
}
// ロケールと色は、最初に必要になったとき 1 回だけ判定する(遅延評価)。
export function createReporter(env: ReportEnv): Reporter;
```

- `report` は、`definition`・`flow`・`state`・`runner` の型を `import type` で読むだけで、それらを変更しない。環境変数と端末の判定は、`ReportEnv` を通して受け取る(`process.env` を直接読まない)。

#### `src/cli/`

3 章と 4 章(`Deps`)。

```ts
// run は、引数を解析し、サブコマンドを実行して、終了コード(0〜255)を返す。process.exit は呼ばない。
// 想定外の内部の失敗は、ここでは捕まえない(src/bin の入口が捕まえる。9 章)。
export function run(args: readonly string[], d: Deps): Promise<number>;
export function systemDeps(): Deps; // 本番の実体の束
```

#### `src/bin/conductor.ts`

```ts
#!/usr/bin/env node
// 骨格(実装ではない): 最初に内部の誤りの捕捉を設定し、run を await し、flush して、終了コードで 1 回だけ終わる。
async function main(): Promise<void>;
```

---

## 3. 依存の向き【規定】

矢印は「import する」。**循環させない**。`tsc --noEmit` が通ることに加えて、循環の検査(候補: `madge --circular`、`eslint-plugin-import` の `no-cycle`。どちらを使うかは実装時に確認。11 章)を品質ゲートに含める。

```text
src/bin ─▶ cli

cli ─▶ report, runner, state, flow, definition, convert, runid, version

report ─▶ state, runner, flow, definition        (型だけ。cli を import しない)

runner ─▶ flow, definition
state  ─▶ flow, definition, runid
flow   ─▶ definition
convert ─▶ definition   (木の読み取りを共有。ほかの src/ のディレクトリには依存しない)
runid  ─▶ (標準モジュールだけ)
definition ─▶ (YAML のライブラリ、標準モジュールだけ)
version ─▶ (なし)

test/support ─▶ cli(型だけ), flow, definition, runner, state, runid, convert
```

図(下へ向かって依存する):

```text
                       src/bin
                          │
                         cli  ─────────┬────────────┐
        ┌──────┬──────┬───┼──────┐     │            │
     report  runner  state  convert version        runid
        │ ╲      │ ╲    │ ╲      │
        │  ╲     │  ╲   │  ╲     └─▶ definition(convert は木の読み取りを使う)
        │   ╲    ▼   ╲  ▼   ╲
        │    ╲──▶ flow ◀──── (state も runner も flow を使う)
        ▼          │
     (state, runner, flow, definition の型を読む)
                   ▼
              definition
```

規則:

1. **`cli` が最上位**(`src/bin` を除く)。他のディレクトリは `cli` を import しない。`report` も `cli` を import しない(環境の判定は `ReportEnv` の形で受け取る)。
2. **`definition` と `flow` が下層**。`definition` は、`src/` のどのディレクトリにも依存しない。`flow` は `definition` だけに依存する。入出力に触れない(`definition.load` の読み取りを除く)。
3. **`state` と `runner` は、`flow` と `definition` の型を使うが、互いに依存しない**。`state` は「ステートを読み書きする」だけで、ジョブを起動しない。`runner` は「ジョブを起動して待つ」だけで、ステートに触れない。両者をつなぐのは `cli`。`state` が OS の違い(ディレクトリの同期の有無)を知る必要があるときは、`cli` が `PlatformKind` から `StateStoreOptions.syncDirectory` を決めて渡す。
4. **`convert` が使う `src/` のディレクトリは `definition` だけ**で、木の読み取り(`definition.readTree`。03 章)を共有するためである。定義の検証(`definition.load` の段階 4・5)は呼ばない。
5. **`report` は、データを受け取って文を作る**。判断をしない。何を表示するかは `cli` が決める。
6. **OS ごとの分岐は、`src/runner/platform.ts` の 1 か所だけ**([07-process-and-signals.md](07-process-and-signals.md) の 8 章)。`process.platform` を読むのは `detectPlatform()` だけ。`state` と `report` と `cli` は、`cli` が `Deps.env.platform()` で得た `PlatformKind` を、引数で受け取って使う。
7. **型の置き場所は、依存の最下層**(2.1 節)。型だけを使うために、上の層や横の層を import しない。上の層の型が要るときは、型を下の層へ移す。
8. **import は、相手のディレクトリの `index.ts` を通す**。ディレクトリの内側のファイルを、外から直接 import しない。ただし、テストは、同じディレクトリの内部のファイルを直接 import してよい。
9. **`test/support/` は `src/cli/` の型を `import type` してよい**。`src/` は `test/` を import しないので、循環しない。`Deps` の各インターフェースのメソッドのシグネチャは、`cli` の型ではなく、下層のディレクトリの型だけで書く(4 章)。偽実装が、`cli` の型を import しなくても、構造的な型付け(メソッドの形が合うこと)で、インターフェースを満たせるようにするため。
10. 強制の方法: ESLint の `no-restricted-imports`(または `import/no-restricted-paths`)に、上の表の許可されない組み合わせを書く(例: `src/definition/**` から `../flow/**` を禁止)。具体的な設定は実装時に確認(11 章)。

---

## 4. `Deps`(差し替え口)とテスト用の偽実装【規定】

### 4.1 方針

`cli` の流れは、環境・時刻・ファイル(定義・ステート・変換)・プロセス・シグナル・標準出力/標準エラーを、**まとめて 1 つの `Deps` オブジェクト**で受け取る。各フィールドは TypeScript の `interface` で定義する。
本番の入口(`systemDeps`)は本番の実体を渡し、テストは偽の実体を渡す。純粋な判断(`flow` の関数、`state` の記録関数、`report` の文の組み立て)は、interface を通さず、`cli` が直接呼ぶ(差し替え不要)。

### 4.2 interface の定義(`src/cli/deps.ts`)

```ts
// 環境(環境変数、標準出力が端末か、OS の種別)。
export interface Env {
  getenv(name: string): string | undefined; // 未設定は undefined
  stdoutIsTerminal(): boolean;
  platform(): PlatformKind;
  cwd(): string;                            // 必要なとき(定義ファイルの相対パスの扱いなど。04・05 章)
}

// 時刻。ステートの一時ファイル名などに使う。
export interface Clock {
  now(): Date;
}

// 標準出力・標準エラーへの書き込み。書き込みの完了まで待つ。失敗は値で返す(拒否しない)。
export type WriteResult = { readonly ok: true } | { readonly ok: false };
export interface Sink {
  writeOut(text: string): Promise<WriteResult>;
  writeErr(text: string): Promise<WriteResult>;
}

// 定義ファイルの読み込み。
export interface Definitions {
  load(path: string, expectedPipelineName: string | null): Promise<LoadResult>;
  looseRunIdEnv(path: string): Promise<string>;
}

// ステートの読み書き(ファイルシステム)。state.StateStore と同じ形。
export interface States {
  read(definitionPath: string, runId: RunId | null): Promise<StateReadResult>;
  readRaw(definitionPath: string, runId: RunId | null): Promise<StateReadResult>;
  write(definitionPath: string, runId: RunId | null, s: PipelineState, d: Durability): Promise<StateError | null>;
  reset(definitionPath: string, runId: RunId | null): Promise<StateError | null>;
  resetAll(definitionPath: string): Promise<ResetAllResult>;
}

// 変換。
export interface Converter {
  convert(inputPath: string, to: 'yaml' | 'json'): Promise<ConvertResult>; // { ok: true; text } | { ok: false; error }
}

// プロセス(ジョブの起動と待ち)。
export interface Process {
  checkCommand(c: Command): CommandResult;
  runJob(job: Job, sig: Interrupt): Promise<JobExecution>;
  wait(secs: bigint, sig: Interrupt): Promise<WaitOutcome>; // リトライの待機
}

// シグナルの監視。
export interface Signals {
  // 受け取った最初のシグナルの番号を sig に立てる監視を始める。
  // 'alreadyRegistered' なら、流れは成功と同様に続ける。'failed' は、登録の失敗(終了コード 70)。
  start(sig: Interrupt): SignalStart;
}

export interface Deps {
  readonly env: Env;
  readonly clock: Clock;
  readonly out: Sink;
  readonly defs: Definitions;
  readonly states: States;
  readonly converter: Converter;
  readonly proc: Process;     // 変数名 process は、グローバルの process と紛らわしいので使わない
  readonly signals: Signals;
}
```

- 引数 `args` は、`run` の引数で受け取る(`Env` に入れない)。
- **ファイルシステムの差し替え**は、`Definitions`・`States`・`Converter` の 3 つの interface が担う。本番の実装は、実ファイルを読み書きする。偽の実装は、メモリに持つ(ファイルなし)。ステートの書き込みの原子的な置換の失敗や、ディスクの異常の検査は、`state` の内側のテスト(一時ディレクトリの実ファイル)で行う。
- `Sink` は、書き込みの失敗を値(`{ ok: false }`)で返す(拒否にしない)。標準出力と標準エラーを混ぜない。**書き込みは `cli` の 1 か所の補助関数を通す**(順序を保つ。書き込みの完了まで `await` する。[07-process-and-signals.md](07-process-and-signals.md) の 9 章)。
- `Process.wait` は `waitRetry` を包む。偽の実装は、待ち時間を記録するだけで実際には待たない。
- `Signals.start` は同期の関数。`Process.runJob` と `Process.wait` と `Definitions`、`States`、`Converter`、`Sink` のメソッドは、すべて拒否されない `Promise` を返す(失敗は値で返す。9 章)。

### 4.3 本番の実装(`src/cli/system.ts`)【目安】

| 名前 | interface | 中身 |
|------|-----------|------|
| `systemEnv` | `Env` | `process.env[name]`、`process.stdout.isTTY === true`、`detectPlatform()`、`process.cwd()`。端末の判定の挙動(UNIX 系と Windows の違い、`isTTY` が `undefined` のとき)は、実装時に 3 OS で確かめる(未確認) |
| `systemClock` | `Clock` | `new Date()` |
| `stdSink` | `Sink` | `process.stdout.write(text, callback)` / `process.stderr.write(...)` を `Promise` で包み、コールバックの完了まで待つ。`process.stdout` と `process.stderr` に `'error'` のリスナーを付ける([07-process-and-signals.md](07-process-and-signals.md) の 9.3 節) |
| `systemDefs` | `Definitions` | `definition.load`、`definition.looseRunIdEnv` |
| `systemStates` | `States` | `state.createStateStore({ now: () => new Date(), syncDirectory: detectPlatform() !== 'windows' })` を持ち、メソッドを委譲 |
| `systemConverter` | `Converter` | `convert.convertFile` |
| `systemProcess` | `Process` | `runner.checkCommand`、`runner.runJob`、`runner.waitRetry` |
| `systemSignals` | `Signals` | `runner.startWatcher(sig).result`(`stop` は本番では呼ばない) |

### 4.4 偽の実装(`test/support/`)【規定】

偽の実装は、次の振る舞いを持つ(名前は実装者が決める)。すべて `src/cli/` の interface を構造的に満たす(`satisfies Env` などで、コンパイル時に確かめる)。

| 偽の実装 | 振る舞い | 主な検査 |
|-----------|----------|----------|
| `FakeEnv` | 環境変数を表(`Map`)で与える。`getenv` を呼んだ回数と名前を記録する。`stdoutIsTerminal`、`platform`、`cwd` の値を設定する | ロケール(`LC_ALL`・`LC_MESSAGES`・`LANG`)・`NO_COLOR`・run ID の環境変数。ロケールの判定を**遅延**して、最初に必要になったとき 1 回だけであること(成功・スキップで `-v` なしなら、読まない) |
| `FakeClock` | 固定の時刻を返す。進める操作を持つ | 一時ファイル名の時刻 |
| `BufSink` | `writeOut`・`writeErr` を、別々の文字列に溜める。呼び出しの順序の記録も持つ。「n 回目の書き込みで失敗する」設定を持つ | 出力の中身、`cannot write to standard output`(`watch` の 65、`convert` の 67) |
| `FakeDefs` | パスごとに、決めた `Definition` や誤りの並びを返す。または、実ファイル(一時ディレクトリ)を `definition.load` で読む版 | 定義の誤り(64)の経路 |
| `MemStates` | ステートをメモリに持つ(ファイルなし)。読み込み・書き込み・削除の呼び出しを記録する。「読み込みで `StateError` を返す」「書き込みで失敗する」設定を持つ | 大きなジョブ数(1,000 個)の流れの測定、ステートの異常系(65) |
| `FakeConverter` | 入力パスごとに、決めた文字列かエラーを返す | `convert` の 67 |
| `ScriptProcess` | ジョブIDごとに、台本の `JobExecution`(終了コード・シグナル終了・起動失敗・中断)を順に返す。呼び出しの記録(何が、何回、どの順で実行されたか)を持つ。`checkCommand` は、台本の誤り、または `runner.checkCommand` を呼ぶ。`wait` は、待ち時間(秒の `bigint`)を記録するだけ。「`wait` が中断を返す」設定を持つ | 分類・遷移・再開・リトライ(待ち時間の列 1・2・4・…)・中断の経路。実際の待ちをしない |
| `FakeSignals` | `start` の戻り値を設定する(`'started'`、`'alreadyRegistered'`、`'failed'`)。「開始時に、すでに `sig.raise(n)` を呼ぶ」設定を持つ | 登録の失敗(70)、登録済み、開始時にすでに合図が立っている |
| 補助 | 一時ディレクトリ(`node:fs/promises` の `mkdtemp`)に、定義ファイルを書く。実行して、ステートファイルの内容を読み戻す。終了後に削除する | 実ファイルを使うテスト(受け入れテストの下位) |

- 実プロセスを使うテスト(`runner` の内側と受け入れテスト)は、`test/jobs/` のテストジョブを使う([11-quality-and-build.md](11-quality-and-build.md))。
- JavaScript はイベントループが 1 つなので、偽の実装にロックは要らない。ただし、シグナルのハンドラから呼ばれうる偽(`Interrupt` を使うもの)は、`await` の途中で状態が変わりうる前提で作る。

---

## 5. 実行のシーケンス

### 5.1 入口

```text
src/bin/conductor.ts: main():
    内部の誤りの捕捉を設定する                           # 最初に。uncaughtException / unhandledRejection / try-catch(07 の 10 章)
    code = await run(process.argv.slice(2), systemDeps())
    標準出力・標準エラーの未完了の書き込みを待つ
    process.exit(code)                                    # 終了は、ここで 1 回だけ

cli.run(args, d):
    parsed = 引数を解析する(04 章)
      誤り           → 使い方の誤りを標準エラーに表示、return 68
      help           → 使い方全文を標準出力へ、return 0
      version        → `conductor 0.8.0` を標準出力へ、return 0
      コマンド       → run / pipeline / watch / reset / validate / convert のどれかへ
```

### 5.2 `pipeline` の実行

`prepare`(前処理)の順序と、そこでの失敗の終了コードは、[05-execution-flow.md](05-execution-flow.md) と [10-errors-and-exit-codes.md](10-errors-and-exit-codes.md) に固定されている。すべての `await` は直列で、並行に走らせない(7.1 節)。

```text
cli: pipeline
 ├─ prepare
 │    ├─ sig = new Interrupt();  r = d.signals.start(sig)           → 'alreadyRegistered' は続行 / 'failed' は 70(定義の検査より前)
 │    ├─ await d.defs.load(...)                                     → 失敗 64(パイプライン名の照合を含む)
 │    ├─ d.proc.checkCommand(全ジョブ)                              → 失敗 64(誤りのあるジョブすべてを 1 ジョブ 1 行で報告)
 │    ├─ run ID の決定(--run-id | 環境変数。runid)                 → 失敗 68
 │    ├─ await d.states.read(...)  (完了のものは「空」)              → 失敗 65
 │    └─ state.checkConsistency(...)                                → 食い違い 65
 └─ for job in 定義のジョブ(記述順):
      ├─ flow.decideBeforeRun(...) → run / skip / reject
      │    ├─ reject → report: 表示(await)、return 66
      │    ├─ skip   → state.applySkip → 変化があれば await d.states.write(durable)、続行(完了なら finish)
      │    └─ run:
      │         ├─ ステートが空なら state.initialState → await d.states.write(fast)
      │         ├─ リトライ込みの実行
      │         │    loop { r = await d.proc.runJob(job, sig)
      │         │           dec = flow.decideRetry(job, r, retriesDone, lastWait)
      │         │           リトライなら report: 知らせ(await d.out.writeErr。常に標準エラー) → w = await d.proc.wait(secs, sig)
      │         │                    w.interrupted なら「起動しなかった中断」の JobExecution(started=false, interrupted=true, signalNumber)を作って抜ける }
      │         ├─ flow.classifyAndDecide(..., RunMode=Pipeline)
      │         ├─ report: 表示(-v のときの結果行・遷移行。常時のエラー行)(await)
      │         ├─ state.applyOutcome → await d.states.write(durable)
      │         ├─ エラーまたは分類なし → return flow.exitCodeFor(...)
      │         ├─ 完了 → finish(--remove-on-complete なら await d.states.reset)、return 0
      │         └─ sig に合図が立っている(sig.number() !== null) → flow.resumePointOnInterrupt → state.applyInterrupt → await d.states.write(durable)、return 128 + 番号
 └─ ループを抜けても完了していない → "cannot determine whether the pipeline is complete"、return 65
```

- `run` は、`prepare`(ジョブIDの存在の検査を含む)のあと、指定した 1 ジョブについて同じ「判断 → 実行 → 分類 → 記録」を行う。分類と次の動きは `RunMode = Individual` で求める。
- `watch`・`reset`・`validate`・`convert` は、シグナルを準備しない。それぞれ、ステートの読み込み(`readRaw`)・ステートの削除(`reset`、`resetAll`)・定義の検査(`defs.load` と `proc.checkCommand`)・変換(`converter.convert`)を呼ぶ。
- `pipeline` は、次のジョブを「次の動き」の戻り値で決めない。記述順に全ジョブを見て、起動前の判断で決める(スキップ・再開は、ステートが運ぶ)。
- 合図の確認は、`cli` が `sig.number()` で行う。`runner.runJob` と `runner.waitRetry` が、待機中の合図を扱う([07-process-and-signals.md](07-process-and-signals.md))。

### 5.3 責務の流れ(データ)

```text
定義ファイル ──definition.load──▶ Definition ─┐
ステートファイル ──state.read──▶ PipelineState ┤
                                              ▼
                       flow.decideBeforeRun ──▶ run / skip / reject
runner.runJob ──▶ JobExecution ──flow.classifyAndDecide──▶ (JobOutcome, NextStepResult)
                                         │
        state.apply*(結果・スキップ・中断) ──▶ 新しい PipelineState ──state.write──▶ ステートファイル
JobOutcome の報告する終了コード ──flow.exitCodeFor──▶ conductor の終了コード
```

### 5.4 中断の伝わり方

- 中断の合図(`runner.Interrupt`)は、「立った/立っていない」と「最初のシグナル番号」を持つ。シグナルのハンドラ(`process.on`)が `raise` で立て、待ち(`runJob`、`waitRetry`)と `cli` の流れが読む。すべて同じスレッドで動く。
- ハンドラは同期で、`raise` を呼ぶだけ。`raise` が `Promise`(`whenRaised`)を解決し、待つ側が次のマイクロタスクで動く。詳細は [07-process-and-signals.md](07-process-and-signals.md) の 5〜7 章。

---

## 6. ファイル単位の責務表【目安】

実装者は、ファイルの分け方を変えてよい。ただし、ディレクトリの責務と依存の向き(2・3 章)と、OS 分岐の置き場所(`platform.ts` だけ)は守る。

| パス | 責務 |
|------|------|
| `src/bin/conductor.ts` | 入口(`main`。内部の誤りの捕捉、`run` の `await`、flush、`process.exit` を 1 回) |
| `src/version.ts` | 版の定数 |
| `src/definition/index.ts` | バレル(公開する名前) |
| `src/definition/types.ts` | 定義の型(`Definition` ほか) |
| `src/definition/errors.ts` | 誤りの種類(`DefinitionErrorKind`)と `DefinitionError` |
| `src/definition/load.ts` | `load`(場所の決定、読み取り、書式の判定、段階の組み立て) |
| `src/definition/intake.ts` | ファイルの読み取り、大きさの上限、拡張子の判定 |
| `src/definition/tree.ts`、`read-yaml.ts`、`read-json.ts` | 木の型、YAML/JSON の読み取り(位置つき。展開と上限を含む。`convert` と共有。03 章) |
| `src/definition/depth.ts` | 入れ子の深さの検査 |
| `src/definition/walk.ts` | 構造・型・遷移・終了コード・名前の検査 |
| `src/definition/loose.ts` | `run_id_env` のゆるい読み取り |
| `src/convert/index.ts`、`convert.ts` | YAML ⇔ JSON の変換の入口(`convertFile`、`convertText`) |
| `src/convert/write-yaml.ts`、`write-json.ts`、`error.ts` | YAML の書き出し、JSON の書き出し、誤り(読み取りは `definition.readTree`) |
| `src/runid/index.ts`、`runid.ts` | run ID の型、検査、決定 |
| `src/flow/types.ts` | 実行時の値の型 |
| `src/flow/classify.ts` | 分類 |
| `src/flow/decide.ts` | 起動前の判断 |
| `src/flow/next-step.ts` | 次の動き |
| `src/flow/resume.ts` | 中断時の再開位置 |
| `src/flow/retry.ts` | リトライの判断と待ち時間(`bigint` の飽和の計算) |
| `src/flow/exit.ts` | 終了コードの定数と範囲合わせ |
| `src/state/types.ts`、`location.ts` | ファイル用の型、場所と名前 |
| `src/state/record.ts` | 記録関数(純粋) |
| `src/state/consistency.ts` | 整合性の検査(純粋) |
| `src/state/reader.ts` | 読み込み(14 段階の検査、異常系) |
| `src/state/writer.ts` | 書き込み(一時ファイル + 同期 + 名前の付け替え)、`reset` |
| `src/state/reset-all.ts` | `resetAll` |
| `src/state/sync-dir.ts` | ディレクトリの同期(`syncDirectory` の真偽で動作を決める。`process.platform` は読まない) |
| `src/runner/split.ts` | 分割と検査 |
| `src/runner/run-job.ts` | ジョブの起動、待ち、結果の取り出し、転送の判断 |
| `src/runner/wait-retry.ts` | リトライの待機 |
| `src/runner/interrupt.ts` | 中断の合図 |
| `src/runner/watcher.ts` | シグナル監視(登録は 1 回、ハンドラは `raise` だけ) |
| `src/runner/platform.ts` | OS 別の部分(`detectPlatform`、`platformFor`。`process.platform` を読む唯一の場所) |
| `src/report/locale.ts` | ロケールと色の判定(遅延) |
| `src/report/catalog.ts` | メッセージカタログ(英語・日本語) |
| `src/report/format.ts` | 行の組み立て(`-v`、誤り) |
| `src/report/escape.ts` | エスケープと切り詰め |
| `src/report/watch.ts` | `watch` の表 |
| `src/report/usage.ts` | 使い方 |
| `src/cli/args.ts` | 引数の字句の解析と検査 |
| `src/cli/deps.ts`、`system.ts` | `Deps` の interface と本番の実装 |
| `src/cli/cli.ts` | `run`(入口の分岐、終了コード) |
| `src/cli/prepare.ts` | 前処理(`run`・`pipeline` に共通) |
| `src/cli/run-cmd.ts`、`pipeline-cmd.ts` | `run`・`pipeline` の流れ |
| `src/cli/exec-job.ts` | リトライ込みの 1 ジョブの実行、記録、表示 |
| `src/cli/aux-cmd.ts` | `watch`・`reset`・`validate`・`convert` |
| `test/support/*.ts` | 4.4 節の偽の実装と補助 |
| `test/acceptance/*.test.ts` | 実際の `dist/bin/conductor.js` を起動する受け入れテスト |
| `test/jobs/*` | テストジョブ |

単体テストは、`test/` の下に、`src/` と同じ構成で置く(例: `test/runner/split.test.ts`)。受け入れテストの置き場所と実行の方法は [11-quality-and-build.md](11-quality-and-build.md)。

---

## 7. 非同期の方針【規定】

### 7.1 基本

| 項目 | 方針 |
|------|------|
| 基本 | `async`/`await` を使う。コールバックの API(`process.stdout.write` のコールバックなど)は、`Promise` に包む。`.then` の連鎖を、ライブラリ的な小さな補助の中以外で書かない |
| スレッド | ユーザーのコードは、イベントループの 1 つのスレッドで動く。データ競合は無い。`worker_threads`、`cluster`、`child_process` の `fork` を使わない |
| 並行の実行 | 流れの制御は、`Promise.all`・`Promise.race` などで仕事を並行に走らせない。ジョブ、出力、ステートの書き込みは、すべて直列に `await` する。例外: `waitRetry` の内側での、タイマーと合図の `Promise.race`(待ちの打ち切りのため。07 章) |
| 投げっぱなしの禁止 | `await` も `.then` もない `Promise` を作らない。型情報つきの ESLint の規則 `@typescript-eslint/no-floating-promises`、`no-misused-promises` をエラーにする |
| 同期のハンドラ | `process.on` のシグナルのハンドラと、子のイベントのハンドラは、同期の関数。仕事は、値を立てる、`Promise` を解決するだけ |
| 共有する可変の値 | `runner.Interrupt` と、シグナル登録の印(`watcher.ts` のモジュール内の変数)だけ |
| 待ち方 | `setTimeout` と `Promise`(`waitRetry`)、子の `close` イベントの `Promise`(`runJob`)。ポーリングをしない |
| キャンセル | `AbortController` を使わない。中断は `Interrupt` で伝える(`spawn` の `signal` オプションは、子を強制終了するため使わない。07 章 4.2 節) |
| イベントループを塞がない | 同期の子プロセスの API、`Atomics.wait`、ビジーループを使わない。07 章 9.1 節 |
| 出力 | 標準出力・標準エラーへの書き込みは、`Sink` を通し、完了まで `await` する。終了の前に flush を待つ(07 章 9.3 節) |
| 排他制御 | 同じ定義への 2 つの `conductor` の同時の実行を、排他しない(両方が動き、ステートは、どちらも壊れた形では読めない。原子的な置換による) |

### 7.2 イベント源の一覧(誰が所有し、どう終わるか)【規定】

| イベント源 | 起こす者(所有者) | 数 | 読む | 書く | 終わる条件 |
|------------|-------------------|-----|------|------|------------|
| シグナルのリスナー(`process.on`) | `startWatcher`(`run`・`pipeline` のみ) | プロセスで 0 または 1 組(最大 4 つのリスナー) | OS のシグナル | `Interrupt.raise` だけ | プロセスの終了まで動き続ける(本番で `stop` は呼ばない)。テストでは `stop` で外す |
| 子プロセスのイベント(`spawn`・`error`・`exit`・`close`) | `runJob` | 実行中のジョブごとに 1(同時に最大 1) | 子の終了 | `runJob` の内側の状態と、`Promise` の解決だけ | `close`(または起動失敗の `error`)で解決し、`once` のハンドラが残らない |
| タイマー(`setTimeout`) | `waitRetry` | 同時に最大 1 | 時間の経過 | `Promise` の解決だけ | 経過、または合図で打ち切って `clearTimeout` する。残さない |

- 出力(標準出力・標準エラー)とステートへの書き込みは、流れの制御(`cli`)の 1 本の `await` の連なりだけが行う。リスナーとハンドラは、それらに触れない。
- リソースのリークがないこと: テストの終わりに、リスナー(`process.listenerCount('SIGTERM')` など)、タイマー、子プロセスが残っていない(`runJob` が戻った時点で、子のイベントのハンドラは外れている)。

---

## 8. `package.json` と `tsconfig.json` の規定

### 8.1 `package.json`

次の項目を満たす(版の数字は実装時に確認。`<...>` は実装時に確認する値)。

```json
{
  "name": "conductor",
  "version": "0.8.0",
  "description": "<1 行の説明>",
  "type": "module",
  "bin": { "conductor": "dist/bin/conductor.js" },
  "exports": { "./package.json": "./package.json" },
  "engines": { "node": ">=<最小の LTS の版>" },
  "files": ["dist"],
  "scripts": {
    "clean": "node --eval \"require('node:fs').rmSync('dist', { recursive: true, force: true })\"",
    "build": "npm run clean && tsc -p tsconfig.build.json",
    "typecheck": "tsc --noEmit",
    "lint": "eslint .",
    "format": "prettier --write .",
    "format:check": "prettier --check .",
    "test": "vitest run",
    "test:coverage": "vitest run --coverage",
    "prepack": "npm run build"
  },
  "dependencies": { "<YAML のライブラリ>": "<版>" },
  "devDependencies": {
    "typescript": "<版>", "eslint": "<版>", "typescript-eslint": "<版>", "prettier": "<版>",
    "vitest": "<版>", "@vitest/coverage-v8": "<版>", "@types/node": "<版>"
  }
}
```

| 項目 | 規定 |
|------|------|
| `type` | `"module"`【規定】(ESM) |
| `bin` | `conductor` → `dist/bin/conductor.js`。そのファイルの先頭は `#!/usr/bin/env node`。`tsc` は、ソースの先頭のシバンを出力にそのまま残す(実装時に確認)。実行権限は、`npm pack`/`npm install` が `bin` のファイルに付ける(Windows では、`npm` がラッパーを作る。実装時に確認) |
| `exports` | ライブラリとして公開しないので、`package.json` だけを出す。`main` と `types` を書かない。`exports` を書くと、ここに無いパスの import は、外部から拒否される |
| `engines` | `node` に最小版(最新の LTS 系列の 1 つ前の LTS 系列の最小の版)を書く。CI の Node の版(最小版と最新版)と一致することを、検査する(11 章) |
| `files` | `["dist"]`。`src/`、`test/`、設定ファイルを含めない。`package.json` と、ライセンスなど `npm` が自動で含めるファイルは、書かなくてよい。含まれるファイルを、`npm pack --dry-run` で確かめる |
| `scripts` | 上の名前を持つ(中身は実装者が決めてよい)。品質ゲートの項目([11-quality-and-build.md](11-quality-and-build.md))を、これらのスクリプトで実行できること |
| `dependencies` | YAML のライブラリ 1 つだけ。それ以外を足さない【規定】 |
| `devDependencies` | 1 章の開発時の依存。ほかは、実装時に追加してよいが、実行時の依存を増やさない。`package-lock.json` をコミットする。CI は `npm ci` を使う |
| `version` | `src/version.ts` の `VERSION` と一致する(テストで検査する)【規定】 |
| `private` | 付けない(npm への公開は人が手で行う。ワークフローは公開しない。11 章)。誤公開を避けたいときは、`"private": true` にして、公開のときに外してよい |

### 8.2 `tsconfig.json`

型検査用の `tsconfig.json`(`src/` と `test/` と設定ファイルを含み、出力しない)と、ビルド用の `tsconfig.build.json`(`src/` だけを `dist/` に出力する)の 2 つに分ける。

```jsonc
// tsconfig.json
{
  "compilerOptions": {
    "target": "ES2023",
    "lib": ["ES2023"],
    "module": "NodeNext",
    "moduleResolution": "NodeNext",
    "types": ["node"],
    "strict": true,
    "noUncheckedIndexedAccess": true,
    "exactOptionalPropertyTypes": true,
    "noImplicitOverride": true,
    "noImplicitReturns": true,
    "noFallthroughCasesInSwitch": true,
    "noUnusedLocals": true,
    "noUnusedParameters": true,
    "verbatimModuleSyntax": true,
    "isolatedModules": true,
    "forceConsistentCasingInFileNames": true,
    "skipLibCheck": true,
    "noEmit": true
  },
  "include": ["src", "test", "*.config.ts"]
}
```

```jsonc
// tsconfig.build.json
{
  "extends": "./tsconfig.json",
  "compilerOptions": {
    "noEmit": false,
    "rootDir": "src",
    "outDir": "dist",
    "sourceMap": false,
    "declaration": false,
    "removeComments": false
  },
  "include": ["src"]
}
```

| 項目 | 規定 |
|------|------|
| `strict` | `true`【規定】。`noUncheckedIndexedAccess` と `exactOptionalPropertyTypes` も `true`(配列と辞書の添字の結果が `undefined` を含むため、範囲外の添字による例外を、型で防ぐ) |
| `module`、`moduleResolution` | `NodeNext`【規定】。相対 import に `.js` を付ける |
| `target`、`lib` | `ES2023`(サポートする Node が実装するもの。`bigint`、`Array.prototype.at`、`structuredClone` などを使える)。最小の Node の版が対応しない構文を使わないこと。実装時に確認 |
| `types` | `["node"]`。ブラウザの型(`dom`)を入れない |
| `verbatimModuleSyntax` | `true`。型だけの import は `import type` と書く(実行時の import を残す・消すが明確になる) |
| `rootDir`、`outDir` | `src` と `dist`(`dist/bin/conductor.js` が `bin` の指す先になる) |
| `sourceMap`、`declaration` | `false`(配布物を小さく。利用者にスタックを見せない) |
| `any` | 使わない。ESLint の `@typescript-eslint/no-explicit-any` をエラーにする。`unknown` を使い、絞り込む |
| 非 `null` 表明(`!`)、`as` による型の偽り | 製品のコードで使わない(外部の入力に対して例外を起こさない規定。07 章 10 章)。ESLint の `no-non-null-assertion` をエラーにする |
| 出力の確認 | `npm run build` のあと、`dist/` を再生成して差分が出ないこと(任意。11 章) |

---

## 9. エラー処理の方針【規定】

| 項目 | 方針 |
|------|------|
| 基本 | 失敗は、戻り値(判別可能な和集合の値。`{ ok: true, ... } \| { ok: false, error }`、または `T \| null`)で返す。外部の入力(引数・環境・ファイル・OS の結果)に対して、例外を投げない。`async` の関数は、**拒否されない `Promise`** を返す(失敗は値で返す) |
| 例外を許す場所 | 実装の不具合(論理の矛盾。起こるはずのない状態)だけ。入口(`src/bin`)の捕捉が、固定の 1 行を標準エラーに出して終了コード 70 にする(07 章 10 章)。`cli` の流れの内側で `catch` して握りつぶさない。Node の API が例外を投げる場合は、その場で `try/catch` して、値にする(例: `spawn` の引数の誤り、`JSON` 構文解析の例外) |
| 終了 | `process.exit` は `src/bin/conductor.ts` だけ(と、内部の誤りのハンドラ)。`cli.run` は、終了コードを返す。`process.exitCode` への代入、`throw` による終了を使わない |
| 種類 | ディレクトリごとに、**種類(文字列リテラルの和集合)を持つエラー型**を公開する: `DefinitionError`、`CommandError`、`StateError`、変換の誤り(`ConvertError`)。いずれも `Error` を継承し、`kind` と、固定の文(`message`)を持つ。呼び出し側は、`kind` で `switch` し、網羅性を型で検査する(`never` への代入)。文字列を比べて分岐しない |
| 表示 | 利用者への表示の文は、`report` が、種類と引数(ジョブID、パスなど)から組み立てる(メッセージカタログ)。**`err.message` の文字列や、ライブラリ(YAML、Node の `Error`)のエラー文、`errno` の名前を、そのまま表示しない**(入力の値や、内部の情報が出るため。[10-errors-and-exit-codes.md](10-errors-and-exit-codes.md))。スタックトレースを出さない |
| 原因 | `new Error(message, { cause })` で原因を包んでよい。ただし、表示に使わない。デバッグ用に残してもよいが、利用者への出力に出さない |
| 複数の誤り | 定義の検査・`command` の検査は、誤りを**すべて集めて**返す(配列)。1 件で止めない |
| 値で返す失敗 | ジョブの起動の失敗(`JobExecution.startFailure`)、出力の書き込みの失敗(`Sink` の `{ ok: false }`)は、値として扱い、終了コードに結びつける |
| 終了コードの決定 | 「失敗の種類 → 終了コード」の対応は `cli` の 1 か所で行う([10-errors-and-exit-codes.md](10-errors-and-exit-codes.md)) |
| 想定外 | 入口で捕まえ、固定の英語 1 行 `conductor: error: internal error` と終了コード 70([07-process-and-signals.md](07-process-and-signals.md) の 10 章) |
| 静的検査 | ESLint(typescript-eslint の推奨と型情報つきの規則)で、`no-floating-promises`、`no-misused-promises`、`no-explicit-any`、`no-non-null-assertion`、`switch-exhaustiveness-check` をエラーにする(11 章)。意図して結果を無視する場合(シグナルの送信の失敗など)は、`void` と、理由の注釈を付ける |

---

## 10. 設計判断のチェックリスト(確認)と未確認の事項

### 10.1 設計判断

| # | 判断 | この版の決定 |
|---|------|--------------|
| 1 | ディレクトリ構成 | 2 章のとおり固定 |
| 2 | 実行時の値の型の置き場所 | `src/flow/`(`state` と `runner` は、`flow` に依存する。`flow` は `state` と `runner` に依存しない) |
| 3 | ステートの書式 | JSON(`JSON.stringify` は直接使わず、ファイル用の型を通して固定の順序で書く)。[06-state-file.md](06-state-file.md) |
| 4 | 差し替え口 | `Deps` オブジェクト(interface のフィールド)。4 章 |
| 5 | シグナル | `process.on`。リスナーは 1 組。合図は `runner.Interrupt` |
| 6 | 待機 | `close` イベントの `Promise` と `setTimeout`。ポーリングなし |
| 7 | OS 別 | `src/runner/platform.ts` の 1 か所。`PlatformKind` を、必要な部品に引数で渡す |
| 8 | エラー | 種類を持つエラー型、`kind` の `switch`、`report` が文を作る |
| 9 | 終了 | `process.exit` は入口の 1 か所 |
| 10 | 想定外の失敗 | `uncaughtException`・`unhandledRejection`・入口の `try/catch` で 70 |
| 11 | 整数 | 64 ビット符号なしの飽和演算は `bigint`。終了コードは `number`。範囲外の検査は字句から `bigint` |
| 12 | ファイル入出力 | `node:fs/promises`(非同期) |
| 13 | import | 相手のディレクトリの `index.ts` を通す。相対 import に `.js` を付ける |

### 10.2 未確認の事項

- サポートする Node の版(最新の LTS 系列と、その 1 つ前の LTS 系列の具体的な番号)。実装時に確認する。
- 循環の検査と import の制限の道具(`madge`、`eslint-plugin-import`、`no-restricted-imports` の設定)の選定と具体的な設定。実装時に確認する。
- `tsc` が、ソースの先頭の `#!/usr/bin/env node` を出力に残すこと、`npm` が `bin` に実行権限を付けること、Windows での `bin` のラッパーの動作。実装時に 3 OS で確かめる。
- 端末の判定(`process.stdout.isTTY`)が、UNIX 系と Windows の両方で、利用者が期待する結果になるか。実装時に 3 OS で確かめる。外部ライブラリ(端末の判定用)は、この仕様では採らない前提。
- `exports` に `package.json` だけを出したときに、`npm pack`、`npm install -g`、`npx` の経路で `bin` が動くか。実装時に確かめる。
- `exactOptionalPropertyTypes` と `noUncheckedIndexedAccess` を併用して、実装が不自然に複雑にならないか(複雑なら緩めてよい。`strict` は緩めない)。
- `definition.load` の引数をパスにするか、ファイルシステムの抽象にするか(裁量とした。本番はファイルシステムを読み、テストは一時ディレクトリで動かす)。
- `test/support/` の偽実装が、`src/cli/deps.ts` の interface を満たすことを、`satisfies` で確かめる形が、`tsc --noEmit` で問題なく書けること。実装時に確かめる。
- Node の標準モジュール(`node:child_process`、`node:fs/promises`、`process.stdout`)の挙動の記述は、07 章に【実測】の印を付けたものを除き、公式のドキュメントの知識によるもので、コードを実行して確かめていない([07-process-and-signals.md](07-process-and-signals.md) の 12 章)。
