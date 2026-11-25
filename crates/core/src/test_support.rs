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

//! 単体テストの共通の補助(定義とジョブの組み立て)。各モジュールの補助は、これと構造体の更新の
//! 書き方で、違う所だけを書く。

use crate::types::{Command, JobDefinition, PipelineDefinition};

/// ID `id`、コマンド `c` のジョブ(ほかは空。成功コードは既定)。
pub(crate) fn job(id: &str) -> JobDefinition {
    JobDefinition {
        id: id.into(),
        command: Command::Line("c".into()),
        warning_exit_codes: Vec::new(),
        success_exit_codes: None,
        transitions: Vec::new(),
        retry: None,
        env: Vec::new(),
        env_file: Vec::new(),
    }
}

/// 名前 `name`、版 1 で、ジョブ `jobs` を持つ定義(ほかは空)。
pub(crate) fn definition(name: &str, jobs: Vec<JobDefinition>) -> PipelineDefinition {
    PipelineDefinition {
        name: name.into(),
        description: None,
        run_id_env: None,
        remove_on_complete: false,
        format_version: 1,
        job: jobs,
        env: Vec::new(),
        env_file: Vec::new(),
        log_dir: None,
    }
}
