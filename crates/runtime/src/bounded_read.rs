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

//! 上限つきの読み取り(`.env` の読み込みが使う。NFR4.3、NFR4.7)。
//!
//! 通常のファイルを、開く前にパスで種別を見て(通常のファイルでなければ開かない。名前付きパイプで
//! 待たない)、開いたものの種別と大きさを見てから、上限 + 1 バイトまでで読む(形 B)。失敗は、すべて
//! [`ReadFailure`] の種類で返し、パニックしない。誤りの種類の写し方と表示は、呼び出し側が決める。

use std::fs::{self, File};
use std::io::{self, ErrorKind, Read};
use std::path::Path;

/// [`read_regular_file`] の失敗。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ReadFailure {
    /// ファイルが無い。
    Missing,
    /// 種別やメタデータを確かめられない、通常のファイルでない、開けない、または読めない。
    Unreadable,
    /// 大きさが上限を超える。
    TooLarge,
}

/// 通常のファイルを、上限 `limit` バイトまでで読む(形 B)。
///
/// 1. パスのメタデータ(シンボリックリンクは辿る): 無い → `Missing`、取れない → `Unreadable`。
///    通常のファイルでない(ディレクトリ、名前付きパイプ、デバイスなど)→ `Unreadable`(開かない)。
/// 2. 開く: 無い(1 のあとに消えた)→ `Missing`、開けない(読む権限がないなど)→ `Unreadable`。
/// 3. 開いたものから種別と大きさ: 取れない・通常のファイルでない → `Unreadable`、上限超 → `TooLarge`。
/// 4. 上限 + 1 バイトまで読む([`read_opened`]): 失敗 → `Unreadable`、超えれば `TooLarge`。
pub(crate) fn read_regular_file(path: &Path, limit: u64) -> Result<Vec<u8>, ReadFailure> {
    let open_failure = |e: io::Error| {
        if e.kind() == ErrorKind::NotFound {
            ReadFailure::Missing
        } else {
            ReadFailure::Unreadable
        }
    };
    // 1. 種別をパスで見る。
    let metadata = fs::metadata(path).map_err(open_failure)?;
    if !metadata.is_file() {
        return Err(ReadFailure::Unreadable);
    }
    // 2. 開く(読む権限がなければ、大きさにかかわらず `Unreadable`)。
    let file = File::open(path).map_err(open_failure)?;
    // 3. 開いたものから種別と大きさを見る。
    let opened = file.metadata().map_err(|_| ReadFailure::Unreadable)?;
    if !opened.is_file() {
        return Err(ReadFailure::Unreadable);
    }
    if opened.len() > limit {
        return Err(ReadFailure::TooLarge);
    }
    // 4. 上限 + 1 バイトまで読む。
    read_opened(file, limit)
}

/// 開いた読み手から、上限 + 1 バイトまで読み、超えれば `TooLarge`(報告された大きさより長く続く
/// 入力でも、そこで止まる。確かめたあとに伸びたファイルのため)。読み取りの失敗は `Unreadable`。
fn read_opened<R: Read>(reader: R, limit: u64) -> Result<Vec<u8>, ReadFailure> {
    let bytes = read_limited(reader, limit).map_err(|_| ReadFailure::Unreadable)?;
    if bytes.len() as u64 > limit {
        return Err(ReadFailure::TooLarge);
    }
    Ok(bytes)
}

/// 読み取りを、上限 + 1 バイトまでに制限する(終わりのない入力でも、そこで止まる)。
pub(crate) fn read_limited<R: Read>(reader: R, limit: u64) -> io::Result<Vec<u8>> {
    let mut bytes = Vec::new();
    reader
        .take(limit.saturating_add(1))
        .read_to_end(&mut bytes)?;
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn 上限_1_つぶんだけ読む() {
        let endless = io::repeat(b'a');
        let bytes = read_limited(endless, 10).expect("読める");
        assert_eq!(bytes.len(), 11);
        let bytes = read_limited(&b"abc"[..], 10).expect("読める");
        assert_eq!(bytes, b"abc");
    }
}
