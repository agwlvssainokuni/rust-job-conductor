//! 引数の解釈(ArgParser。NFR4.1、BR1.1〜BR1.6)。
//!
//! `lexopt` で、引数を字句(オプションの名前、値、位置引数)に分け、コマンドごとの文法の表で検査する。
//! 引数は、シェルを介さず、解釈も展開もしない。パスの値(`--config`、convert の入力)は、
//! `OsString` のまま保つ。
//!
//! # 文法
//!
//! | コマンド | 位置引数 | `-v` | `--config` | `--to` |
//! |----------|----------|------|------------|--------|
//! | `run` | ジョブID(1) | 可 | 可 | 不可 |
//! | `pipeline` | パイプライン名(1) | 可 | 可 | 不可 |
//! | `watch` | なし | 不可 | 可 | 不可 |
//! | `reset` | なし | 不可 | 可 | 不可 |
//! | `convert` | 入力ファイル(1) | 不可 | 不可 | 必須(`yaml` か `toml`) |
//!
//! - `--config path` と `--config=path`(`--to` も同じ)の両方を受け付ける。
//! - `--` は、オプションの終わりで、以降は、すべて位置引数。
//! - `-v` は、ちょうど1回。2回以上(`-vv` を含む)は `invalidValue`。`--config`、`--to` の重複も同じ。
//! - `--help`・`-h`・`--version`・`-V` は、`--` より前に現れたとき、ほかの誤りより先に処理する。

use std::ffi::OsString;

use conductor_converter::TargetFormat;
use lexopt::prelude::*;

use crate::types::{CliCommand, CommandKind, ParsedArgs, UsageError, UsageErrorKind as Kind};

/// 字句に分けた 1 つの引数。
enum Token {
    Verbose,
    Config(OsString),
    To(OsString),
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
            let known = matches!(option.as_str(), "--verbose" | "-v");
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
    let mut unknown: Option<String> = None;
    let mut positionals: Vec<OsString> = Vec::new();
    for token in tokens {
        match token {
            Token::Verbose => verbose_count += 1,
            Token::Config(v) => config.push(v),
            Token::To(v) => to.push(v),
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
        "convert" => CommandKind::Convert,
        _ => return Err(UsageError::with_subject(Kind::UnknownCommand, first)),
    };

    // 許されない場所のオプション。
    let takes_verbose = matches!(kind, CommandKind::Run | CommandKind::Pipeline);
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

    let mut command = CliCommand::new(kind);
    command.verbose = verbose_count == 1;
    command.config_path = config.into_iter().next();

    let wants_positional = !matches!(kind, CommandKind::Watch | CommandKind::Reset);
    let operand = positionals.next();
    let extra = positionals.next();
    if wants_positional {
        if operand.is_none() {
            return Err(UsageError::new(Kind::MissingArgument));
        }
    } else if let Some(unexpected) = &operand {
        return Err(unexpected_argument(unexpected));
    }
    if let Some(unexpected) = &extra {
        return Err(unexpected_argument(unexpected));
    }

    match kind {
        CommandKind::Run => command.job_id = operand.map(to_text).transpose()?,
        CommandKind::Pipeline => command.pipeline_name = operand.map(to_text).transpose()?,
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
