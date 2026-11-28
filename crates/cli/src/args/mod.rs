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

//! 引数の解釈(ArgParser。NFR4.1、BR1.1〜BR1.6)。
//!
//! `lexopt` で、引数を字句(オプションの名前、値、位置引数)に分け、コマンドごとの文法の表で検査する。
//! 引数は、シェルを介さず、解釈も展開もしない。パスの値(`--config`、convert の入力)は、
//! `OsString` のまま保つ。
//!
//! # 文法
//!
//! | コマンド | 位置引数 | `-v` | `--config` | `--to` | `--run-id` | `--all` | `--remove-on-complete` | `--dry-run` | `--assume` | `--format` | `--state` | `--force` | `--follow` | `--interval` |
//! |----------|----------|------|------------|--------|------------|---------|------------------------|-------------|------------|------------|-----------|-----------|------------|--------------|
//! | `run` | ジョブID(1) | 可 | 可 | 不可 | 可 | 不可 | 可 | 可 | `--dry-run` と一緒のときだけ | 不可 | 不可 | 不可 | 不可 | 不可 |
//! | `pipeline` | パイプライン名(1) | 可 | 可 | 不可 | 可 | 不可 | 可 | 可 | `--dry-run` と一緒のときだけ | 不可 | 不可 | 不可 | 不可 | 不可 |
//! | `watch` | なし | 不可 | 可 | 不可 | 可 | 不可 | 不可 | 不可 | 不可 | 不可 | 不可 | 不可 | 可 | `--follow` と一緒のときだけ |
//! | `reset` | なし | 不可 | 可 | 不可 | 可 | 可(`--run-id` とは併用不可) | 不可 | 不可 | 不可 | 不可 | 不可 | 不可 | 不可 | 不可 |
//! | `validate` | パイプライン名(0 か 1) | 可 | 可 | 不可 | 不可 | 不可 | 不可 | 不可 | 不可 | 不可 | 不可 | 不可 | 不可 | 不可 |
//! | `convert` | 入力ファイル(1) | 不可 | 不可 | 必須(`yaml` か `toml`) | 不可 | 不可 | 不可 | 不可 | 不可 | 不可 | 不可 | 不可 | 不可 | 不可 |
//! | `graph` | パイプライン名(0 か 1) | 可 | 可 | 不可 | `--state` と一緒のときだけ | 不可 | 不可 | 不可 | 不可 | 可 | 可 | 不可 | 不可 | 不可 |
//! | `init` | パイプライン名(0 か 1) | 不可 | 可(`-` は標準出力) | 不可 | 不可 | 不可 | 不可 | 不可 | 不可 | 不可 | 不可 | 可 | 不可 | 不可 |
//!
//! - `watch` だけが、`--follow`(値なし。1 回だけ)と、`--interval <秒>`(値あり。1 回だけ)を取る(v0.10.0 の U7。
//!   BR1.1、BR1.2)。`--interval` は、`--follow` と一緒のときだけ。値は 1〜3600 の整数の秒(数字だけ。符号・小数点・
//!   空白を許さない)で、誤りは `invalidValue`(対象は `--interval` で、入力の値は、誤りの文に含めない)。
//!   省略時は 2 秒。
//! - `init` だけが、`--force`(値なし。1 回だけ)を取る(U4。BR1.1、BR1.5、BR2.2、BR3.1)。`init` は、`--config` と
//!   `--force` 以外のオプションを、すべて許さない(68)。名前が空、または空白だけなら `invalidValue`
//!   (対象は固定で、名前の値は含めない)。`--config -` は標準出力(`InitTarget::Stdout`)。
//! - `graph` だけが、`--format <text|mermaid|dot>`(値あり)と `--state`(値なし)を取る。どちらも 1 回だけ。
//!   ほかのコマンドでは、許されないオプション。
//! - `run` と `pipeline` だけが、`--dry-run`(値なし。1 回だけ)と、`--assume <ジョブID>=<終了コード>`
//!   (複数回)を取る(U3。BR2.1、BR2.2)。`--assume` は、`--dry-run` と一緒のときだけ。ほかのコマンドでは、
//!   許されないオプション。`--assume` の誤りは、`invalidValue`(対象は `--assume` で、入力の値は、誤りの文に含めない):
//!   `=` がない、終了コードが整数でない、0〜255 の範囲外、同じジョブID の重複。ジョブID が定義にあるかは、
//!   定義を読んだあとに、流れが検査する。
//! - `--remove-on-complete` は、値を取らない。指定すると、完了のときにステートファイルを削除する(FR2.1)。
//!   2 回以上の指定は、`-v` と同じく `invalidValue`。
//! - `--config path` と `--config=path`(`--to`、`--run-id` も同じ)の両方を受け付ける。
//! - `--run-id` の値は、空(run ID なしに固定)か、`RunId` の規則(`A-Za-z0-9._-`、64 文字まで、
//!   先頭が `.` でない)を満たすこと。違反は `invalidValue`(値は、誤りの文に含めない)。
//! - `--` は、オプションの終わりで、以降は、すべて位置引数。
//! - `-v` は、ちょうど1回。2回以上(`-vv` を含む)は `invalidValue`。`--config`、`--to` の重複も同じ。
//! - `--help`・`-h`・`--version`・`-V` は、`--` より前に現れたとき、ほかの誤りより先に処理する。

use std::ffi::OsString;

use conductor_converter::TargetFormat;
use conductor_core::Assumption;
use lexopt::prelude::*;

use crate::types::CommandKind as C;
use crate::types::{
    CliCommand, CommandKind, GraphFormat, MAX_FOLLOW_INTERVAL_SECS, MIN_FOLLOW_INTERVAL_SECS,
    ParsedArgs, RUN_ID_OPTION, RunIdArg, UsageError, UsageErrorKind as Kind,
};

/// オプションの識別子(`-h`・`--help`・`-V`・`--version` を除く。BR1.1)。
///
/// 並びは [`OPTIONS`] の行の並びと同じ(コンパイル時に検査する)。
#[derive(Clone, Copy, PartialEq, Eq)]
enum Opt {
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
struct OptSpec {
    opt: Opt,
    /// 長い名前(`--` を除く)。
    long: &'static str,
    /// 値を取るか(`--name value` と `--name=value`)。
    takes_value: bool,
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
fn spec(opt: Opt) -> &'static OptSpec {
    &OPTIONS[opt as usize]
}

/// 長い名前から、表の行を引く。
fn spec_by_long(name: &str) -> Option<&'static OptSpec> {
    OPTIONS.iter().find(|s| s.long == name)
}

impl OptSpec {
    /// コマンド `kind` で使えるか。`dry_run` は `--dry-run` が 1 回以上あるか(`--assume` の条件)、
    /// `follow` は `--follow` が 1 回以上あるか(`--interval` の条件)。
    fn allows(&self, kind: CommandKind, dry_run: bool, follow: bool) -> bool {
        match self.allowed {
            Allowed::Commands(kinds) => kinds.contains(&kind),
            Allowed::WithDryRun => dry_run,
            Allowed::WithFollow => follow,
        }
    }
}

impl Opt {
    /// 誤りの対象の綴り。`-v` だけ短い形、ほかは `--<長い名前>`。
    fn subject(self) -> String {
        match self {
            Opt::Verbose => "-v".to_string(),
            _ => format!("--{}", spec(self).long),
        }
    }
}

/// 検査の順の表の 1 つ(BR1.3)。
#[derive(Clone, Copy)]
enum Check {
    /// 1 回以上あり、そのコマンドで使えない → `optionNotAllowed`(対象は綴り)。
    NotAllowed(Opt),
    /// 2 回以上ある → `invalidValue`(対象は綴り)。
    Duplicate(Opt),
}

/// 「使えない」と「重複」の検査の順(BR1.3)。仕様書 04 章 4.2 の 6・7 の順を写す。
/// 誤りの報告の順は、この並びで決まる。`--assume` は重複を許す(同じジョブID の重複は、値の検査で誤り)。
const CHECKS: &[Check] = &[
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
struct Given(Vec<Vec<Option<OsString>>>);

impl Given {
    fn new() -> Self {
        Given(OPTIONS.iter().map(|_| Vec::new()).collect())
    }

    fn push(&mut self, opt: Opt, value: Option<OsString>) {
        self.0[opt as usize].push(value);
    }

    /// 指定の回数。
    fn count(&self, opt: Opt) -> usize {
        self.0[opt as usize].len()
    }

    /// ちょうど 1 回か(CHECKS のあとは、`--assume` 以外は 0 回か 1 回)。
    fn once(&self, opt: Opt) -> bool {
        self.count(opt) == 1
    }

    /// 値を、指定の順に取り出す。
    fn take(&mut self, opt: Opt) -> Vec<OsString> {
        std::mem::take(&mut self.0[opt as usize])
            .into_iter()
            .flatten()
            .collect()
    }
}

/// 字句に分けた 1 つの引数。
enum Token {
    /// 表にあるオプション。値を取るオプションなら `Some`(値は、字句の段階では検査しない)。
    Opt(Opt, Option<OsString>),
    /// 文法にないオプション(表示用の綴り)。
    Unknown(String),
    Positional(OsString),
}

/// 特別なオプション(コマンドの解釈より先に処理する)。
#[derive(Clone, Copy)]
enum Special {
    Help,
    Version,
}

/// 引数(プログラム名を除く)を解釈する。
///
/// # Errors
///
/// 使い方の誤りは [`UsageError`] で返す(ジョブは実行されない。終了コード 68)。
pub fn parse_args<I>(args: I) -> Result<ParsedArgs, UsageError>
where
    I: IntoIterator<Item = OsString>,
{
    let mut parser = lexopt::Parser::from_args(args);
    let mut tokens: Vec<Token> = Vec::new();
    let mut special: Option<Special> = None;
    let mut lex_error: Option<UsageError> = None;

    loop {
        let arg = match parser.next() {
            Ok(Some(arg)) => arg,
            Ok(None) => break,
            // 値の取り残し(`--bogus=x`)。最初の誤りを記録し、解釈を続ける。
            Err(lexopt::Error::UnexpectedValue { option, .. }) => {
                lex_error.get_or_insert_with(|| map_unexpected_value(&option));
                continue;
            }
            // lexopt 0.3.2 の `next` は `UnexpectedValue` しか返さない。版を上げたときの守り
            // (`lexopt::Error` の枝を網羅するため残す)。続けられない誤りとして、そこで終える。
            Err(_) => {
                lex_error.get_or_insert(UsageError::new(Kind::InvalidValue));
                break;
            }
        };
        match arg {
            Short('h') | Long("help") => {
                special.get_or_insert(Special::Help);
            }
            Short('V') | Long("version") => {
                special.get_or_insert(Special::Version);
            }
            Short('v') => tokens.push(Token::Opt(Opt::Verbose, None)),
            Short(c) => tokens.push(Token::Unknown(format!("-{c}"))),
            // 長い名前は表で引き、値を取るかで分ける(BR1.2)。
            Long(name) => match spec_by_long(name) {
                Some(spec) if spec.takes_value => match parser.value() {
                    Ok(v) => tokens.push(Token::Opt(spec.opt, Some(v))),
                    Err(_) => {
                        lex_error.get_or_insert(UsageError::with_subject(
                            Kind::MissingValue,
                            spec.opt.subject(),
                        ));
                        break;
                    }
                },
                Some(spec) => tokens.push(Token::Opt(spec.opt, None)),
                None => tokens.push(Token::Unknown(format!("--{name}"))),
            },
            Value(v) => tokens.push(Token::Positional(v)),
        }
    }

    match special {
        Some(Special::Help) => return Ok(ParsedArgs::Help),
        Some(Special::Version) => return Ok(ParsedArgs::Version),
        None => {}
    }
    if let Some(e) = lex_error {
        return Err(e);
    }
    build_command(tokens)
}

/// `lexopt` の値の取り残し(`UnexpectedValue`)を、使い方の誤りにする。
///
/// `--bogus=x` は知らないオプション、`--verbose=x` は誤った値。
/// 知っている、値を取らないオプションは、表の `takes_value` が偽の行と `-v`(BR1.2)。
fn map_unexpected_value(option: &str) -> UsageError {
    let known = option == "-v"
        || option
            .strip_prefix("--")
            .is_some_and(|long| spec_by_long(long).is_some_and(|spec| !spec.takes_value));
    if known {
        UsageError::with_subject(Kind::InvalidValue, option.to_string())
    } else {
        UsageError::with_subject(Kind::UnknownOption, option.to_string())
    }
}

/// 位置引数を、文字列にする。UTF-8 でなければ `invalidValue`。
fn to_text(value: OsString) -> Result<String, UsageError> {
    value
        .into_string()
        .map_err(|v| UsageError::with_subject(Kind::InvalidValue, v.to_string_lossy().into_owned()))
}

fn build_command(tokens: Vec<Token>) -> Result<ParsedArgs, UsageError> {
    let mut given = Given::new();
    let mut unknown: Option<String> = None;
    let mut positionals: Vec<OsString> = Vec::new();
    for token in tokens {
        match token {
            Token::Opt(opt, value) => given.push(opt, value),
            Token::Unknown(name) => {
                unknown.get_or_insert(name);
            }
            Token::Positional(v) => positionals.push(v),
        }
    }

    // 知らないオプションは、コマンドより先に誤りにする。
    if let Some(name) = unknown {
        return Err(UsageError::with_subject(Kind::UnknownOption, name));
    }

    let mut positionals = positionals.into_iter();
    let Some(first) = positionals.next() else {
        return Err(UsageError::new(Kind::MissingArgument));
    };
    let first = to_text(first)?;
    let Some(kind) = CommandKind::from_name(&first) else {
        return Err(UsageError::with_subject(Kind::UnknownCommand, first));
    };

    // 許されない場所のオプションと重複。CHECKS の順に検査し、最初の誤りを報告する(BR1.3)。
    let dry_run = given.count(Opt::DryRun) > 0;
    let follow = given.count(Opt::Follow) > 0;
    for check in CHECKS {
        match *check {
            Check::NotAllowed(opt) => {
                if given.count(opt) > 0 && !spec(opt).allows(kind, dry_run, follow) {
                    return Err(UsageError::with_subject(
                        Kind::OptionNotAllowed,
                        opt.subject(),
                    ));
                }
            }
            Check::Duplicate(opt) => {
                if given.count(opt) > 1 {
                    return Err(UsageError::with_subject(Kind::InvalidValue, opt.subject()));
                }
            }
        }
    }
    // ここから先、`--assume` 以外のオプションは 0 回か 1 回。

    let run_id = given.take(Opt::RunId).into_iter().next();
    // `--all` と `--run-id` は、一緒に使えない。
    if given.once(Opt::All) && run_id.is_some() {
        return Err(UsageError::with_subject(
            Kind::InvalidValue,
            Opt::All.subject(),
        ));
    }

    // `graph` の `--run-id` は、`--state` と一緒のときだけ(BR4.5)。
    if kind == CommandKind::Graph && run_id.is_some() && !given.once(Opt::State) {
        return Err(UsageError::with_subject(
            Kind::OptionNotAllowed,
            Opt::RunId.subject(),
        ));
    }

    let mut command = CliCommand::new(kind);
    command.verbose = given.once(Opt::Verbose);
    command.force = given.once(Opt::Force);
    command.config_path = given.take(Opt::Config).into_iter().next();
    command.all = given.once(Opt::All);
    command.remove_on_complete = given.once(Opt::RemoveOnComplete);
    command.dry_run.dry_run = given.once(Opt::DryRun);
    command.dry_run.assumptions = parse_assumptions(given.take(Opt::Assume))?;
    command.graph.with_state = given.once(Opt::State);
    if let Some(value) = given.take(Opt::Format).into_iter().next() {
        command.graph.format = parse_graph_format(&value)?;
    }
    command.follow = given.once(Opt::Follow);
    command.interval = given
        .take(Opt::Interval)
        .into_iter()
        .next()
        .map(|value| parse_interval(&value))
        .transpose()?;
    // 値の検査は、ここで行う。誤りの文に、値は含めない(対象は、オプションの名前だけ)。
    command.run_id = run_id.map(parse_run_id_option).transpose()?;
    let to = given.take(Opt::To);

    let operand = positionals.next();
    let extra = positionals.next();
    match (operand_rule(kind), &operand) {
        (Operand::Required, None) => return Err(UsageError::new(Kind::MissingArgument)),
        (Operand::Forbidden, Some(unexpected)) => return Err(unexpected_argument(unexpected)),
        _ => {}
    }
    if let Some(unexpected) = &extra {
        return Err(unexpected_argument(unexpected));
    }

    match kind {
        CommandKind::Run => command.job_id = operand.map(to_text).transpose()?,
        CommandKind::Pipeline | CommandKind::Validate | CommandKind::Graph => {
            command.pipeline_name = operand.map(to_text).transpose()?;
        }
        CommandKind::Convert => {
            command.convert_input = operand;
            let Some(target) = to.into_iter().next() else {
                return Err(UsageError::with_subject(
                    Kind::MissingArgument,
                    Opt::To.subject(),
                ));
            };
            command.convert_target = Some(parse_target(target)?);
        }
        CommandKind::Init => {
            if let Some(name) = operand.map(to_text).transpose()? {
                // 定義の検査と同じ規則(空でない。空白だけでもない)。値は、誤りの文に含めない(BR2.2)。
                if name.trim().is_empty() {
                    return Err(UsageError::with_subject(
                        Kind::InvalidValue,
                        "pipeline name",
                    ));
                }
                command.pipeline_name = Some(name);
            }
        }
        CommandKind::Watch | CommandKind::Reset => {}
    }
    Ok(ParsedArgs::Command(command))
}

/// `--assume` の値の検査(BR2.1)。`<ジョブID>=<終了コード>`。最初の `=` で分ける。
/// 誤りの文には、入力の値を含めない(対象は `--assume` だけ)。
fn parse_assumptions(values: Vec<OsString>) -> Result<Vec<Assumption>, UsageError> {
    let invalid = || UsageError::with_subject(Kind::InvalidValue, Opt::Assume.subject());
    let mut out: Vec<Assumption> = Vec::with_capacity(values.len());
    for value in values {
        let text = value.into_string().map_err(|_| invalid())?;
        let (job_id, code) = text.split_once('=').ok_or_else(invalid)?;
        let code: i32 = code.parse().map_err(|_| invalid())?;
        if !Assumption::exit_code_in_range(code) {
            return Err(invalid());
        }
        if out.iter().any(|a| a.job_id == job_id) {
            return Err(invalid());
        }
        out.push(Assumption::new(job_id, code));
    }
    Ok(out)
}

/// `--format` の値の検査。`text`、`mermaid`、`dot` のどれか(BR1.1)。誤りの文に、値は含めない。
fn parse_graph_format(value: &OsString) -> Result<GraphFormat, UsageError> {
    match value.to_str() {
        Some("text") => Ok(GraphFormat::Text),
        Some("mermaid") => Ok(GraphFormat::Mermaid),
        Some("dot") => Ok(GraphFormat::Dot),
        _ => Err(UsageError::with_subject(
            Kind::InvalidValue,
            Opt::Format.subject(),
        )),
    }
}

/// `--interval` の値の検査(v0.10.0 の U7 の BR1.2)。1〜3600 の整数の秒。数字だけを受け付ける(符号、小数点、
/// 空白、空は誤り)。誤りの文に、値は含めない(対象は `--interval` だけ)。
fn parse_interval(value: &OsString) -> Result<u32, UsageError> {
    let invalid = || UsageError::with_subject(Kind::InvalidValue, Opt::Interval.subject());
    let text = value.to_str().ok_or_else(invalid)?;
    if text.is_empty() || !text.bytes().all(|b| b.is_ascii_digit()) {
        return Err(invalid());
    }
    let secs: u32 = text.parse().map_err(|_| invalid())?;
    if (MIN_FOLLOW_INTERVAL_SECS..=MAX_FOLLOW_INTERVAL_SECS).contains(&secs) {
        Ok(secs)
    } else {
        Err(invalid())
    }
}

/// `--run-id` の値の検査。空は有効(run ID なしに固定する)。UTF-8 でない値、規則に反する値は、
/// `invalidValue`(対象は `--run-id` で、値は含めない)。
fn parse_run_id_option(value: OsString) -> Result<RunIdArg, UsageError> {
    let text = value
        .into_string()
        .map_err(|_| UsageError::with_subject(Kind::InvalidValue, Opt::RunId.subject()))?;
    RunIdArg::parse(&text)
}

/// 位置引数(オペランド)の扱い(BR1.1。文法の表の「位置引数」の列)。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Operand {
    /// 必須(無ければ `missingArgument`)。
    Required,
    /// 省略できる。
    Optional,
    /// 書けない(書けば `unexpectedArgument`)。
    Forbidden,
}

/// コマンドごとの位置引数の扱い。新しいコマンドを足したら、ここで決める(網羅の `match`)。
fn operand_rule(kind: CommandKind) -> Operand {
    match kind {
        CommandKind::Run | CommandKind::Pipeline | CommandKind::Convert => Operand::Required,
        CommandKind::Validate | CommandKind::Graph | CommandKind::Init => Operand::Optional,
        CommandKind::Watch | CommandKind::Reset => Operand::Forbidden,
    }
}

fn unexpected_argument(value: &OsString) -> UsageError {
    UsageError::with_subject(
        Kind::UnexpectedArgument,
        value.to_string_lossy().into_owned(),
    )
}

/// `--to` の値の検査。`yaml` か `toml`(大文字小文字を区別する)。UTF-8 でない値、ほかの値は、
/// `invalidValue`(対象は `--to` で、入力の値は含めない。BR3.1)。
fn parse_target(value: OsString) -> Result<TargetFormat, UsageError> {
    match value.to_str() {
        Some("yaml") => Ok(TargetFormat::Yaml),
        Some("toml") => Ok(TargetFormat::Toml),
        _ => Err(UsageError::with_subject(
            Kind::InvalidValue,
            Opt::To.subject(),
        )),
    }
}
