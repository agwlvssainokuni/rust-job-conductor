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

mod common;

use std::collections::HashSet;

use common::SECRET;
use common::kinds::{DEFINITION_ERROR_KINDS, ENV_FILE_ERROR_KINDS};
use conductor_core::{
    DefinitionErrorKind, EffectiveEntry, EnvEntry, EnvFileError, EnvFileErrorKind, EnvVar,
    summarize_errors,
};

#[test]
fn 値を持つ型の_debug_は値を伏せる() {
    let var = EnvVar {
        name: "TOKEN".into(),
        value: SECRET.into(),
    };
    let entry = EnvEntry {
        name: "TOKEN".into(),
        value: SECRET.into(),
        line: 1,
    };
    let eff = EffectiveEntry {
        name: "TOKEN".into(),
        value: SECRET.into(),
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
fn 定義の誤りの種類は_33_種で表示が空でない() {
    // 33 種であることと重複の無いことは、写しの型とコンパイル時の確かめが保つ。
    for k in DEFINITION_ERROR_KINDS {
        assert!(!k.label().is_empty());
    }
}

#[test]
fn env_file_の誤りの種類は_13_種で理由が重複しない() {
    // 13 種であることと重複の無いことは、写しの型とコンパイル時の確かめが保つ。
    let reasons: HashSet<_> = ENV_FILE_ERROR_KINDS.iter().map(|k| k.reason()).collect();
    assert_eq!(reasons.len(), 13);
    for k in ENV_FILE_ERROR_KINDS {
        assert!(!k.label().is_empty());
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

/// `.env` の誤りの種類の文(13 種)。変更前の文を固定の文字列で書く(1h、1j、D10)。
const ENV_FILE_KIND_TEXTS: [(EnvFileErrorKind, &str); 13] = [
    (
        EnvFileErrorKind::FileMissing,
        "環境変数ファイルが存在しません",
    ),
    (EnvFileErrorKind::Unreadable, "環境変数ファイルを読めません"),
    (EnvFileErrorKind::TooLarge, "環境変数ファイルが大きすぎます"),
    (
        EnvFileErrorKind::NotUtf8,
        "環境変数ファイルが UTF-8 ではありません",
    ),
    (
        EnvFileErrorKind::TooManyEntries,
        "環境変数ファイルの項目が多すぎます",
    ),
    (
        EnvFileErrorKind::InvalidName,
        "環境変数の名前が正しくありません",
    ),
    (
        EnvFileErrorKind::ReservedName,
        "予約された環境変数の名前です",
    ),
    (
        EnvFileErrorKind::CaseConflict,
        "大文字小文字だけが違う環境変数の名前があります",
    ),
    (
        EnvFileErrorKind::MissingEquals,
        "`KEY=VALUE` の形ではありません",
    ),
    (
        EnvFileErrorKind::UnterminatedQuote,
        "引用符が閉じていません",
    ),
    (EnvFileErrorKind::InvalidEscape, "許されないエスケープです"),
    (
        EnvFileErrorKind::TrailingGarbage,
        "閉じた引用符のあとに余計な文字があります",
    ),
    (EnvFileErrorKind::NulInValue, "値に NUL が含まれています"),
];

/// 定義の誤りの種類の文(33 種)。変更前の文を固定の文字列で書く(1j、D10)。
const DEFINITION_KIND_TEXTS: [(DefinitionErrorKind, &str); 34] = [
    (
        DefinitionErrorKind::FileMissing,
        "定義ファイルが存在しません",
    ),
    (DefinitionErrorKind::Unreadable, "定義ファイルを読めません"),
    (DefinitionErrorKind::TooLarge, "定義ファイルが大きすぎます"),
    (DefinitionErrorKind::Syntax, "文法の誤りです"),
    (
        DefinitionErrorKind::TypeMismatch,
        "値の型が正しくありません",
    ),
    (DefinitionErrorKind::UnknownKey, "定義にないキーです"),
    (DefinitionErrorKind::VersionMissing, "書式の版がありません"),
    (
        DefinitionErrorKind::VersionUnsupported,
        "対応していない書式の版です",
    ),
    (
        DefinitionErrorKind::UnknownTarget,
        "遷移先のジョブが定義にありません",
    ),
    (
        DefinitionErrorKind::ErrorTransition,
        "エラー結果には遷移先を書けません",
    ),
    (
        DefinitionErrorKind::BackwardOrSelfTarget,
        "遷移先は、記述順で後ろのジョブまたは @end のみです",
    ),
    (
        DefinitionErrorKind::OverlappingExitCodes,
        "成功と警告の終了コードが重なっています",
    ),
    (
        DefinitionErrorKind::ZeroAsWarning,
        "0 は警告の終了コードにできません",
    ),
    (
        DefinitionErrorKind::PipelineNameMismatch,
        "パイプライン名が定義と一致しません",
    ),
    (
        DefinitionErrorKind::DuplicateJobId,
        "ジョブIDが重複しています",
    ),
    (
        DefinitionErrorKind::InvalidJobId,
        "ジョブIDが正しくありません",
    ),
    (DefinitionErrorKind::NoJobs, "ジョブが 1 つもありません"),
    (DefinitionErrorKind::EmptyField, "必須の値が空です"),
    (
        DefinitionErrorKind::DuplicateTransition,
        "同じ結果の遷移が重複しています",
    ),
    (
        DefinitionErrorKind::InvalidEnvName,
        "環境変数の名前が正しくありません",
    ),
    (DefinitionErrorKind::RetryNotTable, "retry は表で書きます"),
    (
        DefinitionErrorKind::RetryMissingKey,
        "retry の必須のキーがありません",
    ),
    (
        DefinitionErrorKind::RetryExitCodesEmpty,
        "retry の終了コードの一覧が空です",
    ),
    (
        DefinitionErrorKind::RetryExitCodeInvalid,
        "retry の終了コードが正しくありません",
    ),
    (
        DefinitionErrorKind::RetryExitCodesOverlap,
        "retry の終了コードが、成功または警告の終了コードと重なっています",
    ),
    (
        DefinitionErrorKind::RetryMaxInvalid,
        "retry の最大回数が正しくありません",
    ),
    (
        DefinitionErrorKind::RetryDurationInvalid,
        "retry の間隔が正しくありません",
    ),
    (
        DefinitionErrorKind::RetryBackoffIncomplete,
        "retry の倍率と間隔の上限は、両方を書くか、両方とも書きません",
    ),
    (
        DefinitionErrorKind::RetryMultiplierInvalid,
        "retry の倍率が正しくありません",
    ),
    (
        DefinitionErrorKind::RetryMaxIntervalTooSmall,
        "retry の間隔の上限が、間隔より小さくなっています",
    ),
    (
        DefinitionErrorKind::ReservedEnvName,
        "予約された環境変数の名前です",
    ),
    (
        DefinitionErrorKind::EnvNameCaseConflict,
        "大文字小文字だけが違う環境変数の名前があります",
    ),
    (
        DefinitionErrorKind::InvalidEnvValue,
        "環境変数の値が正しくありません",
    ),
    (
        DefinitionErrorKind::UnsetEnvConflict,
        "消す環境変数の名前が、足す環境変数の名前と重なっています",
    ),
];

#[test]
fn 誤りの理由は種類の固定の文で_種類の文は変更前と同じ() {
    // `.env` の誤りの理由は、種類の固定の文(フィールドの写しを持たない。1h)。
    for kind in ENV_FILE_ERROR_KINDS {
        let e = EnvFileError::new("a.env", Some(1), kind);
        assert!(e.to_string().contains(kind.reason()), "{kind:?}");
    }
    // 種類の文は、変更前の文と 1 文字も違わない(表は ALL と同じ順で、すべての種類を持つ)。
    let env_kinds: Vec<EnvFileErrorKind> = ENV_FILE_KIND_TEXTS.iter().map(|(k, _)| *k).collect();
    assert_eq!(env_kinds, ENV_FILE_ERROR_KINDS);
    for (kind, text) in ENV_FILE_KIND_TEXTS {
        assert_eq!(kind.label(), text, "{kind:?}");
    }
    let def_kinds: Vec<DefinitionErrorKind> =
        DEFINITION_KIND_TEXTS.iter().map(|(k, _)| *k).collect();
    assert_eq!(def_kinds, DEFINITION_ERROR_KINDS);
    for (kind, text) in DEFINITION_KIND_TEXTS {
        assert_eq!(kind.label(), text, "{kind:?}");
    }
}
