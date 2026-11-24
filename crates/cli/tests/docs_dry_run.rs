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

//! README の `--dry-run` の説明の検査(U3。BR6.2、FR5.2)。README を読むだけで、書き込まない。
//! 計画の例と実際の出力を比べるテストは、単体テスト(`src/tests/docs.rs`)にある。

mod docs_support;

use docs_support::{assert_docs_check_words, assert_readme_has, readme};

#[test]
fn readme_に_dry_run_の必須の語句と版の表記がある() {
    let text = readme();
    assert_readme_has(
        &text,
        &[
            "--dry-run",
            "--assume",
            "## dry-run の使い方(v0.9.0 以降)",
            "使われなかった仮定",
            "計画が示さないこと",
            "再開位置",
        ],
    );
    // docs-check の必須語句は残っている
    assert_docs_check_words(&text);
}
