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

//! 誤りの種類の一覧(テストの写し。BR1.7)。
//!
//! 一覧の定数、種類の位置を返す網羅の `match`(`_` を使わない)、一覧の i 番目の位置が i であることの
//! コンパイル時の確かめを並べて置く。種類を足すと `match` がコンパイルで止まり、一覧の並びの誤りや
//! 重複も、コンパイルで止まる。

use conductor_converter::ConversionErrorKind as C;

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
