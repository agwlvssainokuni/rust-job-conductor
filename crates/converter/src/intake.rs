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

//! 取り込み(Intake)。入力ファイルを開き、大きさを確認して読み取る。
//!
//! この Unit で、ファイルシステムに触れるのはここだけである(NFR4.5)。読み取りは
//! 「上限 + 1 バイト」までに制限し、確認と読み取りのあいだにファイルが変わっても、
//! 上限を超えて読まない(NFR4.2)。失敗の順序は BR1.12 の 1〜4:
//! fileMissing → unreadable(権限・通常でない)→ tooLarge → unreadable(UTF-8 でない)。

use std::fs::File;
use std::io::{ErrorKind, Read};
use std::path::Path;

use crate::error::{ConversionError, ConversionErrorKind, reason};

/// 入力の大きさの上限(1 MiB。NFR4.2)。
pub(crate) const MAX_INPUT_BYTES: u64 = 1024 * 1024;

/// 表示用のファイル名。
pub(crate) fn file_label(path: &Path) -> String {
    path.display().to_string()
}

fn error(file: &str, kind: ConversionErrorKind, why: &str) -> ConversionError {
    ConversionError::new(file, None, None, kind, why)
}

fn io_reason(kind: ErrorKind) -> &'static str {
    match kind {
        ErrorKind::PermissionDenied => reason::PERMISSION,
        _ => reason::READ_FAILED,
    }
}

/// 入力ファイルを読み取って、文字列として返す。
pub(crate) fn read_input(path: &Path) -> Result<String, ConversionError> {
    let file = file_label(path);

    // BR1.12 の 1〜2: 先に開く(ない → fileMissing、権限など → unreadable)
    let handle = File::open(path).map_err(|e| match e.kind() {
        ErrorKind::NotFound => error(
            &file,
            ConversionErrorKind::FileMissing,
            reason::FILE_MISSING,
        ),
        k => {
            // Windows ではディレクトリを開くと権限のエラーになるので、通常でないことを見分ける
            let not_regular = std::fs::metadata(path).is_ok_and(|m| !m.is_file());
            let why = if not_regular {
                reason::NOT_REGULAR
            } else {
                io_reason(k)
            };
            error(&file, ConversionErrorKind::Unreadable, why)
        }
    })?;
    // BR1.12 の 3: 大きさは、開いたハンドルから取る(権限のないファイルは、ここへ来ない)
    let meta = handle
        .metadata()
        .map_err(|e| error(&file, ConversionErrorKind::Unreadable, io_reason(e.kind())))?;
    if !meta.is_file() {
        return Err(error(
            &file,
            ConversionErrorKind::Unreadable,
            reason::NOT_REGULAR,
        ));
    }
    if meta.len() > MAX_INPUT_BYTES {
        return Err(too_large(&file));
    }

    // BR1.12 の 4: 上限 + 1 バイトまでの読み取りと、UTF-8 の確認
    let bytes = match read_bounded(handle, MAX_INPUT_BYTES) {
        Ok(Some(bytes)) => bytes,
        Ok(None) => return Err(too_large(&file)),
        Err(e) => {
            return Err(error(
                &file,
                ConversionErrorKind::Unreadable,
                io_reason(e.kind()),
            ));
        }
    };
    String::from_utf8(bytes)
        .map_err(|_| error(&file, ConversionErrorKind::Unreadable, reason::NOT_UTF8))
}

fn too_large(file: &str) -> ConversionError {
    error(file, ConversionErrorKind::TooLarge, reason::INPUT_TOO_LARGE)
}

/// `limit` バイトまでを読む。`limit` を超える入力は `None`(「上限 + 1 バイト」で止める)。
///
/// 終わりのない入力でも、`limit + 1` バイトを読んだところで止まる。
pub(crate) fn read_bounded<R: Read>(reader: R, limit: u64) -> std::io::Result<Option<Vec<u8>>> {
    let mut buf = Vec::new();
    reader.take(limit.saturating_add(1)).read_to_end(&mut buf)?;
    if buf.len() as u64 > limit {
        Ok(None)
    } else {
        Ok(Some(buf))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    struct Endless {
        served: u64,
    }
    impl Read for Endless {
        fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
            self.served += buf.len() as u64;
            buf.fill(b'a');
            Ok(buf.len())
        }
    }

    fn write_file(dir: &Path, name: &str, bytes: &[u8]) -> std::path::PathBuf {
        let p = dir.join(name);
        let mut f = File::create(&p).unwrap();
        f.write_all(bytes).unwrap();
        p
    }

    #[test]
    fn missing_file_is_file_missing() {
        let dir = tempfile::tempdir().unwrap();
        let e = read_input(&dir.path().join("none.yaml")).unwrap_err();
        assert_eq!(e.kind, ConversionErrorKind::FileMissing);
        assert!(e.line.is_none());
    }

    #[test]
    fn exactly_one_mib_is_accepted_and_one_more_is_too_large() {
        let dir = tempfile::tempdir().unwrap();
        let ok = write_file(dir.path(), "ok.txt", &vec![b'a'; MAX_INPUT_BYTES as usize]);
        assert_eq!(read_input(&ok).unwrap().len() as u64, MAX_INPUT_BYTES);
        let big = write_file(
            dir.path(),
            "big.txt",
            &vec![b'a'; MAX_INPUT_BYTES as usize + 1],
        );
        let e = read_input(&big).unwrap_err();
        assert_eq!(e.kind, ConversionErrorKind::TooLarge);
    }

    #[test]
    fn endless_reader_stops_at_limit_plus_one() {
        let mut r = Endless { served: 0 };
        let out = read_bounded(&mut r, 1000).unwrap();
        assert!(out.is_none());
        // 上限 + 1 バイトより、大きく読み進めない(Take が、読み取りの大きさを絞る)
        assert!(r.served < 1000 + 1 + 64 * 1024, "served {}", r.served);
        let small = read_bounded(&b"abc"[..], 3).unwrap();
        assert_eq!(small.as_deref(), Some(&b"abc"[..]));
        assert!(read_bounded(&b"abcd"[..], 3).unwrap().is_none());
    }

    #[test]
    fn invalid_utf8_is_unreadable() {
        let dir = tempfile::tempdir().unwrap();
        let p = write_file(
            dir.path(),
            "bad.yaml",
            &[b'a', b':', b' ', 0xff, 0xfe, b'\n'],
        );
        let e = read_input(&p).unwrap_err();
        assert_eq!(e.kind, ConversionErrorKind::Unreadable);
        assert_eq!(e.reason, reason::NOT_UTF8);
    }

    #[test]
    fn directory_is_unreadable() {
        let dir = tempfile::tempdir().unwrap();
        let e = read_input(dir.path()).unwrap_err();
        assert_eq!(e.kind, ConversionErrorKind::Unreadable);
        assert_eq!(e.reason, reason::NOT_REGULAR);
    }

    #[test]
    fn too_large_wins_over_invalid_utf8() {
        // BR1.12: 3) tooLarge は 4) UTF-8 でない より先
        let dir = tempfile::tempdir().unwrap();
        let p = write_file(
            dir.path(),
            "x.bin",
            &vec![0xffu8; MAX_INPUT_BYTES as usize + 1],
        );
        assert_eq!(
            read_input(&p).unwrap_err().kind,
            ConversionErrorKind::TooLarge
        );
    }

    #[cfg(unix)]
    fn set_mode(p: &Path, mode: u32) {
        // 本体の検査(OS 固有の API を使わない)に触れないよう、`use` を使わずに書く
        let perm = <std::fs::Permissions as std::os::unix::fs::PermissionsExt>::from_mode(mode);
        std::fs::set_permissions(p, perm).unwrap();
    }

    /// root などで権限が効かない環境では、確かめられないので、確認をやめる。
    #[cfg(unix)]
    fn permissions_enforced(p: &Path) -> bool {
        File::open(p).is_err()
    }

    #[cfg(unix)]
    #[test]
    fn mode_000_file_is_unreadable() {
        let dir = tempfile::tempdir().unwrap();
        let p = write_file(dir.path(), "locked.yaml", b"a: 1\n");
        set_mode(&p, 0o000);
        if permissions_enforced(&p) {
            let e = read_input(&p).unwrap_err();
            assert_eq!(e.kind, ConversionErrorKind::Unreadable);
            assert_eq!(e.reason, reason::PERMISSION);
        }
        set_mode(&p, 0o644);
    }

    #[cfg(unix)]
    #[test]
    fn mode_000_file_over_1_mib_is_unreadable_not_too_large() {
        // BR1.12: 2) 権限は 3) 大きさより先
        let dir = tempfile::tempdir().unwrap();
        let p = write_file(
            dir.path(),
            "locked-big.yaml",
            &vec![b'a'; MAX_INPUT_BYTES as usize + 1],
        );
        set_mode(&p, 0o000);
        if permissions_enforced(&p) {
            let e = read_input(&p).unwrap_err();
            assert_eq!(e.kind, ConversionErrorKind::Unreadable);
            assert_eq!(e.reason, reason::PERMISSION);
        }
        set_mode(&p, 0o644);
    }

    #[test]
    fn io_errors_map_to_fixed_reasons() {
        // OS 固有の処理(権限の変更)を使わず、対応の関数で確かめる(NFR3.1)
        assert_eq!(io_reason(ErrorKind::PermissionDenied), reason::PERMISSION);
        assert_eq!(io_reason(ErrorKind::Other), reason::READ_FAILED);
        assert_eq!(io_reason(ErrorKind::UnexpectedEof), reason::READ_FAILED);
    }
}
