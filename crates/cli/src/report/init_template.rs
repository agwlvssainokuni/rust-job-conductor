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

//! 定義のひな形(InitTemplate。U4 の BR2.1〜BR2.4)。純粋な関数だけ。

use crate::types::Language;

/// 日本語のひな形の、パイプライン名の前まで(`name = "` を含む)。
const HEAD_JA: &str = r##"# conductor の定義ファイルのひな形(conductor init が作った)。
# 検査: conductor validate / 実行計画: conductor pipeline <名前> --dry-run / 図: conductor graph
format_version = 1

[pipeline]
name = ""##;

/// 日本語のひな形の、パイプライン名のあと(閉じの `"` を含む)。
const TAIL_JA: &str = r##""
description = "ひな形のパイプライン"

# 全ジョブに足す環境変数。値は、書いたとおりに渡す(展開しない)。
# .env ファイルから読むときは、[pipeline] に env_file = ".env" を足す。
[pipeline.env]
GREETING = "hello"

# hello: 最初のジョブ。
# Windows では、echo は実行ファイルではないので、command = ["cmd", "/c", "echo", "hello"] のように書く。
[[job]]
id = "hello"
command = ["echo", "hello"]

# flaky: 終了コード 75 は「一時的な失敗」。5 秒待って、最大 3 回、実行し直す。
# 待ち時間を伸ばすには、multiplier と max_interval を、一緒に書く(例: multiplier = 2、max_interval = "1m")。
[[job]]
id = "flaky"
command = ["echo", "flaky"]

[job.retry]
exit_codes = [75]
max = 3
interval = "5s"
"##;

/// 英語のひな形の、パイプライン名の前まで(`name = "` を含む)。
const HEAD_EN: &str = r##"# A starter conductor definition (created by conductor init).
# Check: conductor validate / plan: conductor pipeline <name> --dry-run / diagram: conductor graph
format_version = 1

[pipeline]
name = ""##;

/// 英語のひな形の、パイプライン名のあと(閉じの `"` を含む)。
const TAIL_EN: &str = r##""
description = "Starter pipeline"

# Environment variables added to every job. Values are passed as written (no expansion).
# To read a .env file, add env_file = ".env" to [pipeline].
[pipeline.env]
GREETING = "hello"

# hello: the first job.
# On Windows echo is not an executable; write command = ["cmd", "/c", "echo", "hello"] instead.
[[job]]
id = "hello"
command = ["echo", "hello"]

# flaky: exit code 75 means "a temporary failure". Wait 5 seconds and run again, up to 3 times.
# To lengthen the wait, write multiplier and max_interval together (for example multiplier = 2, max_interval = "1m").
[[job]]
id = "flaky"
command = ["echo", "flaky"]

[job.retry]
exit_codes = [75]
max = 3
interval = "5s"
"##;

/// 定義のひな形を作る(BR2.1〜BR2.3)。入力は、ロケールとパイプライン名だけ(利用者の値は入らない)。
///
/// `name` は、TOML の基本文字列として、`"` と `\` と制御文字(U+0000〜U+001F、U+007F は `\uXXXX`)を
/// エスケープして、`[pipeline].name` に入れる。名前は、コメントには書かない。UTF-8、LF、末尾に改行 1 つ。
pub fn render_template(lang: Language, name: &str) -> String {
    let (head, tail) = match lang {
        Language::Ja => (HEAD_JA, TAIL_JA),
        Language::En => (HEAD_EN, TAIL_EN),
    };
    let mut out = String::with_capacity(head.len() + tail.len() + name.len());
    out.push_str(head);
    escape_basic_string(name, &mut out);
    out.push_str(tail);
    out
}

/// TOML の基本文字列の中身にするエスケープ(BR2.2)。
fn escape_basic_string(name: &str, out: &mut String) {
    for c in name.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            c if (c as u32) < 0x20 || c == '\u{7F}' => {
                out.push_str(&format!("\\u{:04X}", c as u32));
            }
            c => out.push(c),
        }
    }
}
