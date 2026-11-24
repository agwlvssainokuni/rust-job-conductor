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

//! 表示の安全(Escape。NFR4.2)。入力に由来する文字列を、端末に出せる形にする。
//!
//! 本体は conductor-core の `escape_display` で、ここは名前・パスの薄い包みだけを持つ。
//! conductor-converter は core に依存しないため、同じ規則の写しを持ち、単体テストの
//! `src/tests/escape_table.rs` と converter の単体テストの、同じ期待値の表で揃える。

use conductor_core::escape_display;

/// ジョブID・キー名などを表示するときの最大の文字数(NFR4.2。conductor-core と同じ 64)。
const MAX_DISPLAY_CHARS: usize = conductor_core::MAX_KEY_DISPLAY_CHARS;

/// ジョブID・名前など(64 文字で切る)。
pub fn escape_name(input: &str) -> String {
    escape_display(input, Some(MAX_DISPLAY_CHARS))
}

/// 64 文字で切らずに、エスケープだけ(パス、図と計画のジョブID・コマンドの語など)。
pub fn escape_full(input: &str) -> String {
    escape_display(input, None)
}
