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

//! README の例を比べる単体テストの補助(README の読み取り)。README を読むだけで、書き込まない。
//!
//! 統合テストの `tests/docs_support/mod.rs` の `readme` の写し(本体は変えない)。

use std::path::{Path, PathBuf};

/// リポジトリのルートの README(改行は `\n` にそろえる)。
pub fn readme() -> String {
    let root: PathBuf = Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("..");
    std::fs::read_to_string(root.join("README.md"))
        .expect("README.md を読める")
        .replace("\r\n", "\n")
}
