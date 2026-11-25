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

//! 補助のコマンド(W4〜W6、WF6、U4 の WF2。BR7.1〜BR7.3): `watch`、`reset`、`validate`、`convert`、
//! `graph`、`init`。
//!
//! シグナルの準備は行わない(既定の動作で終わる。BR4.1)。`graph` と `init` の規則の番号は、
//! [`run_graph`]・[`run_init`] の説明にある。シグナルの準備をする `run`・`pipeline`・`--dry-run` は、
//! 1 コマンド 1 ファイル(`run.rs`、`pipeline.rs`、`dry_run.rs`)。

use std::path::Path;

use conductor_core::PipelineDefinition;
use conductor_core::graph::{
    StateOverlay, build_graph, layout_graph, overlay_from_state, pending_overlay,
};
use conductor_runtime::{CreateError, CreateOutcome};

use crate::ctx::{Ctx, StateRead, definition_path};
use crate::exit::{EXIT_CONVERSION, EXIT_EXISTS, EXIT_OK, EXIT_STATE};
use crate::report::graph_dot::render_dot;
use crate::report::graph_mermaid::render_mermaid;
use crate::report::graph_text::render_text;
use crate::report::init_template::render_template;
use crate::report::{DisplayEvent, StateContext};
use crate::run_id::aux_run_id;
use crate::types::{CliCommand, GraphFormat, InitTarget, UsageError, UsageErrorKind};

/// `conductor watch`: ステートを 1 回だけ一覧にして終わる(BR7.1)。所要時間の列のため、表を作る前に
/// 今の時刻を 1 回だけ取る(v0.10.0 の U6 の BR4.1)。
///
/// 定義ファイルは検証しない。環境変数の名前だけは、`--run-id` がないとき、定義から、ゆるく読む(FR4.3)。
pub(crate) fn run_watch(ctx: &mut Ctx<'_>, cmd: &CliCommand) -> i32 {
    let path = definition_path(cmd);
    let run_id = match aux_run_id(ctx, cmd, &path) {
        Ok(run_id) => run_id,
        Err(error) => return ctx.usage_error(&error),
    };
    let context = StateContext::for_read(run_id.as_ref());
    // 完了のステートも読む(FR3.1)。`WatchEmpty` は、ファイルなし・長さ 0 のときだけ。
    let state = match ctx.runtime.read_state_raw(&path, run_id.as_ref()) {
        Ok(state) => state,
        Err(error) => return ctx.state_error(&error, context),
    };
    // 今の時刻は、表を作る前に 1 回だけ取る(動いている最中のジョブの経過時間。v0.10.0 の U6 の BR4.1)。
    let output = match &state {
        Some(state) => DisplayEvent::WatchTable {
            state,
            now: ctx.runtime.now(),
        },
        None => DisplayEvent::WatchEmpty,
    };
    // 標準出力に書けない(閉じたパイプなど)。パニックにせず、65 で終わる(security-design の差分)。
    ctx.show_output(&output, &DisplayEvent::OutputFailed)
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
        Err(error) => return ctx.usage_error(&error),
    };
    match ctx.runtime.reset_state(&path, run_id.as_ref()) {
        Ok(()) => EXIT_OK,
        Err(error) => ctx.state_error(&error, StateContext::Plain),
    }
}

/// `conductor validate [<name>]`: 定義の検証、全ジョブの `command` の検査、`pipeline` と同じ範囲の
/// `.env` の読み取りと検査を行う(重複キーは警告で、誤りではない)。
///
/// ジョブは実行しない。シグナルの準備、ステートの読み書き、リセットもしない。
/// 成功は何も出さない(`-v` のときだけ、標準エラーに 1 行)。誤りは `run` と同じ表示で 64。
pub(crate) fn run_validate(ctx: &mut Ctx<'_>, cmd: &CliCommand) -> i32 {
    match ctx.check_definition_and_env(cmd) {
        Ok(checked) => {
            ctx.show(&DisplayEvent::DefinitionValid {
                job_count: checked.definition.job.len(),
            });
            EXIT_OK
        }
        Err(code) => code,
    }
}

/// `conductor convert <file> --to <format>`: 変換結果を、全部作ってから、標準出力に 1 回で書く(BR7.3)。
///
/// 入力か形式が無ければ、使い方の誤り(68)。`parse_args` を通る限り起こらない、内部の不変条件の守り。
pub(crate) fn run_convert(ctx: &mut Ctx<'_>, cmd: &CliCommand) -> i32 {
    let (Some(input), Some(target)) = (&cmd.convert_input, cmd.convert_target) else {
        let error = UsageError::new(UsageErrorKind::MissingArgument);
        return ctx.usage_error(&error);
    };
    match ctx.runtime.convert(Path::new(input), target) {
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

/// `conductor graph` を実行し、終了コードを返す(WF6。BR4.3〜BR4.6、BR6.1〜BR6.3)。
///
/// 定義の検査(`validate` と同じ。`.env` を含む)→ (`--state` のとき)ステートの読み取りと照合 →
/// 図の構造と配置 → 整形 → 標準出力に 1 回で書く。ジョブは実行せず、ステートは読むだけで、
/// 書かない・消さない。シグナルの準備もしない(BR4.6、BR6.2)。環境変数は、図にも出力にも含めない。
pub(crate) fn run_graph(ctx: &mut Ctx<'_>, cmd: &CliCommand) -> i32 {
    let (path, definition) = match ctx.check_definition_and_env(cmd) {
        Ok(checked) => (checked.path, checked.definition),
        Err(code) => return code,
    };
    let overlay = if cmd.graph.with_state {
        match read_overlay(ctx, cmd, &path, &definition) {
            Ok(overlay) => Some(overlay),
            Err(code) => return code,
        }
    } else {
        None
    };

    let graph = build_graph(&definition, overlay.as_ref());
    let lang = ctx.reporter.language();
    let text = match cmd.graph.format {
        GraphFormat::Text => {
            let layout = layout_graph(&graph);
            render_text(lang, &definition, &graph, &layout, cmd.verbose)
        }
        // `-v` は、Mermaid には影響しない(BR3.1)。
        GraphFormat::Mermaid => render_mermaid(lang, &graph),
        // `-v` は、DOT にも影響しない(U4。BR2.5)。
        GraphFormat::Dot => render_dot(lang, &graph),
    };
    // 標準出力に書けない(閉じたパイプなど)。`watch` と同じに、65 で終わる(BR6.3)。
    ctx.show_output(&DisplayEvent::Graph(&text), &DisplayEvent::OutputFailed)
}

/// `--state`: run ID を決め、ステートを(完了したものも含めて)読み、定義と照合して、重ね合わせにする
/// (BR4.3、BR4.4)。ステートが無ければ、すべて未実行。読むだけで、書かない(BR4.6)。
fn read_overlay(
    ctx: &mut Ctx<'_>,
    cmd: &CliCommand,
    path: &Path,
    definition: &PipelineDefinition,
) -> Result<StateOverlay, i32> {
    let run_id = ctx.definition_run_id(cmd, definition)?;
    // `read_state` は、完了したステートを「無い」として返すため、使わない(`Raw`)。
    let state = ctx.read_checked_state(path, definition, run_id.as_ref(), StateRead::Raw)?;
    Ok(match state {
        Some(state) => overlay_from_state(&state),
        None => pending_overlay(definition),
    })
}

/// `conductor init` を実行し、終了コードを返す(U4 の WF2。BR1.1〜BR1.6、BR3.1、BR3.2)。
///
/// ひな形を作り、ファイルに排他的に書く(または標準出力に出す)。定義、`.env`、ステートは読まず、
/// シグナルの準備もしない(BR1.1)。引数の誤り(68)は、`parse_args` が、ファイルに触れる前に返している。
pub(crate) fn run_init(ctx: &mut Ctx<'_>, cmd: &CliCommand) -> i32 {
    let options = cmd.init_options();
    let text = render_template(ctx.reporter.language(), &options.pipeline_name);
    match &options.target {
        InitTarget::Stdout => {
            // ひな形だけを 1 回で書く。`--force` は意味がないので、無視する(BR1.5)。
            ctx.show_output(
                &DisplayEvent::InitTemplate(&text),
                &DisplayEvent::InitStdoutFailed,
            )
        }
        InitTarget::File(path) => {
            let path = Path::new(path);
            match ctx
                .runtime
                .create_file(path, text.as_bytes(), options.force)
            {
                Ok(CreateOutcome::Created) => {
                    ctx.show(&DisplayEvent::InitCreated { path });
                    EXIT_OK
                }
                Ok(CreateOutcome::Replaced) => {
                    ctx.show(&DisplayEvent::InitReplaced { path });
                    EXIT_OK
                }
                Err(CreateError::AlreadyExists) => {
                    ctx.show(&DisplayEvent::InitExists { path });
                    EXIT_EXISTS
                }
                Err(error) => {
                    ctx.show(&DisplayEvent::InitFailed { path, error });
                    EXIT_STATE
                }
            }
        }
    }
}
