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

//! 表示の安全のエスケープ(Escape。NFR4.2)と、1 行を足す整形の補助。
//!
//! 親のモジュールが `pub(crate) use` で今の名前のまま出し、下位のモジュール(計画・watch・図の整形)は
//! `super::` で引く。

use conductor_core::escape_display;

/// ジョブID・キー名などを表示するときの最大の文字数(NFR4.2。conductor-core と同じ 64)。
const MAX_DISPLAY_CHARS: usize = conductor_core::MAX_KEY_DISPLAY_CHARS;

/// ジョブID・名前など(64 文字で切る)。
pub(crate) fn escape_name(input: &str) -> String {
    escape_display(input, Some(MAX_DISPLAY_CHARS))
}

/// 64 文字で切らずに、エスケープだけ(パス、図と計画のジョブID・コマンドの語など)。
pub(crate) fn escape_full(input: &str) -> String {
    escape_display(input, None)
}

/// 1 行を足す(末尾の空白を取り除き、改行を足す)。graph・計画・watch の整形が使う。
pub(crate) fn push_line(out: &mut String, line: &str) {
    out.push_str(line.trim_end());
    out.push('\n');
}
