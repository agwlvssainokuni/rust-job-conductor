//! 補助のコマンド(W4〜W6。BR7.1〜BR7.3): `watch`、`reset`、`convert`。
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
