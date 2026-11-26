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

//! 起動前の判断(`decide_before_run`、ステートからの `decide_from_state`)。BR3.5、BR3.8。

use super::{SimulationMarks, position_of, record_at};
use crate::types::{
    Decision, JobState, JobStatus, PipelineDefinition, PipelineState, RejectReason, SkipCause,
    SkipReason,
};

/// ジョブを起動する前に、実行・スキップ・拒否を決める(W2、BR3.5)。
///
/// 次の順に、最初に当てはまるものを採る。
///
/// 1. `skipped(byTransition)` の記録 → `Skip(SkipTarget)`
/// 2. `skipped(byEnd)` の記録、または `end_reached` で未実行 → `Skip(AfterEnd)`
/// 3. `success` / `warning` で記録済み → `Skip(AlreadyExecuted)`
/// 4. `error` で記録済み、または `resume_job_id` と一致 → `Run`
/// 5. 記録がすべて pending(または空)で、記述順で最初 → `Run`。規則 6 に含まれる(前のジョブが
///    無いので、規則 6 が `Run` を返す)。コードは規則 6 で判定する。
/// 6. 記述順で前のジョブがすべて、成功・警告・スキップで、このジョブが pending → `Run`
/// 7. それ以外 → `Reject(OutOfOrder)`
///
/// 契約 C2 の入力(ジョブID、定義、各ジョブの状態)に加えて、`resume_job_id` と `end_reached`
/// (`PipelineState` の同名のフィールド)を受け取る。BR3.5 の 2 と 4 が、これらを使うため。
/// 定義にないジョブIDは、順序外として拒否する(パニックしない)。
///
/// 個別実行で `@end` に到達したあと(`end_reached = true`)は、`NextStep.skipped_job_ids` が空なので、
/// 後ろのジョブは記録なし(pending)のまま。スケジューラが呼ぶたびに、2 で `Skip(AfterEnd)` を返す。
/// Cli は、呼ばれたジョブだけを [`apply_skip`](super::apply_skip) で `skipped(byEnd)` として記録する。
/// 完了と再開位置は `apply_skip` が書く(最後のジョブをスキップとして記録し、`pending` が無ければ
/// `completed = true`。BR3.6)。[`resume_point_on_interrupt`](super::resume_point_on_interrupt) は、
/// `pipeline` のジョブのあいだの中断にだけ使う。
///
/// 位置と目印は、ここで自分で求める(位置は `job_index`、目印は先頭から)。本体は
/// [`decide_with`] で、計画の模擬の [`decide_at`] と同じ本体を通る(v0.10.1 の BR3.4)。
fn decide_before_run(
    job_id: &str,
    definition: &PipelineDefinition,
    job_states: &[JobState],
    resume_job_id: Option<&str>,
    end_reached: bool,
) -> Decision {
    let Some(index) = definition.job_index(job_id) else {
        return Decision::Reject(RejectReason::OutOfOrder);
    };
    let input = DecideInput {
        job_id,
        index,
        definition,
        job_states,
        resume_job_id,
        end_reached,
    };
    decide_with(&input, &mut SimulationMarks::new())
}

/// 判断の本体への入力。`index` は、`job_id` の定義の記述順の位置(呼び出し側が確かめた値)。
struct DecideInput<'a> {
    job_id: &'a str,
    index: usize,
    definition: &'a PipelineDefinition,
    job_states: &'a [JobState],
    resume_job_id: Option<&'a str>,
    end_reached: bool,
}

/// 起動前の判断の本体(規則 1〜7。[`decide_before_run`] の説明のとおり)。実行と模擬が共に通る
/// 1 か所(v0.10.1 の BR3.4)。
///
/// 記録は位置で引き([`record_at`]。位置が合えば O(1)、合わなければ探す)、規則 6 の前のジョブの
/// 走査は、目印の [`SimulationMarks`] の「済んだ前置き」から続ける。
fn decide_with(input: &DecideInput<'_>, marks: &mut SimulationMarks) -> Decision {
    let DecideInput {
        job_id,
        index,
        definition,
        job_states,
        resume_job_id,
        end_reached,
    } = *input;
    let (status, skip_reason) = record_at(job_states, index, job_id);

    // 1, 2
    if status == JobStatus::Skipped {
        return match skip_reason {
            Some(SkipReason::ByEnd) => Decision::Skip(SkipCause::AfterEnd),
            _ => Decision::Skip(SkipCause::SkipTarget),
        };
    }
    if end_reached && status == JobStatus::Pending {
        return Decision::Skip(SkipCause::AfterEnd);
    }
    // 3
    if matches!(status, JobStatus::Success | JobStatus::Warning) {
        return Decision::Skip(SkipCause::AlreadyExecuted);
    }
    // 4
    if status == JobStatus::Error || resume_job_id == Some(job_id) {
        return Decision::Run;
    }
    // 5 は 6 に含まれる(ここまでで `status` は pending。`index == 0` なら前のジョブが無い)。
    // 6
    if marks.predecessors_settled(definition, job_states, index) {
        return Decision::Run;
    }
    // 7
    Decision::Reject(RejectReason::OutOfOrder)
}

/// 計画の模擬の、起動前の判断(内部の入口。v0.10.1 の BR3.4、FR3.3)。
///
/// `index` は、模擬が定義の順に進めている位置。`marks` は、模擬が持ち回る目印。判断は
/// [`decide_from_state`] と同じ本体([`decide_with`])で決め、同じ入力には同じ判断になる。
/// `index` のジョブが `job_id` でなければ、`job_index` で探し直す(定義になければ順序外)。
pub(crate) fn decide_at(
    job_id: &str,
    index: usize,
    definition: &PipelineDefinition,
    state: &PipelineState,
    marks: &mut SimulationMarks,
) -> Decision {
    let Some(index) = position_of(definition, index, job_id) else {
        return Decision::Reject(RejectReason::OutOfOrder);
    };
    let input = DecideInput {
        job_id,
        index,
        definition,
        job_states: &state.job_states,
        resume_job_id: state.resume_job_id.as_deref(),
        end_reached: state.end_reached,
    };
    decide_with(&input, marks)
}

/// ステート(無いこともある)から、ジョブを起動する前の判断を決める(W2、BR3.5)。
///
/// ステートが無い(`None`)なら、各ジョブの状態の記録は空、再開位置なし、`end_reached` は偽として、
/// `decide_before_run`(このモジュールの中だけの関数)を呼ぶ。`run`・`pipeline` と、計画(`plan_run`・`plan_pipeline`)が使う。
pub fn decide_from_state(
    job_id: &str,
    definition: &PipelineDefinition,
    state: Option<&PipelineState>,
) -> Decision {
    let (job_states, resume_job_id, end_reached) = match state {
        Some(state) => (
            state.job_states.as_slice(),
            state.resume_job_id.as_deref(),
            state.end_reached,
        ),
        None => (&[][..], None, false),
    };
    decide_before_run(job_id, definition, job_states, resume_job_id, end_reached)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::JobStatus::*;

    fn def(n: usize) -> PipelineDefinition {
        crate::test_support::definition(
            "p",
            (0..n)
                .map(|i| crate::test_support::job(&format!("j{i}")))
                .collect(),
        )
    }

    fn st(id: &str, status: JobStatus, reason: Option<SkipReason>) -> JobState {
        JobState {
            job_id: id.into(),
            status,
            exit_code: None,
            skip_reason: reason,
            attempts: vec![],
        }
    }

    #[test]
    fn 規則ごとの判断() {
        use Decision::*;
        use RejectReason::*;
        use SkipCause::*;
        let d = def(4);
        let skipped_t = |id| st(id, Skipped, Some(SkipReason::ByTransition));
        let skipped_e = |id| st(id, Skipped, Some(SkipReason::ByEnd));

        // (名前, 呼ぶジョブ, 状態, resume, endReached, 期待)
        type Case<'a> = (
            &'a str,
            &'a str,
            Vec<JobState>,
            Option<&'a str>,
            bool,
            Decision,
        );
        let cases: Vec<Case> = vec![
            (
                "1 遷移でスキップ",
                "j1",
                vec![st("j0", Success, None), skipped_t("j1")],
                None,
                false,
                Skip(SkipTarget),
            ),
            (
                "2 @end でスキップ",
                "j2",
                vec![skipped_e("j2")],
                None,
                true,
                Skip(AfterEnd),
            ),
            (
                "2 @end 到達済みで記録なし",
                "j3",
                vec![st("j0", Success, None)],
                None,
                true,
                Skip(AfterEnd),
            ),
            (
                "3 成功で実行済み",
                "j0",
                vec![st("j0", Success, None)],
                None,
                false,
                Skip(AlreadyExecuted),
            ),
            (
                "3 警告で実行済み",
                "j0",
                vec![st("j0", Warning, None)],
                None,
                false,
                Skip(AlreadyExecuted),
            ),
            (
                "4 エラーの再実行",
                "j1",
                vec![st("j0", Success, None), st("j1", Error, None)],
                Some("j1"),
                false,
                Run,
            ),
            (
                "4 エラーで記録済み(resume なし)",
                "j1",
                vec![st("j1", Error, None)],
                None,
                false,
                Run,
            ),
            (
                "4 resume と一致",
                "j2",
                vec![st("j0", Success, None), skipped_t("j1")],
                Some("j2"),
                false,
                Run,
            ),
            ("5 ステートが空で最初", "j0", vec![], None, false, Run),
            (
                "5 すべて pending で最初",
                "j0",
                vec![st("j0", Pending, None), st("j1", Pending, None)],
                None,
                false,
                Run,
            ),
            (
                "6 順番どおり",
                "j2",
                vec![st("j0", Success, None), st("j1", Warning, None)],
                None,
                false,
                Run,
            ),
            (
                "6 間がスキップ済み",
                "j3",
                vec![st("j0", Success, None), skipped_t("j1"), skipped_t("j2")],
                None,
                false,
                Run,
            ),
            (
                "7 ステートが空で最初でない",
                "j1",
                vec![],
                None,
                false,
                Reject(OutOfOrder),
            ),
            (
                "7 前が未実行",
                "j2",
                vec![st("j0", Success, None)],
                None,
                false,
                Reject(OutOfOrder),
            ),
            (
                "7 前がエラー",
                "j2",
                vec![st("j0", Success, None), st("j1", Error, None)],
                None,
                false,
                Reject(OutOfOrder),
            ),
        ];
        for (name, job, states, resume, end, expected) in cases {
            assert_eq!(
                decide_before_run(job, &d, &states, resume, end),
                expected,
                "{name}"
            );
        }
    }

    #[test]
    fn 定義にないジョブは順序外() {
        assert_eq!(
            decide_before_run("zz", &def(2), &[], None, false),
            Decision::Reject(RejectReason::OutOfOrder)
        );
    }

    #[test]
    fn 記録のないジョブは_pending_と同じ() {
        let d = def(2);
        let with_pending = vec![st("j0", Success, None), st("j1", Pending, None)];
        let without = vec![st("j0", Success, None)];
        assert_eq!(
            decide_before_run("j1", &d, &with_pending, None, false),
            decide_before_run("j1", &d, &without, None, false)
        );
    }

    #[test]
    fn スキップの理由がないスキップは遷移のスキップ扱い() {
        assert_eq!(
            decide_before_run("j1", &def(2), &[st("j1", Skipped, None)], None, false),
            Decision::Skip(SkipCause::SkipTarget)
        );
    }

    #[test]
    fn ステートが無いときは_記録が空で再開位置も_end_もないときと同じ判断() {
        let d = def(3);
        let state =
            |job_states: Vec<JobState>, resume: Option<&str>, end_reached: bool| PipelineState {
                pipeline_name: "p".into(),
                definition_path: "t.toml".into(),
                job_states,
                resume_job_id: resume.map(str::to_string),
                end_reached,
                completed: false,
                run_id: None,
            };

        // (1) ステートが無い: すべてのジョブで、記録が空・再開位置なし・end_reached 偽と同じ。
        for id in ["j0", "j1", "j2"] {
            assert_eq!(
                decide_from_state(id, &d, None),
                decide_before_run(id, &d, &[], None, false),
                "{id}"
            );
        }
        assert_eq!(decide_from_state("j0", &d, None), Decision::Run);
        assert_eq!(
            decide_from_state("j1", &d, None),
            Decision::Reject(RejectReason::OutOfOrder)
        );

        // (2) end_reached が渡る: 先頭が成功で end_reached なら、2 つ目は @end の後。
        let ended = state(
            vec![
                st("j0", Success, None),
                st("j1", Pending, None),
                st("j2", Pending, None),
            ],
            None,
            true,
        );
        assert_eq!(
            decide_from_state("j1", &d, Some(&ended)),
            Decision::Skip(SkipCause::AfterEnd)
        );

        // (3) 再開位置が渡る: 前のジョブが pending でも、再開位置のジョブは実行する。
        let resumed = state(
            vec![
                st("j0", Pending, None),
                st("j1", Pending, None),
                st("j2", Pending, None),
            ],
            Some("j1"),
            false,
        );
        assert_eq!(decide_from_state("j1", &d, Some(&resumed)), Decision::Run);

        // 定義にないジョブIDは、ステートの有無によらず順序外。
        assert_eq!(
            decide_from_state("zz", &d, None),
            Decision::Reject(RejectReason::OutOfOrder)
        );
        assert_eq!(
            decide_from_state("zz", &d, Some(&resumed)),
            Decision::Reject(RejectReason::OutOfOrder)
        );
    }
}
