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

use conductor_converter::TargetFormat;

use crate::ctx::{Ctx, definition_path};
use crate::exit::{EXIT_CONVERSION, EXIT_OK, EXIT_STATE, EXIT_USAGE};
use crate::report::{DisplayEvent, StateContext};
use crate::types::{CliCommand, UsageError, UsageErrorKind};

/// `conductor watch`: ステートを 1 回だけ一覧にして終わる(BR7.1)。定義ファイルは読まない。
pub(crate) fn run_watch(ctx: &mut Ctx<'_>, cmd: &CliCommand) -> i32 {
    let path = definition_path(cmd);
    let state = match ctx.runtime.read_state(&path) {
        Ok(state) => state,
        Err(error) => return ctx.state_error(&error, StateContext::Read),
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
pub(crate) fn run_reset(ctx: &mut Ctx<'_>, cmd: &CliCommand) -> i32 {
    let path = definition_path(cmd);
    match ctx.runtime.reset_state(&path) {
        Ok(()) => EXIT_OK,
        Err(error) => ctx.state_error(&error, StateContext::Plain),
    }
}

/// `conductor validate [<name>]`: 定義の検証と、全ジョブの `command` の検査だけを行う。
///
/// ジョブは実行しない。シグナルの準備、ステートの読み書き、リセットもしない。
/// 成功は何も出さない(`-v` のときだけ、標準エラーに 1 行)。誤りは `run` と同じ表示で 64。
pub(crate) fn run_validate(ctx: &mut Ctx<'_>, cmd: &CliCommand) -> i32 {
    match ctx.check_definition(cmd, cmd.pipeline_name.as_deref()) {
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
