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

use std::path::{Path, PathBuf};

fn readme() -> String {
    let root: PathBuf = Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("..");
    std::fs::read_to_string(root.join("README.md"))
        .expect("README.md を読める")
        .replace("\r\n", "\n")
}

#[test]
fn readme_に_dry_run_の必須の語句と版の表記がある() {
    let text = readme();
    for word in [
        "--dry-run",
        "--assume",
        "## dry-run の使い方(v0.9.0 以降)",
        "使われなかった仮定",
        "計画が示さないこと",
        "再開位置",
    ] {
        assert!(text.contains(word), "README に {word} がない");
    }
    // docs-check の必須語句は残っている
    for word in [
        "SHA-256",
        "Gatekeeper",
        "終了コードの表",
        "ステートファイルの場所",
        "at-least-once",
        "シグナル",
        "convert",
    ] {
        assert!(text.contains(word), "docs-check の語句 {word} がない");
    }
}

#[test]
fn readme_の計画の例は_実際の出力と一致する() {
    let text = readme();
    let start = text.find("## dry-run の使い方").expect("節がある");
    let rest = &text[start..];
    let open = rest.find("```text\n").expect("例がある") + "```text\n".len();
    let body = &rest[open..];
    let example = &body[..body.find("```").expect("閉じている")];

    // 例の定義: a が終了コード 3 の警告で `@end` へ進む。`--assume a=3`。
    let definition = "format_version = 1\n\n[pipeline]\nname = \"demo\"\n\n\
        [[job]]\nid = \"a\"\ncommand = \"echo a\"\nwarning_exit_codes = [3]\n\n\
        [[job.transitions]]\non = \"warning\"\nnext = \"@end\"\n\n\
        [[job]]\nid = \"b\"\ncommand = \"echo b\"\n\n\
        [[job]]\nid = \"c\"\ncommand = \"echo c\"\n";
    let def = conductor_core::parse_definition(definition, "c.toml", None).expect("正しい定義");
    let commands: Vec<Vec<String>> = def
        .job
        .iter()
        .map(|j| vec!["echo".to_string(), j.id.clone()])
        .collect();
    let files = conductor_core::LoadedEnvFiles::new();
    let assumptions = [conductor_core::Assumption::new("a", 3)];
    let plan = conductor_core::plan_pipeline(&conductor_core::PlanInput {
        definition: &def,
        state: None,
        commands: &commands,
        env_files: &files,
        run_id: None,
        assumptions: &assumptions,
    })
    .expect("計画");
    let actual = conductor_cli::report::plan_text::render_plan(conductor_cli::Language::Ja, &plan);
    assert_eq!(example, actual);
}
