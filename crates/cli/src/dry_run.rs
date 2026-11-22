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

//! `--dry-run` の流れ(`pipeline` と `run`。WF2〜WF4。BR1.1、BR3.4、BR4.1〜BR4.5)。
//!
//! 準備(`prepare_checked`。定義とコマンドの分割、`--assume` のジョブID、run ID、`.env`、ステートの
//! 読み込みと照合)→ 計画の生成(`conductor-core`)→ 整形 → 標準出力に 1 回で書く。
//! ジョブは起動せず、ステートは読むだけで書かない・消さない。シグナルの準備もしない(BR1.1)。
//! 環境変数の値は、計画にも出力にも含めない(BR5.1)。

use conductor_core::{PlanInput, plan_pipeline, plan_run};

use crate::ctx::{Ctx, FlowTarget};
use crate::exit::EXIT_REJECTED;
use crate::report::DisplayEvent;
use crate::report::plan_text::render_plan;
use crate::types::{CliCommand, UsageError, UsageErrorKind};

/// `run` と `pipeline` の入口(共通)。位置引数 `arg` を、`target` で対象にする。
///
/// 位置引数が無ければ、使い方の誤り(68)。コマンドからは `parse_args` が先に止めるが、公開の型
/// `CliCommand` を直接組み立てて呼ばれたときの守り。
/// `--dry-run` なら、計画を出して、その終了コードを `Err` で返す。どちらでもなければ、位置引数を返す。
pub(crate) fn enter_flow<'c>(
    ctx: &mut Ctx<'_>,
    cmd: &CliCommand,
    arg: Option<&'c str>,
    target: fn(&'c str) -> FlowTarget<'c>,
) -> Result<&'c str, i32> {
    let Some(arg) = arg else {
        let error = UsageError::new(UsageErrorKind::MissingArgument);
        return Err(ctx.usage_error(&error));
    };
    // `--dry-run`: ジョブを実行せず、ステートに書かず、シグナルも準備せずに、計画を出す(U3)。
    if cmd.dry_run.dry_run {
        return Err(run_dry_run(ctx, cmd, target(arg)));
    }
    Ok(arg)
}

/// `--dry-run` を実行し、終了コードを返す(計画を出せたら 0。BR4.4)。
pub(crate) fn run_dry_run(ctx: &mut Ctx<'_>, cmd: &CliCommand, target: FlowTarget<'_>) -> i32 {
    let assumptions = &cmd.dry_run.assumptions;
    let assume_ids: Vec<&str> = assumptions.iter().map(|a| a.job_id.as_str()).collect();
    let checked = match ctx.prepare_checked(cmd, target, &assume_ids) {
        Ok(checked) => checked,
        Err(code) => return code,
    };
    let definition = &checked.definition;

    // コマンドの分割は、定義の検査で済んでいる。計画には、コマンド名と引数を並べた語の列で渡す。
    let commands: Vec<Vec<String>> = checked
        .commands
        .iter()
        .map(|line| {
            let mut words = Vec::with_capacity(line.args.len() + 1);
            words.push(line.program.clone());
            words.extend(line.args.iter().cloned());
            words
        })
        .collect();

    let input = PlanInput {
        definition,
        state: checked.state.as_ref(),
        commands: &commands,
        env_files: &checked.env_files,
        run_id: checked.run_id.as_ref().map(|r| r.as_str()),
        assumptions,
    };
    let planned = match target {
        FlowTarget::Pipeline(_) => plan_pipeline(&input),
        FlowTarget::Run(job_id) => plan_run(job_id, &input),
    };
    let plan = match planned {
        Ok(plan) => plan,
        Err(rejection) => {
            // 本番と同じ表示と終了コード(BR4.3)。
            ctx.show(&DisplayEvent::Rejected {
                job_id: &rejection.job_id,
                reason: rejection.reason,
            });
            return EXIT_REJECTED;
        }
    };

    // 計画の全体を作ってから、標準出力に 1 回で書く(BR3.4)。
    let text = render_plan(ctx.reporter.language(), &plan);
    ctx.show_output(&DisplayEvent::Plan(&text), &DisplayEvent::OutputFailed)
}
