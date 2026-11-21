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

//! `conductor graph` の流れ(WF6。BR4.3〜BR4.6、BR6.1〜BR6.3)。
//!
//! 定義の検査(`validate` と同じ。`.env` を含む)→ (`--state` のとき)ステートの読み取りと照合 →
//! 図の構造と配置 → 整形 → 標準出力に 1 回で書く。ジョブは実行せず、ステートは読むだけで、
//! 書かない・消さない。シグナルの準備もしない(BR4.6、BR6.2)。環境変数は、図にも出力にも含めない。

use std::path::Path;

use conductor_core::graph::{
    StateOverlay, build_graph, layout_graph, overlay_from_state, pending_overlay,
};
use conductor_core::{DEFAULT_RUN_ID_ENV, PipelineDefinition};
use conductor_runtime::{ConsistencyResult, StateLocation, check_consistency};

use crate::args::resolve_run_id;
use crate::aux_commands::check_definition_and_env;
use crate::ctx::{Ctx, read_context};
use crate::exit::{EXIT_OK, EXIT_STATE, EXIT_USAGE};
use crate::report::DisplayEvent;
use crate::report::graph_mermaid::render_mermaid;
use crate::report::graph_text::render_text;
use crate::types::{CliCommand, GraphFormat};

/// `conductor graph` を実行し、終了コードを返す。
pub(crate) fn run_graph(ctx: &mut Ctx<'_>, cmd: &CliCommand) -> i32 {
    let (path, definition) = match check_definition_and_env(ctx, cmd) {
        Ok(checked) => checked,
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
    };
    if ctx.show(&DisplayEvent::Graph(&text)) {
        EXIT_OK
    } else {
        // 標準出力に書けない(閉じたパイプなど)。`watch` と同じに、65 で終わる(BR6.3)。
        ctx.show(&DisplayEvent::OutputFailed);
        EXIT_STATE
    }
}

/// `--state`: run ID を決め、ステートを(完了したものも含めて)読み、定義と照合して、重ね合わせにする
/// (BR4.3、BR4.4)。ステートが無ければ、すべて未実行。読むだけで、書かない(BR4.6)。
fn read_overlay(
    ctx: &mut Ctx<'_>,
    cmd: &CliCommand,
    path: &Path,
    definition: &PipelineDefinition,
) -> Result<StateOverlay, i32> {
    let env_name = definition
        .run_id_env
        .as_deref()
        .unwrap_or(DEFAULT_RUN_ID_ENV);
    let run_id = match resolve_run_id(cmd.run_id.as_deref(), env_name, ctx.env) {
        Ok(run_id) => run_id,
        Err(error) => {
            ctx.show(&DisplayEvent::Usage(&error));
            return Err(EXIT_USAGE);
        }
    };
    // `read_state` は、完了したステートを「無い」として返すため、使わない。
    let state = match ctx.runtime.read_state_raw(path, run_id.as_ref()) {
        Ok(state) => state,
        Err(error) => return Err(ctx.state_error(&error, read_context(run_id.is_some()))),
    };
    let Some(state) = state else {
        return Ok(pending_overlay(definition));
    };
    if let ConsistencyResult::Mismatch(kind) = check_consistency(&state, definition) {
        let state_path = StateLocation::of(path, run_id.as_ref())
            .path()
            .to_path_buf();
        ctx.show(&DisplayEvent::ConsistencyMismatch {
            state_path: &state_path,
            kind,
            with_run_id: run_id.is_some(),
        });
        return Err(EXIT_STATE);
    }
    Ok(overlay_from_state(&state))
}
