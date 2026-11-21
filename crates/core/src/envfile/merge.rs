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
    let mut entries: Vec<EffectiveEntry> = Vec::new();

    push_files(
        &mut entries,
        &def.env_file,
        files,
        EnvOrigin::PipelineEnvFile,
    );
    push_vars(&mut entries, &def.env, EnvOrigin::PipelineEnv);
    push_files(&mut entries, &job.env_file, files, EnvOrigin::JobEnvFile);
    push_vars(&mut entries, &job.env, EnvOrigin::JobEnv);

    put(
        &mut entries,
        "CONDUCTOR_PIPELINE",
        ctx.pipeline_name,
        EnvOrigin::Builtin,
    );
    put(
        &mut entries,
        "CONDUCTOR_JOB_ID",
        ctx.job_id,
        EnvOrigin::Builtin,
    );
    if let Some(run_id) = ctx.run_id {
        put(&mut entries, "CONDUCTOR_RUN_ID", run_id, EnvOrigin::Builtin);
    }
    put(
        &mut entries,
        "CONDUCTOR_ATTEMPT",
        &ctx.attempt.to_string(),
        EnvOrigin::Builtin,
    );

    EffectiveEnv {
        job_id: job.id.clone(),
        entries,
    }
}

/// 同じ名前があれば取り除き、末尾に足す(BR4.3)。
fn put(entries: &mut Vec<EffectiveEntry>, name: &str, value: &str, origin: EnvOrigin) {
    entries.retain(|e| e.name != name);
    entries.push(EffectiveEntry {
        name: name.to_string(),
        value: value.to_string(),
        origin,
    });
}

fn push_vars(entries: &mut Vec<EffectiveEntry>, vars: &[EnvVar], origin: EnvOrigin) {
    for v in vars {
        put(entries, &v.name, &v.value, origin);
    }
}

fn push_files(
    entries: &mut Vec<EffectiveEntry>,
    refs: &[String],
    files: &LoadedEnvFiles,
    origin: EnvOrigin,
) {
    for r in refs {
        if let Some(content) = files.get(r) {
            for e in &content.entries {
                put(entries, &e.name, &e.value, origin);
            }
        }
    }
}
