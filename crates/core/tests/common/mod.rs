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

//! core の統合テストの共通の補助(秘密の値、定義の検査の結果、計った時間の中央値、ステート、
//! ジョブと実行結果)。
#![allow(dead_code)]

use std::time::Duration;

use conductor_core::DefinitionErrorKind as K;
use conductor_core::{
    JobDefinition, JobExecution, JobState, JobStatus, PipelineState, SkipReason, parse_definition,
};

/// 誤りの文やステートに現れてはいけない、秘密の値(NFR4.6)。
pub const SECRET: &str = "s3cr3t-VALUE-do-not-print";

/// 検査の結果を `(種類, 行)` の並びで返す。受理なら空。
pub fn run(text: &str) -> Vec<(K, Option<usize>)> {
    match parse_definition(text, "t.toml", None) {
        Ok(_) => Vec::new(),
        Err(errs) => errs.into_iter().map(|e| (e.kind, e.line)).collect(),
    }
}

/// 検査の誤りの種類の並び。受理なら空。
pub fn kinds(text: &str) -> Vec<K> {
    run(text).into_iter().map(|(k, _)| k).collect()
}

/// 計った時間の中央値。
pub fn median(mut samples: Vec<Duration>) -> Duration {
    samples.sort();
    samples[samples.len() / 2]
}

/// ID `id`、コマンド `c` のジョブの定義(ほかは既定)。
pub fn job(id: &str) -> JobDefinition {
    JobDefinition {
        id: id.to_string(),
        command: "c".into(),
        warning_exit_codes: vec![],
        success_exit_codes: None,
        transitions: vec![],
        retry: None,
        env: vec![],
        env_file: vec![],
    }
}

/// ジョブ `job_id`([`job`])が、終了コード `code` で終わった実行結果。
pub fn exited(job_id: &str, code: i32) -> JobExecution {
    JobExecution::exited(&job(job_id), code)
}

/// パイプライン `p` のステート(定義のキーとパスは `t.toml`)。`jobs` はジョブ ID と状態の並び
/// (終了コードは無し。`Skipped` の理由は `ByTransition`)。
pub fn state(jobs: &[(&str, JobStatus)], resume: Option<&str>, completed: bool) -> PipelineState {
    PipelineState {
        definition_key: "t.toml".into(),
        pipeline_name: "p".into(),
        definition_path: "t.toml".into(),
        job_order: jobs.iter().map(|(id, _)| id.to_string()).collect(),
        job_states: jobs
            .iter()
            .map(|(id, status)| JobState {
                job_id: id.to_string(),
                status: *status,
                exit_code: None,
                skip_reason: (*status == JobStatus::Skipped).then_some(SkipReason::ByTransition),
            })
            .collect(),
        resume_job_id: resume.map(str::to_string),
        end_reached: false,
        completed,
        run_id: None,
    }
}
