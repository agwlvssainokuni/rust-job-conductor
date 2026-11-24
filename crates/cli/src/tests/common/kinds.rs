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

//! 誤りの種類の一覧(テストの写し。BR1.7)。表示の全種類のテストが使う。
//!
//! 一覧ごとに、一覧の定数、種類の位置を返す網羅の `match`(`_` を使わない)、一覧の i 番目の位置が
//! i であることのコンパイル時の確かめを並べて置く。種類を足すと `match` がコンパイルで止まり、
//! 一覧の並びの誤りや重複も、コンパイルで止まる。core と converter の一覧は、それぞれのテストにも
//! 写しがある(どちらも網羅の `match` で止まるので、片方だけが古くなることは無い)。

use crate::types::UsageErrorKind as U;
use conductor_converter::ConversionErrorKind as C;
use conductor_core::DefinitionErrorKind as D;
use conductor_core::EnvFileErrorKind as E;
use conductor_runtime::CreateError as R;

/// 定義の誤りの種類のすべて(テストの写し。種類を足したら、下の `match` がコンパイルで止まるので、
/// ここにも足す)。
pub const DEFINITION_ERROR_KINDS: [D; 33] = [
    D::FileMissing,
    D::Unreadable,
    D::TooLarge,
    D::Syntax,
    D::TypeMismatch,
    D::UnknownKey,
    D::VersionMissing,
    D::VersionUnsupported,
    D::UnknownTarget,
    D::ErrorTransition,
    D::BackwardOrSelfTarget,
    D::OverlappingExitCodes,
    D::ZeroAsWarning,
    D::PipelineNameMismatch,
    D::DuplicateJobId,
    D::InvalidJobId,
    D::NoJobs,
    D::EmptyField,
    D::DuplicateTransition,
    D::InvalidEnvName,
    D::RetryNotTable,
    D::RetryMissingKey,
    D::RetryExitCodesEmpty,
    D::RetryExitCodeInvalid,
    D::RetryExitCodesOverlap,
    D::RetryMaxInvalid,
    D::RetryDurationInvalid,
    D::RetryBackoffIncomplete,
    D::RetryMultiplierInvalid,
    D::RetryMaxIntervalTooSmall,
    D::ReservedEnvName,
    D::EnvNameCaseConflict,
    D::InvalidEnvValue,
];

/// 定義の誤りの種類の、一覧の中の位置(網羅の `match`。`_` を使わない)。
const fn definition_position(kind: D) -> usize {
    match kind {
        D::FileMissing => 0,
        D::Unreadable => 1,
        D::TooLarge => 2,
        D::Syntax => 3,
        D::TypeMismatch => 4,
        D::UnknownKey => 5,
        D::VersionMissing => 6,
        D::VersionUnsupported => 7,
        D::UnknownTarget => 8,
        D::ErrorTransition => 9,
        D::BackwardOrSelfTarget => 10,
        D::OverlappingExitCodes => 11,
        D::ZeroAsWarning => 12,
        D::PipelineNameMismatch => 13,
        D::DuplicateJobId => 14,
        D::InvalidJobId => 15,
        D::NoJobs => 16,
        D::EmptyField => 17,
        D::DuplicateTransition => 18,
        D::InvalidEnvName => 19,
        D::RetryNotTable => 20,
        D::RetryMissingKey => 21,
        D::RetryExitCodesEmpty => 22,
        D::RetryExitCodeInvalid => 23,
        D::RetryExitCodesOverlap => 24,
        D::RetryMaxInvalid => 25,
        D::RetryDurationInvalid => 26,
        D::RetryBackoffIncomplete => 27,
        D::RetryMultiplierInvalid => 28,
        D::RetryMaxIntervalTooSmall => 29,
        D::ReservedEnvName => 30,
        D::EnvNameCaseConflict => 31,
        D::InvalidEnvValue => 32,
    }
}

// 一覧の i 番目の位置が i であること(並びの誤りと重複を、コンパイルで止める)。
const _: () = {
    let mut i = 0;
    while i < DEFINITION_ERROR_KINDS.len() {
        assert!(definition_position(DEFINITION_ERROR_KINDS[i]) == i);
        i += 1;
    }
};

/// `.env` の誤りの種類のすべて(テストの写し。種類を足したら、下の `match` がコンパイルで止まるので、
/// ここにも足す)。
pub const ENV_FILE_ERROR_KINDS: [E; 13] = [
    E::FileMissing,
    E::Unreadable,
    E::TooLarge,
    E::NotUtf8,
    E::TooManyEntries,
    E::InvalidName,
    E::ReservedName,
    E::CaseConflict,
    E::MissingEquals,
    E::UnterminatedQuote,
    E::InvalidEscape,
    E::TrailingGarbage,
    E::NulInValue,
];

/// `.env` の誤りの種類の、一覧の中の位置(網羅の `match`。`_` を使わない)。
const fn env_file_position(kind: E) -> usize {
    match kind {
        E::FileMissing => 0,
        E::Unreadable => 1,
        E::TooLarge => 2,
        E::NotUtf8 => 3,
        E::TooManyEntries => 4,
        E::InvalidName => 5,
        E::ReservedName => 6,
        E::CaseConflict => 7,
        E::MissingEquals => 8,
        E::UnterminatedQuote => 9,
        E::InvalidEscape => 10,
        E::TrailingGarbage => 11,
        E::NulInValue => 12,
    }
}

// 一覧の i 番目の位置が i であること(並びの誤りと重複を、コンパイルで止める)。
const _: () = {
    let mut i = 0;
    while i < ENV_FILE_ERROR_KINDS.len() {
        assert!(env_file_position(ENV_FILE_ERROR_KINDS[i]) == i);
        i += 1;
    }
};

/// 変換の失敗の種類のすべて(テストの写し。種類を足したら、下の `match` がコンパイルで止まるので、
/// ここにも足す)。
pub const CONVERSION_ERROR_KINDS: [C; 6] = [
    C::FileMissing,
    C::Unreadable,
    C::TooLarge,
    C::Syntax,
    C::Unrepresentable,
    C::RootNotMap,
];

/// 変換の失敗の種類の、一覧の中の位置(網羅の `match`。`_` を使わない)。
const fn conversion_position(kind: C) -> usize {
    match kind {
        C::FileMissing => 0,
        C::Unreadable => 1,
        C::TooLarge => 2,
        C::Syntax => 3,
        C::Unrepresentable => 4,
        C::RootNotMap => 5,
    }
}

// 一覧の i 番目の位置が i であること(並びの誤りと重複を、コンパイルで止める)。
const _: () = {
    let mut i = 0;
    while i < CONVERSION_ERROR_KINDS.len() {
        assert!(conversion_position(CONVERSION_ERROR_KINDS[i]) == i);
        i += 1;
    }
};

/// ファイルを作れなかった理由のすべて(テストの写し。種類を足したら、下の `match` がコンパイルで
/// 止まるので、ここにも足す)。
pub const CREATE_ERRORS: [R; 5] = [
    R::AlreadyExists,
    R::ParentMissing,
    R::IsDirectory,
    R::PermissionDenied,
    R::OtherIo,
];

/// ファイルを作れなかった理由の、一覧の中の位置(網羅の `match`。`_` を使わない)。
const fn create_position(error: R) -> usize {
    match error {
        R::AlreadyExists => 0,
        R::ParentMissing => 1,
        R::IsDirectory => 2,
        R::PermissionDenied => 3,
        R::OtherIo => 4,
    }
}

// 一覧の i 番目の位置が i であること(並びの誤りと重複を、コンパイルで止める)。
const _: () = {
    let mut i = 0;
    while i < CREATE_ERRORS.len() {
        assert!(create_position(CREATE_ERRORS[i]) == i);
        i += 1;
    }
};

/// 使い方の誤りの種類のすべて(テストの写し。種類を足したら、下の `match` がコンパイルで止まるので、
/// ここにも足す)。
pub const USAGE_ERROR_KINDS: [U; 8] = [
    U::UnknownCommand,
    U::MissingArgument,
    U::UnexpectedArgument,
    U::UnknownOption,
    U::OptionNotAllowed,
    U::MissingValue,
    U::InvalidValue,
    U::UnknownJob,
];

/// 使い方の誤りの種類の、一覧の中の位置(網羅の `match`。`_` を使わない)。
const fn usage_position(kind: U) -> usize {
    match kind {
        U::UnknownCommand => 0,
        U::MissingArgument => 1,
        U::UnexpectedArgument => 2,
        U::UnknownOption => 3,
        U::OptionNotAllowed => 4,
        U::MissingValue => 5,
        U::InvalidValue => 6,
        U::UnknownJob => 7,
    }
}

// 一覧の i 番目の位置が i であること(並びの誤りと重複を、コンパイルで止める)。
const _: () = {
    let mut i = 0;
    while i < USAGE_ERROR_KINDS.len() {
        assert!(usage_position(USAGE_ERROR_KINDS[i]) == i);
        i += 1;
    }
};
