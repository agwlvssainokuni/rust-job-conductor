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

//! 実行計画(`--dry-run`。U3。BR1.1〜BR1.8、BR2.1〜BR2.4、BR4.3、BR5.1)。
//!
//! 計画の型(この `mod.rs`)と、計画の生成([`simulate`])を持つ。計画の生成は純粋な関数で、
//! ファイルにも、プロセスにも、環境変数にも触れない。実行と同じ判断(`decide_before_run`、
//! `classify_and_decide`)を、実行の代わりに、仮定した終了コードで進める(BR1.2)。
//!
//! 環境変数の値は、この module のどの型にも持たない。持つのは、名前と出どころだけ(BR5.1、FR1.12)。

pub mod simulate;

use crate::envfile::EnvOrigin;
use crate::types::{Classification, DecisionReason, RetryPolicy};

pub use simulate::{PlanInput, plan_pipeline, plan_run};

/// 仮定する終了コードの最小(プロセスの終了コードの範囲。BR2.1)。
pub const ASSUME_EXIT_CODE_MIN: i32 = 0;
/// 仮定する終了コードの最大(BR2.1)。
pub const ASSUME_EXIT_CODE_MAX: i32 = 255;

/// `--assume <ジョブID>=<終了コード>` の 1 つ分(BR2.1)。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Assumption {
    /// 定義にあるジョブの ID。
    pub job_id: String,
    /// 仮定する終了コード(0〜255)。
    pub exit_code: i32,
}

impl Assumption {
    pub fn new(job_id: impl Into<String>, exit_code: i32) -> Self {
        Self {
            job_id: job_id.into(),
            exit_code,
        }
    }

    /// 終了コードが、仮定できる範囲(0〜255)に入っているか(BR2.1)。
    pub fn exit_code_in_range(exit_code: i32) -> bool {
        (ASSUME_EXIT_CODE_MIN..=ASSUME_EXIT_CODE_MAX).contains(&exit_code)
    }
}

/// 計画の中のジョブの扱い(BR1.3)。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlanDisposition {
    /// 実行する。
    WillRun,
    /// 実行済み(成功・警告で記録済み)。
    AlreadyExecuted,
    /// 遷移でスキップ。
    SkippedByTransition,
    /// `@end` のあとでスキップ。
    SkippedAfterEnd,
    /// 模擬で、エラーで停止したあとのため、実行されない。
    NotReached,
}

/// 模擬での、ジョブの結果(`WillRun` のジョブだけ。BR1.7)。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SimulatedResult {
    /// 仮定した(または既定の)終了コード。
    pub exit_code: i32,
    /// 分類(既存の分類の結果)。
    pub classification: Classification,
    /// `--assume` で指定したか(`false` は、既定の成功)。
    pub assumed: bool,
    /// 終了コードが、リトライの対象に含まれるか(注記のため。模擬ではリトライを繰り返さない)。
    pub retryable: bool,
}

/// 環境変数の名前と出どころ(U1 の名前の要約の 1 項目。値は持たない。BR5.1)。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnvNameEntry {
    pub name: String,
    pub origin: EnvOrigin,
}

/// 計画の 1 つのジョブ(契約 C7 の job_entry)。
///
/// `WillRun` でないジョブにも、コマンドと環境変数の名前とリトライの設定を持たせる
/// (表示では、`WillRun` のジョブだけに出す。BR3.3)。環境変数の値は持たない(BR5.1)。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlanEntry {
    pub job_id: String,
    pub disposition: PlanDisposition,
    /// 再開位置のジョブか。
    pub is_resume: bool,
    /// 分割後のコマンド(先頭がプログラム)。
    pub command_words: Vec<String>,
    /// 渡される環境変数の名前と出どころ(試行 1)。
    pub env_names: Vec<EnvNameEntry>,
    pub retry: Option<RetryPolicy>,
    /// `WillRun` のときだけ。
    pub result: Option<SimulatedResult>,
}

/// 計画の方式。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlanMode {
    /// `pipeline <name> --dry-run`。
    Pipeline,
    /// `run <job> --dry-run`。
    Individual,
}

/// 再開位置の種類(BR1.4)。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResumeKind {
    /// ステートが無い(完了したステートを含む)。最初から。
    FromStart,
    /// 前回の停止で、再開位置がある。
    AfterStop,
    /// `run` の対象のジョブ。
    Individual,
}

/// 再開位置(または `run` の対象)。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResumeInfo {
    pub kind: ResumeKind,
    /// `AfterStop` と `Individual` のとき。
    pub job_id: Option<String>,
}

/// 模擬の最後(BR1.3)。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlanEnd {
    /// 最後のジョブまで終わる。
    Completed,
    /// `@end` に到達して終わる。
    EndReached,
    /// エラーで止まる(止まったジョブの ID)。
    StoppedAtError(String),
}

/// `run <job> --dry-run` の、仮定の結果による次の動き(BR1.6)。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NextStepNote {
    pub next_job_id: Option<String>,
    pub skipped_job_ids: Vec<String>,
    pub completed: bool,
    /// `@end` に到達したとき(後ろに未処理のジョブがあれば、呼ばれたときにスキップされる)。
    pub end_reached: bool,
    /// エラーで停止するとき。
    pub stopped: bool,
}

/// 実行計画の全体(契約 C7 の出力)。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutionPlan {
    pub pipeline_name: String,
    pub run_id: Option<String>,
    pub mode: PlanMode,
    pub resume: ResumeInfo,
    /// 指定された仮定(指定順)。
    pub assumptions: Vec<Assumption>,
    /// 模擬で使われなかった仮定のジョブ ID(指定順。BR2.3)。
    pub unused_assumptions: Vec<String>,
    /// `pipeline` は、定義の記述順のすべてのジョブ。`run` は、対象の 1 ジョブ。
    pub entries: Vec<PlanEntry>,
    /// 模擬の最後。`run` のときは参考の値(表示しない)。
    pub end: PlanEnd,
    /// `run` のときだけ(BR1.6)。対象が `WillRun` のときのみ。
    pub next_step_note: Option<NextStepNote>,
}

/// 計画を作れない拒否(本番なら順序外として拒否されるとき。BR4.3)。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlanRejection {
    /// 拒否されたジョブの ID。
    pub job_id: String,
    pub reason: DecisionReason,
}
