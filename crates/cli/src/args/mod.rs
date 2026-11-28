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

mod options;
mod tokens;
mod values;

use std::ffi::OsString;

use lexopt::prelude::*;

use crate::types::{CliCommand, CommandKind, ParsedArgs, UsageError, UsageErrorKind as Kind};
use options::{
    CHECKS, Check, Given, Operand, Opt, operand_rule, spec, spec_by_long, unexpected_argument,
};
use tokens::{Special, Token, map_unexpected_value, to_text};
use values::{
    parse_assumptions, parse_graph_format, parse_interval, parse_run_id_option, parse_target,
};

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

/// 字句から、使い方の指定を組み立てる。
///
/// 次の順に関数を呼び、最初の誤りを返す(誤りの報告の順は、この順と、各関数の中の順で決まる)。
///
/// 1. 字句を集める(知らないオプション)
/// 2. コマンドの名前
/// 3. オプションの「使えない」と「重複」、組み合わせ
/// 4. オプションの値
/// 5. 位置引数の数
/// 6. コマンドごとの位置引数
fn build_command(tokens: Vec<Token>) -> Result<ParsedArgs, UsageError> {
    let (mut given, positionals) = collect_tokens(tokens)?;
    let mut positionals = positionals.into_iter();
    let kind = command_kind(&mut positionals)?;
    check_options(&given, kind)?;
    let (mut command, to) = fill_options(&mut given, kind)?;
    let operand = check_operands(kind, &mut positionals)?;
    fill_operand(kind, &mut command, operand, to)?;
    Ok(ParsedArgs::Command(command))
}

/// 手順 1: 字句を、オプションの指定と位置引数に分ける。知らないオプションがあれば、最初のものを誤りにする。
fn collect_tokens(tokens: Vec<Token>) -> Result<(Given, Vec<OsString>), UsageError> {
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
    Ok((given, positionals))
}

/// 手順 2: 位置引数の先頭から、コマンドの種類を決める。無ければ `missingArgument`、知らなければ
/// `unknownCommand`。
fn command_kind(positionals: &mut std::vec::IntoIter<OsString>) -> Result<CommandKind, UsageError> {
    let Some(first) = positionals.next() else {
        return Err(UsageError::new(Kind::MissingArgument));
    };
    let first = to_text(first)?;
    let Some(kind) = CommandKind::from_name(&first) else {
        return Err(UsageError::with_subject(Kind::UnknownCommand, first));
    };
    Ok(kind)
}

/// 手順 3: オプションの「使えない」と「重複」と、組み合わせの検査。値は取り出さない(数だけを見る)。
fn check_options(given: &Given, kind: CommandKind) -> Result<(), UsageError> {
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

    let has_run_id = given.count(Opt::RunId) > 0;
    // `--all` と `--run-id` は、一緒に使えない。
    if given.once(Opt::All) && has_run_id {
        return Err(UsageError::with_subject(
            Kind::InvalidValue,
            Opt::All.subject(),
        ));
    }

    // `graph` の `--run-id` は、`--state` と一緒のときだけ(BR4.5)。
    if kind == CommandKind::Graph && has_run_id && !given.once(Opt::State) {
        return Err(UsageError::with_subject(
            Kind::OptionNotAllowed,
            Opt::RunId.subject(),
        ));
    }
    Ok(())
}

/// 手順 4: オプションの値を、コマンドに入れる。値の検査の順は `--assume` → `--format` → `--interval` →
/// `--run-id`。`--to` の値は、コマンドごとの位置引数(`convert`)で使うので、そのまま返す。
fn fill_options(
    given: &mut Given,
    kind: CommandKind,
) -> Result<(CliCommand, Vec<OsString>), UsageError> {
    let run_id = given.take(Opt::RunId).into_iter().next();
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
    Ok((command, to))
}

/// 手順 5: 位置引数の数の検査。コマンドの名前の後ろの、1 つ目の位置引数を返す。
fn check_operands(
    kind: CommandKind,
    positionals: &mut std::vec::IntoIter<OsString>,
) -> Result<Option<OsString>, UsageError> {
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
    Ok(operand)
}

/// 手順 6: コマンドごとに、位置引数(と `convert` の `--to`)をコマンドに入れる。
fn fill_operand(
    kind: CommandKind,
    command: &mut CliCommand,
    operand: Option<OsString>,
    to: Vec<OsString>,
) -> Result<(), UsageError> {
    match kind {
        CommandKind::Run => command.job_id = operand.map(to_text).transpose()?,
        CommandKind::Pipeline | CommandKind::Validate | CommandKind::Graph => {
            command.pipeline_name = operand.map(to_text).transpose()?;
        }
        CommandKind::Convert => convert_operand(command, operand, to)?,
        CommandKind::Init => init_operand(command, operand)?,
        CommandKind::Watch | CommandKind::Reset => {}
    }
    Ok(())
}

/// `convert` の入力ファイルと、`--to` の形式。`--to` が無ければ `missingArgument`(対象は `--to`)。
fn convert_operand(
    command: &mut CliCommand,
    operand: Option<OsString>,
    to: Vec<OsString>,
) -> Result<(), UsageError> {
    command.convert_input = operand;
    let Some(target) = to.into_iter().next() else {
        return Err(UsageError::with_subject(
            Kind::MissingArgument,
            Opt::To.subject(),
        ));
    };
    command.convert_target = Some(parse_target(target)?);
    Ok(())
}

/// `init` のパイプライン名。空、または空白だけなら `invalidValue`。
fn init_operand(command: &mut CliCommand, operand: Option<OsString>) -> Result<(), UsageError> {
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
    Ok(())
}
