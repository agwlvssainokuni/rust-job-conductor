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

//! 層の統合(WF4。BR4.1〜BR4.4、BR5.1〜BR5.3)。純粋な関数。

use std::collections::HashMap;

use super::{EffectiveEntry, EffectiveEnv, EnvContext, EnvOrigin, LoadedEnvFiles};
use crate::types::{EnvVar, JobDefinition, PipelineDefinition};

/// 層を、後ろが勝つ順(`[pipeline].env_file` < `[pipeline.env]` < `[job].env_file` < `[job.env]`
/// < 自動の変数)に重ねて、ジョブに足す環境を作る。引き継いだ環境は含めない(BR4.1、BR4.4)。
///
/// 完全に同じ名前は、後ろの層が勝って 1 つになる。大文字小文字だけが違う名前は、別の名前として、
/// 層の順に並べる(BR4.3)。`files` から引けない `env_file` の記述は、何も足さない
/// (読み取りは、実行の前に済んでいる前提)。
pub fn merge_env(
    def: &PipelineDefinition,
    job: &JobDefinition,
    files: &LoadedEnvFiles,
    ctx: &EnvContext<'_>,
) -> EffectiveEnv {
    // その場で作る値は、先に作って借りる。
    let attempt = ctx.attempt.to_string();
    let mut items: Vec<Item<'_>> = Vec::new();

    collect_files(&mut items, &def.env_file, files, EnvOrigin::PipelineEnvFile);
    collect_vars(&mut items, &def.env, EnvOrigin::PipelineEnv);
    collect_files(&mut items, &job.env_file, files, EnvOrigin::JobEnvFile);
    collect_vars(&mut items, &job.env, EnvOrigin::JobEnv);

    items.push(("CONDUCTOR_PIPELINE", ctx.pipeline_name, EnvOrigin::Builtin));
    items.push(("CONDUCTOR_JOB_ID", ctx.job_id, EnvOrigin::Builtin));
    if let Some(run_id) = ctx.run_id {
        // `run_id::DEFAULT_RUN_ID_ENV` とは意味が別(あちらは run ID を読む名前で `run_id_env` で変わる。
        // こちらはジョブに渡す名前で固定)。同じ値でも、まとめない。
        items.push(("CONDUCTOR_RUN_ID", run_id, EnvOrigin::Builtin));
    }
    items.push(("CONDUCTOR_ATTEMPT", &attempt, EnvOrigin::Builtin));

    EffectiveEnv {
        job_id: job.id.clone(),
        entries: keep_last(&items),
    }
}

/// 層の順に並べた 1 項目(名前, 値, 出どころ)。同じ名前を重ねて含む。
type Item<'a> = (&'a str, &'a str, EnvOrigin);

/// 名前ごとの最後の位置を表で引き、最後の位置の項目だけを、並びの順に残す(BR4.3)。
///
/// 完全に同じ名前は、後ろの層が勝って 1 つになり、最後に足した位置に並ぶ。大文字小文字だけが
/// 違う名前は、別の名前。項目の数に比例する時間で済む(BR2.2)。
fn keep_last(items: &[Item<'_>]) -> Vec<EffectiveEntry> {
    let mut last: HashMap<&str, usize> = HashMap::with_capacity(items.len());
    for (i, (name, _, _)) in items.iter().enumerate() {
        last.insert(name, i);
    }
    items
        .iter()
        .enumerate()
        .filter(|(i, (name, _, _))| last.get(name) == Some(i))
        .map(|(_, (name, value, origin))| EffectiveEntry {
            name: (*name).to_string(),
            value: (*value).to_string(),
            origin: *origin,
        })
        .collect()
}

fn collect_vars<'a>(items: &mut Vec<Item<'a>>, vars: &'a [EnvVar], origin: EnvOrigin) {
    for v in vars {
        items.push((&v.name, &v.value, origin));
    }
}

fn collect_files<'a>(
    items: &mut Vec<Item<'a>>,
    refs: &[String],
    files: &'a LoadedEnvFiles,
    origin: EnvOrigin,
) {
    for r in refs {
        if let Some(content) = files.get(r) {
            for e in &content.entries {
                items.push((&e.name, &e.value, origin));
            }
        }
    }
}
