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

//! ステートの読み込み(StateReader。BR2.1〜BR2.3、NFR4.3、NFR4.5、NFR4.7)。
//!
//! 開いたハンドルの大きさ確認 → 上限 + 1 バイトまでの読み取り → 長さ 0 は空 → UTF-8 →
//! TOML → 版の確認(先) → 型付きの構造への走査と内容の矛盾の検査 → `completed` は空として返す。
//! 失敗の表示は、kind ごとの固定の文と、ステートファイルのパスだけで組み立てる。

use std::collections::HashSet;
use std::fs::File;
use std::io::{self, Read};
use std::path::Path;

use conductor_core::{JobState, JobStatus, PipelineState, SkipReason};

use crate::error::{StateError, StateErrorKind, reason};
use crate::types::StateLocation;

/// ステートファイルの大きさの上限(1 MiB)。
pub(crate) const MAX_STATE_BYTES: u64 = 1024 * 1024;

/// 書式の版(初版)。
pub(crate) const FORMAT_VERSION: i64 = 1;

/// ステートを読む(契約 C3 の `read_state`。W2)。
///
/// 入力は、定義ファイルのパス。ステートファイルの場所は、同じディレクトリの
/// `<定義ファイル名>.state`(BR2.1)。
///
/// - ファイルが存在しない、長さが 0、または `completed` が `true`(完了の初期化が、失敗または
///   中断されて残ったもの)のとき、`Ok(None)`(空のステート)。
/// - 読めない: `unreadable`。大きさが上限(1 MiB)を超える: `tooLarge`。TOML として読めない、
///   内容に誤りや矛盾がある: `corrupt`。版が 1 でない: `versionMismatch`(版の確認は、
///   ほかの検査より先)。
///
/// ファイルは、読み取りでは消さない。
pub fn read_state(definition_path: &Path) -> Result<Option<PipelineState>, StateError> {
    read_state_file(StateLocation::of(definition_path).path())
}

/// ステートファイルのパスを、直接指定して読む。
pub(crate) fn read_state_file(path: &Path) -> Result<Option<PipelineState>, StateError> {
    let fail = |kind, reason| StateError::new(path, kind, reason);

    let file = match File::open(path) {
        Ok(f) => f,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(_) => return Err(fail(StateErrorKind::Unreadable, reason::UNREADABLE)),
    };
    let size = file
        .metadata()
        .map_err(|_| fail(StateErrorKind::Unreadable, reason::UNREADABLE))?
        .len();
    if size > MAX_STATE_BYTES {
        return Err(fail(StateErrorKind::TooLarge, reason::TOO_LARGE));
    }
    let bytes = read_limited(file, MAX_STATE_BYTES)
        .map_err(|_| fail(StateErrorKind::Unreadable, reason::UNREADABLE))?;
    if bytes.len() as u64 > MAX_STATE_BYTES {
        return Err(fail(StateErrorKind::TooLarge, reason::TOO_LARGE));
    }
    parse_state(&bytes).map_err(|e| StateError::new(path, e.kind, e.reason))
}

/// 読み取りを、上限 + 1 バイトまでに制限する(終わりのない入力でも、そこで止まる)。
pub(crate) fn read_limited<R: Read>(reader: R, limit: u64) -> io::Result<Vec<u8>> {
    let mut bytes = Vec::new();
    reader
        .take(limit.saturating_add(1))
        .read_to_end(&mut bytes)?;
    Ok(bytes)
}

/// パスを持たない、解釈の失敗。
struct ParseFailure {
    kind: StateErrorKind,
    reason: &'static str,
}

fn corrupt(reason: &'static str) -> ParseFailure {
    ParseFailure {
        kind: StateErrorKind::Corrupt,
        reason,
    }
}

/// 読み取ったバイト列を、ステートにする。空のステートは `Ok(None)`。
fn parse_state(bytes: &[u8]) -> Result<Option<PipelineState>, ParseFailure> {
    if bytes.is_empty() {
        return Ok(None);
    }
    let text = std::str::from_utf8(bytes).map_err(|_| corrupt(reason::NOT_UTF8))?;
    let table: toml::Table = text.parse().map_err(|_| corrupt(reason::NOT_TOML))?;

    // 版の確認は、ほかのキーや型の検査より先(将来の版が、未知のキーで corrupt にならないように)。
    match table.get("format_version") {
        None => return Err(corrupt(reason::NO_VERSION)),
        Some(toml::Value::Integer(v)) if *v == FORMAT_VERSION => {}
        Some(toml::Value::Integer(_)) => {
            return Err(ParseFailure {
                kind: StateErrorKind::VersionMismatch,
                reason: reason::UNSUPPORTED_VERSION,
            });
        }
        Some(_) => return Err(corrupt(reason::BAD_VERSION_TYPE)),
    }

    let state = walk(&table)?;
    if state.completed {
        // 完了のまま残ったステートは、空として扱う(BR2.3)。
        return Ok(None);
    }
    Ok(Some(state))
}

const TOP_KEYS: [&str; 9] = [
    "format_version",
    "definition_key",
    "definition_path",
    "pipeline_name",
    "job_order",
    "jobs",
    "resume_job_id",
    "end_reached",
    "completed",
];

const JOB_KEYS: [&str; 4] = ["id", "status", "exit_code", "skip_reason"];

fn required<'a>(table: &'a toml::Table, key: &str) -> Result<&'a toml::Value, ParseFailure> {
    table.get(key).ok_or_else(|| corrupt(reason::MISSING_KEY))
}

fn as_string(value: &toml::Value) -> Result<String, ParseFailure> {
    value
        .as_str()
        .map(str::to_string)
        .ok_or_else(|| corrupt(reason::WRONG_TYPE))
}

fn as_bool(value: &toml::Value) -> Result<bool, ParseFailure> {
    value.as_bool().ok_or_else(|| corrupt(reason::WRONG_TYPE))
}

fn parse_status(text: &str) -> Result<JobStatus, ParseFailure> {
    match text {
        "pending" => Ok(JobStatus::Pending),
        "success" => Ok(JobStatus::Success),
        "warning" => Ok(JobStatus::Warning),
        "error" => Ok(JobStatus::Error),
        "skipped" => Ok(JobStatus::Skipped),
        _ => Err(corrupt(reason::BAD_STATUS)),
    }
}

fn parse_skip_reason(text: &str) -> Result<SkipReason, ParseFailure> {
    match text {
        "byTransition" => Ok(SkipReason::ByTransition),
        "byEnd" => Ok(SkipReason::ByEnd),
        _ => Err(corrupt(reason::BAD_SKIP_REASON)),
    }
}

fn walk(table: &toml::Table) -> Result<PipelineState, ParseFailure> {
    if table.keys().any(|k| !TOP_KEYS.contains(&k.as_str())) {
        return Err(corrupt(reason::UNKNOWN_KEY));
    }

    let definition_key = as_string(required(table, "definition_key")?)?;
    let definition_path = as_string(required(table, "definition_path")?)?;
    let pipeline_name = as_string(required(table, "pipeline_name")?)?;
    let end_reached = as_bool(required(table, "end_reached")?)?;
    let completed = as_bool(required(table, "completed")?)?;

    let job_order = required(table, "job_order")?
        .as_array()
        .ok_or_else(|| corrupt(reason::WRONG_TYPE))?
        .iter()
        .map(as_string)
        .collect::<Result<Vec<_>, _>>()?;

    let jobs = required(table, "jobs")?
        .as_array()
        .ok_or_else(|| corrupt(reason::WRONG_TYPE))?;
    let mut job_states = Vec::with_capacity(jobs.len());
    for job in jobs {
        let job = job.as_table().ok_or_else(|| corrupt(reason::WRONG_TYPE))?;
        job_states.push(walk_job(job)?);
    }

    let resume_job_id = match table.get("resume_job_id") {
        None => None,
        Some(v) => Some(as_string(v)?),
    };

    // 内容の矛盾の検査(BR2.3)。
    let mut seen = HashSet::new();
    if !job_states.iter().all(|j| seen.insert(j.job_id.as_str())) {
        return Err(corrupt(reason::DUPLICATE_JOB));
    }
    let same_order = job_states.len() == job_order.len()
        && job_states
            .iter()
            .zip(&job_order)
            .all(|(j, id)| &j.job_id == id);
    if !same_order {
        return Err(corrupt(reason::ORDER_MISMATCH));
    }
    if let Some(resume) = &resume_job_id {
        if !job_order.contains(resume) {
            return Err(corrupt(reason::RESUME_NOT_IN_ORDER));
        }
        if completed {
            return Err(corrupt(reason::COMPLETED_WITH_RESUME));
        }
    }

    Ok(PipelineState {
        definition_key,
        pipeline_name,
        definition_path,
        job_order,
        job_states,
        resume_job_id,
        end_reached,
        completed,
    })
}

fn walk_job(job: &toml::Table) -> Result<JobState, ParseFailure> {
    if job.keys().any(|k| !JOB_KEYS.contains(&k.as_str())) {
        return Err(corrupt(reason::UNKNOWN_KEY));
    }
    let job_id = as_string(required(job, "id")?)?;
    let status = parse_status(&as_string(required(job, "status")?)?)?;
    let exit_code = match job.get("exit_code") {
        None => None,
        Some(v) => {
            let n = v.as_integer().ok_or_else(|| corrupt(reason::WRONG_TYPE))?;
            Some(i32::try_from(n).map_err(|_| corrupt(reason::WRONG_TYPE))?)
        }
    };
    let skip_reason = match job.get("skip_reason") {
        None => None,
        Some(v) => Some(parse_skip_reason(&as_string(v)?)?),
    };

    if (status == JobStatus::Skipped) != skip_reason.is_some() {
        return Err(corrupt(reason::SKIP_REASON_MISMATCH));
    }
    let has_result = matches!(
        status,
        JobStatus::Success | JobStatus::Warning | JobStatus::Error
    );
    if exit_code.is_some() && !has_result {
        return Err(corrupt(reason::EXIT_CODE_MISMATCH));
    }
    Ok(JobState {
        job_id,
        status,
        exit_code,
        skip_reason,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const VALID: &str = r#"
format_version = 1
definition_key = "conductor.toml"
definition_path = "conductor.toml"
pipeline_name = "p"
job_order = ["a", "b"]
end_reached = false
completed = false
resume_job_id = "b"

[[jobs]]
id = "a"
status = "success"
exit_code = 0

[[jobs]]
id = "b"
status = "pending"
"#;

    fn kind_of(text: &str) -> (StateErrorKind, &'static str) {
        match parse_state(text.as_bytes()) {
            Err(e) => (e.kind, e.reason),
            Ok(_) => panic!("拒否されるはず: {text}"),
        }
    }

    #[test]
    fn 正しいステートを読める() {
        let state = parse_state(VALID.as_bytes())
            .ok()
            .flatten()
            .expect("読める");
        assert_eq!(state.pipeline_name, "p");
        assert_eq!(state.job_order, ["a", "b"]);
        assert_eq!(state.resume_job_id.as_deref(), Some("b"));
        assert_eq!(state.job_states[0].status, JobStatus::Success);
        assert_eq!(state.job_states[0].exit_code, Some(0));
        assert_eq!(state.job_states[1].status, JobStatus::Pending);
    }

    #[test]
    fn 長さ_0_は空() {
        assert!(matches!(parse_state(b""), Ok(None)));
    }

    #[test]
    fn 上限_1_つぶんだけ読む() {
        let endless = io::repeat(b'a');
        let bytes = read_limited(endless, 10).expect("読める");
        assert_eq!(bytes.len(), 11);
        let bytes = read_limited(&b"abc"[..], 10).expect("読める");
        assert_eq!(bytes, b"abc");
    }

    #[test]
    fn 版を先に確認する() {
        let future = "format_version = 2\nunknown_key = 1\njobs = 5\n";
        assert_eq!(kind_of(future).0, StateErrorKind::VersionMismatch);
        assert_eq!(
            kind_of("format_version = 0\n").0,
            StateErrorKind::VersionMismatch
        );
        assert_eq!(
            kind_of("a = 1\n"),
            (StateErrorKind::Corrupt, reason::NO_VERSION)
        );
        assert_eq!(
            kind_of("format_version = \"1\"\n"),
            (StateErrorKind::Corrupt, reason::BAD_VERSION_TYPE)
        );
    }

    #[test]
    fn 壊れた_toml_と不正な_utf8() {
        assert_eq!(
            kind_of("format_version = = 1"),
            (StateErrorKind::Corrupt, reason::NOT_TOML)
        );
        match parse_state(&[0xff, 0xfe, b'a']) {
            Err(e) => assert_eq!(
                (e.kind, e.reason),
                (StateErrorKind::Corrupt, reason::NOT_UTF8)
            ),
            Ok(_) => panic!("拒否されるはず"),
        }
    }

    #[test]
    fn 未知のキー() {
        let t = VALID.replace("completed = false", "completed = false\nextra = 1");
        assert_eq!(kind_of(&t), (StateErrorKind::Corrupt, reason::UNKNOWN_KEY));
        let t = VALID.replace("status = \"pending\"", "status = \"pending\"\nextra = 1");
        assert_eq!(kind_of(&t), (StateErrorKind::Corrupt, reason::UNKNOWN_KEY));
    }

    #[test]
    fn 型の誤りと必須のキーの欠落() {
        let t = VALID.replace("pipeline_name = \"p\"", "pipeline_name = 5");
        assert_eq!(kind_of(&t), (StateErrorKind::Corrupt, reason::WRONG_TYPE));
        let t = VALID.replace("end_reached = false\n", "");
        assert_eq!(kind_of(&t), (StateErrorKind::Corrupt, reason::MISSING_KEY));
        let t = VALID.replace("job_order = [\"a\", \"b\"]", "job_order = \"a\"");
        assert_eq!(kind_of(&t), (StateErrorKind::Corrupt, reason::WRONG_TYPE));
        let t = VALID.replace("job_order = [\"a\", \"b\"]", "job_order = [1, 2]");
        assert_eq!(kind_of(&t), (StateErrorKind::Corrupt, reason::WRONG_TYPE));
        let t = VALID.replace("exit_code = 0", "exit_code = \"0\"");
        assert_eq!(kind_of(&t), (StateErrorKind::Corrupt, reason::WRONG_TYPE));
        let t = VALID.replace("exit_code = 0", "exit_code = 99999999999");
        assert_eq!(kind_of(&t), (StateErrorKind::Corrupt, reason::WRONG_TYPE));
        let t = VALID.replace("id = \"a\"\n", "");
        assert_eq!(kind_of(&t), (StateErrorKind::Corrupt, reason::MISSING_KEY));
        let t = VALID.replace("resume_job_id = \"b\"", "resume_job_id = 3");
        assert_eq!(kind_of(&t), (StateErrorKind::Corrupt, reason::WRONG_TYPE));
        let t = VALID.replace("[[jobs]]\nid = \"a\"", "[[jobs]]\nid = 1");
        assert_eq!(kind_of(&t), (StateErrorKind::Corrupt, reason::WRONG_TYPE));
        let t = VALID.replace("\n[[jobs]]\nid = \"a\"\nstatus = \"success\"\nexit_code = 0\n\n[[jobs]]\nid = \"b\"\nstatus = \"pending\"\n", "jobs = [1]\n");
        assert_eq!(kind_of(&t), (StateErrorKind::Corrupt, reason::WRONG_TYPE));
        let t = VALID.replace("\n[[jobs]]\nid = \"a\"\nstatus = \"success\"\nexit_code = 0\n\n[[jobs]]\nid = \"b\"\nstatus = \"pending\"\n", "jobs = \"x\"\n");
        assert_eq!(kind_of(&t), (StateErrorKind::Corrupt, reason::WRONG_TYPE));
    }

    #[test]
    fn status_と_skip_reason_の不正() {
        let t = VALID.replace("status = \"pending\"", "status = \"done\"");
        assert_eq!(kind_of(&t), (StateErrorKind::Corrupt, reason::BAD_STATUS));
        // skipped なのに skip_reason が無い
        let t = VALID.replace("status = \"pending\"", "status = \"skipped\"");
        assert_eq!(
            kind_of(&t),
            (StateErrorKind::Corrupt, reason::SKIP_REASON_MISMATCH)
        );
        // skipped でないのに skip_reason がある
        let t = VALID.replace(
            "status = \"pending\"",
            "status = \"pending\"\nskip_reason = \"byEnd\"",
        );
        assert_eq!(
            kind_of(&t),
            (StateErrorKind::Corrupt, reason::SKIP_REASON_MISMATCH)
        );
        let t = VALID.replace(
            "status = \"pending\"",
            "status = \"skipped\"\nskip_reason = \"other\"",
        );
        assert_eq!(
            kind_of(&t),
            (StateErrorKind::Corrupt, reason::BAD_SKIP_REASON)
        );
    }

    #[test]
    fn exit_code_の矛盾() {
        let t = VALID.replace(
            "status = \"pending\"",
            "status = \"pending\"\nexit_code = 1",
        );
        assert_eq!(
            kind_of(&t),
            (StateErrorKind::Corrupt, reason::EXIT_CODE_MISMATCH)
        );
        let t = VALID.replace(
            "status = \"pending\"",
            "status = \"skipped\"\nskip_reason = \"byEnd\"\nexit_code = 1",
        );
        assert_eq!(
            kind_of(&t),
            (StateErrorKind::Corrupt, reason::EXIT_CODE_MISMATCH)
        );
    }

    #[test]
    fn id_の重複と並びの不一致() {
        let t = VALID.replace("id = \"b\"", "id = \"a\"");
        assert_eq!(
            kind_of(&t),
            (StateErrorKind::Corrupt, reason::DUPLICATE_JOB)
        );
        let t = VALID.replace("job_order = [\"a\", \"b\"]", "job_order = [\"b\", \"a\"]");
        assert_eq!(
            kind_of(&t),
            (StateErrorKind::Corrupt, reason::ORDER_MISMATCH)
        );
        let t = VALID.replace("job_order = [\"a\", \"b\"]", "job_order = [\"a\"]");
        assert_eq!(
            kind_of(&t),
            (StateErrorKind::Corrupt, reason::ORDER_MISMATCH)
        );
    }

    #[test]
    fn resume_job_id_の矛盾() {
        let t = VALID.replace("resume_job_id = \"b\"", "resume_job_id = \"zz\"");
        assert_eq!(
            kind_of(&t),
            (StateErrorKind::Corrupt, reason::RESUME_NOT_IN_ORDER)
        );
        let t = VALID.replace("completed = false", "completed = true");
        assert_eq!(
            kind_of(&t),
            (StateErrorKind::Corrupt, reason::COMPLETED_WITH_RESUME)
        );
    }

    #[test]
    fn completed_のステートは空() {
        let t = VALID
            .replace("completed = false", "completed = true")
            .replace("resume_job_id = \"b\"\n", "");
        assert!(matches!(parse_state(t.as_bytes()), Ok(None)));
    }

    #[test]
    fn resume_job_id_は省略できる() {
        let t = VALID.replace("resume_job_id = \"b\"\n", "");
        let state = parse_state(t.as_bytes()).ok().flatten().expect("読める");
        assert_eq!(state.resume_job_id, None);
    }
}
