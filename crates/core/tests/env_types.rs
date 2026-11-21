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

//! 環境変数の型と誤りの種類(U1。BR8.1)。値が表示に出ないこと、種類の網羅。

use std::collections::HashSet;

use conductor_core::{
    DefinitionErrorKind, EffectiveEntry, EnvEntry, EnvFileError, EnvFileErrorKind, EnvOrigin,
    EnvVar, summarize_errors,
};

const SECRET: &str = "s3cr3t-VALUE-do-not-print";

#[test]
fn 値を持つ型の_debug_は値を伏せる() {
    let var = EnvVar {
        name: "TOKEN".into(),
        value: SECRET.into(),
    };
    let entry = EnvEntry {
        name: "TOKEN".into(),
        value: SECRET.into(),
        line: 3,
    };
    let eff = EffectiveEntry {
        name: "TOKEN".into(),
        value: SECRET.into(),
        origin: EnvOrigin::JobEnv,
    };
    for shown in [
        format!("{var:?}"),
        format!("{entry:?}"),
        format!("{eff:?}"),
        format!("{:?}", vec![var.clone()]),
    ] {
        assert!(!shown.contains(SECRET), "{shown}");
        assert!(shown.contains("TOKEN"), "名前は出る: {shown}");
    }
}

#[test]
fn 定義の誤りの種類は_33_種で識別子が重複せず表示が空でない() {
    assert_eq!(DefinitionErrorKind::ALL.len(), 33);
    let codes: HashSet<_> = DefinitionErrorKind::ALL.iter().map(|k| k.code()).collect();
    assert_eq!(codes.len(), 33);
    for k in DefinitionErrorKind::ALL {
        assert!(!k.to_string().is_empty());
    }
    for code in ["reservedEnvName", "envNameCaseConflict", "invalidEnvValue"] {
        assert!(codes.contains(code), "{code}");
    }
}

#[test]
fn env_file_の誤りの種類は_13_種で識別子と理由が重複しない() {
    assert_eq!(EnvFileErrorKind::ALL.len(), 13);
    let codes: HashSet<_> = EnvFileErrorKind::ALL.iter().map(|k| k.code()).collect();
    assert_eq!(codes.len(), 13);
    let reasons: HashSet<_> = EnvFileErrorKind::ALL.iter().map(|k| k.reason()).collect();
    assert_eq!(reasons.len(), 13);
    for k in EnvFileErrorKind::ALL {
        assert!(!k.to_string().is_empty());
        assert!(!k.reason().is_empty());
    }
}

#[test]
fn env_file_の誤りの表示は行の有無で形が変わり値を含まない() {
    let with_line = EnvFileError::new("a.env", Some(7), EnvFileErrorKind::MissingEquals);
    let shown = with_line.to_string();
    assert!(shown.starts_with("a.env:7: "), "{shown}");
    assert!(shown.contains(EnvFileErrorKind::MissingEquals.reason()));

    let no_line = EnvFileError::new("b.env", None, EnvFileErrorKind::FileMissing);
    assert!(no_line.to_string().starts_with("b.env: "));

    let ctrl = EnvFileError::new("a\nb\x1b[0m.env", Some(1), EnvFileErrorKind::TooLarge);
    let shown = ctrl.to_string();
    assert!(
        !shown.contains('\n') && !shown.contains('\u{1b}'),
        "{shown}"
    );
    assert!(!shown.contains(SECRET));
}

#[test]
fn 誤りの要約は_20_件で打ち切り残りの件数を返す() {
    let make = |n: usize| -> Vec<EnvFileError> {
        (1..=n)
            .map(|i| EnvFileError::new("a.env", Some(i), EnvFileErrorKind::InvalidName))
            .collect()
    };
    let e = make(20);
    assert_eq!(summarize_errors(&e).0.len(), 20);
    assert_eq!(summarize_errors(&e).1, 0);
    let e = make(25);
    let (shown, rest) = summarize_errors(&e);
    assert_eq!((shown.len(), rest), (20, 5));
    assert_eq!(shown[19].line, Some(20));
}
