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

//! 一時ファイルと親のディレクトリの扱い(ステートの書き込みと、`init` の出力先の置き換えが共有する。
//! BR2.8)。`state` と `file_create` と `platform` は、ここを使い、互いの内部を使わない。

use std::fs::{self, File, OpenOptions};
use std::io;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

/// 一時ファイルの名前が重なったとき、連番を進めて作り直す回数の上限。
const TEMP_ATTEMPTS: u32 = 16;

/// 一時ファイルの名前の連番(プロセスで 1 つ。ステートと `init` の出力先が共有する)。
static SEQUENCE: AtomicU32 = AtomicU32::new(0);

/// ディレクトリのパスが空なら、実行ディレクトリ(`.`)にする。ステートファイルと `init` の出力先の
/// 親のディレクトリを開くとき、置き換えの同期と古い一時ファイルの後始末に使う 1 か所(BR2.8)。
pub(crate) fn dir_or_current(dir: &Path) -> &Path {
    if dir.as_os_str().is_empty() {
        Path::new(".")
    } else {
        dir
    }
}

/// 対象のファイル `target_name` の一時ファイルの名前の接頭辞 `.<名前>.tmp.`(名前の作り方と、
/// 古い一時ファイルの後始末が使う)。
pub(crate) fn temp_prefix(target_name: &str) -> String {
    format!(".{target_name}.tmp.")
}

/// 一時ファイルの名前 `.<名前>.tmp.<プロセス番号>.<時刻>.<連番>`。
pub(crate) fn temp_name(target_name: &str, pid: u32, nanos: u128, seq: u32) -> String {
    format!("{}{pid}.{nanos}.{seq}", temp_prefix(target_name))
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

/// ディレクトリ `dir` に、対象のファイル `target_name` の一時ファイルを、新規作成で作る。
///
/// 名前は `.<名前>.tmp.<プロセス番号>.<時刻>.<連番>`。名前が重なったら連番を進めて、
/// 最大 16 回まで作り直す([`create_temp`])。`dir` は、呼び出し側が渡したまま使う。
fn create_temp_beside(dir: &Path, target_name: &str) -> io::Result<(PathBuf, File)> {
    let pid = std::process::id();
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    create_temp(dir, |attempt| {
        let seq = SEQUENCE
            .fetch_add(1, Ordering::Relaxed)
            .wrapping_add(attempt);
        temp_name(target_name, pid, nanos, seq)
    })
}

/// [`replace_via_temp`] の失敗。誤りの種類の写し方は、呼び出し側が決める。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ReplaceError {
    /// 一時ファイルを作れない。
    CreateTemp(io::ErrorKind),
    /// 書き込み・同期・置き換えのどれかの失敗(作った一時ファイルは消した)。
    WriteOrRename(io::ErrorKind),
}

/// 一時ファイルに書いて、対象のファイル `target` を置き換える(ステートの書き込みと、`init --force`
/// の出力先の置き換えが共有する 1 か所。BR2.8)。
///
/// 順は、`dir` に対象の名前 `target_name` の一時ファイルを新規作成([`create_temp_beside`])→
/// `write` で書く → 閉じる(Windows では、開いたままのファイルは、移せず、消せない)→ `rename` で
/// 置き換える。書き込みか置き換えに失敗したら、作った一時ファイルを消す。
pub(crate) fn replace_via_temp(
    dir: &Path,
    target_name: &str,
    target: &Path,
    write: impl FnOnce(&mut File) -> io::Result<()>,
) -> Result<(), ReplaceError> {
    let (temp_path, mut file) =
        create_temp_beside(dir, target_name).map_err(|e| ReplaceError::CreateTemp(e.kind()))?;
    let written = write(&mut file);
    drop(file);
    match written.and_then(|()| fs::rename(&temp_path, target)) {
        Ok(()) => Ok(()),
        Err(error) => {
            let _ = fs::remove_file(&temp_path);
            Err(ReplaceError::WriteOrRename(error.kind()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    #[test]
    fn 一時ファイルの名前の形() {
        assert_eq!(
            temp_name("c.toml.state", 7, 123, 4),
            ".c.toml.state.tmp.7.123.4"
        );
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
    fn 対象の隣に接頭辞つきの一時ファイルを作り_続けて呼ぶと別の名前() {
        let dir = tempfile::tempdir().expect("一時ディレクトリ");
        let (first, first_file) = create_temp_beside(dir.path(), "c.toml.state").expect("作れる");
        let (second, second_file) = create_temp_beside(dir.path(), "c.toml.state").expect("作れる");
        drop((first_file, second_file));

        for path in [&first, &second] {
            assert_eq!(path.parent(), Some(dir.path()), "dir に作る");
            let name = path
                .file_name()
                .and_then(|n| n.to_str())
                .expect("名前がある");
            assert!(name.starts_with(".c.toml.state.tmp."), "接頭辞: {name}");
            assert!(path.is_file(), "新しいファイルがある");
        }
        assert_ne!(first, second, "連番が進み、別の名前になる");

        let missing = dir.path().join("no-such-dir");
        assert!(
            create_temp_beside(&missing, "c.toml.state").is_err(),
            "存在しないディレクトリでは失敗する"
        );
    }
}
