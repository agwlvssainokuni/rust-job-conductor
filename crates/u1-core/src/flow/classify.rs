//! 分類と、スケジューラへ返す終了コード(BR2.1〜BR2.4)。

use crate::types::{Classification, JobDefinition, JobExecution, JobOutcome};

/// 起動に失敗したときの終了コード(BR2.3)。
pub const EXIT_START_FAILURE: i32 = 127;

/// シグナルによる終了コードの基準(128 + シグナル番号)。
pub const EXIT_SIGNAL_BASE: i32 = 128;

/// 成功コードに含まれない 0 のときに返す終了コード(BR2.2)。
pub const EXIT_UNLISTED_ZERO: i32 = 1;

/// `128 + シグナル番号`。番号が `None` のときは `128 + 0 = 128` になる(番号が取れなかった場合の
/// 既定の挙動で、仕様どおり。呼び出し側は、取れたときは必ず番号を渡す)。
fn signal_exit(signal: Option<i32>) -> i32 {
    EXIT_SIGNAL_BASE.saturating_add(signal.unwrap_or(0))
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
/// 1. 起動しなかった中断: 分類しない(`classification` は `None`)。`128 + signal_number`(BR2.4)。
/// 2. 中断(起動済み): エラー、`128 + signal_number`。ジョブ自身の終了コードは見ない。
/// 3. 起動失敗: エラー、127。
/// 4. シグナル終了: エラー、`128 + シグナル番号`。
/// 5. 上のどれでもない: 終了コードを、成功・警告・エラーに分類する(BR2.1)。
///    成功・警告は 0、エラーはジョブの終了コード。成功コードに含まれない 0 は 1 とし、
///    `conductor_error` を `true` にする(BR2.2)。
///
/// 起動していないのに中断でもない(あり得ない組み合わせ)は、起動失敗と同じ扱いにする。
/// 起動して終わったのに終了コードがない場合は、エラー(1、`conductor_error`)とする。
pub fn classify(job: &JobDefinition, execution: &JobExecution) -> JobOutcome {
    if !execution.started {
        return if execution.interrupted {
            outcome(job, None, signal_exit(execution.signal_number), false)
        } else {
            outcome(job, Some(Classification::Error), EXIT_START_FAILURE, false)
        };
    }
    if execution.interrupted {
        return outcome(
            job,
            Some(Classification::Error),
            signal_exit(execution.signal_number),
            false,
        );
    }
    if execution.start_failure {
        return outcome(job, Some(Classification::Error), EXIT_START_FAILURE, false);
    }
    if let Some(sig) = execution.terminated_by_signal {
        return outcome(
            job,
            Some(Classification::Error),
            signal_exit(Some(sig)),
            false,
        );
    }
    let Some(code) = execution.exit_code else {
        return outcome(job, Some(Classification::Error), EXIT_UNLISTED_ZERO, true);
    };
    if job.effective_success_codes().contains(&code) {
        outcome(job, Some(Classification::Success), 0, false)
    } else if job.warning_exit_codes.contains(&code) {
        outcome(job, Some(Classification::Warning), 0, false)
    } else if code == 0 {
        outcome(job, Some(Classification::Error), EXIT_UNLISTED_ZERO, true)
    } else {
        outcome(job, Some(Classification::Error), code, false)
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
        }
    }

    fn exec() -> JobExecution {
        JobExecution {
            job_id: "a".into(),
            command: "c".into(),
            started: true,
            exit_code: Some(0),
            terminated_by_signal: None,
            start_failure: false,
            interrupted: false,
            signal_number: None,
        }
    }

    fn code(c: i32) -> JobExecution {
        JobExecution {
            exit_code: Some(c),
            ..exec()
        }
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
        let e = JobExecution {
            exit_code: None,
            start_failure: true,
            ..exec()
        };
        let o = classify(&job(None, vec![]), &e);
        assert_eq!(o.classification, Some(Classification::Error));
        assert_eq!(o.reported_exit_code, 127);
    }

    #[test]
    fn シグナル終了は_128_足す番号() {
        let e = JobExecution {
            exit_code: None,
            terminated_by_signal: Some(9),
            ..exec()
        };
        let o = classify(&job(None, vec![]), &e);
        assert_eq!(o.classification, Some(Classification::Error));
        assert_eq!(o.reported_exit_code, 137);
    }

    #[test]
    fn 優先順位_中断は終了コードとシグナルより先() {
        let j = job(None, vec![]);
        // 中断 + 終了コード 0 + シグナル 9 + 起動失敗: 中断が勝つ
        let e = JobExecution {
            exit_code: Some(0),
            terminated_by_signal: Some(9),
            start_failure: true,
            interrupted: true,
            signal_number: Some(15),
            ..exec()
        };
        let o = classify(&j, &e);
        assert_eq!(o.classification, Some(Classification::Error));
        assert_eq!(o.reported_exit_code, 143);
    }

    #[test]
    fn 優先順位_起動失敗はシグナル終了より先() {
        let e = JobExecution {
            exit_code: None,
            terminated_by_signal: Some(9),
            start_failure: true,
            ..exec()
        };
        assert_eq!(classify(&job(None, vec![]), &e).reported_exit_code, 127);
    }

    #[test]
    fn 起動しなかった中断は分類しない() {
        let e = JobExecution {
            started: false,
            exit_code: None,
            interrupted: true,
            signal_number: Some(2),
            ..exec()
        };
        let o = classify(&job(None, vec![]), &e);
        assert_eq!(o.classification, None);
        assert_eq!(o.reported_exit_code, 130);
        assert!(!o.conductor_error);
    }

    #[test]
    fn 想定外の組み合わせでもパニックしない() {
        let j = job(None, vec![]);
        let not_started = JobExecution {
            started: false,
            exit_code: None,
            ..exec()
        };
        assert_eq!(classify(&j, &not_started).reported_exit_code, 127);
        let no_code = JobExecution {
            exit_code: None,
            ..exec()
        };
        let o = classify(&j, &no_code);
        assert_eq!(o.classification, Some(Classification::Error));
        assert!(o.conductor_error);
        let huge = JobExecution {
            interrupted: true,
            signal_number: Some(i32::MAX),
            ..exec()
        };
        assert_eq!(classify(&j, &huge).reported_exit_code, i32::MAX);
    }
}
