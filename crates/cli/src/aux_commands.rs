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

//! 補助のコマンド(W4〜W6。BR7.1〜BR7.3): `watch`、`reset`、`validate`、`convert`。
//!
//! シグナルの準備は行わない(既定の動作で終わる。BR4.1)。

use std::path::PathBuf;

use conductor_converter::TargetFormat;
use conductor_core::PipelineDefinition;

use crate::ctx::{Ctx, aux_run_id, definition_path, read_context};
use crate::exit::{EXIT_CONVERSION, EXIT_OK, EXIT_STATE, EXIT_USAGE};
use crate::report::{DisplayEvent, StateContext};
use crate::types::{CliCommand, UsageError, UsageErrorKind};

/// `conductor watch`: ステートを 1 回だけ一覧にして終わる(BR7.1)。
///
/// 定義ファイルは検証しない。環境変数の名前だけは、`--run-id` がないとき、定義から、ゆるく読む(FR4.3)。
pub(crate) fn run_watch(ctx: &mut Ctx<'_>, cmd: &CliCommand) -> i32 {
    let path = definition_path(cmd);
    let run_id = match aux_run_id(ctx, cmd, &path) {
        Ok(run_id) => run_id,
        Err(error) => {
            ctx.show(&DisplayEvent::Usage(&error));
            return EXIT_USAGE;
        }
    };
    let context = read_context(run_id.is_some());
    // 完了のステートも読む(FR3.1)。`WatchEmpty` は、ファイルなし・長さ 0 のときだけ。
    let state = match ctx.runtime.read_state_raw(&path, run_id.as_ref()) {
        Ok(state) => state,
        Err(error) => return ctx.state_error(&error, context),
    };
    let written = match &state {
        Some(state) => ctx.show(&DisplayEvent::WatchTable(state)),
        None => ctx.show(&DisplayEvent::WatchEmpty),
    };
    if written {
        EXIT_OK
    } else {
        // 標準出力に書けない(閉じたパイプなど)。パニックにせず、65 で終わる(security-design の差分)。
        ctx.show(&DisplayEvent::OutputFailed);
        EXIT_STATE
    }
}

/// `conductor reset`: ステートを消す(BR7.2)。読み込みも検査も迂回する。ステートがなくても成功。
///
/// `--all` のときは、その定義のステートを、run ID の有無によらず、すべて消す(FR5)。
/// 環境変数は見ない。読めなくて残したファイルは、標準エラーに警告する(終了コードは 0)。
pub(crate) fn run_reset(ctx: &mut Ctx<'_>, cmd: &CliCommand) -> i32 {
    let path = definition_path(cmd);
    if cmd.all {
        return match ctx.runtime.reset_all_states(&path) {
            Ok(report) => {
                if !report.kept.is_empty() {
                    ctx.show(&DisplayEvent::ResetAllKept {
                        names: &report.kept,
                    });
                }
                EXIT_OK
            }
            Err(error) => ctx.state_error(&error, StateContext::Plain),
        };
    }
    let run_id = match aux_run_id(ctx, cmd, &path) {
        Ok(run_id) => run_id,
        Err(error) => {
            ctx.show(&DisplayEvent::Usage(&error));
            return EXIT_USAGE;
        }
    };
    match ctx.runtime.reset_state(&path, run_id.as_ref()) {
        Ok(()) => EXIT_OK,
        Err(error) => ctx.state_error(&error, StateContext::Plain),
    }
}

/// `validate` と `graph` が共有する検査(BR6.1): 定義の検証(64)、全ジョブの `command` の検査(64)、
/// `.env` の読み取り(64)。重複キーの警告は、誤りでないので、標準エラーに出して続ける(BR7.2)。
///
/// ジョブは実行しない。シグナルの準備、ステートの読み書き、リセットもしない。値は、どこにも出さない。
/// 誤りは、`run` と同じ表示で、終了コードを `Err` で返す。
pub(crate) fn check_definition_and_env(
    ctx: &mut Ctx<'_>,
    cmd: &CliCommand,
) -> Result<(PathBuf, PipelineDefinition), i32> {
    let (path, definition) = ctx.check_definition(cmd, cmd.pipeline_name.as_deref())?;
    // `pipeline` と同じ範囲の `.env` を読む(BR7.1)。重複キーの警告は、誤りでないので 0(BR7.2)。
    let loaded = ctx.load_env_files(&path, &definition, None)?;
    let warnings: Vec<_> = loaded
        .files()
        .iter()
        .flat_map(|f| f.warnings.iter().cloned())
        .collect();
    if !warnings.is_empty() {
        ctx.show(&DisplayEvent::EnvWarnings(&warnings));
    }
    Ok((path, definition))
}

/// `conductor validate [<name>]`: 定義の検証と、全ジョブの `command` の検査だけを行う。
///
/// ジョブは実行しない。シグナルの準備、ステートの読み書き、リセットもしない。
/// 成功は何も出さない(`-v` のときだけ、標準エラーに 1 行)。誤りは `run` と同じ表示で 64。
pub(crate) fn run_validate(ctx: &mut Ctx<'_>, cmd: &CliCommand) -> i32 {
    match check_definition_and_env(ctx, cmd) {
        Ok((_, definition)) => {
            ctx.show(&DisplayEvent::DefinitionValid {
                job_count: definition.job.len(),
            });
            EXIT_OK
        }
        Err(code) => code,
    }
}

/// `conductor convert <file> --to <format>`: 変換結果を、全部作ってから、標準出力に 1 回で書く(BR7.3)。
pub(crate) fn run_convert(ctx: &mut Ctx<'_>, cmd: &CliCommand) -> i32 {
    let (Some(input), Some(target)) = (&cmd.convert_input, cmd.convert_target) else {
        let error = UsageError::new(UsageErrorKind::MissingArgument);
        ctx.show(&DisplayEvent::Usage(&error));
        return EXIT_USAGE;
    };
    let target: TargetFormat = target;
    match ctx.runtime.convert(std::path::Path::new(input), target) {
        Ok(text) => match ctx.sink.write_out(text.as_bytes()) {
            Ok(()) => EXIT_OK,
            Err(_) => {
                ctx.show(&DisplayEvent::OutputFailed);
                EXIT_CONVERSION
            }
        },
        Err(error) => {
            // 失敗のとき、標準出力には何も書かない(NFR4.4)。
            ctx.show(&DisplayEvent::Conversion(&error));
            EXIT_CONVERSION
        }
    }
}
