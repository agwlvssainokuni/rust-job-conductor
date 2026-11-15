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

//! ステートと定義の照合(Consistency。BR2.5)。純粋な関数。

use conductor_core::{PipelineDefinition, PipelineState};

use crate::types::{ConsistencyResult, MismatchKind};

/// ステートと定義の食い違いを検知する(契約 C3 の `check_consistency`。W4)。
///
/// パイプライン名 → ジョブIDの集合 → 並び、の順に比べる。`definition_key` と `definition_path` は、
/// 照合に使わない(定義ファイルの置き場所の移動で、食い違いにしないため)。
pub fn check_consistency(
    state: &PipelineState,
    definition: &PipelineDefinition,
) -> ConsistencyResult {
    if state.pipeline_name != definition.name {
        return ConsistencyResult::Mismatch(MismatchKind::PipelineName);
    }
    let defined: Vec<&str> = definition.jobs.iter().map(|j| j.id.as_str()).collect();
    let recorded: Vec<&str> = state.job_order.iter().map(String::as_str).collect();
    if defined == recorded {
        return ConsistencyResult::Ok;
    }
    let mut a = defined;
    let mut b = recorded;
    a.sort_unstable();
    b.sort_unstable();
    if a == b {
        ConsistencyResult::Mismatch(MismatchKind::JobOrder)
    } else {
        ConsistencyResult::Mismatch(MismatchKind::JobIds)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use conductor_core::{JobDefinition, JobState};

    fn definition(name: &str, ids: &[&str]) -> PipelineDefinition {
        PipelineDefinition {
            name: name.into(),
            description: None,
            format_version: 1,
            jobs: ids
                .iter()
                .map(|id| JobDefinition {
                    id: (*id).into(),
                    command: "echo".into(),
                    warning_exit_codes: vec![],
                    success_exit_codes: None,
                    transitions: vec![],
                })
                .collect(),
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
