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

//! オプションの表と検査の順(ArgParser の一部。BR1.1、BR1.3)。
//!
//! オプションの性質(名前、値を取るか、使える場所)と、「使えない」と「重複」の検査の順、
//! 指定の集計、コマンドごとの位置引数の扱いを持つ。親のモジュールの字句と組み立てから引かれる。

use std::ffi::OsString;

use crate::types::CommandKind as C;
use crate::types::{CommandKind, RUN_ID_OPTION, UsageError, UsageErrorKind as Kind};

/// オプションの識別子(`-h`・`--help`・`-V`・`--version` を除く。BR1.1)。
///
/// 並びは [`OPTIONS`] の行の並びと同じ(コンパイル時に検査する)。
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Opt {
    Verbose,
    Config,
    To,
    RunId,
    All,
    RemoveOnComplete,
    DryRun,
    Force,
    Assume,
    Format,
    State,
    Follow,
    Interval,
}

/// オプションを使える場所(BR1.1)。
enum Allowed {
    /// 並びのコマンドで使える。
    Commands(&'static [CommandKind]),
    /// `--dry-run` があるときだけ使える(`--assume`。BR2.2)。
    WithDryRun,
    /// `--follow` があるときだけ使える(`--interval`。v0.10.0 の U7 の BR1.2)。
    WithFollow,
}

/// オプションの表の 1 行(BR1.1)。
pub(super) struct OptSpec {
    pub(super) opt: Opt,
    /// 長い名前(`--` を除く)。
    long: &'static str,
    /// 値を取るか(`--name value` と `--name=value`)。
    pub(super) takes_value: bool,
    allowed: Allowed,
}

/// `convert` 以外のすべてのコマンド。
const ALL_BUT_CONVERT: &[CommandKind] = &[
    C::Run,
    C::Pipeline,
    C::Watch,
    C::Reset,
    C::Validate,
    C::Graph,
    C::Init,
];
const RUN_AND_PIPELINE: &[CommandKind] = &[C::Run, C::Pipeline];

/// オプションの表(BR1.1)。オプションの性質は、ここにだけ置く。
///
/// `--run-id` の `graph` は、`--state` と一緒のときだけ(BR4.5。組み立ての段階で、表の検査のあとに見る)。
const OPTIONS: &[OptSpec] = &[
    OptSpec {
        opt: Opt::Verbose,
        long: "verbose",
        takes_value: false,
        allowed: Allowed::Commands(&[C::Run, C::Pipeline, C::Validate, C::Graph]),
    },
    OptSpec {
        opt: Opt::Config,
        long: "config",
        takes_value: true,
        allowed: Allowed::Commands(ALL_BUT_CONVERT),
    },
    OptSpec {
        opt: Opt::To,
        long: "to",
        takes_value: true,
        allowed: Allowed::Commands(&[C::Convert]),
    },
    OptSpec {
        opt: Opt::RunId,
        long: "run-id",
        takes_value: true,
        allowed: Allowed::Commands(&[C::Run, C::Pipeline, C::Watch, C::Reset, C::Graph]),
    },
    OptSpec {
        opt: Opt::All,
        long: "all",
        takes_value: false,
        allowed: Allowed::Commands(&[C::Reset]),
    },
    OptSpec {
        opt: Opt::RemoveOnComplete,
        long: "remove-on-complete",
        takes_value: false,
        allowed: Allowed::Commands(RUN_AND_PIPELINE),
    },
    OptSpec {
        opt: Opt::DryRun,
        long: "dry-run",
        takes_value: false,
        allowed: Allowed::Commands(RUN_AND_PIPELINE),
    },
    OptSpec {
        opt: Opt::Force,
        long: "force",
        takes_value: false,
        allowed: Allowed::Commands(&[C::Init]),
    },
    OptSpec {
        opt: Opt::Assume,
        long: "assume",
        takes_value: true,
        allowed: Allowed::WithDryRun,
    },
    OptSpec {
        opt: Opt::Format,
        long: "format",
        takes_value: true,
        allowed: Allowed::Commands(&[C::Graph]),
    },
    OptSpec {
        opt: Opt::State,
        long: "state",
        takes_value: false,
        allowed: Allowed::Commands(&[C::Graph]),
    },
    OptSpec {
        opt: Opt::Follow,
        long: "follow",
        takes_value: false,
        allowed: Allowed::Commands(&[C::Watch]),
    },
    OptSpec {
        opt: Opt::Interval,
        long: "interval",
        takes_value: true,
        allowed: Allowed::WithFollow,
    },
];

// `OPTIONS` の行は、`Opt` の並びと同じ順に置く(`spec` が添字で引くため)。
const _: () = {
    assert!(OPTIONS.len() == Opt::Interval as usize + 1);
    let mut i = 0;
    while i < OPTIONS.len() {
        assert!(OPTIONS[i].opt as usize == i);
        i += 1;
    }
};

/// `full` が `--` と `long` をつないだ綴りか(コンパイル時の検査用)。
const fn is_long_spelling(full: &str, long: &str) -> bool {
    let full = full.as_bytes();
    let long = long.as_bytes();
    if full.len() != long.len() + 2 || full[0] != b'-' || full[1] != b'-' {
        return false;
    }
    let mut i = 0;
    while i < long.len() {
        if full[i + 2] != long[i] {
            return false;
        }
        i += 1;
    }
    true
}

// `types` の `RUN_ID_OPTION`(`RunIdArg::parse` の誤りの対象)は、表の `--run-id` の綴りと同じ。
const _: () = assert!(is_long_spelling(
    RUN_ID_OPTION,
    OPTIONS[Opt::RunId as usize].long
));

/// 識別子から、表の行を引く。
pub(super) fn spec(opt: Opt) -> &'static OptSpec {
    &OPTIONS[opt as usize]
}

/// 長い名前から、表の行を引く。
pub(super) fn spec_by_long(name: &str) -> Option<&'static OptSpec> {
    OPTIONS.iter().find(|s| s.long == name)
}

impl OptSpec {
    /// コマンド `kind` で使えるか。`dry_run` は `--dry-run` が 1 回以上あるか(`--assume` の条件)、
    /// `follow` は `--follow` が 1 回以上あるか(`--interval` の条件)。
    pub(super) fn allows(&self, kind: CommandKind, dry_run: bool, follow: bool) -> bool {
        match self.allowed {
            Allowed::Commands(kinds) => kinds.contains(&kind),
            Allowed::WithDryRun => dry_run,
            Allowed::WithFollow => follow,
        }
    }
}

impl Opt {
    /// 誤りの対象の綴り。`-v` だけ短い形、ほかは `--<長い名前>`。
    pub(super) fn subject(self) -> String {
        match self {
            Opt::Verbose => "-v".to_string(),
            _ => format!("--{}", spec(self).long),
        }
    }
}

/// 検査の順の表の 1 つ(BR1.3)。
#[derive(Clone, Copy)]
pub(super) enum Check {
    /// 1 回以上あり、そのコマンドで使えない → `optionNotAllowed`(対象は綴り)。
    NotAllowed(Opt),
    /// 2 回以上ある → `invalidValue`(対象は綴り)。
    Duplicate(Opt),
}

/// 「使えない」と「重複」の検査の順(BR1.3)。仕様書 04 章 4.2 の 6・7 の順を写す。
/// 誤りの報告の順は、この並びで決まる。`--assume` は重複を許す(同じジョブID の重複は、値の検査で誤り)。
pub(super) const CHECKS: &[Check] = &[
    Check::NotAllowed(Opt::Verbose),
    Check::NotAllowed(Opt::Config),
    Check::NotAllowed(Opt::To),
    Check::NotAllowed(Opt::RunId),
    Check::NotAllowed(Opt::All),
    Check::NotAllowed(Opt::RemoveOnComplete),
    Check::NotAllowed(Opt::DryRun),
    Check::NotAllowed(Opt::Force),
    Check::Duplicate(Opt::Force),
    Check::NotAllowed(Opt::Assume),
    Check::Duplicate(Opt::DryRun),
    Check::NotAllowed(Opt::Format),
    Check::NotAllowed(Opt::State),
    // `--follow` を `--interval` より先に見る(`watch` 以外の `--follow --interval` は、`--follow` の誤り。
    // v0.10.0 の U7 の BR1.2)。
    Check::NotAllowed(Opt::Follow),
    Check::NotAllowed(Opt::Interval),
    Check::Duplicate(Opt::Format),
    Check::Duplicate(Opt::State),
    Check::Duplicate(Opt::Follow),
    Check::Duplicate(Opt::Interval),
    Check::Duplicate(Opt::Verbose),
    Check::Duplicate(Opt::Config),
    Check::Duplicate(Opt::To),
    Check::Duplicate(Opt::RunId),
    Check::Duplicate(Opt::All),
    Check::Duplicate(Opt::RemoveOnComplete),
];

/// オプションごとの指定の並び(`Opt` で引く。BR1.3)。数は並びの長さ。値を取らないオプションは `None` を積む。
pub(super) struct Given(Vec<Vec<Option<OsString>>>);

impl Given {
    pub(super) fn new() -> Self {
        Given(OPTIONS.iter().map(|_| Vec::new()).collect())
    }

    pub(super) fn push(&mut self, opt: Opt, value: Option<OsString>) {
        self.0[opt as usize].push(value);
    }

    /// 指定の回数。
    pub(super) fn count(&self, opt: Opt) -> usize {
        self.0[opt as usize].len()
    }

    /// ちょうど 1 回か(CHECKS のあとは、`--assume` 以外は 0 回か 1 回)。
    pub(super) fn once(&self, opt: Opt) -> bool {
        self.count(opt) == 1
    }

    /// 値を、指定の順に取り出す。
    pub(super) fn take(&mut self, opt: Opt) -> Vec<OsString> {
        std::mem::take(&mut self.0[opt as usize])
            .into_iter()
            .flatten()
            .collect()
    }
}

/// 位置引数(オペランド)の扱い(BR1.1。文法の表の「位置引数」の列)。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Operand {
    /// 必須(無ければ `missingArgument`)。
    Required,
    /// 省略できる。
    Optional,
    /// 書けない(書けば `unexpectedArgument`)。
    Forbidden,
}

/// コマンドごとの位置引数の扱い。新しいコマンドを足したら、ここで決める(網羅の `match`)。
pub(super) fn operand_rule(kind: CommandKind) -> Operand {
    match kind {
        CommandKind::Run | CommandKind::Pipeline | CommandKind::Convert => Operand::Required,
        CommandKind::Validate | CommandKind::Graph | CommandKind::Init => Operand::Optional,
        CommandKind::Watch | CommandKind::Reset => Operand::Forbidden,
    }
}

pub(super) fn unexpected_argument(value: &OsString) -> UsageError {
    UsageError::with_subject(
        Kind::UnexpectedArgument,
        value.to_string_lossy().into_owned(),
    )
}
