//! ソースの検索(`src/` だけ。`src/bin/` の補助の実行ファイルと `tests/` は対象外)。
//! NFR3.1(OS 固有のコードは `platform` だけ)、NFR4.1(シェルを呼ばない)、NFR4.7(外部の入力で
//! パニックしない書き方)、NFR4.8(`unsafe` は 1 か所だけ)。

use std::path::{Path, PathBuf};

fn src_files() -> Vec<(PathBuf, String)> {
    fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
        for entry in std::fs::read_dir(dir).expect("読める").flatten() {
            let path = entry.path();
            if path.is_dir() {
                if path.file_name().is_some_and(|n| n == "bin") {
                    continue;
                }
                walk(&path, out);
            } else if path.extension().is_some_and(|e| e == "rs") {
                out.push(path);
            }
        }
    }
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut files = Vec::new();
    walk(&root, &mut files);
    files.sort();
    files
        .into_iter()
        .map(|p| {
            let text = std::fs::read_to_string(&p).expect("読める");
            (p, text)
        })
        .collect()
}

fn in_platform(path: &Path) -> bool {
    path.components().any(|c| c.as_os_str() == "platform")
}

/// コメントでない行(`//` で始まらない行)。
fn code_lines(text: &str) -> impl Iterator<Item = &str> {
    text.lines().filter(|l| !l.trim_start().starts_with("//"))
}

/// テストモジュールより前の部分(製品のコード)。
fn production_part(text: &str) -> &str {
    match text.find("#[cfg(test)]") {
        Some(i) => &text[..i],
        None => text,
    }
}

/// 単語としての `unsafe`(`unsafe_code` は含めない)の出現の数。
fn count_unsafe(line: &str) -> usize {
    let bytes = line.as_bytes();
    let mut count = 0;
    let mut from = 0;
    while let Some(i) = line[from..].find("unsafe") {
        let at = from + i;
        let before_ok =
            at == 0 || !(bytes[at - 1].is_ascii_alphanumeric() || bytes[at - 1] == b'_');
        let after = at + "unsafe".len();
        let after_ok =
            after >= bytes.len() || !(bytes[after].is_ascii_alphanumeric() || bytes[after] == b'_');
        if before_ok && after_ok {
            count += 1;
        }
        from = after;
    }
    count
}

#[test]
fn unsafe_は_platform_の_unix_系の_1_か所だけ() {
    let mut found = Vec::new();
    for (path, text) in src_files() {
        let n: usize = code_lines(&text).map(count_unsafe).sum();
        if n > 0 {
            found.push((path, n));
        }
    }
    assert_eq!(found.len(), 1, "{found:?}");
    let (path, n) = &found[0];
    assert!(path.ends_with("platform/unix.rs"), "{path:?}");
    assert_eq!(*n, 1, "unsafe の呼び出しは 1 回だけ");
}

#[test]
fn クレートは_unsafe_を拒否し_許すのは_platform_の_unix_系だけ() {
    for (path, text) in src_files() {
        if path.ends_with("src/lib.rs") {
            assert!(text.contains("#![deny(unsafe_code)]"));
        }
        let allows = code_lines(&text).any(|l| l.contains("allow(unsafe_code)"));
        assert_eq!(allows, path.ends_with("platform/unix.rs"), "{path:?}");
    }
}

#[test]
fn os_固有の分岐は_platform_の外にない() {
    let needles = [
        "cfg(unix)",
        "cfg(windows)",
        "cfg!(unix)",
        "cfg!(windows)",
        "target_os",
        "target_family",
        "os::unix",
        "os::windows",
        "libc::",
    ];
    for (path, text) in src_files() {
        if in_platform(&path) {
            continue;
        }
        for line in code_lines(&text) {
            for needle in needles {
                assert!(!line.contains(needle), "{path:?}: {line}");
            }
        }
    }
}

#[test]
fn シェルを呼ばない() {
    let forbidden = [
        "\"sh\"",
        "\"cmd\"",
        "\"/bin/sh\"",
        "\"cmd.exe\"",
        "\"bash\"",
        "\"powershell\"",
        "\"-c\"",
        "\"/C\"",
    ];
    for (path, text) in src_files() {
        for line in code_lines(&text) {
            for needle in forbidden {
                assert!(!line.contains(needle), "{path:?}: {line}");
            }
            if line.contains("Command::new(") {
                let ok = line.contains("Command::new(&line.program)")
                    || (path.ends_with("platform/unix.rs")
                        && line.contains("Command::new(\"sleep\")"));
                assert!(ok, "{path:?}: {line}");
            }
        }
    }
}

#[test]
fn 製品のコードに_パニックしうる書き方がない() {
    let forbidden = [
        ".unwrap()",
        ".expect(",
        "panic!(",
        "unreachable!(",
        "todo!(",
        "unimplemented!(",
    ];
    for (path, text) in src_files() {
        for line in code_lines(production_part(&text)) {
            for needle in forbidden {
                assert!(!line.contains(needle), "{path:?}: {line}");
            }
        }
    }
}
