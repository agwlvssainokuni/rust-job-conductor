//! 取り込み(Intake)。定義ファイルの場所を決め、大きさを確認して読み取る。
//!
//! このクレートで、ファイルシステムに触れるのはここだけである(NFR4.5)。
//! 読み取りは「上限 + 1 バイト」までに制限し、確認と読み取りのあいだにファイルが
//! 変わっても、上限を超えて読まない(NFR4.4)。

use std::fs::File;
use std::io::{ErrorKind, Read};
use std::path::{Path, PathBuf};

use crate::error::{DefinitionError, DefinitionErrorKind};

/// 定義ファイルの大きさの上限(1 MiB。NFR4.4)。
pub const MAX_DEFINITION_BYTES: u64 = 1024 * 1024;

/// 既定の定義ファイル名(BR4.1)。
pub const DEFAULT_DEFINITION_FILE: &str = "conductor.toml";

/// 定義ファイルの場所を決める(BR4.1)。指定があればそれ、なければ実行ディレクトリの `conductor.toml`。
pub fn resolve_path(path: Option<&Path>) -> PathBuf {
    match path {
        Some(p) => p.to_path_buf(),
        None => PathBuf::from(DEFAULT_DEFINITION_FILE),
    }
}

/// 表示用のファイル名。
pub fn file_label(path: &Path) -> String {
    path.display().to_string()
}

fn error(file: &str, kind: DefinitionErrorKind, reason: &str) -> DefinitionError {
    DefinitionError::new(file, None, kind, reason)
}

fn io_reason(kind: ErrorKind) -> &'static str {
    match kind {
        ErrorKind::PermissionDenied => "読み取りの権限がありません",
        _ => "読み取りに失敗しました",
    }
}

/// 定義ファイルを読み取って、文字列として返す。
///
/// 失敗は、次の種類の `DefinitionError` として返す。
/// 存在しない: `fileMissing`、上限超過: `tooLarge`、
/// 権限がない・通常のファイルでない・UTF-8 でない: `unreadable`(理由で区別)。
pub fn read_definition(path: &Path) -> Result<String, DefinitionError> {
    let file = file_label(path);

    let meta = std::fs::metadata(path).map_err(|e| match e.kind() {
        ErrorKind::NotFound => error(
            &file,
            DefinitionErrorKind::FileMissing,
            "指定の場所にファイルがありません",
        ),
        k => error(&file, DefinitionErrorKind::Unreadable, io_reason(k)),
    })?;

    if !meta.is_file() {
        return Err(error(
            &file,
            DefinitionErrorKind::Unreadable,
            "通常のファイルではありません",
        ));
    }
    if meta.len() > MAX_DEFINITION_BYTES {
        return Err(too_large(&file));
    }

    let handle = File::open(path)
        .map_err(|e| error(&file, DefinitionErrorKind::Unreadable, io_reason(e.kind())))?;
    let mut bytes = Vec::new();
    handle
        .take(MAX_DEFINITION_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| error(&file, DefinitionErrorKind::Unreadable, io_reason(e.kind())))?;
    if bytes.len() as u64 > MAX_DEFINITION_BYTES {
        return Err(too_large(&file));
    }

    String::from_utf8(bytes).map_err(|_| {
        error(
            &file,
            DefinitionErrorKind::Unreadable,
            "文字コードが UTF-8 ではありません",
        )
    })
}

fn too_large(file: &str) -> DefinitionError {
    error(
        file,
        DefinitionErrorKind::TooLarge,
        "ファイルの大きさが上限(1 MiB)を超えています",
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kind_of(r: Result<String, DefinitionError>) -> DefinitionErrorKind {
        r.expect_err("失敗するはず").kind
    }

    #[test]
    fn 場所の既定は_conductor_toml() {
        assert_eq!(resolve_path(None), PathBuf::from("conductor.toml"));
        assert_eq!(
            resolve_path(Some(Path::new("x/y.toml"))),
            PathBuf::from("x/y.toml")
        );
    }

    #[test]
    fn 存在しないと_file_missing() {
        let dir = tempfile::tempdir().unwrap();
        let k = kind_of(read_definition(&dir.path().join("none.toml")));
        assert_eq!(k, DefinitionErrorKind::FileMissing);
    }

    #[test]
    fn ディレクトリは_unreadable() {
        let dir = tempfile::tempdir().unwrap();
        let e = read_definition(dir.path()).unwrap_err();
        assert_eq!(e.kind, DefinitionErrorKind::Unreadable);
        assert!(e.reason.contains("通常のファイル"));
    }

    #[test]
    fn ちょうど上限は受理し_上限超過は_too_large() {
        let dir = tempfile::tempdir().unwrap();
        let ok = dir.path().join("ok.toml");
        std::fs::write(&ok, "#".repeat(MAX_DEFINITION_BYTES as usize)).unwrap();
        assert_eq!(
            read_definition(&ok).unwrap().len() as u64,
            MAX_DEFINITION_BYTES
        );

        let big = dir.path().join("big.toml");
        std::fs::write(&big, "#".repeat(MAX_DEFINITION_BYTES as usize + 1)).unwrap();
        assert_eq!(
            kind_of(read_definition(&big)),
            DefinitionErrorKind::TooLarge
        );
    }

    #[test]
    fn utf8_でない内容は_unreadable() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("bad.toml");
        std::fs::write(&p, [0x66, 0xff, 0xfe]).unwrap();
        let e = read_definition(&p).unwrap_err();
        assert_eq!(e.kind, DefinitionErrorKind::Unreadable);
        assert!(e.reason.contains("UTF-8"));
    }

    #[test]
    fn 空のファイルは空の文字列() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("empty.toml");
        std::fs::write(&p, "").unwrap();
        assert_eq!(read_definition(&p).unwrap(), "");
    }

    #[cfg(unix)]
    #[test]
    fn 権限がないと_unreadable() {
        use std::os::unix::fs::PermissionsExt;
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("secret.toml");
        std::fs::write(&p, "x").unwrap();
        std::fs::set_permissions(&p, std::fs::Permissions::from_mode(0o000)).unwrap();
        // root では読めてしまうため、読めた場合は確認しない。
        if let Err(e) = read_definition(&p) {
            assert_eq!(e.kind, DefinitionErrorKind::Unreadable);
            assert!(e.reason.contains("権限"));
        }
    }
}
