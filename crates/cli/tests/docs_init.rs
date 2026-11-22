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

//! README の `init` の説明の検査(U4。BR4.2、FR5.2)。README を読むだけで、書き込まない。

mod docs_support;

use conductor_cli::Language;
use conductor_cli::report::init_template::render_template;
use docs_support::{assert_docs_check_words, assert_readme_has, readme};

#[test]
fn readme_に_init_の必須の語句と版の表記と終了コード_69_がある() {
    let text = readme();
    assert_readme_has(
        &text,
        &[
            "conductor init",
            "--force",
            "--config -",
            "## init の使い方(v0.9.0 以降)",
            "| 69 |",
            "既にあります",
        ],
    );
    // docs-check の必須語句は残っている
    assert_docs_check_words(&text);
}

#[test]
fn readme_のひな形の例は_実際の日本語のひな形と一致する() {
    let text = readme();
    let start = text.find("## init の使い方").expect("節がある");
    let rest = &text[start..];
    let open = rest.find("```toml\n").expect("例がある") + "```toml\n".len();
    let body = &rest[open..];
    let example = &body[..body.find("```").expect("閉じている")];
    assert_eq!(example, render_template(Language::Ja, "my-pipeline"));
}
