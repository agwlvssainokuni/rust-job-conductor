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

//! エラーの型と、表示用の組み立て(Describe)。
//!
//! 表示は、kind ごとの固定の文と、ステートファイルのパスだけで組み立てる。入力の値と、
//! ライブラリのエラー文は含めない(NFR4.5)。

use std::path::{Path, PathBuf};
use thiserror::Error;

/// command の文字列を分けられない理由の種類(BR1.1)。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandErrorKind {
    /// 閉じていない引用符。
    UnclosedQuote,
    /// コマンド名がない(空、空白だけ)。
    Empty,
}

impl CommandErrorKind {
    /// 種類の理由(固定の文。入力の値は含めない)。
    pub fn reason(self) -> &'static str {
        match self {
            Self::UnclosedQuote => reason::UNCLOSED_QUOTE,
            Self::Empty => reason::EMPTY_COMMAND,
        }
    }
}

/// command の文字列を分けられないときの報告。ジョブを起動する前に検知できる。表示は、種類の理由
/// ([`CommandErrorKind::reason`])。
#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error("{}", .kind.reason())]
pub struct CommandError {
    pub kind: CommandErrorKind,
}

/// ステートの読み書きの失敗の種類。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StateErrorKind {
    Unreadable,
    TooLarge,
    Corrupt,
    VersionMismatch,
    WriteFailed,
    RemoveFailed,
}

impl StateErrorKind {
    /// 種類の文(日本語。表示と、cli の日本語の表示が使う。D10)。
    pub fn label(self) -> &'static str {
        match self {
            Self::Unreadable => "ステートファイルを読めません",
            Self::TooLarge => "ステートファイルが大きすぎます",
            Self::Corrupt => "ステートファイルが壊れています",
            Self::VersionMismatch => "ステートファイルの版が合いません",
            Self::WriteFailed => "ステートファイルを書き込めません",
            Self::RemoveFailed => "ステートファイルを削除できません",
        }
    }
}

/// ステートの読み書きの失敗の報告(契約 C3 の `failure`)。
///
/// 表示は、kind ごとの固定の文、ステートファイルのパス、固定の理由だけで組み立てる。
#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error("{}: {} ({})", .kind.label(), .path.display(), .reason)]
pub struct StateError {
    pub path: PathBuf,
    pub kind: StateErrorKind,
    /// 固定の文。入力の値と、ライブラリのエラー文は含めない。
    pub reason: &'static str,
}

impl StateError {
    pub(crate) fn new(path: &Path, kind: StateErrorKind, reason: &'static str) -> Self {
        Self {
            path: path.to_path_buf(),
            kind,
            reason,
        }
    }
}

/// 固定の理由の文。
pub(crate) mod reason {
    pub const UNCLOSED_QUOTE: &str = "引用符が閉じていません";
    pub const EMPTY_COMMAND: &str = "コマンド名がありません";

    pub const UNREADABLE: &str = "開く、または読み取ることができません";
    pub const TOO_LARGE: &str = "大きさが上限(1 MiB)を超えています";
    // 文の「1 MiB」とステートファイルの上限の定数が食い違えば、コンパイルできない。
    const _: () = assert!(crate::state::reader::MAX_STATE_BYTES == 1024 * 1024);
    pub const NOT_UTF8: &str = "UTF-8 の文字列ではありません";
    pub const NOT_TOML: &str = "TOML として読めません";
    pub const NO_VERSION: &str = "format_version がありません";
    pub const BAD_VERSION_TYPE: &str = "format_version が整数ではありません";
    pub const UNSUPPORTED_VERSION: &str = "対応していない版です(reset が必要です)";
    pub const UNKNOWN_KEY: &str = "知らないキーがあります";
    pub const MISSING_KEY: &str = "必須のキーがありません";
    pub const WRONG_TYPE: &str = "値の型が誤っています";
    pub const BAD_STATUS: &str = "status の値が誤っています";
    pub const BAD_SKIP_REASON: &str = "skip_reason の値が誤っています";
    pub const SKIP_REASON_MISMATCH: &str = "skip_reason が status と合いません";
    pub const EXIT_CODE_MISMATCH: &str = "exit_code が status と合いません";
    pub const DUPLICATE_JOB: &str = "ジョブIDが重複しています";
    pub const ORDER_MISMATCH: &str = "job の並びが job_order と一致しません";
    pub const RESUME_NOT_IN_ORDER: &str = "resume_job_id が job_order にありません";
    pub const COMPLETED_WITH_RESUME: &str = "completed と resume_job_id が両立しません";
    pub const RUN_ID_MISMATCH: &str = "run_id がファイル名の run ID と一致しません";
    /// 動いたジョブ(success・warning・error)に試行が無い(v0.10.0 の U6 の BR3.1。v0.9.9 までのステート)。
    pub const NO_ATTEMPT: &str = "動いたジョブに試行の記録がありません(古い形式の可能性があります)";
    /// 試行の時刻が `YYYY-MM-DDTHH:MM:SSZ` の形でない(U6 の BR2.1)。
    pub const BAD_TIME: &str = "時刻の書式が誤っています";
    /// 最後でない試行に `ended_at` が無い(U6 の BR3.2)。
    pub const UNFINISHED_ATTEMPT: &str = "終わりの時刻の無い試行が、最後の試行ではありません";

    pub const WRITE_FAILED: &str = "書き込みに失敗しました";
    pub const SERIALIZE_FAILED: &str = "内容を TOML にできません";
    pub const REMOVE_FAILED: &str = "削除に失敗しました";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn 理由の文にステートのキーとしての_jobs_が残らない() {
        for text in [
            reason::UNKNOWN_KEY,
            reason::MISSING_KEY,
            reason::WRONG_TYPE,
            reason::DUPLICATE_JOB,
            reason::ORDER_MISMATCH,
            reason::RESUME_NOT_IN_ORDER,
        ] {
            assert!(!text.contains("jobs"), "{text}");
        }
    }

    #[test]
    fn コマンドエラーの表示は固定の文() {
        let e = CommandError {
            kind: CommandErrorKind::UnclosedQuote,
        };
        assert_eq!(e.to_string(), reason::UNCLOSED_QUOTE);
        let e = CommandError {
            kind: CommandErrorKind::Empty,
        };
        assert_eq!(e.to_string(), reason::EMPTY_COMMAND);
    }

    #[test]
    fn 全_kind_の表示にパスと固定の文だけが入る() {
        let kinds = [
            StateErrorKind::Unreadable,
            StateErrorKind::TooLarge,
            StateErrorKind::Corrupt,
            StateErrorKind::VersionMismatch,
            StateErrorKind::WriteFailed,
            StateErrorKind::RemoveFailed,
        ];
        let mut labels = std::collections::HashSet::new();
        for kind in kinds {
            let e = StateError::new(Path::new("dir/x.state"), kind, reason::NOT_TOML);
            let text = e.to_string();
            assert!(text.contains("dir/x.state"), "{text}");
            assert!(text.contains(reason::NOT_TOML), "{text}");
            assert!(labels.insert(kind.label()), "kind ごとに文が違う");
        }
    }

    #[test]
    fn 等しさはフィールドで決まる() {
        let a = StateError::new(Path::new("a"), StateErrorKind::Corrupt, reason::NOT_TOML);
        let b = StateError::new(Path::new("a"), StateErrorKind::Corrupt, reason::NOT_TOML);
        let c = StateError::new(Path::new("b"), StateErrorKind::Corrupt, reason::NOT_TOML);
        assert_eq!(a, b);
        assert_ne!(a, c);
    }
}
