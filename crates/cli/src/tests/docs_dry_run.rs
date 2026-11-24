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
//! README の例と実際の出力を比べる単体テスト。必須の語句の検査は統合テストの同じ名前のファイル。

use super::docs_support::readme;

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
    let actual = crate::report::plan_text::render_plan(crate::types::Language::Ja, &plan);
    assert_eq!(example, actual);
}
