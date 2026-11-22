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

use std::path::Path;

/// ディレクトリのパスが空なら、実行ディレクトリ(`.`)にする。ステートファイルと `init` の出力先の
/// 親のディレクトリを開くとき、置き換えの同期と古い一時ファイルの後始末に使う 1 か所(BR2.8)。
pub(crate) fn dir_or_current(dir: &Path) -> &Path {
    if dir.as_os_str().is_empty() {
        Path::new(".")
    } else {
        dir
    }
}
