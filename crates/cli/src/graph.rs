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

use conductor_core::PipelineDefinition;
use conductor_core::graph::{
    StateOverlay, build_graph, layout_graph, overlay_from_state, pending_overlay,
};

use crate::ctx::{Ctx, StateRead};
use crate::report::DisplayEvent;
use crate::report::graph_mermaid::render_mermaid;
use crate::report::graph_text::render_text;
use crate::types::{CliCommand, GraphFormat};

/// `conductor graph` を実行し、終了コードを返す。
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
