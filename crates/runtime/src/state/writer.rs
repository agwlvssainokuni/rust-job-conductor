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

//! ステートの書き込みと初期化(StateWriter、Reset。BR2.2、BR2.4、BR2.6、NFR4.4)。
//!
//! 一時ファイルを新規作成で作り、全内容を書き、`Durable` のときだけ同期し、置き換える(`rename`)。
//! どの段階で失敗しても、ステートファイルは直前の完全な内容のままで、自分が作った一時ファイルは消す。

use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use conductor_core::{JobState, PipelineState, RunId};
// 単体テスト(`mod tests` の `use super::*`)が使う。
#[cfg(test)]
use conductor_core::JobStatus;

use crate::error::{StateError, StateErrorKind, reason};
use crate::platform;
use crate::state::reader::FORMAT_VERSION;
use crate::types::{Durability, StateLocation};

/// 一時ファイルの名前が重なったとき、連番を進めて作り直す回数の上限。
const TEMP_ATTEMPTS: u32 = 16;

/// 残った一時ファイルを、後始末で消す古さ(24 時間)。
const STALE_AFTER: Duration = Duration::from_secs(24 * 60 * 60);

static SEQUENCE: AtomicU32 = AtomicU32::new(0);

/// ステートを書く(契約 C3 の `write_state`。W3)。
///
/// 入力は、定義ファイルのパス(ステートファイルの場所は、同じディレクトリの
/// `<定義ファイル名>.state`)、ステート、耐久性の指定。
///
/// - 同じディレクトリに、一時ファイル `.<ステートファイル名>.tmp.<プロセス番号>.<時刻>.<連番>` を、
///   新規作成で作り(既存なら連番を進めて、最大 16 回)、全内容(TOML、`format_version = 1`)を書く。
/// - [`Durability::Durable`] のときだけ、内容を同期し、置き換えのあと、UNIX 系でディレクトリも
///   同期する(失敗は無視)。[`Durability::Fast`] は、同期を省く(起動前の初期ステート)。
/// - 成功した `Durable` の書き込みのあと、同じ名前の型で、更新から 24 時間以上たった一時ファイルを
///   消す(失敗は無視)。
/// - 失敗したら、自分が作った一時ファイルを消し、`writeFailed`。ステートファイルは変わらない。
pub fn write_state(
    definition_path: &Path,
    run_id: Option<&RunId>,
    state: &PipelineState,
    durability: Durability,
) -> Result<(), StateError> {
    write_state_file(
        StateLocation::of(definition_path, run_id).path(),
        state,
        durability,
    )
}

pub(crate) fn write_state_file(
    path: &Path,
    state: &PipelineState,
    durability: Durability,
) -> Result<(), StateError> {
    let fail = || StateError::new(path, StateErrorKind::WriteFailed, reason::WRITE_FAILED);
    let text = serialize(state).ok_or_else(|| {
        StateError::new(path, StateErrorKind::WriteFailed, reason::SERIALIZE_FAILED)
    })?;

    let dir = path.parent().unwrap_or_else(|| Path::new(""));
    let state_name = path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .ok_or_else(fail)?;
    let pid = std::process::id();
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);

    let (temp_path, mut file) = create_temp(dir, |attempt| {
        let seq = SEQUENCE
            .fetch_add(1, Ordering::Relaxed)
            .wrapping_add(attempt);
        temp_name(&state_name, pid, nanos, seq)
    })
    .map_err(|_| fail())?;

    let written = persist(&mut file, text.as_bytes(), durability, |f| f.sync_all());
    // 置き換えと後始末の前に、閉じる(Windows では、開いたままのファイルは、移せず、消せない)。
    drop(file);
    let result = written.and_then(|()| fs::rename(&temp_path, path));
    if result.is_err() {
        let _ = fs::remove_file(&temp_path);
        return Err(fail());
    }

    if durability == Durability::Durable {
        platform::sync_dir(dir);
        remove_stale_temps(dir, &state_name, SystemTime::now());
    }
    Ok(())
}

/// ステートを初期化する(契約 C3 の `reset_state`。W5。BR2.6)。
///
/// 読まずに(破損、食い違い、版の不一致の検査をせず)削除する。存在しなければ、何もせず成功。
/// 削除に失敗したら `removeFailed`。
pub fn reset_state(definition_path: &Path, run_id: Option<&RunId>) -> Result<(), StateError> {
    let location = StateLocation::of(definition_path, run_id);
    match fs::remove_file(location.path()) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(_) => Err(StateError::new(
            location.path(),
            StateErrorKind::RemoveFailed,
            reason::REMOVE_FAILED,
        )),
    }
}

/// 一時ファイルの名前。
pub(crate) fn temp_name(state_name: &str, pid: u32, nanos: u128, seq: u32) -> String {
    format!(".{state_name}.tmp.{pid}.{nanos}.{seq}")
}

/// 一時ファイルを、新規作成で作る(既存なら失敗するため、他のファイルを上書きしない)。
/// 名前が重なったら、`name_for(試行の番号)` で名前を変えて、最大 16 回まで作り直す。
pub(crate) fn create_temp(
    dir: &Path,
    mut name_for: impl FnMut(u32) -> String,
) -> io::Result<(PathBuf, File)> {
    let mut last = io::Error::from(io::ErrorKind::AlreadyExists);
    for attempt in 0..TEMP_ATTEMPTS {
        let path = dir.join(name_for(attempt));
        match OpenOptions::new().write(true).create_new(true).open(&path) {
            Ok(file) => return Ok((path, file)),
            Err(e) if e.kind() == io::ErrorKind::AlreadyExists => last = e,
            Err(e) => return Err(e),
        }
    }
    Err(last)
}

/// 内容を書き、`Durable` のときだけ同期する。
pub(crate) fn persist<W: Write>(
    file: &mut W,
    bytes: &[u8],
    durability: Durability,
    sync: impl FnOnce(&W) -> io::Result<()>,
) -> io::Result<()> {
    file.write_all(bytes)?;
    file.flush()?;
    if durability == Durability::Durable {
        sync(file)?;
    }
    Ok(())
}

/// 同じ名前の型で、更新から 24 時間以上たった一時ファイルを消す(失敗は無視)。
pub(crate) fn remove_stale_temps(dir: &Path, state_name: &str, now: SystemTime) {
    let prefix = format!(".{state_name}.tmp.");
    let dir = if dir.as_os_str().is_empty() {
        Path::new(".")
    } else {
        dir
    };
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let name = entry.file_name();
        if !name.to_string_lossy().starts_with(&prefix) {
            continue;
        }
        let Ok(meta) = entry.metadata() else {
            continue;
        };
        if !meta.is_file() {
            continue;
        }
        let stale = meta
            .modified()
            .ok()
            .and_then(|m| now.duration_since(m).ok())
            .is_some_and(|age| age >= STALE_AFTER);
        if stale {
            let _ = fs::remove_file(entry.path());
        }
    }
}

fn job_table(job: &JobState) -> toml::Table {
    let mut t = toml::Table::new();
    t.insert("id".into(), toml::Value::String(job.job_id.clone()));
    t.insert(
        "status".into(),
        toml::Value::String(job.status.as_str().into()),
    );
    if let Some(code) = job.exit_code {
        t.insert("exit_code".into(), toml::Value::Integer(i64::from(code)));
    }
    if let Some(r) = job.skip_reason {
        t.insert("skip_reason".into(), toml::Value::String(r.as_str().into()));
    }
    t
}

/// ステートを、TOML の文字列にする(E3)。`definition_key` と `definition_path` は、同じ文字列。
pub(crate) fn serialize(state: &PipelineState) -> Option<String> {
    let mut t = toml::Table::new();
    t.insert(
        "format_version".into(),
        toml::Value::Integer(FORMAT_VERSION),
    );
    t.insert(
        "definition_key".into(),
        toml::Value::String(state.definition_path.clone()),
    );
    t.insert(
        "definition_path".into(),
        toml::Value::String(state.definition_path.clone()),
    );
    t.insert(
        "pipeline_name".into(),
        toml::Value::String(state.pipeline_name.clone()),
    );
    t.insert(
        "job_order".into(),
        toml::Value::Array(
            state
                .job_order
                .iter()
                .map(|id| toml::Value::String(id.clone()))
                .collect(),
        ),
    );
    t.insert(
        "job".into(),
        toml::Value::Array(
            state
                .job_states
                .iter()
                .map(|j| toml::Value::Table(job_table(j)))
                .collect(),
        ),
    );
    if let Some(id) = &state.resume_job_id {
        t.insert("resume_job_id".into(), toml::Value::String(id.clone()));
    }
    // run ID なしのステートには、キーを書かない(形を変えない。NFR1)。
    if let Some(id) = &state.run_id {
        t.insert("run_id".into(), toml::Value::String(id.clone()));
    }
    t.insert(
        "end_reached".into(),
        toml::Value::Boolean(state.end_reached),
    );
    t.insert("completed".into(), toml::Value::Boolean(state.completed));
    toml::to_string(&t).ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    fn state() -> PipelineState {
        PipelineState {
            definition_key: "d/conductor.toml".into(),
            pipeline_name: "p".into(),
            definition_path: "d/conductor.toml".into(),
            job_order: vec!["a".into(), "b".into()],
            job_states: vec![
                JobState {
                    job_id: "a".into(),
                    status: JobStatus::Error,
                    exit_code: Some(3),
                    skip_reason: None,
                },
                JobState::pending("b"),
            ],
            resume_job_id: Some("a".into()),
            end_reached: false,
            completed: false,
            run_id: None,
        }
    }

    #[test]
    fn run_id_はあるときだけ直列化される() {
        let without = serialize(&state()).expect("直列化できる");
        assert!(!without.contains("run_id"));
        let mut s = state();
        s.run_id = Some("r-1".into());
        let with = serialize(&s).expect("直列化できる");
        assert!(with.contains("run_id = \"r-1\""));
    }

    #[test]
    fn 一時ファイルの名前の形() {
        assert_eq!(
            temp_name("c.toml.state", 7, 123, 4),
            ".c.toml.state.tmp.7.123.4"
        );
    }

    #[test]
    fn 直列化に定義のパスが_2_つのキーで入る() {
        let text = serialize(&state()).expect("直列化できる");
        assert!(text.contains("format_version = 1"));
        assert_eq!(text.matches("\"d/conductor.toml\"").count(), 2);
        assert!(text.contains("resume_job_id = \"a\""));
        assert!(text.contains("exit_code = 3"));
    }

    #[test]
    fn 名前が重なったら連番を進める() {
        let dir = tempfile::tempdir().expect("一時ディレクトリ");
        std::fs::write(dir.path().join("t0"), "existing").expect("書ける");
        std::fs::write(dir.path().join("t1"), "existing").expect("書ける");
        let (path, _file) = create_temp(dir.path(), |n| format!("t{n}")).expect("作れる");
        assert_eq!(path.file_name().and_then(|n| n.to_str()), Some("t2"));
        assert_eq!(
            std::fs::read_to_string(dir.path().join("t0"))
                .ok()
                .as_deref(),
            Some("existing"),
            "既存のファイルは上書きされない"
        );
    }

    #[test]
    fn 名前が_16_回とも重なれば失敗() {
        let dir = tempfile::tempdir().expect("一時ディレクトリ");
        std::fs::write(dir.path().join("same"), "x").expect("書ける");
        let calls = Cell::new(0);
        let result = create_temp(dir.path(), |_| {
            calls.set(calls.get() + 1);
            "same".to_string()
        });
        assert!(result.is_err());
        assert_eq!(calls.get(), 16);
    }

    #[test]
    fn 作成のそれ以外の失敗はすぐに返す() {
        let calls = Cell::new(0);
        let result = create_temp(Path::new("/definitely/not/existing/dir"), |_| {
            calls.set(calls.get() + 1);
            "x".to_string()
        });
        assert!(result.is_err());
        assert_eq!(calls.get(), 1);
    }

    #[test]
    fn durable_のときだけ同期する() {
        let synced = Cell::new(0);
        let mut buf: Vec<u8> = Vec::new();
        persist(&mut buf, b"abc", Durability::Durable, |_| {
            synced.set(synced.get() + 1);
            Ok(())
        })
        .expect("書ける");
        assert_eq!((buf.as_slice(), synced.get()), (&b"abc"[..], 1));

        let mut buf2: Vec<u8> = Vec::new();
        persist(&mut buf2, b"abc", Durability::Fast, |_| {
            synced.set(synced.get() + 1);
            Ok(())
        })
        .expect("書ける");
        assert_eq!(synced.get(), 1, "Fast は同期しない");
    }

    #[test]
    fn 同期の失敗は書き込みの失敗() {
        let mut buf: Vec<u8> = Vec::new();
        let r = persist(&mut buf, b"x", Durability::Durable, |_| {
            Err(io::Error::other("sync"))
        });
        assert!(r.is_err());
    }

    #[test]
    fn 古い一時ファイルだけを消す() {
        let dir = tempfile::tempdir().expect("一時ディレクトリ");
        let old = dir.path().join(".c.toml.state.tmp.1.1.1");
        let fresh = dir.path().join(".c.toml.state.tmp.2.2.2");
        let other = dir.path().join(".other.state.tmp.1.1.1");
        let plain = dir.path().join("c.toml.state");
        for p in [&old, &fresh, &other, &plain] {
            std::fs::write(p, "x").expect("書ける");
        }
        let two_days = Duration::from_secs(48 * 60 * 60);
        let f = OpenOptions::new().write(true).open(&old).expect("開ける");
        f.set_modified(SystemTime::now() - two_days)
            .expect("更新時刻を変えられる");
        let g = OpenOptions::new().write(true).open(&other).expect("開ける");
        g.set_modified(SystemTime::now() - two_days)
            .expect("更新時刻を変えられる");
        drop((f, g));

        remove_stale_temps(dir.path(), "c.toml.state", SystemTime::now());
        assert!(!old.exists(), "古い一時ファイルは消える");
        assert!(fresh.exists(), "新しい一時ファイルは残る");
        assert!(other.exists(), "別のステートの一時ファイルは触らない");
        assert!(plain.exists(), "ステートファイルは触らない");
    }

    #[test]
    fn 読めないディレクトリの後始末は何もしない() {
        remove_stale_temps(
            Path::new("/definitely/not/existing"),
            "x",
            SystemTime::now(),
        );
    }
}
