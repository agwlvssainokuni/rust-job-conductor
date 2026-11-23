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

//! `.env` の読み取り(RuntimeEnvLoader。WF3。BR1.7、BR3.1、BR3.3、BR3.4、BR6.4)。
//!
//! 解析は `conductor-core` の `parse_env_file` に任せる。ここは、パスの解決と、
//! ファイルの大きさの検査、上限 + 1 バイトまでの読み取り、UTF-8 の確認だけを行う。
//! 誤りは、すべてのファイル分を集めて返す。誤りに、値は含めない(BR8.1)。

use std::path::{Path, PathBuf};

use conductor_core::envfile::MAX_ENV_FILE_BYTES;
use conductor_core::{
    EnvFileContent, EnvFileError, EnvFileErrorKind, LoadedEnvFiles, parse_env_file,
};

use crate::bounded_read::{ReadFailure, read_regular_file};

/// 定義ファイルのパスと、読む対象の `env_file` の記述から、すべての `.env` を読む。
///
/// - 相対パスは、定義ファイルのあるディレクトリが基準。絶対パスはそのまま(`..` も許す。BR1.7)。
/// - 解決した結果が同じパスは、1 回だけ読み、同じ内容を共有する(BR6.4)。
/// - 無い、読めない、1 MiB を超える、UTF-8 でない、書式の誤りは、すべて集めて返す。
pub fn load_env_files(
    definition_path: &Path,
    refs: &[String],
) -> Result<LoadedEnvFiles, Vec<EnvFileError>> {
    let base = definition_path.parent().unwrap_or_else(|| Path::new(""));
    let mut loaded = LoadedEnvFiles::new();
    let mut errors: Vec<EnvFileError> = Vec::new();
    // 解決済みのパス → 読み取った内容の位置(読めなかったパスは `None`。誤りを重複して報告しない)
    let mut seen: Vec<(PathBuf, Option<usize>)> = Vec::new();

    for reference in refs {
        let resolved = resolve(base, reference);
        let known = seen.iter().find(|(p, _)| *p == resolved).map(|(_, i)| *i);
        let index = match known {
            Some(index) => index,
            None => {
                let index = match read_one(&resolved) {
                    Ok(content) => Some(loaded.push_file(content)),
                    Err(mut e) => {
                        errors.append(&mut e);
                        None
                    }
                };
                seen.push((resolved, index));
                index
            }
        };
        if let Some(index) = index {
            loaded.link(reference.clone(), index);
        }
    }

    if errors.is_empty() {
        Ok(loaded)
    } else {
        Err(errors)
    }
}

/// 相対なら (定義ファイルのディレクトリ)/(パス)、絶対ならそのまま(BR1.7)。`~` と変数は展開しない。
fn resolve(base: &Path, reference: &str) -> PathBuf {
    let path = Path::new(reference);
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        base.join(path)
    }
}

/// 1 つのファイルを読んで解析する(3b。定義ファイルの読み込みと同じ順)。
///
/// 1. パスのメタデータ: 無い → `FileMissing`、取れない → `Unreadable`。通常のファイルでない
///    (ディレクトリ、名前付きパイプなど)→ `Unreadable`(開かない。大きさは見ない)。
/// 2. 開く: 無い(1 のあとに消えた)→ `FileMissing`、開けない(読む権限がないなど)→ `Unreadable`。
/// 3. 開いたものから大きさ: 取れない・通常のファイルでない → `Unreadable`、上限超 → `TooLarge`。
/// 4. 上限 + 1 バイトまで読む: 失敗 → `Unreadable`、超えれば `TooLarge`。
/// 5. UTF-8 でない → `NotUtf8`。
///
/// v0.9.6 までは、パスのメタデータで大きさを先に見たため、読む権限のない 1 MiB 超の `.env` は `TooLarge`
/// だった。失敗はすべて種類で返し、パニックしない(誤りの表示に値を含めない)。
fn read_one(path: &Path) -> Result<EnvFileContent, Vec<EnvFileError>> {
    let shown = path.display().to_string();
    let fail = |kind: EnvFileErrorKind| vec![EnvFileError::new(shown.clone(), None, kind)];

    // 1〜4(`bounded_read::read_regular_file`。通常のファイルでなければ開かない。名前付きパイプで待たない)。
    let bytes = read_regular_file(path, MAX_ENV_FILE_BYTES as u64).map_err(|failure| {
        fail(match failure {
            ReadFailure::Missing => EnvFileErrorKind::FileMissing,
            ReadFailure::Unreadable => EnvFileErrorKind::Unreadable,
            ReadFailure::TooLarge => EnvFileErrorKind::TooLarge,
        })
    })?;
    let text = String::from_utf8(bytes).map_err(|_| fail(EnvFileErrorKind::NotUtf8))?;
    parse_env_file(&text, &shown)
}
