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

//! 分類と、スケジューラへ返す終了コード(BR2.1〜BR2.4)。

use crate::types::{Classification, ExecutionResult, JobDefinition, JobExecution, JobOutcome};

/// 起動に失敗したときの終了コード(BR2.3)。
pub const EXIT_START_FAILURE: i32 = 127;

/// シグナルによる終了コードの基準(128 + シグナル番号)。
pub const EXIT_SIGNAL_BASE: i32 = 128;

/// 成功コードに含まれない 0 のときに返す終了コード(BR2.2)。
pub const EXIT_UNLISTED_ZERO: i32 = 1;

/// `128 + シグナル番号`(あふれるときは `i32::MAX` で止める)。中断の番号は、型で必ずある(BR1.1)。
fn signal_exit(signal: i32) -> i32 {
    EXIT_SIGNAL_BASE.saturating_add(signal)
}

fn outcome(
    job: &JobDefinition,
    class: Option<Classification>,
    code: i32,
    conductor_error: bool,
) -> JobOutcome {
    JobOutcome {
        job_id: job.id.clone(),
        classification: class,
        reported_exit_code: code,
        conductor_error,
    }
}

/// ジョブの実行の結果を分類し、返す終了コードを決める。
///
/// 判定の優先順位(BR2.3)は、最初に当てはまるものを採る。
///
/// 1. 起動しなかった中断(`NotStarted`): 分類しない(`classification` は `None`)。
///    `128 + シグナル番号`(BR2.4)。
/// 2. 中断(`Exited` で `interrupted_by` あり): エラー、`128 + シグナル番号`。
///    ジョブ自身の終了コードとシグナルは見ない。
/// 3. 起動失敗(`StartFailed`): エラー、127。
/// 4. シグナル終了(`Exited` で `terminated_by_signal` あり): エラー、`128 + シグナル番号`。
/// 5. 上のどれでもない: 終了コードを、成功・警告・エラーに分類する(BR2.1)。
///    成功・警告は 0、エラーはジョブの終了コード。成功コードに含まれない 0 は 1 とし、
///    `conductor_error` を `true` にする(BR2.2)。
///
/// 起動して終わったのに終了コードがない場合は、エラー(1、`conductor_error`)とする。
pub fn classify(job: &JobDefinition, execution: &JobExecution) -> JobOutcome {
    let (exit_code, terminated_by_signal) = match execution.result {
        ExecutionResult::NotStarted { signal } => {
            return outcome(job, None, signal_exit(signal), false);
        }
        ExecutionResult::Exited {
            interrupted_by: Some(signal),
            ..
        } => {
            return outcome(job, Some(Classification::Error), signal_exit(signal), false);
        }
        ExecutionResult::StartFailed => {
            return outcome(job, Some(Classification::Error), EXIT_START_FAILURE, false);
        }
        ExecutionResult::Exited {
            exit_code,
            terminated_by_signal,
            interrupted_by: None,
        } => (exit_code, terminated_by_signal),
    };
    if let Some(sig) = terminated_by_signal {
        return outcome(job, Some(Classification::Error), signal_exit(sig), false);
    }
    let Some(code) = exit_code else {
        return outcome(job, Some(Classification::Error), EXIT_UNLISTED_ZERO, true);
    };
    let c = classify_exit_code(job, code);
    outcome(
        job,
        Some(c.classification),
        c.reported_exit_code,
        c.conductor_error,
    )
}

/// 終了コードの分類の結果([`classify`] の手順 5)。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ExitCodeClass {
    pub(crate) classification: Classification,
    /// スケジューラへ返す終了コード。
    pub(crate) reported_exit_code: i32,
    /// conductor のエラーか(成功コードに含まれない 0。BR2.2)。
    pub(crate) conductor_error: bool,
}

/// 終了コード `code` を、成功・警告・エラーに分類し、返す終了コードを決める([`classify`] の
/// 手順 5。BR2.1、BR2.2)。`--dry-run` の模擬も、同じ規則で結果の語を決める。
///
/// 成功コード → 成功・0、警告コード → 警告・0、成功コードに含まれない 0 → エラー・1・
/// `conductor_error`、そのほか → エラー・その終了コード。
pub(crate) fn classify_exit_code(job: &JobDefinition, code: i32) -> ExitCodeClass {
    let (classification, reported_exit_code, conductor_error) =
        if job.effective_success_codes().contains(&code) {
            (Classification::Success, 0, false)
        } else if job.warning_exit_codes.contains(&code) {
            (Classification::Warning, 0, false)
        } else if code == 0 {
            (Classification::Error, EXIT_UNLISTED_ZERO, true)
        } else {
            (Classification::Error, code, false)
        };
    ExitCodeClass {
        classification,
        reported_exit_code,
        conductor_error,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn job(success: Option<Vec<i32>>, warning: Vec<i32>) -> JobDefinition {
        JobDefinition {
            id: "a".into(),
            command: "c".into(),
            warning_exit_codes: warning,
            success_exit_codes: success,
            transitions: vec![],
            retry: None,
            env: Vec::new(),
            env_file: Vec::new(),
        }
    }

    fn exec(result: ExecutionResult) -> JobExecution {
        JobExecution::new(&job(None, Vec::new()), result)
    }

    fn code(c: i32) -> JobExecution {
        exec(ExecutionResult::Exited {
            exit_code: Some(c),
            terminated_by_signal: None,
            interrupted_by: None,
        })
    }

    #[test]
    fn 終了コードの分類と返す値() {
        let j = job(None, vec![3, 4]);
        // (終了コード, 分類, 返す値, conductorError)
        let cases = [
            (0, Classification::Success, 0, false),
            (3, Classification::Warning, 0, false),
            (4, Classification::Warning, 0, false),
            (1, Classification::Error, 1, false),
            (2, Classification::Error, 2, false),
            (255, Classification::Error, 255, false),
            (-1, Classification::Error, -1, false),
        ];
        for (c, class, reported, ce) in cases {
            let o = classify(&j, &code(c));
            assert_eq!(o.classification, Some(class), "終了コード {c}");
            assert_eq!(o.reported_exit_code, reported, "終了コード {c}");
            assert_eq!(o.conductor_error, ce, "終了コード {c}");
            assert_eq!(o.job_id, "a");
        }
    }

    #[test]
    fn 成功コードを書いたらその値だけが成功で_0_は成功にならない() {
        let j = job(Some(vec![2]), vec![]);
        let two = classify(&j, &code(2));
        assert_eq!(two.classification, Some(Classification::Success));
        assert_eq!(two.reported_exit_code, 0);
        let zero = classify(&j, &code(0));
        assert_eq!(zero.classification, Some(Classification::Error));
        assert_eq!(zero.reported_exit_code, 1);
        assert!(zero.conductor_error);
    }

    #[test]
    fn 成功コードに_0_を含めれば_0_は成功() {
        let j = job(Some(vec![0, 2]), vec![]);
        assert_eq!(
            classify(&j, &code(0)).classification,
            Some(Classification::Success)
        );
    }

    #[test]
    fn 起動失敗は_127() {
        let e = exec(ExecutionResult::StartFailed);
        let o = classify(&job(None, vec![]), &e);
        assert_eq!(o.classification, Some(Classification::Error));
        assert_eq!(o.reported_exit_code, 127);
    }

    #[test]
    fn シグナル終了は_128_足す番号() {
        let e = exec(ExecutionResult::Exited {
            exit_code: None,
            terminated_by_signal: Some(9),
            interrupted_by: None,
        });
        let o = classify(&job(None, vec![]), &e);
        assert_eq!(o.classification, Some(Classification::Error));
        assert_eq!(o.reported_exit_code, 137);
    }

    #[test]
    fn 優先順位_中断は終了コードとシグナルより先() {
        let j = job(None, vec![]);
        // 中断 + 終了コード 0 + シグナル 9: 中断が勝つ(起動失敗と中断は、型の上で同時に作れない)
        let e = exec(ExecutionResult::Exited {
            exit_code: Some(0),
            terminated_by_signal: Some(9),
            interrupted_by: Some(15),
        });
        let o = classify(&j, &e);
        assert_eq!(o.classification, Some(Classification::Error));
        assert_eq!(o.reported_exit_code, 143);
    }

    #[test]
    fn 優先順位_起動失敗はシグナル終了より先() {
        // 起動失敗はシグナルを持てない(優先は型で決まる)。
        let e = exec(ExecutionResult::StartFailed);
        assert_eq!(classify(&job(None, vec![]), &e).reported_exit_code, 127);
    }

    #[test]
    fn 起動しなかった中断は分類しない() {
        let e = exec(ExecutionResult::NotStarted { signal: 2 });
        let o = classify(&job(None, vec![]), &e);
        assert_eq!(o.classification, None);
        assert_eq!(o.reported_exit_code, 130);
        assert!(!o.conductor_error);
    }

    #[test]
    fn 想定外の組み合わせでもパニックしない() {
        let j = job(None, vec![]);
        let no_code = exec(ExecutionResult::Exited {
            exit_code: None,
            terminated_by_signal: None,
            interrupted_by: None,
        });
        let o = classify(&j, &no_code);
        assert_eq!(o.classification, Some(Classification::Error));
        assert!(o.conductor_error);
        let huge = exec(ExecutionResult::Exited {
            exit_code: Some(0),
            terminated_by_signal: None,
            interrupted_by: Some(i32::MAX),
        });
        assert_eq!(classify(&j, &huge).reported_exit_code, i32::MAX);
    }

    /// 3 つの場合の、分類・返す終了コード・`conductor_error`(BR1.3。同値の表の 6 行)。
    #[test]
    fn 三つの場合の分類は表のとおり() {
        let j = job(None, vec![3]);
        let exited = |exit_code, terminated_by_signal, interrupted_by| ExecutionResult::Exited {
            exit_code,
            terminated_by_signal,
            interrupted_by,
        };
        // (結果, 分類, 返す値, conductorError)
        let cases = [
            (ExecutionResult::NotStarted { signal: 2 }, None, 130, false),
            (
                ExecutionResult::StartFailed,
                Some(Classification::Error),
                127,
                false,
            ),
            (
                exited(Some(0), Some(9), Some(15)),
                Some(Classification::Error),
                143,
                false,
            ),
            (
                exited(None, Some(9), None),
                Some(Classification::Error),
                137,
                false,
            ),
            (
                exited(None, None, None),
                Some(Classification::Error),
                1,
                true,
            ),
            (
                exited(Some(3), None, None),
                Some(Classification::Warning),
                0,
                false,
            ),
        ];
        for (result, class, reported, ce) in cases {
            let o = classify(&j, &exec(result));
            assert_eq!(o.classification, class, "{result:?}");
            assert_eq!(o.reported_exit_code, reported, "{result:?}");
            assert_eq!(o.conductor_error, ce, "{result:?}");
        }
    }
}
