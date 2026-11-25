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

//! 定義の新しいキー `env`、`env_file` の読み取りと検査(U1。BR1.1〜BR1.8、FR1.1〜FR1.4、FR1.9、FR1.11、FR1.13)。
//! `unset_env` の読み取りと `env` との重なり(v0.10.0 の U3。BR1.1〜BR1.3、BR2.1)。

mod common;

use common::{SECRET, env_ref, kinds, paths};
use conductor_core::{DefinitionErrorKind as K, PipelineDefinition, parse_definition};

fn parse(text: &str) -> Result<PipelineDefinition, Vec<(K, String)>> {
    parse_definition(text, "t.toml", None)
        .map_err(|errs| errs.into_iter().map(|e| (e.kind, e.to_string())).collect())
}

fn ok(text: &str) -> PipelineDefinition {
    parse(text).unwrap_or_else(|e| panic!("受理されること: {e:?}"))
}

/// パイプラインの追加部分 `p` と、ジョブ `a` の追加部分 `j` を持つ定義。
fn def(p: &str, j: &str) -> String {
    format!(
        "format_version = 1\n[pipeline]\nname = \"p\"\n{p}\n[[job]]\nid = \"a\"\ncommand = \"c\"\n{j}\n"
    )
}

#[test]
fn 正しい_env_と_env_file_を書いた順に読める() {
    let text = def(
        "env_file = \".env\"\n[pipeline.env]\nZ = \"1\"\nA = \"two words\"\nM = \"\"\n",
        "env_file = [\"a.env\", \"b.env\"]\n[job.env]\nJ = \"x\"\n",
    );
    // [pipeline.env] の後ろに [[job]] が来るため、job の表は別に確かめる
    let d = ok(&text);
    let names: Vec<_> = d.env.iter().map(|e| e.name.as_str()).collect();
    assert_eq!(
        names,
        ["Z", "A", "M"],
        "書いた順を保つ(キーの並べ替えをしない)"
    );
    assert_eq!(d.env[1].value, "two words");
    assert_eq!(d.env[2].value, "");
    assert_eq!(paths(&d.env_file), [".env"]);
    assert_eq!(paths(&d.job[0].env_file), ["a.env", "b.env"]);
    assert_eq!(d.job[0].env[0].name, "J");
}

#[test]
fn env_file_は文字列も配列も空の配列も受け付ける() {
    assert_eq!(
        ok(&def("env_file = \"x.env\"\n", "")).env_file,
        [env_ref("x.env", true)]
    );
    assert_eq!(
        ok(&def("env_file = [\"x.env\", \"y.env\"]\n", "")).env_file,
        [env_ref("x.env", true), env_ref("y.env", true)]
    );
    assert!(ok(&def("env_file = []\n", "")).env_file.is_empty());
}

#[test]
fn 新しいキーのない既存の定義は変わらず読めて展開もしない() {
    let d = ok(&def("", ""));
    assert!(d.env.is_empty() && d.env_file.is_empty());
    assert!(d.job[0].env.is_empty() && d.job[0].env_file.is_empty());
    assert_eq!(d.format_version, 1);
    // 値の `${X}` は、そのまま
    let d = ok(&def("[pipeline.env]\nV = \"${HOME}/$X\"\n", ""));
    assert_eq!(d.env[0].value, "${HOME}/$X");
}

#[test]
fn 値が文字列でない_env_file_の要素が文字列でないか空は誤り() {
    assert_eq!(
        kinds(&def("[pipeline.env]\nA = 1\n", "")),
        [K::TypeMismatch]
    );
    assert_eq!(kinds(&def("env = \"x\"\n", "")), [K::TypeMismatch]);
    assert_eq!(kinds(&def("env_file = 3\n", "")), [K::TypeMismatch]);
    assert_eq!(
        kinds(&def("env_file = [\"a\", 2]\n", "")),
        [K::TypeMismatch]
    );
    assert_eq!(kinds(&def("env_file = \"\"\n", "")), [K::EmptyField]);
    assert_eq!(
        kinds(&def("env_file = [\"a\", \"\"]\n", "")),
        [K::EmptyField]
    );
}

#[test]
fn env_file_の表は_path_と_required_を読み_required_を省くと必ず要る() {
    let d = ok(&def(
        "env_file = [\".env\", { path = \"local.env\", required = false }, { path = \"m.env\" }, { path = \"r.env\", required = true }]\n",
        "",
    ));
    assert_eq!(
        d.env_file,
        [
            env_ref(".env", true),
            env_ref("local.env", false),
            env_ref("m.env", true),
            env_ref("r.env", true),
        ]
    );
}

#[test]
fn env_file_は表_1_つでも書け_pipeline_と_job_の両方で読める() {
    let d = ok(&def(
        "env_file = { path = \"p.env\", required = false }\n",
        "env_file = [{ path = \"j.env\" }, \"k.env\"]\n",
    ));
    assert_eq!(d.env_file, [env_ref("p.env", false)]);
    assert_eq!(
        d.job[0].env_file,
        [env_ref("j.env", true), env_ref("k.env", true)]
    );
}

#[test]
fn env_file_の表の誤りは固定の理由で返す() {
    let cases: [(&str, K, &str); 6] = [
        (
            "env_file = { required = false }\n",
            K::TypeMismatch,
            "必須の値 `path` がありません",
        ),
        (
            "env_file = { path = 1 }\n",
            K::TypeMismatch,
            "`path` は文字列で書きます",
        ),
        (
            "env_file = { path = \"\" }\n",
            K::EmptyField,
            "`env_file` に空の文字列は書けません",
        ),
        (
            "env_file = { path = \"a\", required = \"no\" }\n",
            K::TypeMismatch,
            "`required` は真偽値(`true` または `false`)で書きます",
        ),
        (
            "env_file = { path = \"a\", optional = true }\n",
            K::UnknownKey,
            "キー `optional` は定義にありません",
        ),
        (
            "env_file = [\"a\", { path = \"b\" }, 2]\n",
            K::TypeMismatch,
            "`env_file` の配列の要素は、文字列かテーブルで書きます",
        ),
    ];
    for (p, kind, reason) in cases {
        let errs = parse(&def(p, "")).unwrap_err();
        assert_eq!(errs.len(), 1, "{p}: {errs:?}");
        assert_eq!(errs[0].0, kind, "{p}");
        assert!(errs[0].1.contains(reason), "{p}: {}", errs[0].1);
        // ジョブに書いても同じ
        let errs = parse(&def("", p)).unwrap_err();
        assert_eq!(errs.len(), 1, "job {p}: {errs:?}");
        assert_eq!(errs[0].0, kind, "job {p}");
    }
    let errs = parse(&def("env_file = 3\n", "")).unwrap_err();
    assert!(
        errs[0]
            .1
            .contains("`env_file` は、文字列かテーブル、またはその配列で書きます"),
        "{}",
        errs[0].1
    );
}

#[test]
fn env_file_の配列の複数の誤りはすべて集める() {
    assert_eq!(
        kinds(&def(
            "env_file = [{ path = \"\" }, true, { path = \"a\", required = 1, x = 2 }]\n",
            "",
        )),
        // 位置の順(`required = 1` の値は、知らないキー `x` より前)
        [
            K::EmptyField,
            K::TypeMismatch,
            K::TypeMismatch,
            K::UnknownKey
        ]
    );
}

#[test]
fn 名前の規則に合わない名前は誤り() {
    for bad in ["\"1A\"", "\"A-B\"", "\"A B\"", "\"\"", "\"日本\""] {
        let text = def(&format!("env = {{ {bad} = \"v\" }}\n"), "");
        assert_eq!(kinds(&text), [K::InvalidEnvName], "{bad}");
    }
    ok(&def("env = { _A1 = \"v\", a_b = \"w\" }\n", ""));
}

#[test]
fn 予約名は大文字小文字を区別せずに拒否する() {
    for name in [
        "CONDUCTOR_X",
        "conductor_x",
        "Conductor_Run_Id",
        "CONDUCTOR_",
    ] {
        let text = def(&format!("env = {{ {name} = \"v\" }}\n"), "");
        assert_eq!(kinds(&text), [K::ReservedEnvName], "{name}");
        let text = def("", &format!("env = {{ {name} = \"v\" }}\n"));
        assert_eq!(kinds(&text), [K::ReservedEnvName], "job {name}");
    }
    // 接頭辞でない名前は許す
    ok(&def(
        "env = { CONDUCTOR = \"v\", XCONDUCTOR_A = \"w\" }\n",
        "",
    ));
}

#[test]
fn 同じ表の中の大文字小文字だけが違う名前は誤りで別の表なら誤りでない() {
    let text = def("env = { foo = \"1\", FOO = \"2\" }\n", "");
    assert_eq!(kinds(&text), [K::EnvNameCaseConflict]);
    // 別の表(pipeline と job)の同じ綴り、大文字小文字違いは誤りでない
    let d = ok(&def("env = { foo = \"1\" }\n", "env = { FOO = \"2\" }\n"));
    assert_eq!(d.env[0].name, "foo");
    assert_eq!(d.job[0].env[0].name, "FOO");
}

#[test]
fn nul_を含む値は誤り() {
    let text = def("env = { A = \"x\\u0000y\" }\n", "");
    assert_eq!(kinds(&text), [K::InvalidEnvValue]);
}

#[test]
fn unset_env_を_pipeline_と_job_で書いた順に読み同じ名前は最初の_1_つにまとめる() {
    // U3 の BR1.1、BR1.3
    let d = ok(&def(
        "unset_env = [\"HTTP_PROXY\", \"A\", \"HTTP_PROXY\"]\n",
        "unset_env = [\"SECRET_TOKEN\", \"a\", \"SECRET_TOKEN\"]\n",
    ));
    assert_eq!(d.unset_env, ["HTTP_PROXY", "A"]);
    // 大文字小文字だけが違う名前は、まとめない(完全一致でまとめる)
    assert_eq!(d.job[0].unset_env, ["SECRET_TOKEN", "a"]);
    // 空の配列と、書かない定義は空
    assert!(ok(&def("unset_env = []\n", "")).unset_env.is_empty());
    let d = ok(&def("", ""));
    assert!(d.unset_env.is_empty() && d.job[0].unset_env.is_empty());
}

#[test]
fn unset_env_の型の誤りは固定の理由で返す() {
    // U3 の BR1.1
    let cases: [(&str, &str); 3] = [
        (
            "unset_env = \"HTTP_PROXY\"\n",
            "`unset_env` は、文字列の配列で書きます",
        ),
        (
            "unset_env = { A = 1 }\n",
            "`unset_env` は、文字列の配列で書きます",
        ),
        (
            "unset_env = [\"A\", 1]\n",
            "`unset_env` の要素は、文字列で書きます",
        ),
    ];
    for (text, reason) in cases {
        for t in [def(text, ""), def("", text)] {
            let errs = parse(&t).unwrap_err();
            assert_eq!(errs.len(), 1, "{t}: {errs:?}");
            assert_eq!(errs[0].0, K::TypeMismatch, "{t}");
            assert!(errs[0].1.contains(reason), "{t}: {}", errs[0].1);
        }
    }
}

#[test]
fn unset_env_の空の名前と規則違反と予約名は誤りで理由は_unset_env_の文() {
    // U3 の BR1.2
    let cases: [(&str, K, &str); 5] = [
        (
            "\"\"",
            K::EmptyField,
            "`unset_env` に空の文字列は書けません",
        ),
        ("\"1A\"", K::InvalidEnvName, "`unset_env` の名前 `1A` は、"),
        (
            "\"A-B\"",
            K::InvalidEnvName,
            "`unset_env` の名前 `A-B` は、",
        ),
        (
            "\"CONDUCTOR_JOB_ID\"",
            K::ReservedEnvName,
            "`unset_env` の名前 `CONDUCTOR_JOB_ID` は、`CONDUCTOR_` で始まるため書けません",
        ),
        (
            "\"conductor_x\"",
            K::ReservedEnvName,
            "`unset_env` の名前 `conductor_x` は、",
        ),
    ];
    for (name, kind, reason) in cases {
        for t in [
            def(&format!("unset_env = [{name}]\n"), ""),
            def("", &format!("unset_env = [{name}]\n")),
        ] {
            let errs = parse(&t).unwrap_err();
            assert_eq!(errs.len(), 1, "{t}: {errs:?}");
            assert_eq!(errs[0].0, kind, "{t}");
            assert!(errs[0].1.contains(reason), "{t}: {}", errs[0].1);
        }
    }
    // 複数の誤りは、すべて集める(位置の順)
    assert_eq!(
        kinds(&def("unset_env = [\"\", \"1A\", 2, \"CONDUCTOR_A\"]\n", "")),
        [
            K::EmptyField,
            K::InvalidEnvName,
            K::TypeMismatch,
            K::ReservedEnvName
        ]
    );
}

#[test]
fn unset_env_と同じジョブに当てはまる_env_の重なりは大文字小文字を区別せずに誤り() {
    // U3 の BR2.1(比べ方は security-design の 1)
    let conflict = |p: &str, j: &str| {
        let errs = parse(&def(p, j)).unwrap_err();
        assert!(
            errs.iter().all(|(k, _)| *k == K::UnsetEnvConflict),
            "{p} / {j}: {errs:?}"
        );
        assert!(
            errs[0]
                .1
                .contains("`unset_env` の名前が、`env` の名前と重なっています"),
            "{}",
            errs[0].1
        );
        errs.len()
    };
    // ジョブの unset_env とそのジョブの env、パイプラインの env
    assert_eq!(
        conflict("", "unset_env = [\"FOO\"]\n[job.env]\nFOO = \"x\"\n"),
        1
    );
    assert_eq!(
        conflict("[pipeline.env]\nFOO = \"x\"\n", "unset_env = [\"FOO\"]\n"),
        1
    );
    // パイプラインの unset_env とパイプラインの env、ジョブの env
    assert_eq!(
        conflict("unset_env = [\"FOO\"]\n[pipeline.env]\nFOO = \"x\"\n", ""),
        1
    );
    assert_eq!(
        conflict("unset_env = [\"FOO\"]\n", "[job.env]\nFOO = \"x\"\n"),
        1
    );
    // 大文字小文字だけが違う名前も重なり(どの OS でも)
    assert_eq!(
        conflict("", "unset_env = [\"foo\"]\n[job.env]\nFOO = \"x\"\n"),
        1
    );
    assert_eq!(
        conflict("unset_env = [\"Path\"]\n", "[job.env]\nPATH = \"x\"\n"),
        1
    );
    // 名前ごとに 1 つ
    assert_eq!(
        conflict(
            "unset_env = [\"A\", \"B\"]\n[pipeline.env]\nA = \"1\"\nB = \"2\"\n",
            ""
        ),
        2
    );
}

#[test]
fn ほかのジョブの_env_とは重ならず_重なりの誤りは名前の位置の行() {
    // ジョブの unset_env は、ほかのジョブの env と比べない(U3 の BR2.1)
    let text = "format_version = 1\n[pipeline]\nname = \"p\"\n\
                [[job]]\nid = \"a\"\ncommand = \"c\"\nunset_env = [\"FOO\"]\n\
                [[job]]\nid = \"b\"\ncommand = \"c\"\n[job.env]\nFOO = \"x\"\n";
    let d = ok(text);
    assert_eq!(d.job[0].unset_env, ["FOO"]);
    assert_eq!(d.job[1].env[0].name, "FOO");
    // 誤りの行は unset_env の名前の行
    let text = "format_version = 1\n[pipeline]\nname = \"p\"\n\
                [[job]]\nid = \"a\"\ncommand = \"c\"\nunset_env = [\n  \"A\",\n  \"FOO\",\n]\n\
                [job.env]\nFOO = \"x\"\n";
    let errs = parse_definition(text, "t.toml", None).unwrap_err();
    assert_eq!(errs.len(), 1, "{errs:?}");
    assert_eq!(errs[0].kind, K::UnsetEnvConflict);
    assert_eq!(errs[0].line, Some(9));
}

#[test]
fn 誤りの表示に値が含まれない() {
    let cases = [
        def(&format!("env = {{ A = \"{SECRET}\\u0000\" }}\n"), ""),
        def(&format!("env = {{ CONDUCTOR_A = \"{SECRET}\" }}\n"), ""),
        def(&format!("env = {{ a = \"1\", A = \"{SECRET}\" }}\n"), ""),
        def(&format!("env = {{ \"1x\" = \"{SECRET}\" }}\n"), ""),
        def(&format!("env = \"{SECRET}\"\n"), ""),
        def("", &format!("env = {{ A = [\"{SECRET}\"] }}\n")),
        // U3: unset_env の型の誤りと、env との重なり(値に目印)
        def(&format!("unset_env = \"{SECRET}\"\n"), ""),
        def(
            "",
            &format!("unset_env = [\"A\"]\n[job.env]\nA = \"{SECRET}\"\n"),
        ),
    ];
    for text in cases {
        for (_, shown) in parse(&text).unwrap_err() {
            assert!(!shown.contains(SECRET), "{shown}");
        }
    }
}
