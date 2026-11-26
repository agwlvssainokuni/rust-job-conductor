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

//! ログのディレクトリとファイル(RuntimeLog。v0.10.0 の BR3.1〜BR3.4、BR6.1、契約 C6)。
//!
//! 試行ごとに 1 つの新しいファイル `<安全な ID>.<詰めた時刻>.<試行の番号>.log` を、`log_dir` の
//! 直下に作る。ディレクトリが無ければ親を含めて作る(権限は既定のまま)。ファイルは「無いときだけ作る」
//! (`create_new`)で開き、今のファイルを上書きも追記もしない。同じ名前があれば `-2`〜`-1000` を
//! 足して作り直す。UNIX 系は作るときに 0600 を付ける(`platform`)。ログのファイルを消すことはしない
//! (BR6.1)。

use std::fs::{self, File, OpenOptions};
use std::io;
use std::path::PathBuf;

use conductor_core::{Timestamp, format_compact};

use crate::platform;

/// 名前の番号(`-<n>`)の上限(BR3.3)。1 は番号なしの名前。
const MAX_SUFFIX: u32 = 1000;

/// 安全な名前(stem)の長さの上限(バイト。UTF-8 の文字の途中では切らない)。
const MAX_STEM_BYTES: usize = 100;

/// Windows の予約名(大文字小文字を問わない。`security-design.md` の 1)。
const RESERVED_NAMES: [&str; 22] = [
    "CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8",
    "COM9", "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
];

/// 1 回の試行のログの置き場所の指定(`entities.md` の LogTarget)。cli の流れが試行ごとに作る。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogTarget {
    /// ログのディレクトリ(解決済み。相対の `log_dir` は、定義ファイルのディレクトリが基準)。
    pub dir: PathBuf,
    /// 定義のジョブ ID(名前にするときに [`safe_log_stem`] で置き換える)。
    pub job_id: String,
    /// 流れがその試行の前に取った現在の時刻(BR2.3)。
    pub started_at: Timestamp,
    /// 1 から数える試行の番号(`CONDUCTOR_ATTEMPT` と同じ)。
    pub attempt: u32,
}

/// ログの書き込みの結果(`entities.md` と `reliability-design.md` の 1 の LogOutcome)。
///
/// 3 つとも、起きたときにそのログのファイルのパスを持つ(流れが表示の 1 行に使う。BR5.1〜BR5.3)。
/// `log_dir` が無いときは、すべて `None`(`Default`)。
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LogOutcome {
    /// 起動の前にログのディレクトリかファイルを作れなかった(このときジョブは起動していない。BR5.1)。
    pub log_not_created: Option<PathBuf>,
    /// 実行中にファイルへの書き込みに失敗し、以後ファイルに書かなかった(BR5.2)。
    pub write_failed: Option<PathBuf>,
    /// ジョブの終了の後、出力の読み取りを待ち切れなかった(BR4.5、BR5.3)。
    pub output_cut_short: Option<PathBuf>,
}

/// ジョブ ID を、ファイル名に使える安全な名前(stem)にする(BR3.2、`security-design.md` の 1)。
///
/// 1. `/` `\` `:` `*` `?` `"` `<` `>` `|` と、制御文字(U+0000〜U+001F、U+007F〜U+009F)を `_` にする。
/// 2. 先頭の `.` を `_` にする(隠しファイルや `..` を作らない)。
/// 3. 最初の `.` より前の部分から前後の ASCII の空白(` `)を除いた値が、大文字小文字を問わず
///    Windows の予約名なら、先頭に `_` を足す(`NUL.x` → `_NUL.x`、`con` → `_con`、
///    `con ` → `_con_`、`nul .tar.gz` → `_nul .tar.gz`。BR2.1、FR2.1)。空白を除くのは比べる
///    ときだけで、名前そのものからは除かない。
/// 4. 100 バイトまでに切る(UTF-8 の文字の途中では切らない)。
/// 5. 末尾の `.` か空白を `_` にする(Windows が名前から落とすもの)。
///
/// それ以外(日本語など)はそのまま。OS によらず同じ結果になり、ディレクトリの区切りは残らない。
pub fn safe_log_stem(job_id: &str) -> String {
    let mut stem: String = job_id
        .chars()
        .map(|c| if is_unsafe_char(c) { '_' } else { c })
        .collect();
    if stem.starts_with('.') {
        stem.replace_range(..1, "_");
    }
    let head = stem.split('.').next().unwrap_or_default().trim_matches(' ');
    if RESERVED_NAMES
        .iter()
        .any(|name| name.eq_ignore_ascii_case(head))
    {
        stem.insert(0, '_');
    }
    let mut end = stem.len().min(MAX_STEM_BYTES);
    while !stem.is_char_boundary(end) {
        end -= 1;
    }
    stem.truncate(end);
    if stem.ends_with(['.', ' ']) {
        stem.pop();
        stem.push('_');
    }
    stem
}

/// ファイル名に使えない・危ない文字か(BR3.2)。
fn is_unsafe_char(c: char) -> bool {
    matches!(c, '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|')
        || c <= '\u{1f}'
        || ('\u{7f}'..='\u{9f}').contains(&c)
}

/// 番号 `n` のファイル名(`n` が 1 なら番号なし。BR3.1、BR3.3)。
fn file_name(stem: &str, time: &str, attempt: u32, n: u32) -> String {
    if n <= 1 {
        format!("{stem}.{time}.{attempt}.log")
    } else {
        format!("{stem}.{time}.{attempt}-{n}.log")
    }
}

/// 試行のログのファイルを作って開く(BR3.1〜BR3.4)。
///
/// - ディレクトリが無ければ、親を含めて作る。
/// - `<安全な ID>.<詰めた時刻>.<試行の番号>.log` を「無いときだけ作る」で開く。UNIX 系は 0600。
/// - 「すでにある」なら `-2`、`-3` … `-1000` を足して作り直す。
///
/// 成功なら、書き込み用のファイルとそのパス。作れなければ、表示に使うパス(ディレクトリを作れない・
/// 番号を使い切ったときは番号なしの名前、開けなかったときはその名前)を `Err` で返す(BR5.1)。
/// OS のエラーの文は持たない。
pub fn open_log_file(target: &LogTarget) -> Result<(File, PathBuf), PathBuf> {
    let stem = safe_log_stem(&target.job_id);
    let time = format_compact(target.started_at);
    let first = target.dir.join(file_name(&stem, &time, target.attempt, 1));
    if fs::create_dir_all(&target.dir).is_err() {
        return Err(first);
    }
    for n in 1..=MAX_SUFFIX {
        let path = target.dir.join(file_name(&stem, &time, target.attempt, n));
        let mut options = OpenOptions::new();
        options.write(true).create_new(true);
        platform::restrict_to_owner(&mut options);
        match options.open(&path) {
            Ok(file) => return Ok((file, path)),
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(_) => return Err(path),
        }
    }
    Err(first)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn 番号が_1_なら番号なし_2_からは番号つき() {
        assert_eq!(file_name("a", "T", 1, 1), "a.T.1.log");
        assert_eq!(file_name("a", "T", 3, 2), "a.T.3-2.log");
        assert_eq!(
            file_name("a", "T", 1, MAX_SUFFIX),
            format!("a.T.1-{MAX_SUFFIX}.log")
        );
        assert_eq!(MAX_SUFFIX, 1000);
    }

    #[test]
    fn 番号を使い切ったら作らず_番号なしの名前を返す() {
        let dir = tempfile::tempdir().expect("一時ディレクトリ");
        let target = LogTarget {
            dir: dir.path().to_path_buf(),
            job_id: "a".into(),
            started_at: Timestamp::EPOCH,
            attempt: 1,
        };
        for n in 1..=MAX_SUFFIX {
            fs::write(
                dir.path().join(file_name("a", "19700101T000000Z", 1, n)),
                b"",
            )
            .expect("書ける");
        }
        let err = open_log_file(&target).expect_err("番号を使い切る");
        assert_eq!(err, dir.path().join("a.19700101T000000Z.1.log"));
        // 上限の 1 つ前が空いていれば、そこに作る。
        fs::remove_file(
            dir.path()
                .join(file_name("a", "19700101T000000Z", 1, MAX_SUFFIX)),
        )
        .expect("消せる");
        let (_, path) = open_log_file(&target).expect("作れる");
        assert_eq!(
            path,
            dir.path()
                .join(format!("a.19700101T000000Z.1-{MAX_SUFFIX}.log"))
        );
    }

    #[test]
    fn 名前の最長でも_255_バイトを超えない() {
        let stem = safe_log_stem(&"あ".repeat(200));
        assert!(stem.len() <= MAX_STEM_BYTES);
        let name = file_name(&stem, "99991231T235959Z", u32::MAX, MAX_SUFFIX);
        assert!(name.len() <= 255, "{}", name.len());
    }
}
