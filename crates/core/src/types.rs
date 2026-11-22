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

//! 共有する型(Types)。契約 C1 のデータの形だけを持ち、入出力と OS に依存しない。
//!
//! 型の定義は、`entities.md` に従う。値の生成を受け持つのは、このクレートでは
//! 定義(`PipelineDefinition` など)と判断(`Decision`、`JobOutcome`、`NextStep`)で、
//! `JobExecution`、`JobState`、`PipelineState` は conductor-runtime が生成・保持する。
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
    /// run ID を読む環境変数の名前(省略可。形は検査済み。省略時は `CONDUCTOR_RUN_ID`)。
    pub run_id_env: Option<String>,
    /// 完了のときにステートファイルを削除するか(省略時は `false`。FR2.2)。
    pub remove_on_complete: bool,
    /// 書式の版(初版は 1)。
    pub format_version: u32,
    /// ジョブ。定義ファイルに書いた順を保つ(BR1.11)。1 つ以上。
    pub job: Vec<JobDefinition>,
    /// `[pipeline.env]` の内容。書いた順を保つ(FR1.1)。
    pub env: Vec<EnvVar>,
    /// `[pipeline].env_file` の内容。書いた順を保つ。文字列 1 つは要素 1 つの並びにする(FR1.4)。
    pub env_file: Vec<String>,
}

/// 定義の `env` の表の 1 項目(名前と値)。値は秘密として扱い、`Debug` は伏せ字にする(BR8.1)。
#[derive(Clone, PartialEq, Eq)]
pub struct EnvVar {
    /// 環境変数の名前(名前の規則と予約名の検査は済んでいる。BR1.2〜BR1.4)。
    pub name: String,
    /// 値(NUL を含まない。BR1.5)。展開しない(BR1.1)。
    pub value: String,
}

impl std::fmt::Debug for EnvVar {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EnvVar")
            .field("name", &self.name)
            .field("value", &"***")
            .finish()
    }
}

impl PipelineDefinition {
    /// ジョブIDから、記述順の位置(0 始まり)を求める。
    pub fn job_index(&self, job_id: &str) -> Option<usize> {
        self.job.iter().position(|j| j.id == job_id)
    }
}

/// ジョブのコマンド。定義ファイルには、文字列または文字列の配列で書く。ここでは解釈しない。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    /// 1 つの文字列。起動のとき、コマンド名と引数に分ける(空白で区切り、引用符を解く)。
    Line(String),
    /// 文字列の配列。先頭がプログラム、残りが引数で、1 要素が 1 引数。分割も展開もしない。
    Argv(Vec<String>),
}

impl Command {
    /// 起動するプログラムの名前(起動失敗の表示用)。取り出せないとき(空)は `None`。
    pub fn program_name(&self) -> Option<&str> {
        match self {
            Command::Line(line) => line.split_whitespace().next(),
            Command::Argv(argv) => argv.first().map(String::as_str),
        }
    }
}

impl From<&str> for Command {
    fn from(line: &str) -> Self {
        Command::Line(line.to_string())
    }
}

impl From<String> for Command {
    fn from(line: String) -> Self {
        Command::Line(line)
    }
}

/// 1 つのジョブの定義(契約 C1)。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JobDefinition {
    /// ジョブID(空でなく、定義の中で一意。`@` で始めない)。
    pub id: String,
    /// コマンド(空でない)。文字列(`Line`)または文字列の配列(`Argv`)。ここでは解釈しない。
    pub command: Command,
    /// 警告にする終了コード。`success_exit_codes` と重ならず、0 を含まない。
    pub warning_exit_codes: Vec<i32>,
    /// 成功にする終了コード。省略時は `None`(`[0]` として扱う)。書いたら、その値だけが成功。
    pub success_exit_codes: Option<Vec<i32>>,
    /// 結果ごとの遷移(`on` ごとに高々 1 つ)。
    pub transitions: Vec<Transition>,
    /// リトライの設定。`[job.retry]` を書かなければ `None`(リトライしない。BR1.1、FR1.2)。
    pub retry: Option<RetryPolicy>,
    /// `[job.env]` の内容。書いた順を保つ(FR1.1)。
    pub env: Vec<EnvVar>,
    /// `[job].env_file` の内容。書いた順を保つ(FR1.4)。
    pub env_file: Vec<String>,
}

/// ジョブのリトライの設定(`[job.retry]`。BR1.1〜BR1.13、FR1.1〜FR1.9)。
///
/// 値はすべて整数で持つ(浮動小数点は使わない。要件 C1)。検証は、定義の読み取りで済んでいる。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RetryPolicy {
    /// リトライの対象にする終了コード。昇順で重複がなく、各値は 1〜255、空でない(BR1.5、BR1.6)。
    pub exit_codes: Vec<i32>,
    /// リトライの最大回数(1〜100。BR1.9)。
    pub max: u32,
    /// 最初の待ち時間(秒。1 秒〜24 時間。BR1.10)。
    pub interval_secs: u64,
    /// 待ち時間にかける倍率(2〜10)。`max_interval_secs` と同時に指定する(BR1.11、BR1.12)。
    pub multiplier: Option<u32>,
    /// 待ち時間の上限(秒)。`interval_secs` 以上(BR1.10、BR1.13)。
    pub max_interval_secs: Option<u64>,
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

impl TransitionOn {
    /// 定義と表示に使う語(`success`、`warning`)。
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Success => "success",
            Self::Warning => "warning",
        }
    }
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

/// 1 回のジョブの起動の結果(契約 C1。生成は conductor-runtime)。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JobExecution {
    pub job_id: String,
    pub command: Command,
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

/// パイプラインの中の 1 つのジョブの、これまでの状態(契約 C1。保持は conductor-runtime)。
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

/// 定義ファイル 1 つに対する、実行状態の全体(契約 C1。保持は conductor-runtime)。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PipelineState {
    /// 仮。確定は conductor-runtime の Functional Design。
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
    /// run ID(run ID なしのステートでは `None`。ファイル名の run ID と一致する)。
    pub run_id: Option<String>,
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

impl Classification {
    /// 表示に使う語(`success`、`warning`、`error`)。
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Success => "success",
            Self::Warning => "warning",
            Self::Error => "error",
        }
    }

    /// この分類で引く遷移(成功は `success`、警告は `warning`)。エラーは遷移を書けないため `None`(BR1.2)。
    pub fn transition_on(self) -> Option<TransitionOn> {
        match self {
            Self::Success => Some(TransitionOn::Success),
            Self::Warning => Some(TransitionOn::Warning),
            Self::Error => None,
        }
    }
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
    /// この時点でスキップとして記録するジョブ。`ByTransition`(間のジョブ)と、`Pipeline` での
    /// `@end`(以降の全ジョブ、`ByEnd`)で埋まる。`Individual` で `@end` に到達したときは空で、
    /// 以降のジョブは、スケジューラが呼んだときに `decide_before_run` が `Skip(AfterEnd)` を
    /// 返し、Cli が呼ばれたジョブだけを記録する(BR3.6)。
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
            command: "echo".into(),
            warning_exit_codes: vec![],
            success_exit_codes: None,
            transitions: vec![],
            retry: None,
            env: Vec::new(),
            env_file: Vec::new(),
        }
    }

    fn policy() -> RetryPolicy {
        RetryPolicy {
            exit_codes: vec![75],
            max: 3,
            interval_secs: 30,
            multiplier: None,
            max_interval_secs: None,
        }
    }

    #[test]
    fn リトライを持たない定義は_retry_が_none() {
        assert_eq!(job("a").retry, None);
    }

    #[test]
    fn リトライを持つ定義を作れて比較できる() {
        let mut j = job("a");
        j.retry = Some(policy());
        assert_eq!(j.retry.as_ref().map(|r| r.max), Some(3));
        assert_eq!(j, j.clone());
        assert_ne!(j, job("a"));
    }

    #[test]
    fn 倍率と上限つきの設定は値で比較される() {
        let mut p = policy();
        p.multiplier = Some(2);
        p.max_interval_secs = Some(600);
        assert_ne!(p, policy());
        assert_eq!(p, p.clone());
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
            run_id_env: None,
            remove_on_complete: false,
            format_version: 1,
            job: vec![job("a"), job("b")],
            env: Vec::new(),
            env_file: Vec::new(),
        };
        assert_eq!(def.job_index("b"), Some(1));
        assert_eq!(def.job_index("zz"), None);
    }

    #[test]
    fn 文字列から_line_を作れる() {
        assert_eq!(Command::from("echo a"), Command::Line("echo a".into()));
        assert_eq!(
            Command::from(String::from("echo a")),
            Command::Line("echo a".into())
        );
    }

    #[test]
    fn プログラム名は_line_では先頭の語_argv_では先頭の要素() {
        assert_eq!(Command::from("  echo  a b").program_name(), Some("echo"));
        assert_eq!(Command::from("   ").program_name(), None);
        let argv = Command::Argv(vec!["my prog".into(), "x".into()]);
        assert_eq!(argv.program_name(), Some("my prog"));
        assert_eq!(Command::Argv(vec![]).program_name(), None);
    }

    #[test]
    fn 新しいジョブの状態は_pending() {
        let s = JobState::pending("a");
        assert_eq!(s.status, JobStatus::Pending);
        assert_eq!(JobStatus::default(), JobStatus::Pending);
        assert!(s.exit_code.is_none() && s.skip_reason.is_none());
    }
}
