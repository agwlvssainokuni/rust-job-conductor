# 02 アーキテクチャ(パッケージ・依存方向・型の骨格・`Deps`・実行のシーケンス)

対象: `conductor` v0.8.0 と同じ挙動を Python で実装するための、ソースの構成。用語は [00-README.md](00-README.md) の用語集に従う。

この章は、パッケージとモジュールの責務、依存の向き(循環させない)、各パッケージが公開する型と関数の骨格(シグネチャ)、差し替え口(`Deps`)、実行のシーケンス、同期モデル、エラー処理、`pyproject.toml` を定める。骨格は、型の形を示すためのもので、実装コードではない。関数の本体は、各章(03〜10)の規定から書く。

## 規定の強さと確認の表記

| 表記 | 意味 |
|------|------|
| 【規定】 | どの実装も満たす。テストや点検で確かめる |
| 【目安】 | 推奨。異なる分け方にしてよい。ただし、パッケージの責務と依存の向き(2・3 章)は守る |
| 実装時に確認 | 版や道具の選定など、実装の時点で決める値。この文書は決めない |
| 未確認 | 実測も、公式の文書による裏づけも得ていない |

---

## 1. 設計の前提

| 項目 | 内容 |
|------|------|
| 言語・実行環境 | CPython。サポートする版は、実装時点で、公式にセキュリティ修正が続いている版のうち、最小と最新(具体的な版は実装時に確認。この文書の作成時点では、最小が 3.11、最新が 3.15 と思われるが、未確認)。`pyproject.toml` の `requires-python` に最小版を書く(8 章)。この文書の規定は、3.11 以降で使える機能(`typing.assert_never`、`enum.StrEnum`、`match` 文、`dataclass(slots=True)`、`X \| Y` の型の書き方)に依存してよい |
| 型検査 | 型ヒントは、すべての関数・メソッド・属性に付ける。`mypy --strict` を通す(8 章)。`Any` を使わない(外部から来る値は `object` として受け、絞り込む)。`# type: ignore` は、理由の注釈とエラーコードを付けた場合だけ(`ignore-without-code` を有効にする) |
| 注釈の評価 | すべてのモジュールの先頭に `from __future__ import annotations` を書く【目安】(前方参照と、型だけの import の循環を避ける。版による差を減らす) |
| パッケージ名 | 配布名・import 名とも `conductor`(置き換えてよい。置き換えたときは、`pyproject.toml` の `name` と `[project.scripts]` のキー、`src/` 直下のディレクトリ名、import 文の接頭辞を、すべて同じ名前に変える)。`src/` レイアウト(`src/conductor/`) |
| 構成 | 1 つの配布物に、コンソールスクリプト 1 つ(`conductor`)と、`src/conductor/` 配下のパッケージ 7 つ(`definition`、`convert`、`flow`、`state`、`runner`、`report`、`cli`)、モジュール 3 つ(`__main__.py`、`runid.py`、`version.py`)(2 章)。テストの補助(`tests/support/`、`tests/jobs/`)は配布物(wheel)に含めない |
| 起動の方法 | コンソールスクリプト `conductor`(`[project.scripts]` の `conductor = "conductor.__main__:main"`)と、`python -m conductor`。どちらも `main() -> int` を呼び、`sys.exit` に渡す(2 章の `__main__.py`) |
| データ型 | `dataclasses`(`frozen=True`、`slots=True`)、`enum`(文字列の値を持つもの)、`typing.Protocol`、`Literal`。判別共用体は、`Literal` の属性(`ok`、`kind` など)を持つ `dataclass` の `\|` の和で書き、`match` 文と `assert_never` で網羅を検査する |
| 実行時の依存 | YAML の読み取りだけ(1 つのライブラリ。選定は [03-definition-format.md](03-definition-format.md) と [09-converter.md](09-converter.md))。JSON は自前の字句・構文解析で読む。引数の解析は自前(`argparse` は使わない。理由は [04-cli.md](04-cli.md))。ほかは標準ライブラリだけ(`subprocess`、`os`、`pathlib`、`signal`、`sys`、`json`(書き出しだけ)、`dataclasses`、`enum`、`typing`、`tempfile`、`time`、`datetime`、`errno` など) |
| 開発時の依存 | `pytest`、`pytest-cov`、`mypy`、`ruff`、`build`、`pip-audit`(選定と版は実装時に確認。[11-quality-and-build.md](11-quality-and-build.md)) |
| 公開の範囲 | ライブラリとして公開しない。import できる名前は、各パッケージの `__init__.py` の `__all__` に出すものだけ(内部の取り決め)。`py.typed` を置かない |
| 配布物 | sdist と wheel。wheel には `conductor` パッケージだけを含め、`tests/` を含めない |

---

## 2. パッケージ一覧と責務【規定】

| パス | 責務 | 入出力 |
|------|------|--------|
| `src/conductor/__main__.py` | 薄い入口。`main() -> int` を持つ。最初に、内部の誤りの捕捉(`sys.excepthook` の置き換えと、`main` の `try/except`)を設定する。想定外の例外を捕まえ、`conductor: error: internal error` を標準エラーに 1 行出して終了コード 70 にする。本番の `Deps` を作って `cli` の `run` を呼び、終了コードを返す。`if __name__ == "__main__": sys.exit(main())`。終了(`sys.exit`)は、ここだけ(と、07 章 10 章の内部の誤りの経路、07 章 7.6 節の `KeyboardInterrupt` の経路) | プロセスの引数・環境・標準入出力 |
| `src/conductor/definition/` | 定義ファイルの読み込み(場所の決定と読み取り、YAML/JSON の構文解析、入れ子の深さ、`format_version`、構造・型・遷移・終了コード・名前の検査)。誤りを位置(行)つきで返す。**定義の型**(`Definition`、`Job`、`Command`、`RetryPolicy`、`Backoff`、`Transition`)と、誤りの種類(`ErrorKind`。03 章)を持つ。`run_id_env` だけのゆるい読み取りも持つ。木の読み取り(`read_tree`)と JSON の読み取り(`read_json`)は、`convert` と `state` が共有する | ファイルを読む(`load` だけ)。ほかは文字列に対する純粋な処理 |
| `src/conductor/convert/` | YAML ⇔ JSON の変換(`convert --to yaml\|json`)。定義の検証はしない。木の読み取りは `definition` と共有する | 入力ファイル 1 つを読む(`convert_file`)。`convert_text` は文字列に対して純粋 |
| `src/conductor/runid.py` | run ID の値の検査と、決定(`--run-id` と環境変数のどちらを使うか)に使う純粋な関数。**`RunIdError` の型**を持つ | なし |
| `src/conductor/flow/` | 純粋な判断。実行結果の分類(成功・警告・エラー・分類なし)と報告する終了コード、遷移と次の動き、起動前の判断(実行・スキップ・拒否)、中断時の再開位置、リトライの判断と待ち時間(`int` の飽和演算)、終了コードの範囲合わせ。**実行時の値の型**(`JobExecution`、`JobState`、`PipelineState`、`JobOutcome`、`NextStep` など)と、終了コードの定数を持つ | なし(引数だけから結果を決める。ファイル・プロセス・環境変数・時刻に触れない) |
| `src/conductor/state/` | ステートの読み書き。JSON の表現(ファイル用の型と、固定の順序の書き出し)、記録関数(結果・スキップ・中断を、新しい `PipelineState` として作る純粋な関数)、読み込み(14 段階の検査と異常系)、書き込み(一時ファイル + `fsync` + `os.replace` + ディレクトリの同期)、定義との整合性の検査、`reset` と `reset --all`、ステートファイルの場所と名前。**ステートの誤りの型**(`StateError`)を持つ | ステートのファイル(とそのディレクトリ)。記録関数と整合性の検査は純粋 |
| `src/conductor/runner/` | `command` の分割と検査、`subprocess.Popen` によるジョブの起動と待ち、シグナルの転送、リトライの待機、中断の合図(`Interrupt`)、シグナル監視、プラットフォーム層(`platform.py`。`sys.platform` を読む唯一の場所。OS 差は、ここ 1 か所に集める)。**`CommandLine` と `CommandError` の型**を持つ | 子プロセス、時刻(待ち)、OS のシグナル。分割と検査は純粋 |
| `src/conductor/report/` | 出力。メッセージカタログ(英語・日本語)、ロケールと色の判定、`-v` の行、誤りの行、`watch` の表、使い方(`--help`)、入力由来の文字列のエスケープと切り詰め。**文を作るだけ**で、書き込みは `cli` が `Sink` へ行う | 環境変数・端末かどうか・OS の種別は、引数(`ReportEnv`)で受け取る(自分では読まない) |
| `src/conductor/cli/` | 引数解析(誤りの優先順位)、各サブコマンド(`run`・`pipeline`・`watch`・`reset`・`validate`・`convert`)の組み立て、`Deps`(差し替え口の `Protocol`)とその本番の実装、終了コードの決定。上記のすべてを呼ぶ | `Deps` を通した、すべての入出力 |
| `src/conductor/version.py` | 版の定数(`VERSION = "0.8.0"`)。`conductor --version` の表示と、リリースの検査([11-quality-and-build.md](11-quality-and-build.md))が使う。`pyproject.toml` の `version` と一致することを、テストで検査する。実行時に `importlib.metadata` で版を引かない(ソースから動かすときに使えず、起動も遅くなるため) | なし |
| `tests/` | 単体テスト(`src/conductor/` と同じ構成)と受け入れテスト(`tests/acceptance/`。実際に `python -m conductor` またはコンソールスクリプトを起動する)。配布物に含めない | テストだけが使う |
| `tests/support/` | テスト用の偽実装(`Deps` の各 `Protocol` を満たす)、一時ディレクトリで定義やステートを作る補助(4.4 節) | テストだけが使う |
| `tests/jobs/` | 受け入れテスト用のテストジョブ(終了コード、スリープ、標準出力への書き込み、環境変数・引数の出力など。Python のスクリプトで、`sys.executable` で起動する。[11-quality-and-build.md](11-quality-and-build.md))。配布物に含めない | テストだけが起動する |

### 2.1 各パッケージの公開(`__all__`)するものの骨格【規定】

各パッケージの `__init__.py` が、他のパッケージから使われる名前だけを、`__all__` に入れて公開する。他のパッケージからの import は、**必ず相手の `__init__.py` を通す**(`from conductor.definition import Definition`)。`__init__.py` に出さない補助の名前は、パッケージの内側だけで使う(補助の名前は実装者が決める)。パッケージの内側のモジュールは、互いを直接 import する(自分の `__init__` を経由しない。循環 import を避けるため)。型だけを使う import は、`if TYPE_CHECKING:` の内側に置いてよい(依存の向きの規則 3 章は、これも依存として数える)。

型の置き場所の決定(依存の最下層に置く。循環を避けるため。3 章):

| 型 | 置き場所 |
|----|----------|
| 定義の型(`Definition`、`Job`、`Command`、`RetryPolicy`、`Backoff`、`Transition`)、定義の誤り(`DefinitionError`、`ErrorKind`)、木の型(`TreeNode` ほか) | `conductor.definition` |
| `RunIdError`、`RunIdErrorKind` | `conductor.runid` |
| 実行時の値の型(`JobExecution`、`JobState`、`PipelineState`、`JobOutcome`、`NextStep`、`ResumePoint` など)、終了コードの定数 | `conductor.flow` |
| ステートの誤り(`StateError`)、`Durability`、`StateStore` | `conductor.state` |
| `CommandLine`、`CommandError`、`CommandResult`、`Interrupt`、`WaitOutcome`、`Platform`、`PlatformKind`、`ChildLike` | `conductor.runner` |
| `Locale`、`ReportEnv`、`Reporter` | `conductor.report` |
| `Deps` と、その各 `Protocol`(`Env`、`Clock`、`Sink`、`Definitions`、`States`、`Converter`、`Process`、`Signals`) | `conductor.cli` |

#### `conductor.definition`

```python
# 03 章の型が正(骨格の再掲)。ここでは他のパッケージが使う名前だけを示す。
# 「無し」は None で表す。コレクションは tuple(不変)で持つ。整数は int(任意の精度。bool を整数として受理しない検査は 03 章)。
@dataclass(frozen=True, slots=True)
class Definition:
    name: str                                  # pipeline の名前
    description: str | None
    run_id_env: str | None                     # None なら実行時に CONDUCTOR_RUN_ID
    remove_on_complete: bool
    format_version: Literal[1]
    jobs: tuple[Job, ...]                      # 記述順。1 つ以上

@dataclass(frozen=True, slots=True)
class Job:
    id: str
    command: Command
    warning_exit_codes: tuple[int, ...]
    success_exit_codes: tuple[int, ...] | None # None なら既定の (0,)
    transitions: tuple[Transition, ...]        # on ごとに高々 1 つ
    retry: RetryPolicy | None

@dataclass(frozen=True, slots=True)
class LineCommand:                             # 文字列形式
    line: str

@dataclass(frozen=True, slots=True)
class ArgvCommand:                             # 配列形式。1 要素以上
    argv: tuple[str, ...]

Command = LineCommand | ArgvCommand

@dataclass(frozen=True, slots=True)
class Transition:
    on: Literal["success", "warning"]
    next: str | None                           # 次のジョブID。None は end(定義の "end")

@dataclass(frozen=True, slots=True)
class RetryPolicy:
    exit_codes: tuple[int, ...]                # 1 要素以上
    max: int                                   # 1〜100
    interval_secs: int                         # 秒。int(上限の飽和は flow で行う)
    backoff: Backoff | None                    # multiplier と max_interval は両方ある、または両方ない

@dataclass(frozen=True, slots=True)
class Backoff:
    multiplier: int                            # 2〜10
    max_interval_secs: int

# 03 章が正本。誤りは、Exception を継承しない普通の値(dataclass)。種類は 31 種(ERROR_KINDS)。
ErrorKind = Literal["unsupportedFormat", "syntax", ...]   # 03 章の 31 種(文字列リテラルの和)

@dataclass(frozen=True, slots=True)
class DefinitionError:
    kind: ErrorKind
    file: str                                  # 表示用のファイル名
    line: int | None                           # 1 始まり。None は行なし
    column: int | None                         # 順序づけにだけ使い、表示しない
    reason: str                                # 固定の日本語の文。入力の値や、ライブラリのエラー文を含めない

@dataclass(frozen=True, slots=True)
class LoadOk:
    definition: Definition
    ok: Literal[True] = True

@dataclass(frozen=True, slots=True)
class LoadErr:
    errors: tuple[DefinitionError, ...]
    ok: Literal[False] = False

LoadResult = LoadOk | LoadErr

# 署名の正本は 03 章。誤りは、見つけたものをまとめて返す(早い段階の誤りは 1 件で打ち切る。03 章)。例外を送出しない。
def load(path: str, expected_pipeline_name: str | None) -> LoadResult: ...
# ファイルに触れず、文字列に対して 03 章の段階 2〜5 を行う(純粋)。
def parse(text: str, fmt: Literal["yaml", "json"], file_label: str, expected_pipeline_name: str | None) -> LoadResult: ...
# run_id_env だけをゆるく読む(どの失敗でも "" を返し、既定の名前を使う)。
def loose_run_id_env(path: str) -> str: ...
# YAML/JSON を 03 章の規定で木に読む(段階 2〜3)。convert が共有する。
def read_tree(text: str, fmt: Literal["yaml", "json"], file_label: str) -> TreeResult: ...
# 自前の JSON の読み取り(03 章)。state も、ステートの読み込みに使う(06 章)。
def read_json(text: str, options: ReadJsonOptions | None = None) -> ReadJsonResult: ...
# 段階 3 の上限の確認(深さ・要素数・文字の大きさ)。state は深さだけを使う。
def check_limits(root: TreeNode, which: Literal["all", "depthOnly"]) -> DefinitionError | None: ...
# 木の型(TreeNode、MapNode、MapEntry、Pos ほか。03 章)も公開する。
```

- 定義の書式は、拡張子で決める(`.yaml`/`.yml`/`.json`、大文字小文字を区別しない。それ以外は `unsupportedFormat`)。[03-definition-format.md](03-definition-format.md)。
- 終了コードの配列(`warning_exit_codes` など)は、整数の字句から `int` で範囲を検査する(32 ビット符号つきの範囲。`bool`、`float`、`nan`/`inf`、`1.0` は整数でない。03 章)。
- YAML のライブラリの import は、YAML を読む関数の内側に遅らせる【目安】(`--help`・`--version`・使い方の誤りで、ライブラリを読み込まない。起動時間の計測は 11 章)。この 1 か所に限り、関数の内側の import を許す(ruff の `PLC0415` を、このファイルだけ無効にする)。

#### `conductor.runid`

```python
RunIdErrorKind = Literal["Empty", "TooLong", "InvalidChar", "LeadingDot"]

@dataclass(frozen=True, slots=True)
class RunIdError:
    kind: RunIdErrorKind                       # 値は持たない(10 章)

@dataclass(frozen=True, slots=True)
class RunIdOk:
    run_id: str
    ok: Literal[True] = True

@dataclass(frozen=True, slots=True)
class RunIdErr:
    error: RunIdError
    ok: Literal[False] = False

RunIdParse = RunIdOk | RunIdErr

# 値の検査(順 1〜4。05 章)。検査を通った値は、ただの str として扱う。
def parse_run_id(value: str) -> RunIdParse: ...

# 決定(05 章)。--run-id、環境変数の名前、環境変数の読み取りから、run ID(なしは None)を決める。
@dataclass(frozen=True, slots=True)
class ResolveOk:
    run_id: str | None
    ok: Literal[True] = True

@dataclass(frozen=True, slots=True)
class ResolveErr:
    error: RunIdError
    subject: str                               # 表示で使う、どの入力か
    ok: Literal[False] = False

ResolveRunIdResult = ResolveOk | ResolveErr

def resolve_run_id(
    flag_value: str | None,                    # --run-id が指定されなければ None(空文字列は「指定された」)
    env_name: str,                             # 定義の run_id_env、なければ "CONDUCTOR_RUN_ID"
    getenv: Callable[[str], str | None],
) -> ResolveRunIdResult: ...
```

#### `conductor.flow`

```python
# 型と関数の骨格の全体は 05 章が正。次の名前を公開する(snake_case)。
#   型: JobStatus、SkipReason、JobState、PipelineState、JobExecution、Classification、JobOutcome、
#       DecisionReason、Decision、RetryDecision、ResumePoint、RunMode、NextStep、
#       TransitionTarget、FlowJob、FlowDefinition
#   関数: to_flow_definition、classify、classify_and_decide、next_step、decide_before_run、
#         retry_decision、is_retry_target、next_wait_secs、sat_mul_u64、resume_point_on_interrupt、
#         exit_code_for、signal_exit_code、sat_add_i32
#   定数: U64_MAX(2**64 - 1)、終了コードの定数(10 章の全件)
# 05 章と名前が異なる場合は、05 章に合わせる。
# 「無し」は None で表す。コレクションは tuple。ここには、runner、state、report が使う型だけを示す。

# ジョブの実行結果(07 章 4.5 節)
@dataclass(frozen=True, slots=True)
class JobExecution:
    job_id: str
    command: Command                           # 表示とステートには使わない(job_id だけ)
    started: bool                              # False は、起動の前に合図が立っていたときだけ
    exit_code: int | None
    terminated_by_signal: int | None
    start_failure: bool
    interrupted: bool
    signal_number: int | None

@dataclass(frozen=True, slots=True)
class PipelineState:                           # 05 章が正
    definition_key: str
    definition_path: str
    pipeline_name: str
    run_id: str | None
    job_order: tuple[str, ...]
    jobs: tuple[JobState, ...]
    resume_job_id: str | None
    end_reached: bool
    completed: bool
    # 06 章の論理項目の全件

# 終了コードの定数。10 章の全件
EXIT_OK: Final = 0
# ...

# リトライの待ち時間(秒)は int。64 ビット符号なしの上限で飽和する(05 章): min(x, 2**64 - 1)。折り返さない。
def next_wait_secs(policy: RetryPolicy, retries_done: int, last_wait_secs: int) -> int: ...
```

#### `conductor.state`

```python
# DURABLE: 同期してから置き換える。FAST: 同期を省く(起動前の初期ステート)
Durability = Literal["durable", "fast"]

StateErrorKind = Literal["unreadable", "tooLarge", "corrupt", "versionMismatch", "writeFailed", "removeFailed"]
StateErrorDetail = Literal["runIdMismatch"]    # 内部の判別用(reset --all)。表示に使わない

# 型の正本は 06 章。reason は 06 章の固定の日本語文(入力の値や下位のエラー文を含めない)。
class StateError(Exception):
    kind: StateErrorKind
    path: str
    reason: str
    detail: StateErrorDetail | None

# 記録関数(純粋。元を変更せず、新しい値を返す。署名の正本は 05 章)
def initial_state(definition_path: str, definition: FlowDefinition, run_id: str | None) -> PipelineState: ...
def apply_outcome(s: PipelineState, job_id: str, execution: JobExecution, outcome: JobOutcome, nxt: NextStep) -> PipelineState: ...
def apply_skip(s: PipelineState, job_id: str, reason: DecisionReason, definition: FlowDefinition) -> PipelineState: ...
def apply_interrupt(s: PipelineState, point: ResumePoint) -> PipelineState: ...

# 整合性の検査(純粋。06 章が正本)
@dataclass(frozen=True, slots=True)
class ConsistencyOk:
    kind: Literal["ok"] = "ok"

@dataclass(frozen=True, slots=True)
class ConsistencyMismatch:
    what: Literal["pipelineName", "jobIds", "jobOrder"]
    kind: Literal["mismatch"] = "mismatch"

ConsistencyResult = ConsistencyOk | ConsistencyMismatch
def check_consistency(state: PipelineState, definition: FlowDefinition) -> ConsistencyResult: ...

# 入出力の関数(06 章。投げる形): 失敗は StateError を送出する。
#   read_state(path, run_id) -> PipelineState | None            # None は「空」。完了のものも「空」
#   read_state_raw(path, run_id) -> PipelineState | None        # 完了のものも返す(watch)
#   write_state(path, run_id, s, d) -> None                     # writeFailed
#   reset_state(path, run_id) -> None                           # removeFailed
#   reset_all_states(path) -> ResetAllOutcome                   # 06 章
# StateStore は、これらを包んで、StateError を値に変える(Deps の「例外を送出しない」の規則に合わせるため)。
# StateError 以外の例外は捕まえない(想定外の内部失敗。入口が 70 にする)。
@dataclass(frozen=True, slots=True)
class ResetAllOutcome:
    kept: tuple[str, ...]                      # 読めずに残したステートのファイル名(整列済み)

@dataclass(frozen=True, slots=True)
class StateReadOk:
    state: PipelineState | None                # None は「空」
    ok: Literal[True] = True

@dataclass(frozen=True, slots=True)
class StateFail:
    error: StateError
    ok: Literal[False] = False

StateReadResult = StateReadOk | StateFail
StateResult = StateWriteOk | StateFail         # StateWriteOk は ok: Literal[True] だけを持つ
ResetAllResult = ResetAllOk | StateFail        # ResetAllOk は outcome: ResetAllOutcome と ok: Literal[True]

@dataclass(frozen=True, slots=True)
class StateStoreOptions:
    now: Callable[[], datetime]                # 一時ファイル名用
    sync_directory: bool                       # UNIX 系 True、Windows False(cli が PlatformKind から決める)

class StateStore(Protocol):
    def read(self, definition_path: str, run_id: str | None) -> StateReadResult: ...
    def read_raw(self, definition_path: str, run_id: str | None) -> StateReadResult: ...
    def write(self, definition_path: str, run_id: str | None, s: PipelineState, d: Durability) -> StateResult: ...
    def reset(self, definition_path: str, run_id: str | None) -> StateResult: ...
    def reset_all(self, definition_path: str) -> ResetAllResult: ...

def create_state_store(opts: StateStoreOptions) -> StateStore: ...
```

- ステートの JSON の表現(ファイル用の型、キーの順、整数の幅の検査、深さの制限)は [06-state-file.md](06-state-file.md)。`PipelineState` を直接 `json.dumps` しない。`state` の内側の、ファイル用の型(`__init__.py` に出さない)を通し、キーの順を固定して書き出す。読み込みは、`json.loads` ではなく、`definition.read_json` を使う(位置・重複キー・整数の字句・深さの上限のため。06 章)。
- 入出力は、同期の標準ライブラリ(`open`、`os.replace`、`os.fsync`)で行う(7 章)。

#### `conductor.runner`

骨格は [07-process-and-signals.md](07-process-and-signals.md)。公開するのは次のとおり。

```python
@dataclass(frozen=True, slots=True)
class CommandLine:
    program: str
    args: tuple[str, ...]

CommandErrorKind = Literal["unclosedQuote", "empty"]

class CommandError(Exception):                 # メッセージ(args[0])は固定の文(入力の値を含めない)
    kind: CommandErrorKind

CommandResult = CommandOk | CommandErr         # 07 章 3.8 節
def split_command(s: str) -> CommandResult: ...
def check_command(c: Command) -> CommandResult: ...

class Interrupt:                               # 07 章 6 章
    def trigger(self, n: int) -> None: ...
    def number(self) -> int | None: ...

def run_job(job: Job, sig: Interrupt, *, spawn: Spawner = ..., platform: Platform = ..., sleep: Callable[[float], None] = ...) -> JobExecution: ...   # 例外を送出しない
WaitOutcome = Elapsed | Interrupted            # 07 章 7.5 節
def wait_retry(secs: int, sig: Interrupt, *, sleep: Callable[[float], None] = ..., monotonic_ns: Callable[[], int] = ...) -> WaitOutcome: ...   # 例外を送出しない

SignalStart = Literal["started", "alreadyRegistered", "failed"]

class Watcher(Protocol):
    result: SignalStart
    def stop(self) -> None: ...                # 登録したハンドラを元に戻す。本番では呼ばない(テスト用)

def start_watcher(sig: Interrupt) -> Watcher: ...   # 本番: プロセスで 1 回だけ。2 回目は "alreadyRegistered"

PlatformKind = Literal["posix", "windows"]
def detect_platform() -> PlatformKind: ...     # sys.platform を読む唯一の関数
```

#### `conductor.report`

```python
Locale = Literal["en", "ja"]

class ReportEnv(Protocol):
    def getenv(self, name: str) -> str | None: ...
    def stdout_is_terminal(self) -> bool: ...
    def platform(self) -> PlatformKind: ...    # 色を使えるかの判定に使う(Windows は色なし)

class Reporter(Protocol):
    # 文を作る関数群。戻り値は文字列(改行を含む)。書き込みはしない。
    def verbose_result(self, o: JobOutcome, er: JobExecution) -> str: ...
    def definition_errors(self, errs: Sequence[DefinitionError]) -> str: ...
    def command_error(self, path: str, job_id: str, e: CommandError) -> str: ...
    def state_error(self, e: StateError) -> str: ...
    def watch(self, s: PipelineState) -> str: ...      # ほか、必要な引数は 08 章
    def usage(self) -> str: ...
    # ほか、08 章のメッセージの全件

# ロケールと色は、最初に必要になったとき 1 回だけ判定する(遅延評価)。
def create_reporter(env: ReportEnv) -> Reporter: ...
```

- `report` は、`definition`・`flow`・`state`・`runner` の型を読むだけで、それらを変更しない。環境変数と端末の判定は、`ReportEnv` を通して受け取る(`os.environ` と `sys.stdout.isatty()` を直接読まない)。

#### `conductor.cli`

3 章と 4 章(`Deps`)。

```python
# run は、引数を解析し、サブコマンドを実行して、終了コード(0〜255)を返す。sys.exit は呼ばない。
# 想定外の内部の失敗は、ここでは捕まえない(__main__ の入口が捕まえる。9 章)。
def run(args: Sequence[str], d: Deps) -> int: ...
def system_deps() -> Deps: ...                 # 本番の実体の束
```

#### `conductor.__main__`

```python
# 骨格(実装ではない): 最初に内部の誤りの捕捉を設定し、run を呼び、flush して、終了コードを返す。
def main(argv: Sequence[str] | None = None) -> int: ...     # argv が None なら sys.argv[1:]
if __name__ == "__main__":
    sys.exit(main())
```

- コンソールスクリプトの起動用のコードは、`sys.exit(main())` を呼ぶ。したがって `main()` は、終了コードを `return` で返す(`sys.exit` を呼ばない)。
- `main()` は、`cli` を関数の内側で import する【目安】(YAML のライブラリを読むまで、読み込みを遅らせる方針と合わせる。上記の `definition` の注意)。

---

## 3. 依存の向き【規定】

矢印は「import する」。**循環させない**。Python は、import の循環が、実行時の `ImportError`(部分的に初期化されたモジュール)になりうるので、型検査に加えて、循環の検査を品質ゲートに含める(下記の 10 番)。

```text
__main__ ─▶ cli

cli ─▶ report, runner, state, flow, definition, convert, runid, version

report ─▶ state, runner, flow, definition, runid   (型だけ。cli を import しない)

runner ─▶ flow, definition
state  ─▶ flow, definition, runid
flow   ─▶ definition
convert ─▶ definition   (木の読み取りを共有。ほかのパッケージには依存しない)
runid  ─▶ (標準ライブラリだけ)
definition ─▶ (YAML のライブラリ、標準ライブラリだけ)
version ─▶ (なし)

tests/support ─▶ cli(型だけ), flow, definition, runner, state, runid, convert
```

図(下へ向かって依存する):

```text
                     __main__
                        │
                       cli  ─────────┬────────────┐
        ┌──────┬──────┬───┼──────┐   │            │
     report  runner  state  convert version      runid
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

1. **`cli` が最上位**(`__main__` を除く)。他のパッケージは `cli` を import しない。`report` も `cli` を import しない(環境の判定は `ReportEnv` の形で受け取る)。
2. **`definition` と `flow` が下層**。`definition` は、`conductor` のどのパッケージにも依存しない。`flow` は `definition` だけに依存する。入出力に触れない(`definition.load` の読み取りを除く)。
3. **`state` と `runner` は、`flow` と `definition` の型を使うが、互いに依存しない**。`state` は「ステートを読み書きする」だけで、ジョブを起動しない。`runner` は「ジョブを起動して待つ」だけで、ステートに触れない。両者をつなぐのは `cli`。`state` が OS の違い(ディレクトリの同期の有無)を知る必要があるときは、`cli` が `PlatformKind` から `StateStoreOptions.sync_directory` を決めて渡す。
4. **`convert` が使う `conductor` のパッケージは `definition` だけ**で、木の読み取り(`definition.read_tree`。03 章)を共有するためである。定義の検証(`definition.load` の段階 4・5)は呼ばない。
5. **`report` は、データを受け取って文を作る**。判断をしない。何を表示するかは `cli` が決める。
6. **OS ごとの分岐は、`runner/platform.py` の 1 か所だけ**([07-process-and-signals.md](07-process-and-signals.md) の 8 章)。`sys.platform` を読むのは `detect_platform()` だけ。`state` と `report` と `cli` は、`cli` が `Deps.env.platform()` で得た `PlatformKind` を、引数で受け取って使う。
7. **型の置き場所は、依存の最下層**(2.1 節)。型だけを使うために、上の層や横の層を import しない。上の層の型が要るときは、型を下の層へ移す。`report` が `runner` の `PlatformKind` を型として読むのは、この規則の範囲内(`report` は `runner` の下位でも上位でもなく、`runner` は `report` を import しないため、循環しない)。
8. **import は、相手のパッケージの `__init__.py` を通す**。パッケージの内側のモジュールを、外から直接 import しない(`from conductor.flow.classify import classify` と書かない)。ただし、テストは、同じパッケージの内部のモジュールを直接 import してよい。
9. **`tests/support/` は `conductor.cli` の型を import してよい**。`src/conductor/` は `tests/` を import しないので、循環しない。`Deps` の各 `Protocol` のメソッドのシグネチャは、`cli` の型ではなく、下層のパッケージの型だけで書く(4 章)。偽実装が、`cli` の型を import しなくても、構造的な型付け(メソッドの形が合うこと)で、`Protocol` を満たせるようにするため。
10. 強制の方法(2 段): (a)(依存の向きの検査)`tests/test_architecture.py` が、`ast` で `src/conductor/` のすべてのモジュールの import 文を走査し、上の表の許可されない組み合わせ(例: `conductor/definition/**` から `conductor.flow` への import)と、パッケージの内側のモジュールへの外からの直接の import、循環(パッケージ間のグラフ)を、失敗にする。標準ライブラリだけで書ける。(b)(補助)ruff の `flake8-tidy-imports`(`banned-api`)で、製品のコードの禁止の import(`asyncio`、`argparse`、`threading`、`multiprocessing`、`concurrent`、`socket`、`urllib`、`http`、`shlex`、`pickle`)を失敗にする(8 章)。専用の道具(`import-linter` など)を使う場合は、実装時に確認(未確認。この文書は、依存を増やさない `ast` の検査を基本とする)。

---

## 4. `Deps`(差し替え口)とテスト用の偽実装【規定】

### 4.1 方針

`cli` の流れは、環境・時刻・ファイル(定義・ステート・変換)・プロセス・シグナル・標準出力/標準エラーを、**まとめて 1 つの `Deps` オブジェクト**で受け取る。各フィールドは `typing.Protocol` で定義する。
本番の入口(`system_deps`)は本番の実体を渡し、テストは偽の実体を渡す。純粋な判断(`flow` の関数、`state` の記録関数、`report` の文の組み立て)は、`Protocol` を通さず、`cli` が直接呼ぶ(差し替え不要)。

### 4.2 `Protocol` の定義(`cli/deps.py`)

```python
# 環境(環境変数、標準出力が端末か、OS の種別)。
class Env(Protocol):
    def getenv(self, name: str) -> str | None: ...      # 未設定は None
    def stdout_is_terminal(self) -> bool: ...
    def platform(self) -> PlatformKind: ...
    def cwd(self) -> str: ...                           # 必要なとき(定義ファイルの相対パスの扱いなど。04・05 章)

# 時刻。ステートの一時ファイル名などに使う。
class Clock(Protocol):
    def now(self) -> datetime: ...

# 標準出力・標準エラーへの書き込み。書くたびに flush する。失敗は値で返す(例外を送出しない)。True は成功。
class Sink(Protocol):
    def write_out(self, text: str) -> bool: ...
    def write_err(self, text: str) -> bool: ...

# 定義ファイルの読み込み。
class Definitions(Protocol):
    def load(self, path: str, expected_pipeline_name: str | None) -> LoadResult: ...
    def loose_run_id_env(self, path: str) -> str: ...   # どの失敗でも ""(既定の名前を使う)

# ステートの読み書き(ファイルシステム)。state.StateStore と同じ形。
class States(Protocol):
    def read(self, definition_path: str, run_id: str | None) -> StateReadResult: ...
    def read_raw(self, definition_path: str, run_id: str | None) -> StateReadResult: ...
    def write(self, definition_path: str, run_id: str | None, s: PipelineState, d: Durability) -> StateResult: ...
    def reset(self, definition_path: str, run_id: str | None) -> StateResult: ...
    def reset_all(self, definition_path: str) -> ResetAllResult: ...

# 変換。
class Converter(Protocol):
    def convert(self, input_path: str, to: Literal["yaml", "json"]) -> ConvertResult: ...   # ConvertOk(text) | ConvertErr(error)

# プロセス(ジョブの起動と待ち)。
class Process(Protocol):
    def check_command(self, c: Command) -> CommandResult: ...
    def run_job(self, job: Job, sig: Interrupt) -> JobExecution: ...
    def wait(self, secs: int, sig: Interrupt) -> WaitOutcome: ...     # リトライの待機

# シグナルの監視。
class Signals(Protocol):
    # 受け取った最初のシグナルの番号を sig に立てる監視を始める。
    # "alreadyRegistered" なら、流れは成功と同様に続ける。"failed" は、登録の失敗(終了コード 70)。
    def start(self, sig: Interrupt) -> SignalStart: ...

@dataclass(frozen=True, slots=True)
class Deps:
    env: Env
    clock: Clock
    out: Sink
    defs: Definitions
    states: States
    converter: Converter
    proc: Process               # 変数名 process は、標準ライブラリの名前と紛らわしいので使わない
    signals: Signals
```

- 引数 `args` は、`run` の引数で受け取る(`Env` に入れない)。
- **ファイルシステムの差し替え**は、`Definitions`・`States`・`Converter` の 3 つの `Protocol` が担う。本番の実装は、実ファイルを読み書きする。偽の実装は、メモリに持つ(ファイルなし)。ステートの書き込みの原子的な置換の失敗や、ディスクの異常の検査は、`state` の内側のテスト(一時ディレクトリの実ファイル)で行う。
- `Sink` は、書き込みの失敗を値(`False`)で返す(例外を送出しない。`BrokenPipeError` を含む `OSError` を、`Sink` の内側で捕まえる。08 章)。標準出力と標準エラーを混ぜない。**書き込みは `cli` の 1 か所の補助関数を通す**(順序を保つ。書くたびに `flush()` する。[07-process-and-signals.md](07-process-and-signals.md) の 9 章)。
- `Process.wait` は `wait_retry` を包む。偽の実装は、待ち時間を記録するだけで実際には待たない。
- すべてのメソッドは、同期の関数である(`async def` を使わない)。`Process.run_job`、`Process.wait`、`Definitions`、`States`、`Converter`、`Sink` のメソッドは、例外を送出しない(失敗は値で返す。9 章)。

### 4.3 本番の実装(`cli/system.py`)【目安】

| 名前 | `Protocol` | 中身 |
|------|-----------|------|
| `SystemEnv` | `Env` | `os.environ.get(name)`、`sys.stdout.isatty()`(`sys.stdout` が `None` や閉じているときは `False`)、`detect_platform()`、`os.getcwd()`。端末の判定の挙動(UNIX 系と Windows の違い)は、実装時に 3 OS で確かめる(未確認) |
| `SystemClock` | `Clock` | `datetime.now(timezone.utc)` |
| `StdSink` | `Sink` | `sys.stdout.write(text)` と `sys.stdout.flush()`(標準エラーも同じ)を、`try/except OSError`(`BrokenPipeError` を含む)で包み、失敗なら `False`。文字コードと `errors` の設定は 08 章 |
| `SystemDefs` | `Definitions` | `definition.load`、`definition.loose_run_id_env` |
| `SystemStates` | `States` | `state.create_state_store(StateStoreOptions(now=..., sync_directory=detect_platform() != "windows"))` を持ち、メソッドを委譲 |
| `SystemConverter` | `Converter` | `convert.convert_file` |
| `SystemProcess` | `Process` | `runner.check_command`、`runner.run_job`、`runner.wait_retry` |
| `SystemSignals` | `Signals` | `runner.start_watcher(sig).result`(`stop` は本番では呼ばない) |

### 4.4 偽の実装(`tests/support/`)【規定】

偽の実装は、次の振る舞いを持つ(名前は実装者が決める)。すべて `cli/deps.py` の `Protocol` を構造的に満たす(`d: Deps = Deps(env=FakeEnv(), ...)` のように、`mypy --strict` が確かめる)。

| 偽の実装 | 振る舞い | 主な検査 |
|-----------|----------|----------|
| `FakeEnv` | 環境変数を表(`dict`)で与える。`getenv` を呼んだ回数と名前を記録する。`stdout_is_terminal`、`platform`、`cwd` の値を設定する | ロケール(`LC_ALL`・`LC_MESSAGES`・`LANG`)・`NO_COLOR`・run ID の環境変数。ロケールの判定を**遅延**して、最初に必要になったとき 1 回だけであること(成功・スキップで `-v` なしなら、読まない) |
| `FakeClock` | 固定の時刻を返す。進める操作を持つ | 一時ファイル名の時刻 |
| `BufSink` | `write_out`・`write_err` を、別々の文字列に溜める。呼び出しの順序の記録も持つ。「n 回目の書き込みで失敗する(`False` を返す)」設定を持つ | 出力の中身、`cannot write to standard output`(`watch` の 65、`convert` の 67) |
| `FakeDefs` | パスごとに、決めた `Definition` や誤りの並びを返す。または、実ファイル(一時ディレクトリ)を `definition.load` で読む版 | 定義の誤り(64)の経路 |
| `MemStates` | ステートをメモリに持つ(ファイルなし)。読み込み・書き込み・削除の呼び出しを記録する。「読み込みで `StateError` を返す」「書き込みで失敗する」設定を持つ | 大きなジョブ数(1,000 個)の流れの測定、ステートの異常系(65) |
| `FakeConverter` | 入力パスごとに、決めた文字列かエラーを返す | `convert` の 67 |
| `ScriptProcess` | ジョブIDごとに、台本の `JobExecution`(終了コード・シグナル終了・起動失敗・中断)を順に返す。呼び出しの記録(何が、何回、どの順で実行されたか)を持つ。`check_command` は、台本の誤り、または `runner.check_command` を呼ぶ。`wait` は、待ち時間(秒の `int`)を記録するだけ。「`wait` が中断を返す」設定を持つ | 分類・遷移・再開・リトライ(待ち時間の列 1・2・4・…)・中断の経路。実際の待ちをしない |
| `FakeSignals` | `start` の戻り値を設定する(`"started"`、`"alreadyRegistered"`、`"failed"`)。「開始時に、すでに `sig.trigger(n)` を呼ぶ」設定を持つ | 登録の失敗(70)、登録済み、開始時にすでに合図が立っている |
| 補助 | 一時ディレクトリ(`tempfile.TemporaryDirectory`、または pytest の `tmp_path`)に、定義ファイルを書く。実行して、ステートファイルの内容を読み戻す。終了後に削除する | 実ファイルを使うテスト(受け入れテストの下位) |

- 実プロセスを使うテスト(`runner` の内側と受け入れテスト)は、`tests/jobs/` のテストジョブを使う([11-quality-and-build.md](11-quality-and-build.md))。
- スレッドを使わないので、偽の実装にロックは要らない。シグナルを使う偽(`Interrupt` を使うもの)は、`trigger` を、偽の `sleep` や偽の `Spawner` の中から呼ぶ形で作る(別のスレッドや本物のシグナルを使わない)。

---

## 5. 実行のシーケンス

### 5.1 入口

```text
conductor/__main__.py: main(argv=None):
    内部の誤りの捕捉を設定する                           # 最初に。sys.excepthook / try-except(07 の 10 章)
    code = cli.run(argv または sys.argv[1:], system_deps())
    標準出力・標準エラーを flush する
    return code                                           # 終了は、`sys.exit(main())` の 1 か所

cli.run(args, d):
    parsed = 引数を解析する(04 章)
      誤り           → 使い方の誤りを標準エラーに表示、return 68
      help           → 使い方全文を標準出力へ、return 0
      version        → `conductor 0.8.0` を標準出力へ、return 0
      コマンド       → run / pipeline / watch / reset / validate / convert のどれかへ
```

### 5.2 `pipeline` の実行

`prepare`(前処理)の順序と、そこでの失敗の終了コードは、[05-execution-flow.md](05-execution-flow.md) と [10-errors-and-exit-codes.md](10-errors-and-exit-codes.md) に固定されている。すべての呼び出しは直列で、並行に走らせない(7.1 節)。

```text
cli: pipeline
 ├─ prepare
 │    ├─ sig = Interrupt();  r = d.signals.start(sig)             → "alreadyRegistered" は続行 / "failed" は 70(定義の検査より前)
 │    ├─ d.defs.load(...)                                         → 失敗 64(パイプライン名の照合を含む)
 │    ├─ d.proc.check_command(全ジョブ)                           → 失敗 64(誤りのあるジョブすべてを 1 ジョブ 1 行で報告)
 │    ├─ run ID の決定(--run-id | 環境変数。runid)               → 失敗 68
 │    ├─ d.states.read(...)  (完了のものは「空」)                  → 失敗 65
 │    └─ state.check_consistency(...)                              → 食い違い 65
 └─ for job in 定義のジョブ(記述順):
      ├─ flow.decide_before_run(...) → run / skip / reject
      │    ├─ reject → report: 表示、return 66
      │    ├─ skip   → state.apply_skip → 変化があれば d.states.write(durable)、続行(完了なら finish)
      │    └─ run:
      │         ├─ ステートが空なら state.initial_state → d.states.write(fast)
      │         ├─ リトライ込みの実行
      │         │    loop { r = d.proc.run_job(job, sig)
      │         │           dec = flow.retry_decision(job, r, retries_done, last_wait)
      │         │           リトライなら report: 知らせ(d.out.write_err。常に標準エラー) → w = d.proc.wait(secs, sig)
      │         │                    w が中断なら「起動しなかった中断」の JobExecution(started=False, interrupted=True, signal_number)を作って抜ける }
      │         ├─ flow.classify_and_decide(..., RunMode=Pipeline)
      │         ├─ report: 表示(-v のときの結果行・遷移行。常時のエラー行)
      │         ├─ state.apply_outcome → d.states.write(durable)
      │         ├─ エラーまたは分類なし → return flow.exit_code_for(...)
      │         ├─ 完了 → finish(--remove-on-complete なら d.states.reset)、return 0
      │         └─ sig に合図が立っている(sig.number() is not None) → flow.resume_point_on_interrupt → state.apply_interrupt → d.states.write(durable)、return 128 + 番号
 └─ ループを抜けても完了していない → "cannot determine whether the pipeline is complete"、return 65
```

- `run` は、`prepare`(ジョブIDの存在の検査を含む)のあと、指定した 1 ジョブについて同じ「判断 → 実行 → 分類 → 記録」を行う。分類と次の動きは `RunMode = Individual` で求める。
- `watch`・`reset`・`validate`・`convert` は、シグナルを準備しない。それぞれ、ステートの読み込み(`read_raw`)・ステートの削除(`reset`、`reset_all`)・定義の検査(`defs.load` と `proc.check_command`)・変換(`converter.convert`)を呼ぶ。
- `pipeline` は、次のジョブを「次の動き」の戻り値で決めない。記述順に全ジョブを見て、起動前の判断で決める(スキップ・再開は、ステートが運ぶ)。
- 合図の確認は、`cli` が `sig.number()` で行う(立っているか `is not None` で判定する。番号 0 を偽にしない。07 章 6.2 節)。`runner.run_job` と `runner.wait_retry` が、待機中の合図を扱う([07-process-and-signals.md](07-process-and-signals.md))。

### 5.3 責務の流れ(データ)

```text
定義ファイル ──definition.load──▶ Definition ─┐
ステートファイル ──state.read──▶ PipelineState ┤
                                              ▼
                       flow.decide_before_run ──▶ run / skip / reject
runner.run_job ──▶ JobExecution ──flow.classify_and_decide──▶ (JobOutcome, NextStep)
                                         │
        state.apply_*(結果・スキップ・中断) ──▶ 新しい PipelineState ──state.write──▶ ステートファイル
JobOutcome の報告する終了コード ──flow.exit_code_for──▶ conductor の終了コード
```

### 5.4 中断の伝わり方

- 中断の合図(`runner.Interrupt`)は、「立った/立っていない」と「最初のシグナル番号」を持つ。シグナルハンドラ(`signal.signal` で登録した関数)が `trigger` で立て、待ち(`run_job`、`wait_retry`)と `cli` の流れが読む。すべて同じスレッド(メインスレッド)で動く。
- ハンドラは、`trigger` を呼ぶだけ。待つ側は、短い `sleep` と `poll()` の繰り返しの中で、次の周に合図を読む(最大 10 ms〜50 ms 後)。詳細は [07-process-and-signals.md](07-process-and-signals.md) の 5〜7・9 章。

---

## 6. ファイル単位の責務表【目安】

実装者は、ファイルの分け方を変えてよい。ただし、パッケージの責務と依存の向き(2・3 章)と、OS 分岐の置き場所(`platform.py` だけ)は守る。

| パス | 責務 |
|------|------|
| `src/conductor/__main__.py` | 入口(`main`。内部の誤りの捕捉、`cli.run` の呼び出し、flush、終了コードを返す) |
| `src/conductor/version.py` | 版の定数 |
| `src/conductor/definition/__init__.py` | 公開する名前(`__all__`) |
| `src/conductor/definition/types.py` | 定義の型(`Definition` ほか) |
| `src/conductor/definition/errors.py` | 誤りの種類(`ErrorKind`、`ERROR_KINDS`)と `DefinitionError` |
| `src/conductor/definition/load.py` | `load`(場所の決定、読み取り、書式の判定、段階の組み立て) |
| `src/conductor/definition/intake.py` | ファイルの読み取り、大きさの上限、拡張子の判定 |
| `src/conductor/definition/tree.py`、`read_yaml.py`、`read_json.py` | 木の型、YAML/JSON の読み取り(位置つき。展開と上限を含む。`convert` と `state` が共有。03 章) |
| `src/conductor/definition/depth.py` | 入れ子の深さの検査 |
| `src/conductor/definition/walk.py` | 構造・型・遷移・終了コード・名前の検査 |
| `src/conductor/definition/loose.py` | `run_id_env` のゆるい読み取り |
| `src/conductor/convert/__init__.py`、`convert.py` | YAML ⇔ JSON の変換の入口(`convert_file`、`convert_text`) |
| `src/conductor/convert/write_yaml.py`、`write_json.py`、`errors.py` | YAML の書き出し、JSON の書き出し、誤り(`ConvertError`。読み取りは `definition.read_tree`) |
| `src/conductor/runid.py` | run ID の型、検査、決定 |
| `src/conductor/flow/types.py` | 実行時の値の型 |
| `src/conductor/flow/classify.py` | 分類 |
| `src/conductor/flow/decide.py` | 起動前の判断 |
| `src/conductor/flow/next_step.py` | 次の動き |
| `src/conductor/flow/resume.py` | 中断時の再開位置 |
| `src/conductor/flow/retry.py` | リトライの判断と待ち時間(`min(x, 2**64 - 1)` の飽和の計算) |
| `src/conductor/flow/exit_codes.py` | 終了コードの定数と範囲合わせ |
| `src/conductor/state/types.py`、`location.py` | ファイル用の型、場所と名前 |
| `src/conductor/state/record.py` | 記録関数(純粋) |
| `src/conductor/state/consistency.py` | 整合性の検査(純粋) |
| `src/conductor/state/reader.py` | 読み込み(14 段階の検査、異常系) |
| `src/conductor/state/writer.py` | 書き込み(一時ファイル + `fsync` + `os.replace`)、`reset` |
| `src/conductor/state/reset_all.py` | `reset_all` |
| `src/conductor/state/sync_dir.py` | ディレクトリの同期(`sync_directory` の真偽で動作を決める。`sys.platform` は読まない) |
| `src/conductor/runner/split.py` | 分割と検査 |
| `src/conductor/runner/run_job.py` | ジョブの起動、待ち(待機ループ)、結果の取り出し、転送の判断 |
| `src/conductor/runner/wait_retry.py` | リトライの待機 |
| `src/conductor/runner/interrupt.py` | 中断の合図 |
| `src/conductor/runner/watcher.py` | シグナル監視(登録は 1 回、ハンドラは `trigger` だけ、復元) |
| `src/conductor/runner/platform.py` | OS 別の部分(`detect_platform`、`platform_for`、`exit_by_interrupt`。`sys.platform` を読む唯一の場所) |
| `src/conductor/report/locale.py` | ロケールと色の判定(遅延) |
| `src/conductor/report/catalog.py` | メッセージカタログ(英語・日本語) |
| `src/conductor/report/format.py` | 行の組み立て(`-v`、誤り) |
| `src/conductor/report/escape.py` | エスケープと切り詰め |
| `src/conductor/report/watch.py` | `watch` の表 |
| `src/conductor/report/usage.py` | 使い方 |
| `src/conductor/cli/args.py` | 引数の字句の解析と検査 |
| `src/conductor/cli/deps.py`、`system.py` | `Deps` の `Protocol` と本番の実装 |
| `src/conductor/cli/app.py` | `run`(入口の分岐、終了コード) |
| `src/conductor/cli/prepare.py` | 前処理(`run`・`pipeline` に共通) |
| `src/conductor/cli/run_cmd.py`、`pipeline_cmd.py` | `run`・`pipeline` の流れ |
| `src/conductor/cli/exec_job.py` | リトライ込みの 1 ジョブの実行、記録、表示 |
| `src/conductor/cli/aux_cmd.py` | `watch`・`reset`・`validate`・`convert` |
| `tests/support/*.py` | 4.4 節の偽の実装と補助 |
| `tests/test_architecture.py` | 依存の向き・循環・OS 分岐の置き場所・禁止の import の点検(3 章、07 章 8 章) |
| `tests/acceptance/test_*.py` | 実際に `python -m conductor` を起動する受け入れテスト |
| `tests/jobs/*` | テストジョブ |

単体テストは、`tests/` の下に、`src/conductor/` と同じ構成で置く(例: `tests/runner/test_split.py`)。受け入れテストの置き場所と実行の方法は [11-quality-and-build.md](11-quality-and-build.md)。

---

## 7. 同期モデルの方針【規定】

### 7.1 基本

| 項目 | 方針 |
|------|------|
| 基本 | **同期(スレッドなし)**。関数は、普通の関数(`def`)。`async def`、`await`、`asyncio` を使わない |
| スレッド | メインスレッドだけで動く。データ競合は無い。`threading`、`concurrent.futures`、`multiprocessing`、`os.fork` を使わない(製品のコード)。理由: (1) `signal.signal` は、メインスレッドからだけ呼べる【07 章で実測】。(2) 子の回収を 1 か所(メインスレッドの `poll()`)にして、`pid` の再利用の競合を避ける。(3) シグナルハンドラとスレッドの併用は、ハンドラが実行されるスレッドの規則が増える |
| 並行の実行 | 流れの制御は、仕事を並行に走らせない。ジョブ、出力、ステートの書き込みは、すべて直列に呼ぶ |
| 子プロセス | `subprocess.Popen`。待機は、`poll()` と短い `sleep` の繰り返し(07 章 5 章)。`subprocess.run`、`Popen.wait()`(引数なし)、`Popen.communicate` を使わない |
| シグナル | メインスレッドで `signal.signal` のハンドラを登録する。ハンドラは、`Interrupt.trigger` を呼ぶだけ(フラグを立てるだけ)。複雑なことをしない |
| 待ち方 | 待機ループ: 10 ms 以下の `time.sleep`。リトライの待機: 50 ms 以下の `time.sleep` の繰り返しと、単調な時計(`time.monotonic_ns`)。ポーリングの理由: ハンドラがフラグを立てても、PEP 475 により、`time.sleep` と `Popen.wait()` は残りの時間を待ち続け、戻らないため(07 章 5.2 節の実測)。刻んで、次の周でフラグを読む |
| PEP 475(EINTR の自動の再試行) | `time.sleep`、`os.read`、`os.write`、`os.replace`、`os.fsync` などは、シグナルで中断されたとき、ハンドラが例外を送出しなければ、自動で再試行される。自前の再試行を書かない。ハンドラが例外を送出しない規則が前提 |
| 共有する可変の値 | `runner.Interrupt` と、シグナル登録の印(`watcher.py` のモジュール内の変数)だけ |
| キャンセル | 例外による中断(`KeyboardInterrupt` をハンドラから送出する、など)を使わない。中断は `Interrupt` で伝える |
| 出力 | 標準出力・標準エラーへの書き込みは、`Sink` を通し、書くたびに `flush()` する(07 章 9.2 節) |
| 排他制御 | 同じ定義への 2 つの `conductor` の同時の実行を、排他しない(両方が動き、ステートは、どちらも壊れた形では読めない。原子的な置換による) |
| 再帰 | 深い入れ子の入力(定義・ステート・変換)に対して、再帰で読まない(`sys.getrecursionlimit()` の既定は 1000 で、深さの上限(32)より大きいが、解析器の内部の再帰を含めると足りない場合がある)。深さの上限の検査を、解析より前に効かせる、または明示のスタックで読む(03 章)。`sys.setrecursionlimit` を呼ばない |

### 7.2 スレッドの使用表(使わないのが原則)

製品のコードでスレッドを使う箇所は、ない。将来、必要になった場合は、次を表にして、この章を改める。

| 項目 | 書く内容 |
|------|----------|
| スレッドの必要の理由 | なぜ、メインスレッドの `poll()` + `sleep` で足りないのか |
| 所有者 | どのモジュールが作り、止めるか |
| 終了条件 | いつ、どう終わるか(`daemon` に頼らない) |
| 共有する値 | ロックの有無 |
| シグナルとの関係 | `signal.signal` の呼び出しは、メインスレッドから |

### 7.3 イベント源の一覧(誰が所有し、どう終わるか)【規定】

07 章 9.3 節の表を、正本とする。要点: シグナルハンドラ(`start_watcher` が所有。`Interrupt.trigger` だけを書く)、子プロセス(`run_job` が所有。`poll()` が値を返すまで)、待ち(`run_job` の待機ループと `wait_retry`。経過か合図まで)。

---

## 8. `pyproject.toml` の規定

### 8.1 ビルドバックエンドの選定

| 候補 | 評価 |
|------|------|
| `hatchling` | `src/` レイアウトを自動で扱う。wheel と sdist の内容を、設定で細かく指定できる。依存が少ない。PEP 517/621/639 に対応する(実装時に確認)。**採用案** |
| `setuptools` | 最も広く使われている。設定が多く、`src/` レイアウトと、`tests/` の除外に、追加の設定が要る |
| `flit-core` | 設定が最小。sdist の内容の指定に制限がある(未確認) |
| `pdm-backend`、`uv_build` など | 実装時に確認 |

基準: PEP 621 の `[project]` を読む、`src/` レイアウト、wheel から `tests/` を除ける、依存が少ない、保守が続いている。選定は、実装時に確認する(版を決めて断定しない)。この文書は、`hatchling` を例に書く。

### 8.2 `pyproject.toml` の骨格

次の項目を満たす(`<...>` は実装時に確認する値)。

```text
[build-system]
requires = ["hatchling>=<版>"]
build-backend = "hatchling.build"

[project]
name = "conductor"
version = "0.8.0"
description = "<1 行の説明>"
readme = "README.md"
requires-python = ">=<最小の版>"          # 例: ">=3.11"(実装時に確認)
license = "<SPDX の式>"                   # PEP 639。実装時に確認
license-files = ["LICENSE"]
dependencies = ["<YAML のライブラリ>>=<版>,<<次の大きな版>"]
classifiers = [ ... ]                     # 任意

[project.scripts]
conductor = "conductor.__main__:main"

[dependency-groups]                       # PEP 735。使えない環境では [project.optional-dependencies] の dev に置く(実装時に確認)
dev = ["pytest>=<版>", "pytest-cov>=<版>", "mypy>=<版>", "ruff>=<版>", "build>=<版>", "pip-audit>=<版>"]

[tool.hatch.build.targets.wheel]
packages = ["src/conductor"]

[tool.hatch.build.targets.sdist]
include = ["/src", "/tests", "/README.md", "/LICENSE", "/pyproject.toml"]

[tool.ruff]
line-length = 100
target-version = "py<最小の版の数字>"     # 例: "py311"
src = ["src", "tests"]

[tool.ruff.lint]
select = ["E", "F", "W", "I", "B", "UP", "SIM", "C4", "RUF", "S", "T20", "BLE", "PLC0415", "TID", "A", "PTH", "N"]
# 例外: tests/ では "S101"(assert)、"S603"、"S607"(subprocess)、"T20" を許す(per-file-ignores)

[tool.ruff.lint.flake8-tidy-imports.banned-api]
"asyncio".msg = "同期モデル(02 章 7 章)に反する"
"argparse".msg = "引数の解析は自前(04 章)"
"threading".msg = "スレッドを使わない(02 章 7 章)"
"multiprocessing".msg = "使わない"
"concurrent".msg = "使わない"
"shlex".msg = "command の分割は自前(07 章 3 章)"
"pickle".msg = "使わない"

[tool.ruff.format]
# 既定のまま(引用符は二重引用符、行末は LF)

[tool.mypy]
python_version = "<最小の版>"
strict = true
files = ["src", "tests"]
warn_unreachable = true
enable_error_code = ["ignore-without-code", "redundant-expr", "truthy-bool"]

[tool.pytest.ini_options]
minversion = "<版>"
testpaths = ["tests"]
addopts = "-ra --strict-markers --strict-config"
filterwarnings = ["error"]
xfail_strict = true
markers = [
  "acceptance: 実際の起動を伴う受け入れテスト",
  "posix_only: UNIX 系だけで動くテスト",
  "windows_only: Windows だけで動くテスト",
  "slow: 時間のかかるテスト(計測、長い待ち)",
]

[tool.coverage.run]
source = ["conductor"]

[tool.coverage.report]
fail_under = 80
show_missing = true
skip_empty = true
```

### 8.3 各項目の規定

| 項目 | 規定 |
|------|------|
| `[build-system]` | PEP 517 のバックエンドを指定する。`requires` の版は、実装時に確認。ビルドは、PyPA の `build`(`python -m build`)で、sdist と wheel を作る(11 章) |
| `[project]` の `name`、`version` | `name` は `conductor`。`version` は `0.8.0`。`src/conductor/version.py` の `VERSION` と一致する(テストで検査する)【規定】。`version` を動的(`dynamic`)にしない(2 か所の一致を、テストで検査するため) |
| `requires-python` | 最小版を `>=X.Y` で書く(実装時に確認)。CI の Python の版(最小版と最新版)と一致することを、検査する(11 章)。`mypy` の `python_version` と、ruff の `target-version` も、最小版に合わせる |
| `dependencies` | YAML のライブラリ 1 つだけ。それ以外を足さない【規定】。版の範囲は、下限と、次の大きな版の未満(実装時に確認) |
| 開発時の依存 | 1 章の開発時の依存。実行時の依存を増やさない。版の固定(ロックファイル)の方針は、実装時に確認([11-quality-and-build.md](11-quality-and-build.md)) |
| `[project.scripts]` | `conductor = "conductor.__main__:main"`。`main` は、終了コード(`int`)を返す関数。コンソールスクリプトの起動用のコードが、その戻り値で `sys.exit` を呼ぶ(Windows では、`pip` が `conductor.exe` のラッパーを作る。動作は、実装時に 3 OS で確かめる。未確認) |
| `python -m conductor` | `src/conductor/__main__.py` の `if __name__ == "__main__": sys.exit(main())` で動く |
| wheel の中身 | `src/conductor/` だけ(`tests/`、設定ファイルを含めない)。`python -m build` のあと、wheel と sdist の内容を点検する(11 章) |
| sdist の中身 | `src/`、`tests/`(受け入れテストの再現用)、`README.md`、`LICENSE`、`pyproject.toml`。不要なファイル(`.git`、キャッシュ)を含めない |
| `[tool.ruff]` | 行の長さ、`target-version`、`src` を指定する。`ruff check` と `ruff format --check` が、品質ゲート(11 章)。許す例外は、`per-file-ignores` に、理由つきで書く |
| `[tool.ruff.lint]` の `select` | 上の骨格の分類を最低限とする(実装時に調整してよい。緩める場合は理由を書く)。`T20`(`print` の禁止): 製品のコードは、`Sink` を通してだけ出力する。`BLE`(広い `except` の禁止): 入口の 1 か所(`except BaseException`)だけ、理由つきの `# noqa: BLE001` を許す。`S`(セキュリティ): `shell=True` の禁止(`S602` など)を含む |
| `banned-api` | 製品のコードの禁止の import。テストでは、`threading`(07 章 11 章の、メインスレッド以外からの呼び出しの検査)など、必要なものを `per-file-ignores` で許す |
| `[tool.mypy]` | `strict = true`。`src` と `tests` の両方を対象にする(偽実装が `Protocol` を満たすことを、型検査で確かめるため) |
| `[tool.pytest.ini_options]` | `testpaths`、`--strict-markers`、`filterwarnings = ["error"]`(警告を失敗にする。`ResourceWarning` を含む)。`posix_only`・`windows_only` は、`sys.platform` を読む補助(`tests/support/` の中)で、`skipif` を付ける(`tests/` は OS 分岐の点検の対象外) |
| `[tool.coverage]` | 行カバレッジの下限は 80%。Linux で計測する(`pytest --cov=conductor --cov-fail-under=80`)。`[tool.coverage.report]` の `fail_under` と、コマンドラインの値を、同じにする。ブランチの計測は必須としない |
| 版の一致 | `pyproject.toml` の `version`、`src/conductor/version.py` の `VERSION`、リリースのタグ(`v0.8.0`)が一致する(11 章) |

---

## 9. エラー処理の方針【規定】

| 項目 | 方針 |
|------|------|
| 基本 | 失敗は、戻り値(判別可能な和の値。`ok: Literal[True]` と `ok: Literal[False]` を持つ `dataclass` の `\|` の和、または `T \| None`)で返す。外部の入力(引数・環境・ファイル・OS の結果)に対して、例外を送出しない |
| 例外を許す場所 | (1) 実装の不具合(論理の矛盾。起こるはずのない状態)。入口(`__main__`)の捕捉が、固定の 1 行を標準エラーに出して終了コード 70 にする(07 章 10 章)。`cli` の流れの内側で `except Exception` して握りつぶさない。(2) `state` の内側(`StateError` を送出し、`StateStore` が値に変える)。(3) 標準ライブラリが例外を送出する場合は、その場で `try/except` して、値にする(例: `Popen` の `OSError`・`ValueError`、ファイル入出力の `OSError`、`UnicodeDecodeError`) |
| 捕まえる例外の範囲 | 想定する型だけを、狭く捕まえる(`except OSError`、`except (OSError, ValueError)`)。裸の `except:` と、`except Exception:` / `except BaseException:` を、入口の 1 か所以外に書かない。`try` の中身は、失敗しうる 1 つの呼び出しに絞る |
| 終了 | `sys.exit` は `__main__.py` の 1 か所だけ(と、07 章の内部の誤りの経路 `os._exit(70)`、`KeyboardInterrupt` の経路 `exit_by_interrupt`)。`cli.run` は、終了コードを返す。`SystemExit` を、流れの内側で送出しない |
| 種類 | パッケージごとに、**種類(`Literal` の文字列)を持つエラー型**を公開する: `DefinitionError`(値)、`CommandError`(`Exception`)、`StateError`(`Exception`)、変換の誤り(`ConvertError`。`Exception`)、`RunIdError`(値)。例外型は、`Exception` を継承し、`kind` と、固定の文(メッセージ)を持つ。呼び出し側は、`kind` で `match` し、網羅性を型で検査する(`assert_never`)。文字列を比べて分岐しない |
| 表示 | 利用者への表示の文は、`report` が、種類と引数(ジョブID、パスなど)から組み立てる(メッセージカタログ)。**`str(e)` や `e.args` の文字列、ライブラリ(YAML、標準ライブラリ)のエラー文、`errno` の名前を、そのまま表示しない**(入力の値や、内部の情報が出るため。[10-errors-and-exit-codes.md](10-errors-and-exit-codes.md))。スタックトレースを出さない(`traceback` を使わない) |
| 原因 | `raise X(...) from e` で原因をつなげてよい。ただし、表示に使わない。デバッグ用に残してもよいが、利用者への出力に出さない |
| 複数の誤り | 定義の検査・`command` の検査は、誤りを**すべて集めて**返す(`tuple`)。1 件で止めない |
| 値で返す失敗 | ジョブの起動の失敗(`JobExecution.start_failure`)、出力の書き込みの失敗(`Sink` の `False`)は、値として扱い、終了コードに結びつける |
| 終了コードの決定 | 「失敗の種類 → 終了コード」の対応は `cli` の 1 か所で行う([10-errors-and-exit-codes.md](10-errors-and-exit-codes.md)) |
| 想定外 | 入口で捕まえ、固定の英語 1 行 `conductor: error: internal error` と終了コード 70(`SystemExit` を除く。`KeyboardInterrupt` は 07 章 7.6 節)。07 章 10 章 |
| `assert` | 外部の入力の検査に使わない(`python -O` で消える)。起こらないはずの状態の表明に限り、使ってよい(`assert_never` を含む)。その失敗は、実装の不具合として、入口の捕捉で 70 になる |
| 整数の検査 | 外部の入力の整数は、`type(x) is int` で検査する(`bool` は `int` の下位型で、`isinstance(True, int)` が真になる。`float` の `1.0`、`nan`、`inf` も整数でない)。詳細は 03・06 章 |
| 静的検査 | `mypy --strict`、ruff の `BLE`・`T20`・`S` で確かめる(8 章)。意図して例外を握りつぶす場合(シグナルの送信の失敗など)は、`contextlib.suppress(OSError)` か、`except OSError:` に理由の注釈を付ける |

---

## 10. 設計判断のチェックリスト(確認)と未確認の事項

### 10.1 設計判断

| # | 判断 | この版の決定 |
|---|------|--------------|
| 1 | パッケージ構成 | 2 章のとおり固定 |
| 2 | 実行時の値の型の置き場所 | `conductor.flow`(`state` と `runner` は、`flow` に依存する。`flow` は `state` と `runner` に依存しない) |
| 3 | ステートの書式 | JSON(`PipelineState` を直接 `json.dumps` せず、キーの順を固定したファイル用の型を書き出す。読み込みは自前の `read_json`。`json.loads` は製品のコードで使わない)。[06-state-file.md](06-state-file.md) |
| 4 | 差し替え口 | `Deps`(`dataclass`)と各 `Protocol`。4 章 |
| 5 | シグナル | `signal.signal`。ハンドラは 1 組。合図は `runner.Interrupt` |
| 6 | 待機 | `Popen.poll()` + 短い `time.sleep`(ポーリング)。スレッドなし、`asyncio` なし |
| 7 | OS 別 | `runner/platform.py` の 1 か所。`PlatformKind` を、必要な部品に引数で渡す |
| 8 | エラー | 種類を持つエラー型、`kind` の `match`、`report` が文を作る |
| 9 | 終了 | `sys.exit` は入口の 1 か所 |
| 10 | 想定外の失敗 | `sys.excepthook` と `main` の `try/except` で 70 |
| 11 | 整数 | 任意精度の `int`。64 ビット符号なしの飽和は `min(x, 2**64 - 1)`。終了コードの範囲検査は字句から。`bool` を整数にしない |
| 12 | ファイル入出力 | 同期の標準ライブラリ(`open`、`os.replace`、`os.fsync`) |
| 13 | import | 相手のパッケージの `__init__.py` を通す。`from __future__ import annotations` |
| 14 | 依存の検査 | `tests/test_architecture.py`(`ast`)と、ruff の `banned-api` |
| 15 | ビルド | `hatchling`(採用案)、`python -m build` |

### 10.2 未確認の事項

- サポートする Python の版(最小と最新の具体的な番号)。実装時に確認する。この文書の作成時点では、最小が 3.11、最新が 3.15 と思われる(未確認)。
- YAML のライブラリの選定(03・09 章)。実装時に確認する。
- ビルドバックエンドの選定(`hatchling` が採用案。版と、PEP 639 の `license` の書き方、`[dependency-groups]`(PEP 735)を `pip` と `build` の環境が扱えるか)。実装時に確認する。
- `ruff` の規則の集合(8.2 節の `select`)が、製品のコードに対して、不自然な書き換えを強いないか。調整してよい(`strict` の `mypy` は緩めない)。
- `main()` の戻り値を `sys.exit` に渡す形が、`pip` が作る `conductor`(UNIX 系のスクリプト、Windows の `conductor.exe`)の 3 OS で、終了コードを正しく返すこと。実装時に確かめる。
- `python -X importtime` による、`conductor --version` と `--help` の起動時間(YAML のライブラリを読み込まないこと。11 章の計測)。
- 端末の判定(`sys.stdout.isatty()`)が、UNIX 系と Windows の両方で、利用者が期待する結果になるか。実装時に 3 OS で確かめる。
- `tests/support/` の偽実装が、`cli/deps.py` の `Protocol` を満たすことを、`mypy --strict` が問題なく通せること(`Protocol` のメソッドの引数の名前と、構造的な型付けの食い違いが出ないか)。実装時に確かめる。
- `ast` による依存の検査(`tests/test_architecture.py`)が、`if TYPE_CHECKING:` の内側の import を、正しく扱えること。実装時に確かめる。
- 07 章の【実測】は、macOS の Python 3.14.10、3.15.2、3.9.6 で得た。Linux と Windows の挙動は未確認(07 章 13 章)。
