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

//! 排他的なファイルの作成(RuntimeFileCreate。U4 の BR1.3、BR1.6)。

/// ファイルの作成の成功の結果(BR1.4)。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CreateOutcome {
    /// 新規に作った。
    Created,
    /// `--force` で、既にあったファイルを置き換えた。
    Replaced,
}

/// ファイルを作れなかった理由(BR1.2、BR3.2)。表示は、種類の固定の文とパスだけで、OS の
/// エラーの文は含めない。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CreateError {
    /// 既にある(`--force` なし)。終了コード 69。
    AlreadyExists,
    /// 親のディレクトリが無い(作らない。BR1.6)。
    ParentMissing,
    /// 作る先が、ディレクトリ。
    IsDirectory,
    /// 書く権限がない。
    PermissionDenied,
    /// そのほかの入出力の誤り。
    OtherIo,
}

impl CreateError {
    /// すべての種類(網羅のテストに使う)。
    pub const ALL: [CreateError; 5] = [
        Self::AlreadyExists,
        Self::ParentMissing,
        Self::IsDirectory,
        Self::PermissionDenied,
        Self::OtherIo,
    ];
}
