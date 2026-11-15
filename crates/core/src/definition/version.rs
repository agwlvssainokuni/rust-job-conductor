//! 版の確認(VersionCheck)。
//!
//! 書式の版のキー(`format_version`)を確認する。無ければ `versionMissing`、
//! 対応する版(初版は 1)でなければ `versionUnsupported`。通らなければ、他の検査はしない(BR1.7)。

use toml::de::{DeTable, DeValue};

use crate::error::{DefinitionError, DefinitionErrorKind, Locate};

/// 書式の版のキー。
pub const VERSION_KEY: &str = "format_version";

/// この `conductor` が対応する書式の版。
pub const SUPPORTED_FORMAT_VERSION: u32 = 1;

/// 版を確認し、対応する版の値を返す。
pub fn check_version(
    root: &DeTable<'_>,
    file: &str,
    locate: &Locate,
) -> Result<u32, DefinitionError> {
    let Some(value) = root.get(VERSION_KEY) else {
        return Err(DefinitionError::new(
            file,
            None,
            DefinitionErrorKind::VersionMissing,
            "`format_version` が必要です(対応する版は 1 です)",
        ));
    };
    let line = Some(locate.line_of(value.span().start));
    let unsupported = |reason: &str| {
        DefinitionError::new(file, line, DefinitionErrorKind::VersionUnsupported, reason)
    };
    let DeValue::Integer(int) = value.get_ref() else {
        return Err(unsupported(
            "`format_version` は整数で書きます(対応する版は 1 です)",
        ));
    };
    match i64::from_str_radix(int.as_str(), int.radix()) {
        Ok(v) if v == i64::from(SUPPORTED_FORMAT_VERSION) => Ok(SUPPORTED_FORMAT_VERSION),
        _ => Err(unsupported(
            "この版には対応していません(対応する版は 1 です)",
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn check(text: &str) -> Result<u32, DefinitionError> {
        let locate = Locate::new(text);
        let doc = DeTable::parse(text).expect("読み取れる入力");
        check_version(doc.get_ref(), "f.toml", &locate)
    }

    #[test]
    fn 版_1_は受理() {
        assert_eq!(check("format_version = 1\n").unwrap(), 1);
    }

    #[test]
    fn 版がないと_version_missing() {
        let e = check("a = 1\n").unwrap_err();
        assert_eq!(e.kind, DefinitionErrorKind::VersionMissing);
    }

    #[test]
    fn 版_2_は_version_unsupported_で行つき() {
        let e = check("\nformat_version = 2\n").unwrap_err();
        assert_eq!(e.kind, DefinitionErrorKind::VersionUnsupported);
        assert_eq!(e.line, Some(2));
    }

    #[test]
    fn 整数でない版も_version_unsupported() {
        for text in [
            "format_version = \"1\"\n",
            "format_version = 1.0\n",
            "format_version = 9\n",
        ] {
            assert_eq!(
                check(text).unwrap_err().kind,
                DefinitionErrorKind::VersionUnsupported
            );
        }
    }

    #[test]
    fn 負の版や_0_も_version_unsupported() {
        assert!(check("format_version = 0\n").is_err());
        assert!(check("format_version = -1\n").is_err());
    }
}
