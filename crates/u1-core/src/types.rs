//! 共有する型(Types)。契約 C1 のデータの形だけを持ち、入出力と OS に依存しない。
//!
//! 型の定義は、`entities.md` に従う。値の生成を受け持つのは、このクレートでは
//! 定義(`PipelineDefinition` など)と判断(`Decision`、`JobOutcome`、`NextStep`)で、
//! `JobExecution`、`JobState`、`PipelineState` は u2-runtime が生成・保持する。
//!
//! 終了コードは、プロセスの終了コードに合わせて `i32` で持つ。定義ファイルに書かれた
//! `i32` の範囲を超える整数は、読み込み時に `typeMismatch` になる。

/// 定義ファイル 1 つに書かれたパイプラインの定義(契約 C1)。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PipelineDefinition {
    /// パイプラインの名前(空でない)。
    pub name: String,
    /// 説明(省略可)。
    pub description: Option<String>,
    /// 書式の版(初版は 1)。
    pub format_version: u32,
    /// ジョブ。定義ファイルに書いた順を保つ(BR1.11)。1 つ以上。
    pub jobs: Vec<JobDefinition>,
}

impl PipelineDefinition {
    /// ジョブIDから、記述順の位置(0 始まり)を求める。
    pub fn job_index(&self, job_id: &str) -> Option<usize> {
        self.jobs.iter().position(|j| j.id == job_id)
    }
}

/// 1 つのジョブの定義(契約 C1)。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JobDefinition {
    /// ジョブID(空でなく、定義の中で一意。`@` で始めない)。
    pub id: String,
    /// コマンド(空でない)。コマンド名と引数に分けて直接起動する書式。ここでは解釈しない。
    pub command: String,
    /// 警告にする終了コード。`success_exit_codes` と重ならず、0 を含まない。
    pub warning_exit_codes: Vec<i32>,
    /// 成功にする終了コード。省略時は `None`(`[0]` として扱う)。書いたら、その値だけが成功。
    pub success_exit_codes: Option<Vec<i32>>,
    /// 結果ごとの遷移(`on` ごとに高々 1 つ)。
    pub transitions: Vec<Transition>,
}

impl JobDefinition {
    /// 実際に成功として扱う終了コード。省略時は `[0]`(BR1.11、BR2.1)。
    pub fn effective_success_codes(&self) -> &[i32] {
        match &self.success_exit_codes {
            Some(codes) => codes,
            None => &[0],
        }
    }

    /// 指定した結果に対応する遷移を返す。
    pub fn transition_for(&self, on: TransitionOn) -> Option<&Transition> {
        self.transitions.iter().find(|t| t.on == on)
    }
}

/// 遷移を書ける結果。`error` は書けない(BR1.2)。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransitionOn {
    Success,
    Warning,
}

/// 遷移の行き先。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TransitionTarget {
    /// 記述順で後ろのジョブID。
    Job(String),
    /// `@end`(以降のジョブをスキップして終える)。
    End,
}

/// 結果ごとの遷移ルール(独立した識別子を持たない値)。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Transition {
    pub on: TransitionOn,
    pub next: TransitionTarget,
}

/// 1 回のジョブの起動の結果(契約 C1。生成は u2-runtime)。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JobExecution {
    pub job_id: String,
    pub command: String,
    /// 合図が呼び出しの前に立っていて起動しなかったときだけ `false`。
    pub started: bool,
    /// 終了コード(`started` が `true` で、終了コードがあるときのみ)。
    pub exit_code: Option<i32>,
    /// シグナルで終了したときの、シグナル番号。
    pub terminated_by_signal: Option<i32>,
    /// 起動に失敗した。
    pub start_failure: bool,
    /// conductor がシグナルを受け取り、中断の合図が立った。
    pub interrupted: bool,
    /// `interrupted` のとき、受け取ったシグナルの番号。
    pub signal_number: Option<i32>,
}

/// ジョブの状態。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum JobStatus {
    #[default]
    Pending,
    Success,
    Warning,
    Error,
    Skipped,
}

/// スキップの理由(`JobStatus::Skipped` のときのみ)。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SkipReason {
    /// 遷移によるスキップ。
    ByTransition,
    /// `@end` によるスキップ。
    ByEnd,
}

/// パイプラインの中の 1 つのジョブの、これまでの状態(契約 C1。保持は u2-runtime)。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JobState {
    pub job_id: String,
    pub status: JobStatus,
    pub exit_code: Option<i32>,
    pub skip_reason: Option<SkipReason>,
}

impl JobState {
    /// まだ何も起きていないジョブの状態(pending)。
    pub fn pending(job_id: impl Into<String>) -> Self {
        Self {
            job_id: job_id.into(),
            status: JobStatus::Pending,
            exit_code: None,
            skip_reason: None,
        }
    }
}

/// 定義ファイル 1 つに対する、実行状態の全体(契約 C1。保持は u2-runtime)。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PipelineState {
    /// 仮。確定は u2-runtime の Functional Design。
    pub definition_key: String,
    pub pipeline_name: String,
    pub definition_path: String,
    /// 記録時のジョブIDと記述順。
    pub job_order: Vec<String>,
    pub job_states: Vec<JobState>,
    /// 次に実行する、または再開するジョブのID。
    pub resume_job_id: Option<String>,
    pub end_reached: bool,
    pub completed: bool,
}

/// 起動前の判断の理由。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DecisionReason {
    /// 遷移によるスキップ対象(skip)。
    SkipTarget,
    /// `@end` より後ろ(skip)。
    AfterEnd,
    /// 成功・警告で実行済み(skip)。
    AlreadyExecuted,
    /// 順序外の呼び出し(reject のみ)。
    OutOfOrder,
}

/// ジョブを起動する前の判断(契約 C1)。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Decision {
    /// ジョブを実行する。
    Run,
    /// 実行せず、終了コード 0 で終わる。
    Skip(DecisionReason),
    /// 実行せず、ステートも変えない。呼び出し側は理由を表示し、終了コード 66 で終わる(BR3.8)。
    Reject(DecisionReason),
}

/// 結果の分類。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Classification {
    Success,
    Warning,
    Error,
}

/// 実行したジョブの結果の分類と、スケジューラへ返す終了コード。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JobOutcome {
    pub job_id: String,
    /// 分類。ジョブを起動しなかった中断(BR2.4)は分類しないため `None`。
    pub classification: Option<Classification>,
    /// スケジューラへ返す終了コード(BR2.2〜BR2.4)。
    pub reported_exit_code: i32,
    /// 成功コードに含まれない 0 など、conductor 自身が原因を表示するエラーのとき `true`。
    pub conductor_error: bool,
}

/// 結果の分類を受けた、次の動き。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NextStep {
    pub job_id: String,
    /// 次に実行するジョブ。完了または停止のときは `None`。
    pub next_job_id: Option<String>,
    pub skipped_job_ids: Vec<String>,
    pub skip_reason_kind: Option<SkipReason>,
    pub end_reached: bool,
    pub completed: bool,
    /// エラーで停止したときは、そのジョブ。
    pub resume_job_id: Option<String>,
}

/// 実行の方式(BR3.6)。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunMode {
    /// 個別実行(`run <job-id>`)。
    Individual,
    /// 一連の実行(`pipeline <name>`)。
    Pipeline,
}

/// ジョブのあいだで中断されたときの再開位置(契約 C2 の `resume_point_on_interrupt` の出力)。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResumePoint {
    pub resume_job_id: Option<String>,
    pub end_reached: bool,
    pub completed: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn job(id: &str) -> JobDefinition {
        JobDefinition {
            id: id.to_string(),
            command: "echo".to_string(),
            warning_exit_codes: vec![],
            success_exit_codes: None,
            transitions: vec![],
        }
    }

    #[test]
    fn 成功コードの省略時は_0_だけ() {
        assert_eq!(job("a").effective_success_codes(), &[0]);
    }

    #[test]
    fn 成功コードを書いたらその値だけ() {
        let mut j = job("a");
        j.success_exit_codes = Some(vec![2, 3]);
        assert_eq!(j.effective_success_codes(), &[2, 3]);
    }

    #[test]
    fn 遷移を結果から引ける() {
        let mut j = job("a");
        j.transitions.push(Transition {
            on: TransitionOn::Warning,
            next: TransitionTarget::End,
        });
        assert!(j.transition_for(TransitionOn::Success).is_none());
        assert_eq!(
            j.transition_for(TransitionOn::Warning).map(|t| &t.next),
            Some(&TransitionTarget::End)
        );
    }

    #[test]
    fn ジョブの位置は記述順() {
        let def = PipelineDefinition {
            name: "p".into(),
            description: None,
            format_version: 1,
            jobs: vec![job("a"), job("b")],
        };
        assert_eq!(def.job_index("b"), Some(1));
        assert_eq!(def.job_index("zz"), None);
    }

    #[test]
    fn 新しいジョブの状態は_pending() {
        let s = JobState::pending("a");
        assert_eq!(s.status, JobStatus::Pending);
        assert_eq!(JobStatus::default(), JobStatus::Pending);
        assert!(s.exit_code.is_none() && s.skip_reason.is_none());
    }
}
