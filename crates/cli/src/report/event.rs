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

//! 表示する出来事の定義(Reporter の一部。BR6.1〜BR6.5)。
//!
//! 流れが作る [`DisplayEvent`] と、その補足の型(`StateContext`、`ResultDetail`、`FramePrefix`)、
//! 水準(`level`)を持つ。親のモジュールが `pub use` で今の名前のまま出す。

use std::path::Path;

use conductor_converter::ConversionError;
use conductor_core::{
    Classification, DefinitionError, EnvFileError, EnvWarning, MismatchKind, PipelineState,
    RejectReason, RunId, SkipCause, Timestamp, TransitionOn, TransitionTarget,
};
use conductor_runtime::{CommandError, CreateError, StateError};

use crate::types::{Level, UsageError};

/// ステートの誤りを表示するときの文脈(原因に添える補足を決める)。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StateContext {
    /// ステートを読んだ(`reset` を案内する)。
    Read,
    /// run ID つきのステートを読んだ(`reset --run-id <ID>` を案内する)。
    ReadRunId,
    /// ジョブを起動する前の失敗(ジョブは起動していない)。
    JobNotStarted,
    /// ジョブのあとの失敗(ジョブは実行済み)。
    JobExecuted,
    /// 補足なし(`reset` と `reset --all`)。
    Plain,
}

impl StateContext {
    /// ステートを読んだときの表示の文脈。run ID があるときは、案内を `reset --run-id` にする。
    pub(crate) fn for_read(run_id: Option<&RunId>) -> Self {
        match run_id {
            Some(_) => Self::ReadRunId,
            None => Self::Read,
        }
    }
}

/// ジョブの結果の補足(`-v` の `(…)` の中)。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResultDetail {
    /// ジョブの終了コード。
    ExitCode(i32),
    /// ジョブが、シグナルで終了した。
    Signal(i32),
    /// ジョブを起動できなかった。
    StartFailure,
    /// 中断のシグナルを受け取った(`started` は、ジョブが起動していたか)。
    Interrupted { signal: i32, started: bool },
}

/// 表示する内容(entities の DisplayEvent。種類とデータ)。
#[derive(Debug, Clone)]
pub enum DisplayEvent<'a> {
    // ---- error(標準エラー。常に) ----
    DefinitionErrors(&'a [DefinitionError]),
    /// `.env` の誤り(20 件で打ち切り、残りは件数。値は含めない。BR2.8)。
    EnvFileErrors(&'a [EnvFileError]),
    /// `.env` の警告(重複キー。`validate` と `graph` が出す(`check_definition_and_env`)。BR7.2)。
    EnvWarnings(&'a [EnvWarning]),
    CommandErrors {
        definition: &'a Path,
        errors: &'a [(String, CommandError)],
    },
    Usage(&'a UsageError),
    State {
        error: &'a StateError,
        context: StateContext,
    },
    ConsistencyMismatch {
        state_path: &'a Path,
        kind: MismatchKind,
        /// 読んだときの文脈。`Read` か `ReadRunId`(run ID つきのステートなら、案内の文が変わる)。
        context: StateContext,
    },
    /// `reset --all` が、読めなくて残したファイル(警告。終了コードは変わらない)。
    ResetAllKept {
        names: &'a [String],
    },
    Rejected {
        job_id: &'a str,
        reason: RejectReason,
    },
    StartFailure {
        job_id: &'a str,
        command: &'a str,
    },
    SuccessCodeNotListed {
        job_id: &'a str,
    },
    Conversion(&'a ConversionError),
    /// 標準出力に書けなかった(`watch`・`graph`・`--dry-run` は 65、`convert` は 67 で終わる)。
    OutputFailed,
    CompletionUndetermined {
        state_path: &'a Path,
    },
    /// シグナルの準備に失敗した(固定の英語の文。終了コード 70)。
    SignalSetupFailed,
    /// `init` の作る先に、ファイルが既にある(終了コード 69。U4 の BR1.2)。
    InitExists {
        path: &'a Path,
    },
    /// `init` が、ファイルを作れなかった(`already_exists` 以外。終了コード 65。BR3.2)。
    InitFailed {
        path: &'a Path,
        error: CreateError,
    },
    /// `init --config -` で、ひな形を標準出力に書けなかった(終了コード 65。BR1.5)。
    InitStdoutFailed,
    /// ログのファイルを作れず、ジョブを起動しなかった(終了コード 127。v0.10.0 の BR5.1)。
    /// 起動の失敗の表示([`DisplayEvent::StartFailure`])の代わりに出す。
    LogNotCreated {
        path: &'a Path,
    },
    // ---- notice(標準エラー。常に。エラーではない知らせ) ----
    /// ログのファイルへの書き込みに失敗し、途中からファイルに残していない(警告。v0.10.0 の BR5.2)。
    LogWriteFailed {
        path: &'a Path,
    },
    /// ジョブの終了の後も出力が続き、残りをファイルに残していない(警告。v0.10.0 の BR5.3)。
    LogOutputCutShort {
        path: &'a Path,
    },
    /// リトライの前の知らせ(BR2.4、BR2.15)。待機の前に出す。
    RetryScheduled {
        job_id: &'a str,
        /// 直前の実行の終了コード。
        last_exit_code: i32,
        /// 何回目のリトライか(1 から)。
        retry_number: u32,
        /// リトライの最大回数。
        max: u32,
        /// 待ち時間(秒)。
        wait_secs: u64,
    },
    /// `init` が、ファイルを作った(BR1.4)。
    InitCreated {
        path: &'a Path,
    },
    /// `init --force` が、既にあったファイルを置き換えた(BR1.4)。
    InitReplaced {
        path: &'a Path,
    },
    // ---- verbose(標準エラー。`-v` のとき) ----
    JobResult {
        job_id: &'a str,
        /// 分類。`None` は、起動しなかった中断(表示は `success`・`warning`・`error`・`interrupted` のトークン)。
        result: Option<Classification>,
        detail: ResultDetail,
        /// `conductor` が返す終了コード(範囲を合わせたあと)。
        conductor_exit: i32,
    },
    JobSkipped {
        job_id: &'a str,
        reason: SkipCause,
    },
    Transition {
        /// 遷移を引いた結果(表示は `success` か `warning`)。
        on: TransitionOn,
        /// 遷移先(ジョブID、または `@end`。表示は [`TransitionTarget::label`] をエスケープしたもの)。
        target: &'a TransitionTarget,
        skipped: &'a [String],
    },
    Interrupted {
        signal: i32,
    },
    /// `validate` が通った(ジョブの数)。
    DefinitionValid {
        job_count: usize,
    },
    // ---- output(標準出力) ----
    /// `watch` の表。`now` は表示の時点で 1 回だけ取った今の時刻(動いている最中のジョブの経過時間に
    /// 使う。v0.10.0 の U6 の BR4.1)。
    WatchTable {
        state: &'a PipelineState,
        now: Timestamp,
    },
    WatchEmpty,
    /// `watch --follow` の 1 回分の表示(v0.10.0 の U7。BR2.1、BR2.2)。前に `prefix` を置き、続けて
    /// 1 回の `watch` と同じ表(`state` があるとき。時刻は表示の時点の今)か「ステートが無い」を書く。
    WatchFrame {
        prefix: FramePrefix,
        state: Option<(&'a PipelineState, Timestamp)>,
    },
    /// `graph` の図(組み立て済みの text か Mermaid。そのまま出す。BR6.3)。
    Graph(&'a str),
    /// `--dry-run` の計画(組み立て済み。そのまま出す。U3 の BR3.4)。
    Plan(&'a str),
    /// `init --config -` のひな形(組み立て済み。そのまま出す。BR1.5)。
    InitTemplate(&'a str),
    /// 使い方(`--help`)。
    Help,
    /// 版(`--version`)。
    Version,
}

/// `watch --follow` の 1 回分の表示の前に置くもの(v0.10.0 の U7。BR2.2、NFR6.3)。
///
/// どれにするかは、流れが初めに 1 回だけ決めた「画面を消せるか」(UNIX 系の端末)と、何回目の表示かで決める。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FramePrefix {
    /// 何も置かない(追記の 1 回目)。
    Nothing,
    /// 画面を消してカーソルを左上に戻す(UNIX 系の端末。1 回目から毎回)。
    ClearScreen,
    /// 空の 1 行(Windows・パイプ・ファイルの追記の 2 回目から)。
    BlankLine,
}

/// 画面を消してカーソルを左上に戻す制御の文字列(`ESC[H ESC[2J`)。
pub(crate) const CLEAR_SCREEN: &str = "\x1b[H\x1b[2J";

impl FramePrefix {
    /// 描き直しの `n` 回目(0 始まり)の前に置くもの。`clear` は画面を消せるか(流れが初めに 1 回だけ決める)。
    pub fn for_frame(clear: bool, n: usize) -> Self {
        match (clear, n) {
            (true, _) => Self::ClearScreen,
            (false, 0) => Self::Nothing,
            (false, _) => Self::BlankLine,
        }
    }

    /// 前に置く文字列。
    pub fn text(self) -> &'static str {
        match self {
            Self::Nothing => "",
            Self::ClearScreen => CLEAR_SCREEN,
            Self::BlankLine => "\n",
        }
    }
}

impl DisplayEvent<'_> {
    /// 水準(出力先と、表示の条件を決める)。
    pub fn level(&self) -> Level {
        match self {
            Self::DefinitionErrors(_)
            | Self::EnvFileErrors(_)
            | Self::EnvWarnings(_)
            | Self::CommandErrors { .. }
            | Self::Usage(_)
            | Self::State { .. }
            | Self::ConsistencyMismatch { .. }
            | Self::ResetAllKept { .. }
            | Self::Rejected { .. }
            | Self::StartFailure { .. }
            | Self::SuccessCodeNotListed { .. }
            | Self::Conversion(_)
            | Self::OutputFailed
            | Self::SignalSetupFailed
            | Self::InitExists { .. }
            | Self::InitFailed { .. }
            | Self::InitStdoutFailed
            | Self::LogNotCreated { .. }
            | Self::CompletionUndetermined { .. } => Level::Error,
            Self::RetryScheduled { .. }
            | Self::InitCreated { .. }
            | Self::InitReplaced { .. }
            | Self::LogWriteFailed { .. }
            | Self::LogOutputCutShort { .. } => Level::Notice,
            Self::JobResult { .. }
            | Self::JobSkipped { .. }
            | Self::Transition { .. }
            | Self::Interrupted { .. }
            | Self::DefinitionValid { .. } => Level::Verbose,
            Self::WatchTable { .. }
            | Self::WatchEmpty
            | Self::WatchFrame { .. }
            | Self::Graph(_)
            | Self::Plan(_)
            | Self::InitTemplate(_)
            | Self::Help
            | Self::Version => Level::Output,
        }
    }
}
