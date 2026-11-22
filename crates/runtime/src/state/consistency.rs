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

//! ステートと定義の照合(Consistency。BR2.5)。
//!
//! 照合の関数と型は conductor-core の `consistency` に置き、`lib.rs` が再エクスポートする。
//! このファイルは、照合の単体テストをこの場所に残すため、テストが使う名前だけを取り込む。

// 単体テスト(`mod tests` の `use super::*`)が使う。
#[cfg(test)]
use crate::types::{ConsistencyResult, MismatchKind};
#[cfg(test)]
use conductor_core::{PipelineDefinition, PipelineState, check_consistency};

#[cfg(test)]
mod tests {
    use super::*;
    use conductor_core::{JobDefinition, JobState};

    fn definition(name: &str, ids: &[&str]) -> PipelineDefinition {
        PipelineDefinition {
            name: name.into(),
            description: None,
            run_id_env: None,
            remove_on_complete: false,
            format_version: 1,
            job: ids
                .iter()
                .map(|id| JobDefinition {
                    id: (*id).into(),
                    command: "echo".into(),
                    warning_exit_codes: vec![],
                    success_exit_codes: None,
                    transitions: vec![],
                    retry: None,
                    env: vec![],
                    env_file: vec![],
                })
                .collect(),
            env: vec![],
            env_file: vec![],
        }
    }

    fn state(name: &str, ids: &[&str]) -> PipelineState {
        PipelineState {
            definition_key: "other/place.toml".into(),
            pipeline_name: name.into(),
            definition_path: "other/place.toml".into(),
            job_order: ids.iter().map(|s| s.to_string()).collect(),
            job_states: ids.iter().map(|s| JobState::pending(*s)).collect(),
            resume_job_id: None,
            end_reached: false,
            completed: false,
            run_id: None,
        }
    }

    #[test]
    fn 一致すれば_ok_で_パスの違いは見ない() {
        assert_eq!(
            check_consistency(&state("p", &["a", "b"]), &definition("p", &["a", "b"])),
            ConsistencyResult::Ok
        );
    }

    #[test]
    fn パイプライン名の違い() {
        assert_eq!(
            check_consistency(&state("p", &["a"]), &definition("q", &["a"])),
            ConsistencyResult::Mismatch(MismatchKind::PipelineName)
        );
        // 名前が違えば、IDも違っていても、名前が先に報告される。
        assert_eq!(
            check_consistency(&state("p", &["a"]), &definition("q", &["z"])),
            ConsistencyResult::Mismatch(MismatchKind::PipelineName)
        );
    }

    #[test]
    fn ジョブidの追加_削除_改名() {
        let d = definition("p", &["a", "b"]);
        for ids in [&["a"][..], &["a", "b", "c"], &["a", "x"]] {
            assert_eq!(
                check_consistency(&state("p", ids), &d),
                ConsistencyResult::Mismatch(MismatchKind::JobIds),
                "{ids:?}"
            );
        }
    }

    #[test]
    fn 並びだけの違い() {
        assert_eq!(
            check_consistency(&state("p", &["b", "a"]), &definition("p", &["a", "b"])),
            ConsistencyResult::Mismatch(MismatchKind::JobOrder)
        );
    }
}
