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
//! | コマンド | 位置引数 | `-v` | `--config` | `--to` | `--run-id` | `--all` |
//! |----------|----------|------|------------|--------|------------|---------|
//! | `run` | ジョブID(1) | 可 | 可 | 不可 | 可 | 不可 |
//! | `pipeline` | パイプライン名(1) | 可 | 可 | 不可 | 可 | 不可 |
//! | `watch` | なし | 不可 | 可 | 不可 | 可 | 不可 |
//! | `reset` | なし | 不可 | 可 | 不可 | 可 | 可(`--run-id` とは併用不可) |
//! | `validate` | パイプライン名(0 か 1) | 可 | 可 | 不可 | 不可 | 不可 |
//! | `convert` | 入力ファイル(1) | 不可 | 不可 | 必須(`yaml` か `toml`) | 不可 | 不可 |
//!
//! - `--config path` と `--config=path`(`--to`、`--run-id` も同じ)の両方を受け付ける。
//! - `--run-id` の値は、空(run ID なしに固定)か、`RunId` の規則(`A-Za-z0-9._-`、64 文字まで、
//!   先頭が `.` でない)を満たすこと。違反は `invalidValue`(値は、誤りの文に含めない)。
//! - `--` は、オプションの終わりで、以降は、すべて位置引数。
//! - `-v` は、ちょうど1回。2回以上(`-vv` を含む)は `invalidValue`。`--config`、`--to` の重複も同じ。
//! - `--help`・`-h`・`--version`・`-V` は、`--` より前に現れたとき、ほかの誤りより先に処理する。

use std::ffi::OsString;

use conductor_converter::TargetFormat;
use conductor_core::RunId;
use lexopt::prelude::*;

use crate::deps::Env;
use crate::types::{CliCommand, CommandKind, ParsedArgs, UsageError, UsageErrorKind as Kind};

/// 字句に分けた 1 つの引数。
enum Token {
    Verbose,
    Config(OsString),
    To(OsString),
    /// `--run-id <値>`(値は、字句の段階では、検査しない)。
    RunId(OsString),
    /// `--all`。
    All,
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
            Err(e) => {
                if lex_error.is_none() {
                    lex_error = Some(map_lex_error(&e));
                }
                // 値の取り残し(`--bogus=x`)の後も、解釈は続けられる。続けられない誤りは、そこで終える。
                if matches!(e, lexopt::Error::UnexpectedValue { .. }) {
                    continue;
                }
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
            Short('v') | Long("verbose") => tokens.push(Token::Verbose),
            Long("config") => match parser.value() {
                Ok(v) => tokens.push(Token::Config(v)),
                Err(_) => {
                    lex_error
                        .get_or_insert(UsageError::with_subject(Kind::MissingValue, "--config"));
                    break;
                }
            },
            Long("run-id") => match parser.value() {
                Ok(v) => tokens.push(Token::RunId(v)),
                Err(_) => {
                    lex_error
                        .get_or_insert(UsageError::with_subject(Kind::MissingValue, "--run-id"));
                    break;
                }
            },
            Long("all") => tokens.push(Token::All),
            Long("to") => match parser.value() {
                Ok(v) => tokens.push(Token::To(v)),
                Err(_) => {
                    lex_error.get_or_insert(UsageError::with_subject(Kind::MissingValue, "--to"));
                    break;
                }
            },
            Short(c) => tokens.push(Token::Unknown(format!("-{c}"))),
            Long(name) => tokens.push(Token::Unknown(format!("--{name}"))),
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

/// `lexopt` の誤りを、使い方の誤りにする。
fn map_lex_error(e: &lexopt::Error) -> UsageError {
    match e {
        lexopt::Error::MissingValue { .. } => UsageError::new(Kind::MissingValue),
        lexopt::Error::UnexpectedValue { option, .. } => {
            // `--bogus=x` は知らないオプション、`--verbose=x` は誤った値。
            let known = matches!(option.as_str(), "--verbose" | "-v" | "--all");
            if known {
                UsageError::with_subject(Kind::InvalidValue, option.clone())
            } else {
                UsageError::with_subject(Kind::UnknownOption, option.clone())
            }
        }
        _ => UsageError::new(Kind::InvalidValue),
    }
}

/// 位置引数を、文字列にする。UTF-8 でなければ `invalidValue`。
fn to_text(value: OsString) -> Result<String, UsageError> {
    value
        .into_string()
        .map_err(|v| UsageError::with_subject(Kind::InvalidValue, v.to_string_lossy().into_owned()))
}

fn build_command(tokens: Vec<Token>) -> Result<ParsedArgs, UsageError> {
    let mut verbose_count = 0usize;
    let mut config: Vec<OsString> = Vec::new();
    let mut to: Vec<OsString> = Vec::new();
    let mut run_ids: Vec<OsString> = Vec::new();
    let mut all_count = 0usize;
    let mut unknown: Option<String> = None;
    let mut positionals: Vec<OsString> = Vec::new();
    for token in tokens {
        match token {
            Token::Verbose => verbose_count += 1,
            Token::Config(v) => config.push(v),
            Token::To(v) => to.push(v),
            Token::RunId(v) => run_ids.push(v),
            Token::All => all_count += 1,
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
    let kind = match first.as_str() {
        "run" => CommandKind::Run,
        "pipeline" => CommandKind::Pipeline,
        "watch" => CommandKind::Watch,
        "reset" => CommandKind::Reset,
        "validate" => CommandKind::Validate,
        "convert" => CommandKind::Convert,
        _ => return Err(UsageError::with_subject(Kind::UnknownCommand, first)),
    };

    // 許されない場所のオプション。
    let takes_verbose = matches!(
        kind,
        CommandKind::Run | CommandKind::Pipeline | CommandKind::Validate
    );
    let takes_config = !matches!(kind, CommandKind::Convert);
    let takes_to = matches!(kind, CommandKind::Convert);
    if verbose_count > 0 && !takes_verbose {
        return Err(UsageError::with_subject(Kind::OptionNotAllowed, "-v"));
    }
    if !config.is_empty() && !takes_config {
        return Err(UsageError::with_subject(Kind::OptionNotAllowed, "--config"));
    }
    if !to.is_empty() && !takes_to {
        return Err(UsageError::with_subject(Kind::OptionNotAllowed, "--to"));
    }
    let takes_run_id = matches!(
        kind,
        CommandKind::Run | CommandKind::Pipeline | CommandKind::Watch | CommandKind::Reset
    );
    if !run_ids.is_empty() && !takes_run_id {
        return Err(UsageError::with_subject(Kind::OptionNotAllowed, "--run-id"));
    }
    if all_count > 0 && kind != CommandKind::Reset {
        return Err(UsageError::with_subject(Kind::OptionNotAllowed, "--all"));
    }
    // 重複。
    if verbose_count > 1 {
        return Err(UsageError::with_subject(Kind::InvalidValue, "-v"));
    }
    if config.len() > 1 {
        return Err(UsageError::with_subject(Kind::InvalidValue, "--config"));
    }
    if to.len() > 1 {
        return Err(UsageError::with_subject(Kind::InvalidValue, "--to"));
    }
    if run_ids.len() > 1 {
        return Err(UsageError::with_subject(Kind::InvalidValue, "--run-id"));
    }
    if all_count > 1 {
        return Err(UsageError::with_subject(Kind::InvalidValue, "--all"));
    }
    // `--all` と `--run-id` は、一緒に使えない。
    if all_count == 1 && !run_ids.is_empty() {
        return Err(UsageError::with_subject(Kind::InvalidValue, "--all"));
    }

    let mut command = CliCommand::new(kind);
    command.verbose = verbose_count == 1;
    command.config_path = config.into_iter().next();
    command.all = all_count == 1;
    // 値の検査は、ここで行う。誤りの文に、値は含めない(対象は、オプションの名前だけ)。
    command.run_id = run_ids
        .into_iter()
        .next()
        .map(parse_run_id_option)
        .transpose()?;

    // `validate` の位置引数(パイプライン名)は、省略できる。
    let wants_positional = !matches!(
        kind,
        CommandKind::Watch | CommandKind::Reset | CommandKind::Validate
    );
    let allows_operand = !matches!(kind, CommandKind::Watch | CommandKind::Reset);
    let operand = positionals.next();
    let extra = positionals.next();
    if wants_positional {
        if operand.is_none() {
            return Err(UsageError::new(Kind::MissingArgument));
        }
    } else if !allows_operand {
        if let Some(unexpected) = &operand {
            return Err(unexpected_argument(unexpected));
        }
    }
    if let Some(unexpected) = &extra {
        return Err(unexpected_argument(unexpected));
    }

    match kind {
        CommandKind::Run => command.job_id = operand.map(to_text).transpose()?,
        CommandKind::Pipeline | CommandKind::Validate => {
            command.pipeline_name = operand.map(to_text).transpose()?;
        }
        CommandKind::Convert => {
            command.convert_input = operand;
            let Some(target) = to.into_iter().next() else {
                return Err(UsageError::with_subject(Kind::MissingArgument, "--to"));
            };
            command.convert_target = Some(parse_target(target)?);
        }
        CommandKind::Watch | CommandKind::Reset => {}
    }
    Ok(ParsedArgs::Command(command))
}

/// `--run-id` の値の検査。空は有効(run ID なしに固定する)。UTF-8 でない値、規則に反する値は、
/// `invalidValue`(対象は `--run-id` で、値は含めない)。
fn parse_run_id_option(value: OsString) -> Result<String, UsageError> {
    let invalid = || UsageError::with_subject(Kind::InvalidValue, "--run-id");
    let text = value.into_string().map_err(|_| invalid())?;
    if !text.is_empty() {
        RunId::parse(&text).map_err(|_| invalid())?;
    }
    Ok(text)
}

/// run ID を決める(FR1.2〜FR1.4)。
///
/// - `option` が `Some`(`--run-id` あり): 環境変数は見ない。空なら run ID なし(固定)。
/// - `option` が `None`: 環境変数 `env_name` を読み、未定義と空は run ID なし。
///
/// `--run-id` の値は、引数の段階で検査済みだが、ここでも検査する(環境変数の値と同じ関数)。
/// 規則に反する値は、`invalidValue`(対象は、どこから来たか=オプションか環境変数かだけ。値は含めない)。
pub fn resolve_run_id(
    option: Option<&str>,
    env_name: &str,
    env: &dyn Env,
) -> Result<Option<RunId>, UsageError> {
    let (value, subject) = match option {
        Some(v) => (Some(v.to_string()), "--run-id"),
        None => (env.var(env_name), "run ID (environment variable)"),
    };
    match value.as_deref() {
        None | Some("") => Ok(None),
        Some(v) => RunId::parse(v)
            .map(Some)
            .map_err(|_| UsageError::with_subject(Kind::InvalidValue, subject)),
    }
}

fn unexpected_argument(value: &OsString) -> UsageError {
    UsageError::with_subject(
        Kind::UnexpectedArgument,
        value.to_string_lossy().into_owned(),
    )
}

fn parse_target(value: OsString) -> Result<TargetFormat, UsageError> {
    match value.to_str() {
        Some("yaml") => Ok(TargetFormat::Yaml),
        Some("toml") => Ok(TargetFormat::Toml),
        _ => Err(UsageError::with_subject(
            Kind::InvalidValue,
            value.to_string_lossy().into_owned(),
        )),
    }
}
