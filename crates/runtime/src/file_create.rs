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

//! 排他的なファイルの作成(RuntimeFileCreate。U4 の BR1.3、BR1.6)。
//!
//! 新規は `create_new`(ファイルがなければ作る、1 回の操作)で作り、`--force` は、同じディレクトリの
//! 一時ファイル(ステートの書き込みと同じ名前の作り方)に書いて、置き換える。親のディレクトリは作らない。
//! 内容は、そのまま書く(改行を変換しない)。

use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::Path;

use crate::temp_file::{ReplaceError, dir_or_current, replace_via_temp};

/// ファイルの作成の成功の結果(BR1.4)。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CreateOutcome {
    /// 新規に作った。
    Created,
    /// `--force` で、既にあったファイルを置き換えた。
    Replaced,
}

/// ファイルを作れなかった理由(BR1.2、BR3.2)。表示は、種類の固定の文とパスだけで、OS の
/// エラーの文は含めない。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CreateError {
    /// 既にある(`--force` なし)。終了コード 69。
    AlreadyExists,
    /// 親のディレクトリが無い(作らない。BR1.6)。
    ParentMissing,
    /// 作る先が、ディレクトリ。
    IsDirectory,
    /// 書く権限がない。
    PermissionDenied,
    /// そのほかの入出力の誤り。
    OtherIo,
}

impl CreateError {
    /// すべての種類(網羅のテストに使う)。
    pub const ALL: [CreateError; 5] = [
        Self::AlreadyExists,
        Self::ParentMissing,
        Self::IsDirectory,
        Self::PermissionDenied,
        Self::OtherIo,
    ];
}

/// ファイルを作る(WF3。BR1.2、BR1.3、BR1.6)。
///
/// - `force` なし: 排他的に作る。既に、ファイルまたはシンボリックリンク(宙に浮いたリンクを含む)が
///   あれば、何も変えず [`CreateError::AlreadyExists`]。書き込みに失敗したら、作ったファイルを消す。
/// - `force` あり: 同じディレクトリの一時ファイルに書き、置き換える(シンボリックリンクは、リンクそのものを
///   置き換える)。置き換え先がディレクトリなら [`CreateError::IsDirectory`]。失敗したら、一時ファイルを消す。
/// - 親のディレクトリが無ければ、作らず [`CreateError::ParentMissing`]。
///
/// エラーに、OS のエラーの文は含めない(種類だけ)。
pub fn create_file(path: &Path, bytes: &[u8], force: bool) -> Result<CreateOutcome, CreateError> {
    if force {
        replace_file(path, bytes)
    } else {
        create_new_file(path, bytes)
    }
}

/// 新規の作成(`create_new`)。
fn create_new_file(path: &Path, bytes: &[u8]) -> Result<CreateOutcome, CreateError> {
    let mut file = match OpenOptions::new().write(true).create_new(true).open(path) {
        Ok(file) => file,
        Err(error) => return Err(classify_open_error(path, &error)),
    };
    let written = file.write_all(bytes).and_then(|()| file.flush());
    // 消す前に、閉じる(Windows では、開いたままのファイルは、消せない)。
    drop(file);
    if written.is_err() {
        let _ = fs::remove_file(path);
        return Err(CreateError::OtherIo);
    }
    Ok(CreateOutcome::Created)
}

/// `create_new` の失敗を、パスの今の姿で分類する(OS によって、ディレクトリへの `create_new` の
/// 誤りの種類が違うため。BR1.3)。
fn classify_open_error(path: &Path, error: &io::Error) -> CreateError {
    match fs::symlink_metadata(path) {
        Ok(meta) if meta.is_dir() => CreateError::IsDirectory,
        // ファイル、またはシンボリックリンク(宙に浮いたものを含む)。
        Ok(_) => CreateError::AlreadyExists,
        Err(_) => classify_kind(error.kind()),
    }
}

fn classify_kind(kind: io::ErrorKind) -> CreateError {
    match kind {
        io::ErrorKind::NotFound => CreateError::ParentMissing,
        io::ErrorKind::PermissionDenied => CreateError::PermissionDenied,
        _ => CreateError::OtherIo,
    }
}

/// `--force`: 一時ファイルに書いて、置き換える。
fn replace_file(path: &Path, bytes: &[u8]) -> Result<CreateOutcome, CreateError> {
    let dir = dir_or_current(path.parent().unwrap_or(Path::new("")));
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .ok_or(CreateError::OtherIo)?;
    match fs::metadata(dir) {
        Ok(meta) if meta.is_dir() => {}
        Ok(_) => return Err(CreateError::OtherIo),
        Err(error) => return Err(classify_kind(error.kind())),
    }
    let existed = match fs::symlink_metadata(path) {
        Ok(meta) if meta.is_dir() => return Err(CreateError::IsDirectory),
        Ok(_) => true,
        Err(_) => false,
    };

    match replace_via_temp(dir, &name, path, |file| {
        file.write_all(bytes).and_then(|()| file.flush())
    }) {
        Ok(()) => Ok(if existed {
            CreateOutcome::Replaced
        } else {
            CreateOutcome::Created
        }),
        Err(ReplaceError::CreateTemp(kind)) => Err(classify_kind(kind)),
        Err(ReplaceError::WriteOrRename(kind)) => Err(match kind {
            io::ErrorKind::PermissionDenied => CreateError::PermissionDenied,
            _ => CreateError::OtherIo,
        }),
    }
}
