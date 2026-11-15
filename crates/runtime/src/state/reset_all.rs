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

//! `reset --all`(FR5。NFR2)。ある定義のステートを、run ID の有無によらず、すべて消す。
//!
//! 対象の判定は、次のとおり(厳密)。
//! - 一覧にするのは、定義ファイルと同じディレクトリだけ(再帰しない)。
//! - 通常のファイルだけを見る(`symlink_metadata`。シンボリックリンクは辿らず、対象にしない)。
//! - 名前が、`<定義ファイル名>.state`、または `<定義ファイル名>.<有効な run ID>.state` のものだけ。
//!   一時ファイル(`.<ステート名>.tmp.*`)、ほかの名前、ほかのディレクトリは、対象外。
//! - run ID 付きは、中身の `run_id` が名前の run ID と一致するときだけ消す。中身に `run_id` が
//!   ない、または値が違うもの(別の定義のステートでありうる)は、消さず、黙って残す。
//! - 読めない・壊れているものは、消さずに、名前を `kept` に入れる。

use std::fs;
use std::io;
use std::path::Path;

use conductor_core::RunId;

use crate::error::{StateError, StateErrorKind, reason};
use crate::state::reader::{Inspected, inspect_run_state};
use crate::types::StateLocation;

/// `reset --all` の結果。
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ResetAllReport {
    /// 消したファイルの数。
    pub removed: usize,
    /// 読めない・壊れているために残したファイルの名前(並べ替え済み)。
    pub kept: Vec<String>,
}

/// 定義のステートを、すべて消す(FR5)。
pub fn reset_all_states(definition_path: &Path) -> Result<ResetAllReport, StateError> {
    let mut report = ResetAllReport::default();

    // run ID なしのステート。読まずに消す(`reset` と同じ)。通常のファイルだけ。
    let plain = StateLocation::of(definition_path, None);
    if is_regular_file(plain.path()) {
        remove(plain.path(), &mut report)?;
    }

    let Some(base) = definition_path.file_name().and_then(|n| n.to_str()) else {
        return Ok(report);
    };
    let dir = match definition_path.parent() {
        Some(p) if !p.as_os_str().is_empty() => p,
        _ => Path::new("."),
    };
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(report),
        Err(_) => {
            return Err(StateError::new(
                dir,
                StateErrorKind::Unreadable,
                reason::UNREADABLE,
            ));
        }
    };

    let mut candidates: Vec<(String, RunId)> = Vec::new();
    for entry in entries.flatten() {
        let file_name = entry.file_name();
        let Some(name) = file_name.to_str() else {
            continue;
        };
        if let Some(id) = run_id_in_name(base, name) {
            candidates.push((name.to_string(), id));
        }
    }
    candidates.sort_by(|a, b| a.0.cmp(&b.0));

    for (name, id) in candidates {
        let path = dir.join(&name);
        match fs::symlink_metadata(&path) {
            Ok(m) if m.file_type().is_file() => {}
            Ok(_) => continue,
            Err(e) if e.kind() == io::ErrorKind::NotFound => continue,
            Err(_) => {
                report.kept.push(name);
                continue;
            }
        }
        match inspect_run_state(&path, &id) {
            Inspected::Ours => remove(&path, &mut report)?,
            Inspected::NotOurs => {}
            Inspected::Broken => report.kept.push(name),
        }
    }
    Ok(report)
}

/// `<定義ファイル名>.<run-id>.state` の形なら、有効な run ID を返す。
fn run_id_in_name(base: &str, name: &str) -> Option<RunId> {
    let rest = name.strip_prefix(base)?.strip_prefix('.')?;
    let id = rest.strip_suffix(".state")?;
    RunId::parse(id).ok()
}

fn is_regular_file(path: &Path) -> bool {
    fs::symlink_metadata(path)
        .map(|m| m.file_type().is_file())
        .unwrap_or(false)
}

fn remove(path: &Path, report: &mut ResetAllReport) -> Result<(), StateError> {
    match fs::remove_file(path) {
        Ok(()) => {
            report.removed += 1;
            Ok(())
        }
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(_) => Err(StateError::new(
            path,
            StateErrorKind::RemoveFailed,
            reason::REMOVE_FAILED,
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn 名前から_run_id_を取り出す() {
        let id = |n: &str| run_id_in_name("x.toml", n).map(|r| r.as_str().to_string());
        assert_eq!(id("x.toml.a.state"), Some("a".into()));
        assert_eq!(id("x.toml.run-1.2_x.state"), Some("run-1.2_x".into()));
        assert_eq!(id("x.toml.a.b.state"), Some("a.b".into()));
        let long = format!("x.toml.{}.state", "a".repeat(64));
        assert!(id(&long).is_some());
        let too_long = format!("x.toml.{}.state", "a".repeat(65));
        assert_eq!(id(&too_long), None);
    }

    #[test]
    fn 対象外の名前() {
        for n in [
            "x.toml.state",
            "x.toml..state",
            "x.toml...state",
            "x.toml.a b.state",
            "x.toml.a/b.state",
            "x.toml.a.statex",
            "x.toml.a.state.bak",
            "x.toml.a",
            "x.toml.bak",
            "x.toml.é.state",
            ".x.toml.a.state",
            ".x.toml.state.tmp.1.2.3",
            "xx.toml.a.state",
            "x.tomla.state",
            "y.toml.a.state",
            "x.toml.\u{0}.state",
            "x.toml..a.state",
        ] {
            assert_eq!(run_id_in_name("x.toml", n), None, "{n:?}");
        }
    }
}
