// Copyright 2026 agwlvssainokuni
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! conductor-core: conductor の中核ライブラリ(Unit conductor-core)。
//!
//! 定義ファイル(`conductor.toml`)の読み込みと検証(DefinitionLoader)と、
//! ジョブの結果の分類・次の動き・完了・再開位置の判断(Flow)を受け持つ。
//! ジョブの起動(conductor-runtime)、ステートの読み書き(conductor-runtime)、コマンドと表示(conductor-cli)は、
//! このクレートの外にある。
//!
//! # 公開する関数(契約 C2)
//!
//! | 関数 | 役割 | 入出力 | 失敗 |
//! |------|------|--------|------|
//! | [`load_definition`] | 定義ファイルを読み、検証する(W1) | ファイル → [`PipelineDefinition`] | [`DefinitionError`] の並び(複数) |
//! | [`parse_definition`] | 文字列の定義を検証する(ファイルに触れない入口) | 文字列 → [`PipelineDefinition`] | 同上 |
//! | [`decide_from_state`] | ジョブを起動する前の判断(W2、BR3.5) | 引数 → [`Decision`] | なし(純粋) |
//! | [`classify_and_decide`] | 実行後の分類と次の動き(W3、BR2.x、BR3.x) | 引数 → ([`JobOutcome`], [`NextStep`]) | なし(純粋) |
//! | [`resume_point_on_interrupt`] | ジョブのあいだの中断の再開位置(W4、BR3.7) | 引数 → [`ResumePoint`] | なし(純粋) |
//! | [`initial_state`] | 空のステートを、定義から作る(BR2.8) | 定義、パス、run ID → [`PipelineState`] | なし(純粋) |
//! | [`apply_outcome`] | ジョブの結果を記録する(BR2.8) | ステート、結果、次の動き → ステート | なし(純粋) |
//! | [`apply_skip`] | スキップを記録し、完了を判定する(BR2.8) | ステート、理由、定義 → ステート | なし(純粋) |
//! | [`apply_interrupt`] | 中断を記録する(BR2.8) | ステート、[`ResumePoint`] → ステート | なし(純粋) |
//! | [`check_consistency`] | ステートと定義の食い違いを検知する(BR2.5) | ステート、定義 → [`ConsistencyResult`] | なし(純粋) |
//!
//! ステートの更新の関数(`initial_state`、`apply_*`)は、本番の流れと `--dry-run` の模擬が共に使う。
//! `conductor-runtime` は、これらと `check_consistency` を再公開しない(使う側は、この crate から使う)。
//! ほかの公開の関数(`.env` の解析と統合、図、計画、run ID など)は、各モジュールにある。
//!
//! 入出力に触れるのは、[`definition::intake`] だけである。それ以外は、引数だけから結果を決める
//! 純粋な処理で、ファイルにもプロセスにも環境変数にも触れない。
//!
//! # モジュールの構成(logical-components との対応)
//!
//! | モジュール | 部品 | 責務 |
//! |------------|------|------|
//! | [`types`] | 共有する型(Types) | データの形だけ(契約 C1) |
//! | [`error`] | 表示用の組み立て(Describe)、位置の変換(Locate) | 固定の文だけでエラーを表示する。バイトの位置から行番号を求める |
//! | [`definition::intake`] | 取り込み(Intake) | 場所の決定、大きさの確認、上限 + 1 バイトまでの読み取り |
//! | `definition::parse` | 読み取り(Parse) | 位置を保つ文書の木にする。文法の誤りは `syntax` |
//! | `definition::depth` | 深さの検査(Depth) | 入れ子 32 段の上限 |
//! | `definition::version` | 版の確認(VersionCheck) | `format_version` が 1 であること。通らなければ、そこで終える |
//! | `definition::walk` | 構造の検査(Walk) | キー、型、構造の制約、遷移、終了コード、名前。誤りはすべて集める |
//! | [`flow`] | 判断(Flow) | 起動前の判断、分類、次の動き、完了、再開位置 |
//! | `flow::record` | 記録(Recorder) | `initial_state`、`apply_*`(純粋。本番と模擬が共に使う) |
//! | [`consistency`] | 照合(Consistency) | ステートと定義の、パイプライン名、ジョブIDの集合、並び(純粋) |
//! | [`envfile`] | `.env` の解析と層の統合 | `.env` の文字列の解析と、環境変数の層の統合(純粋) |
//! | [`graph`] | 図の構造と配置 | `graph` の図の節・辺と配置(純粋) |
//! | [`plan`] | `--dry-run` の計画 | 実行の模擬と計画(純粋) |
//! | [`run_id`] | run ID の型と規則 | run ID の検査と、環境変数の名前の規則 |
//! | [`time`] | 時刻(CoreTime) | UTC の時刻の整形・読み取り・差(純粋。現在の時刻は runtime が取る) |
//!
//! 定義の読み込みの流れは Intake → Parse → Depth → VersionCheck → Walk の順である。
//!
//! # 呼び出しの順序(Cli から見て)
//!
//! ジョブごとに、[`decide_from_state`] → (`Run` のとき)ジョブの起動 → [`classify_and_decide`] →
//! ステートの書き込み。`Skip` のときは、ジョブを起動せず、必要ならスキップをステートに記録して、
//! 終了コード 0 で終わる。完了と再開位置は [`apply_skip`] が書く(最後のジョブをスキップとして
//! 記録し、`pending` が無ければ `completed = true`)。[`resume_point_on_interrupt`] は、`pipeline` の
//! ジョブのあいだの中断にだけ使う。`Reject` のときは、ジョブを実行せず、
//! ステートも変えず、conductor 自身のエラー(終了コード 66。BR5.1)として扱う。
//!
//! # 守る性質
//!
//! - 外部の入力(定義ファイル)に対して、パニックしない。失敗は、戻り値のエラーで返す(NFR4.7)。
//! - エラーの表示に、入力の値、ライブラリのエラー文、ソースの抜粋を含めない。キー名は、
//!   制御文字をエスケープし、64 文字までに切る(NFR4.6)。
//! - 定義に書かれたコマンド([`Command`]。文字列または文字列の配列)は、データとして扱い、
//!   解釈も起動もしない(NFR4.5)。
//! - `#![forbid(unsafe_code)]`。

#![forbid(unsafe_code)]

pub mod consistency;
pub mod definition;
pub mod envfile;
pub mod error;
pub mod flow;
pub mod graph;
pub mod plan;
pub mod run_id;
#[cfg(test)]
mod test_support;
pub mod time;
pub mod types;

pub use consistency::{ConsistencyResult, MismatchKind, check_consistency};
pub use definition::{load_definition, parse_definition};
pub use envfile::{
    EffectiveEntry, EffectiveEnv, EnvContext, EnvEntry, EnvFileContent, EnvFileError,
    EnvFileErrorKind, EnvWarning, EnvWarningKind, LoadedEnvFiles, env_file_refs, merge_env,
    parse_env_file, summarize_errors,
};
pub use error::{DefinitionError, DefinitionErrorKind, MAX_KEY_DISPLAY_CHARS, escape_display};
pub use flow::{
    RetryDecision, apply_interrupt, apply_outcome, apply_skip, classify_and_decide,
    decide_from_state, initial_state, resume_point_on_interrupt, retry_decision,
};
pub use plan::{
    Assumption, EnvNameEntry, ExecutionPlan, NextStepNote, PlanDisposition, PlanEnd, PlanEntry,
    PlanInput, PlanMode, PlanRejection, ResumeInfo, SimulatedResult, plan_pipeline, plan_run,
};
pub use run_id::{DEFAULT_RUN_ID_ENV, RunId, RunIdError, peek_run_id_env};
pub use time::{Timestamp, duration_secs, format_compact, format_rfc3339, parse_rfc3339};
pub use types::{
    Backoff, Classification, Command, Decision, EnvFileRef, EnvVar, ExecutionResult, JobDefinition,
    JobExecution, JobOutcome, JobState, JobStatus, NextStep, PipelineDefinition, PipelineState,
    RejectReason, ResumePoint, RetryPolicy, RunMode, SkipCause, SkipReason, Transition,
    TransitionOn, TransitionTarget,
};
