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

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use conductor_core::envfile::MAX_ENV_FILE_BYTES;
use conductor_core::{
    EnvFileContent, EnvFileError, EnvFileErrorKind, EnvFileRef, LoadedEnvFiles, parse_env_file,
};

use crate::bounded_read::{ReadFailure, read_regular_file};

/// 定義ファイルのパスと、読む対象の `env_file` の記述から、すべての `.env` を読む。
///
/// - 相対パスは、定義ファイルのあるディレクトリが基準。絶対パスはそのまま(`..` も許す。BR1.7)。
/// - 解決した結果が同じパスは、1 回だけ読み、同じ内容を共有する(BR6.4)。そのパスが必ず要るかは、
///   そこに解決されるすべての記述の `required` の論理和で決める(v0.10.0 の BR2.2)。
/// - 必ず要らないパスが無ければ、誤りにも警告にもせず、そのパスの記述を引き当てに載せない
///   (v0.10.0 の BR2.3)。無いことの判断は `read_regular_file` の `Missing` だけで行う。
/// - 必ず要るパスが無い、読めない、1 MiB を超える、UTF-8 でない、書式の誤りは、`required` によらず
///   すべて集めて返す(v0.10.0 の BR2.4、BR2.5)。誤りの並びは、記述の出現順。
pub fn load_env_files(
    definition_path: &Path,
    refs: &[EnvFileRef],
) -> Result<LoadedEnvFiles, Vec<EnvFileError>> {
    let base = definition_path.parent().unwrap_or_else(|| Path::new(""));

    // 前処理(1 回): 解決済みのパスを最初に現れた順に並べ、required を論理和でまとめる。
    // パス → 位置の表で引くので、記述の数に比例する時間で済む。
    let mut targets: Vec<(PathBuf, bool)> = Vec::new();
    let mut position: HashMap<PathBuf, usize> = HashMap::new();
    // 記述ごとの、`targets` の位置(記述の順)
    let mut target_of: Vec<usize> = Vec::with_capacity(refs.len());
    for r in refs {
        let resolved = resolve(base, &r.path);
        let i = match position.get(&resolved) {
            Some(&i) => {
                targets[i].1 |= r.required;
                i
            }
            None => {
                let i = targets.len();
                position.insert(resolved.clone(), i);
                targets.push((resolved, r.required));
                i
            }
        };
        target_of.push(i);
    }

    // 解決済みのパスごとに 1 回だけ読む(最初に現れた順)。読めなかった・飛ばしたパスは `None`。
    let mut loaded = LoadedEnvFiles::new();
    let mut errors: Vec<EnvFileError> = Vec::new();
    let mut index_of: Vec<Option<usize>> = Vec::with_capacity(targets.len());
    for (path, required) in &targets {
        let index = match read_one(path, *required) {
            Ok(Some(content)) => Some(loaded.push_file(content)),
            Ok(None) => None,
            Err(mut e) => {
                errors.append(&mut e);
                None
            }
        };
        index_of.push(index);
    }

    for (r, &i) in refs.iter().zip(&target_of) {
        if let Some(index) = index_of[i] {
            loaded.link(r.path.clone(), index);
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
///
/// `required` が `false` で、1 か 2 で無い(`Missing`)ときだけ、誤りにせず `Ok(None)` を返す
/// (v0.10.0 の BR2.3)。ほかの失敗は `required` によらず誤り(BR2.4)。
fn read_one(path: &Path, required: bool) -> Result<Option<EnvFileContent>, Vec<EnvFileError>> {
    let shown = path.display().to_string();
    let fail = |kind: EnvFileErrorKind| vec![EnvFileError::new(shown.clone(), None, kind)];

    // 1〜4(`bounded_read::read_regular_file`。通常のファイルでなければ開かない。名前付きパイプで待たない)。
    let bytes = match read_regular_file(path, MAX_ENV_FILE_BYTES as u64) {
        Ok(bytes) => bytes,
        Err(ReadFailure::Missing) if !required => return Ok(None),
        Err(failure) => {
            return Err(fail(match failure {
                ReadFailure::Missing => EnvFileErrorKind::FileMissing,
                ReadFailure::Unreadable => EnvFileErrorKind::Unreadable,
                ReadFailure::TooLarge => EnvFileErrorKind::TooLarge,
            }));
        }
    };
    let text = String::from_utf8(bytes).map_err(|_| fail(EnvFileErrorKind::NotUtf8))?;
    parse_env_file(&text, &shown).map(Some)
}
