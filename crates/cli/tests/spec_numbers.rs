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

//! 仕様書(`docs/spec/rust/`、`docs/spec/language-agnostic/`)に書いた数が、実装と一致すること
//! (v0.10.3 の FR3.1〜FR3.3、BR3.1〜BR3.5)。
//!
//! 実装の値は、ソースとデモのスクリプトを文字列として読んで数える(本番のコードは変えない)。
//! 仕様書は、決まった場所(見出し・文・表)を必ず見て、さらに全体を手がかりの語で探す。
//! 全体の探しでは、版の注記(`v0.N.N まで`・`v0.N.N 以前`・`の時点`)のある行を履歴として除く。
//! リポジトリのファイルを読むだけで、書き込まない。

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

/// 突き合わせる仕様書のディレクトリ(リポジトリからの相対のパス)。
const SPEC_DIRS: [&str; 2] = ["docs/spec/rust", "docs/spec/language-agnostic"];

const R: &str = "docs/spec/rust";
const L: &str = "docs/spec/language-agnostic";

/// 全体の探しで、手がかりの語の終わりから数までの文字数の上限。
/// これより遠い数は、手がかりの語の数ではない(別の話の数)として扱わない。
const MAX_CLUE_DISTANCE: usize = 48;

// ---------------------------------------------------------------------------
// ファイルの読み取り
// ---------------------------------------------------------------------------

/// 読んだファイル。`\r` を除いてから行に分けたもの(BR3.5)。
struct Doc {
    rel: String,
    lines: Vec<String>,
}

fn repo_path(rel: &str) -> PathBuf {
    let mut path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    for part in rel.split('/') {
        path.push(part);
    }
    path
}

fn read_doc(rel: &str) -> Doc {
    let text = match std::fs::read_to_string(repo_path(rel)) {
        Ok(text) => text,
        Err(e) => panic!("{rel} を読めない: {e}"),
    };
    let text = text.replace('\r', "");
    Doc {
        rel: rel.to_string(),
        lines: text.split('\n').map(str::to_string).collect(),
    }
}

/// 突き合わせる仕様書のすべての `.md`(ファイル名の順)。
fn spec_docs() -> Vec<Doc> {
    let mut docs = Vec::new();
    for dir in SPEC_DIRS {
        let entries = match std::fs::read_dir(repo_path(dir)) {
            Ok(entries) => entries,
            Err(e) => panic!("{dir} を読めない: {e}"),
        };
        let mut names: Vec<String> = entries
            .map(|e| match e {
                Ok(e) => e.file_name().to_string_lossy().into_owned(),
                Err(e) => panic!("{dir} の中を読めない: {e}"),
            })
            .filter(|n| n.ends_with(".md"))
            .collect();
        names.sort();
        assert!(!names.is_empty(), "{dir} に仕様書が無い");
        for name in names {
            docs.push(read_doc(&format!("{dir}/{name}")));
        }
    }
    docs
}

fn doc_named<'a>(docs: &'a [Doc], rel: &str) -> &'a Doc {
    match docs.iter().find(|d| d.rel == rel) {
        Some(doc) => doc,
        None => panic!("{rel} が無い"),
    }
}

/// 失敗の文に入れる、行の一部(先頭の 40 文字)。
fn excerpt(line: &str) -> String {
    let trimmed = line.trim();
    let mut out: String = trimmed.chars().take(40).collect();
    if trimmed.chars().count() > 40 {
        out.push('…');
    }
    out
}

// ---------------------------------------------------------------------------
// 補助 1: 列挙の変種を数える
// ---------------------------------------------------------------------------

/// `header` を含む行から、`}` だけの行までの、列挙の変種の名前(宣言の順)。
/// `///`・`//`・`#[` の行と空行を除き、`Name,`・`Name {`・`Name(` の形を 1 つと数える。
fn enum_variants(rel: &str, header: &str) -> Vec<String> {
    let doc = read_doc(rel);
    let Some(start) = doc.lines.iter().position(|l| l.contains(header)) else {
        panic!("{rel} に `{header}` が無い(実装の値を取れない)");
    };
    let mut names = Vec::new();
    for line in &doc.lines[start + 1..] {
        let t = line.trim();
        if t == "}" {
            break;
        }
        if t.is_empty() || t.starts_with("//") || t.starts_with("#[") {
            continue;
        }
        let name: String = t
            .chars()
            .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
            .collect();
        assert!(
            name.chars().next().is_some_and(|c| c.is_ascii_uppercase()),
            "{rel} の `{header}` の中に変種でない行がある(「{}」)",
            excerpt(t)
        );
        names.push(name);
    }
    assert!(
        !names.is_empty(),
        "{rel} の `{header}` の変種が 0 個(実装の値を取れない)"
    );
    names
}

// ---------------------------------------------------------------------------
// 補助 2: 配列の `"…"` を数える
// ---------------------------------------------------------------------------

/// `header` を含む行の `=` の後ろから `];` までにある文字列(`"…"`)の数。
fn array_strings(rel: &str, header: &str) -> u32 {
    let doc = read_doc(rel);
    let Some(start) = doc.lines.iter().position(|l| l.contains(header)) else {
        panic!("{rel} に `{header}` が無い(実装の値を取れない)");
    };
    let joined = doc.lines[start..].join("\n");
    let Some(body_start) = joined.find("= ") else {
        panic!("{rel} の `{header}` に `=` が無い(実装の値を取れない)");
    };
    let body = &joined[body_start..];
    let Some(body_end) = body.find("];") else {
        panic!("{rel} の `{header}` に `];` が無い(実装の値を取れない)");
    };
    let quotes = body[..body_end].matches('"').count();
    assert!(
        quotes.is_multiple_of(2) && quotes > 0,
        "{rel} の `{header}` の文字列を数えられない(実装の値を取れない)"
    );
    (quotes / 2) as u32
}

// ---------------------------------------------------------------------------
// 補助 3: 見出しの下(または文の前)の表の本体の行を数える
// ---------------------------------------------------------------------------

/// `first` の行から始まる表の、本体の行の数(見出しの行と区切りの行を除き、表が切れるまで)。
fn table_body_rows(lines: &[String], first: usize) -> u32 {
    let rows = lines[first..]
        .iter()
        .take_while(|l| l.trim_start().starts_with('|'))
        .count();
    rows.saturating_sub(2) as u32
}

/// `idx` の行(見出し)より後ろで最初の表の、本体の行の数。表が無ければ `None`。
fn table_rows_after(lines: &[String], idx: usize) -> Option<u32> {
    let first = (idx + 1..lines.len()).find(|&i| lines[i].trim_start().starts_with('|'))?;
    Some(table_body_rows(lines, first))
}

/// `idx` の行(文)の直前にある表(間は空行だけ)の、本体の行の数。表が無ければ `None`。
fn table_rows_before(lines: &[String], idx: usize) -> Option<u32> {
    let mut last = idx;
    while last > 0 && lines[last - 1].trim().is_empty() {
        last -= 1;
    }
    let mut first = last;
    while first > 0 && lines[first - 1].trim_start().starts_with('|') {
        first -= 1;
    }
    (first < last).then(|| table_body_rows(lines, first))
}

// ---------------------------------------------------------------------------
// 補助 4: 「N 種」などの数を取り出す
// ---------------------------------------------------------------------------

/// 行の中の「N <unit>」(数と単位の間は空白 1 つ)の、数の始まりの位置(バイト)、数、
/// 単位の終わりの位置。数の直前が `.`・英数字・`_` のもの(`6.2 列`、`v0.9.0` など)は除く。
fn numbers_before_unit(line: &str, unit: &str) -> Vec<(usize, u32, usize)> {
    let pattern = format!(" {unit}");
    let mut out = Vec::new();
    for (i, _) in line.match_indices(&pattern) {
        let digits = line[..i]
            .chars()
            .rev()
            .take_while(char::is_ascii_digit)
            .count();
        if digits == 0 {
            continue;
        }
        let s = i - digits;
        let before = line[..s].chars().next_back();
        if before.is_some_and(|c| c == '.' || c == '_' || c.is_ascii_alphanumeric()) {
            continue;
        }
        if let Ok(n) = line[s..i].parse::<u32>() {
            out.push((s, n, i + pattern.len()));
        }
    }
    out
}

/// 「N <unit>」の N(最初のもの)。
fn first_number_before_unit(line: &str, unit: &str) -> Option<u32> {
    numbers_before_unit(line, unit).first().map(|&(_, n, _)| n)
}

/// `prefix` のすぐ後ろの数(例: `` `JOB_KEYS` `` の後ろの「5」)。数の後ろが `unit` で始まるものだけ。
fn numbers_after_prefix(line: &str, prefix: &str, unit: &str) -> Vec<u32> {
    let mut out = Vec::new();
    for (i, _) in line.match_indices(prefix) {
        let rest = &line[i + prefix.len()..];
        let digits = rest.chars().take_while(char::is_ascii_digit).count();
        if digits == 0 || !rest[digits..].starts_with(unit) {
            continue;
        }
        if let Ok(n) = rest[..digits].parse::<u32>() {
            out.push(n);
        }
    }
    out
}

/// `prefix` のすぐ後ろの数(最初のもの)。
fn first_number_after_prefix(line: &str, prefix: &str, unit: &str) -> Option<u32> {
    numbers_after_prefix(line, prefix, unit).first().copied()
}

/// 行の中の数の範囲(「A〜B」と「A to B」)の、始まりの位置(バイト)と両端(位置の順)。
fn ranges(line: &str) -> Vec<(usize, u32, u32)> {
    let mut out = Vec::new();
    for sep in ["〜", " to "] {
        for (i, _) in line.match_indices(sep) {
            let left = line[..i]
                .chars()
                .rev()
                .take_while(char::is_ascii_digit)
                .count();
            let rest = &line[i + sep.len()..];
            let right = rest.chars().take_while(char::is_ascii_digit).count();
            if left == 0 || right == 0 {
                continue;
            }
            let s = i - left;
            let before = line[..s].chars().next_back();
            if before.is_some_and(|c| c == '.' || c == '+' || c.is_ascii_alphanumeric()) {
                continue;
            }
            if let (Ok(a), Ok(b)) = (line[s..i].parse::<u32>(), rest[..right].parse::<u32>()) {
                out.push((s, a, b));
            }
        }
    }
    out.sort_unstable();
    out
}

// ---------------------------------------------------------------------------
// 補助 5: 版の注記の有無を判断する
// ---------------------------------------------------------------------------

/// 行に版の注記(履歴の数であることの印)があるか。
/// `v0.N.N まで`・`v0.N.x まで`(「までは」「までの」「までに」を含む)、`v0.N.N 以前`、`の時点`。
/// `v0.N.N 以降` は注記に入れない(今の数を書く行に付くため)。
fn has_version_note(line: &str) -> bool {
    if line.contains("の時点") {
        return true;
    }
    line.match_indices("v0.").any(|(i, _)| {
        let rest = &line[i + 3..];
        let minor = rest.chars().take_while(char::is_ascii_digit).count();
        if minor == 0 || !rest[minor..].starts_with('.') {
            return false;
        }
        let rest = &rest[minor + 1..];
        let patch = rest
            .chars()
            .take_while(|c| c.is_ascii_digit() || *c == 'x')
            .count();
        patch > 0 && (rest[patch..].starts_with(" まで") || rest[patch..].starts_with(" 以前"))
    })
}

// ---------------------------------------------------------------------------
// 決まった場所と、全体の探しの共通の部品
// ---------------------------------------------------------------------------

/// 1 つの数の突き合わせの、失敗の集め役。失敗の文には、数の名前、ファイルと行番号、
/// 行の一部、仕様書の値と実装の値を入れる。
struct Checker {
    name: &'static str,
    failures: Vec<String>,
    hits: usize,
}

impl Checker {
    fn new(name: &'static str) -> Self {
        Self {
            name,
            failures: Vec::new(),
            hits: 0,
        }
    }

    fn mismatch(&mut self, doc: &Doc, idx: usize, spec: &str, implementation: &str) {
        self.failures.push(format!(
            "{}: {}:{}: 仕様書は {spec}、実装は {implementation}(「{}」)",
            self.name,
            doc.rel,
            idx + 1,
            excerpt(&doc.lines[idx])
        ));
    }

    fn fail(&mut self, message: String) {
        self.failures.push(format!("{}: {message}", self.name));
    }

    /// `doc` の中で `pred` に合う行の番号(0 始まり)。1 つも無ければ失敗にする(BR3.3)。
    fn anchors(&mut self, doc: &Doc, what: &str, pred: impl Fn(&str) -> bool) -> Vec<usize> {
        let found: Vec<usize> = (0..doc.lines.len())
            .filter(|&i| pred(&doc.lines[i]))
            .collect();
        if found.is_empty() {
            self.fail(format!(
                "{}: 決まった場所(「{what}」)が見つからない",
                doc.rel
            ));
        }
        found
    }

    /// 決まった場所の 1 つの数を突き合わせる。数が取れなければ失敗にする(BR3.3)。
    fn value(&mut self, doc: &Doc, idx: usize, spec: Option<u32>, expected: u32) {
        match spec {
            Some(v) if v == expected => {}
            Some(v) => self.mismatch(doc, idx, &v.to_string(), &expected.to_string()),
            None => self.mismatch(doc, idx, "(数が取れない)", &expected.to_string()),
        }
    }

    /// 全体の探しで当たった 1 件。違えば失敗にする(BR3.4)。
    fn hit(&mut self, doc: &Doc, idx: usize, spec: String, implementation: String) {
        self.hits += 1;
        if spec != implementation {
            self.mismatch(doc, idx, &spec, &implementation);
        }
    }

    /// 全体の探しが 1 件も当たらなければ失敗にし(BR3.4)、失敗を 1 行に 1 件ずつ並べて落とす。
    fn finish_with_search(mut self) {
        if self.hits == 0 {
            self.fail("全体の探しが 1 件も当たらない(手がかりの語が仕様書に無い)".to_string());
        }
        self.finish();
    }

    /// 失敗を 1 行に 1 件ずつ並べて落とす。
    fn finish(self) {
        assert!(self.failures.is_empty(), "{}", self.failures.join("\n"));
    }
}

/// 手がかりの語と、その語が指す数の番号。
struct Clue {
    word: &'static str,
    kind: usize,
}

/// 行の中で、手がかりの語に結び付く「N 種」の数(数の番号と数の組)。
/// - 「N 種の<語>」のように、数が語のすぐ前にあれば、その数を語の数とする(README の形)。
/// - そうでなければ、語より後ろで最も近い「N 種」を語の数とする。ただし、間に別の数の手がかりの
///   語があるとき、または `MAX_CLUE_DISTANCE` 文字より遠いときは、結び付けない。
fn bound_numbers(line: &str, clues: &[Clue], unit: &str) -> Vec<(usize, u32)> {
    let mut found: Vec<(usize, usize, usize)> = Vec::new();
    for clue in clues {
        for (i, _) in line.match_indices(clue.word) {
            found.push((i, i + clue.word.len(), clue.kind));
        }
    }
    found.sort_unstable();
    let numbers = numbers_before_unit(line, unit);
    let mut out = Vec::new();
    for &(start, end, kind) in &found {
        let before = numbers.iter().find(|&&(_, _, unit_end)| {
            line[unit_end..].starts_with("の") && unit_end + "の".len() == start
        });
        if let Some(&(_, n, _)) = before {
            out.push((kind, n));
            continue;
        }
        let Some(&(pos, n, _)) = numbers.iter().find(|&&(pos, _, _)| pos >= end) else {
            continue;
        };
        let blocked = found
            .iter()
            .any(|&(s, _, k)| k != kind && s >= end && s < pos);
        let far = line[end..pos].chars().count() > MAX_CLUE_DISTANCE;
        if !blocked && !far {
            out.push((kind, n));
        }
    }
    out
}

// ---------------------------------------------------------------------------
// 実装の値
// ---------------------------------------------------------------------------

const WALK_SRC: &str = "crates/core/src/definition/walk/mod.rs";
const STATE_READER_SRC: &str = "crates/runtime/src/state/reader.rs";

/// `crates/runtime/src/error.rs` の `mod reason` の `pub const` のうち、直前の `///` の行に
/// 「試行」を含むものの名前。
fn attempt_reasons() -> Vec<String> {
    let rel = "crates/runtime/src/error.rs";
    let doc = read_doc(rel);
    let Some(start) = doc.lines.iter().position(|l| l.contains("mod reason {")) else {
        panic!("{rel} に `mod reason` が無い(実装の値を取れない)");
    };
    let mut names = Vec::new();
    for i in start + 1..doc.lines.len() {
        let t = doc.lines[i].trim();
        if t == "}" {
            break;
        }
        let Some(rest) = t.strip_prefix("pub const ") else {
            continue;
        };
        let prev = doc.lines[i - 1].trim();
        if prev.starts_with("///") && prev.contains("試行") {
            let name: String = rest
                .chars()
                .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
                .collect();
            names.push(name);
        }
    }
    assert!(
        !names.is_empty(),
        "{rel} の試行の理由が 0 個(実装の値を取れない)"
    );
    names
}

/// `crates/cli/src/report/messages.rs` の `pub const HEADER_` の数(`watch` の表の列)。
fn watch_columns() -> u32 {
    let rel = "crates/cli/src/report/messages.rs";
    let n = read_doc(rel)
        .lines
        .iter()
        .filter(|l| l.starts_with("pub const HEADER_"))
        .count();
    assert!(n > 0, "{rel} に `HEADER_` が無い(実装の値を取れない)");
    n as u32
}

/// `crates/cli/src/types.rs` の `const <name>: u32 = N;` の N。
fn cli_const(name: &str) -> u32 {
    let rel = "crates/cli/src/types.rs";
    let doc = read_doc(rel);
    let pattern = format!("const {name}:");
    let Some(line) = doc.lines.iter().find(|l| l.contains(&pattern)) else {
        panic!("{rel} に `{name}` が無い(実装の値を取れない)");
    };
    let value = line
        .split('=')
        .nth(1)
        .map(|v| v.trim().trim_end_matches(';').trim());
    let Some(Ok(n)) = value.map(str::parse::<u32>) else {
        panic!("{rel} の `{name}` の値を読めない(実装の値を取れない)");
    };
    assert!(n > 0, "{rel} の `{name}` が 0(実装の値を取れない)");
    n
}

/// デモの場面の数。`demo/run-demo.sh` の `scene "` の行と、Windows 版の `call :scene` の行が同じ数。
fn demo_scenes() -> u32 {
    let count = |rel: &str, prefix: &str| {
        read_doc(rel)
            .lines
            .iter()
            .filter(|l| l.trim_start().starts_with(prefix))
            .count()
    };
    let unix = count("demo/run-demo.sh", "scene \"");
    let windows = count("demo/windows/run-demo.cmd", "call :scene ");
    assert!(
        unix > 0,
        "demo/run-demo.sh の場面が 0 個(実装の値を取れない)"
    );
    assert_eq!(
        unix, windows,
        "デモの場面の数が UNIX 版({unix})と Windows 版({windows})で違う"
    );
    unix as u32
}

// ---------------------------------------------------------------------------
// 定義の誤り(#1)と `.env` の誤り(#2)の全体の探し
// ---------------------------------------------------------------------------

/// 全体の探しの手がかり。`kind` の 0 が定義の誤り、1 が `.env` の誤り。
const ERROR_KIND_CLUES: [Clue; 5] = [
    Clue {
        word: "DefinitionErrorKind",
        kind: 0,
    },
    Clue {
        word: "定義の誤り",
        kind: 0,
    },
    Clue {
        word: "エラーの種類",
        kind: 0,
    },
    Clue {
        word: "`.env` の誤り",
        kind: 1,
    },
    Clue {
        word: "EnvFileErrorKind",
        kind: 1,
    },
];

/// #1 と #2 の、決まった場所と全体の探し。
/// `heading` は 03 章の見出し(N 種と表)、`count_line` は 08 章の表の後ろの「(N 種。」の文。
fn check_error_kinds(c: &mut Checker, kind: usize, expected: u32, heading: &str, count_line: &str) {
    let docs = spec_docs();
    for dir in [R, L] {
        let doc = doc_named(&docs, &format!("{dir}/03-definition-format.md"));
        for idx in c.anchors(doc, heading, |l| l.starts_with('#') && l.contains(heading)) {
            let n = first_number_before_unit(&doc.lines[idx], "種");
            c.value(doc, idx, n, expected);
            c.value(doc, idx, table_rows_after(&doc.lines, idx), expected);
        }
        let doc = doc_named(&docs, &format!("{dir}/08-output-and-i18n.md"));
        for idx in c.anchors(doc, count_line, |l| l.contains(count_line)) {
            let n = first_number_before_unit(&doc.lines[idx], "種");
            c.value(doc, idx, n, expected);
        }
    }
    for doc in &docs {
        for (idx, line) in doc.lines.iter().enumerate() {
            if has_version_note(line) {
                continue;
            }
            for (k, n) in bound_numbers(line, &ERROR_KIND_CLUES, "種") {
                if k == kind {
                    c.hit(doc, idx, format!("{n} 種"), format!("{expected} 種"));
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// 数ごとのテスト
// ---------------------------------------------------------------------------

#[test]
fn 定義の誤りの種類の数は_仕様書と実装で一致する() {
    let mut c = Checker::new("定義の誤りの種類");
    let expected =
        enum_variants("crates/core/src/error.rs", "pub enum DefinitionErrorKind {").len() as u32;
    check_error_kinds(
        &mut c,
        0,
        expected,
        "種類の一覧(",
        "種。理由の文は 03 の各節",
    );
    c.finish_with_search();
}

#[test]
fn env_の誤りの種類の数は_仕様書と実装で一致する() {
    let mut c = Checker::new("`.env` の誤りの種類");
    let header = "pub enum EnvFileErrorKind {";
    let expected = enum_variants("crates/core/src/envfile/mod.rs", header).len() as u32;
    check_error_kinds(
        &mut c,
        1,
        expected,
        "`.env` の誤りの種類(",
        "種。理由の中の",
    );
    c.finish_with_search();
}

/// #3 と #4: 定義のキーの「上の N つ」と、その直前の表の行。
fn check_definition_keys(c: &mut Checker, expected: u32, fragments: [&str; 2]) {
    let docs = spec_docs();
    for (dir, fragment) in [(R, fragments[0]), (L, fragments[1])] {
        let doc = doc_named(&docs, &format!("{dir}/03-definition-format.md"));
        let pred = |l: &str| l.contains(fragment) && l.contains("上の ");
        for idx in c.anchors(doc, fragment, pred) {
            let n = first_number_after_prefix(&doc.lines[idx], "上の ", " つ");
            c.value(doc, idx, n, expected);
            c.value(doc, idx, table_rows_before(&doc.lines, idx), expected);
        }
    }
}

#[test]
fn パイプラインのキーの数は_仕様書と実装で一致する() {
    let mut c = Checker::new("`[pipeline]` のキー");
    let expected = array_strings(WALK_SRC, "const PIPELINE_KEYS");
    check_definition_keys(
        &mut c,
        expected,
        ["(`PIPELINE_KEYS`)", "それ以外は `unknownKey`(例: `owner`"],
    );
    c.finish();
}

#[test]
fn ジョブのキーの数は_仕様書と実装で一致する() {
    let mut c = Checker::new("`[[job]]` のキー");
    let expected = array_strings(WALK_SRC, "const JOB_KEYS");
    check_definition_keys(
        &mut c,
        expected,
        [
            "(`JOB_KEYS`)。それ以外は",
            "それ以外は `unknownKey`(例: `warn_exit_codes`",
        ],
    );
    c.finish();
}

#[test]
fn ステートの最上位のキーの数は_仕様書と実装で一致する() {
    let mut c = Checker::new("ステートの最上位のキー");
    let expected = array_strings(STATE_READER_SRC, "const TOP_KEYS");
    let docs = spec_docs();
    for (dir, heading) in [
        (R, "### 2.2 最上位のキー"),
        (L, "### 2.1 ステート全体の項目"),
    ] {
        let doc = doc_named(&docs, &format!("{dir}/06-state-file.md"));
        for idx in c.anchors(doc, heading, |l| l.trim() == heading) {
            c.value(doc, idx, table_rows_after(&doc.lines, idx), expected);
        }
    }
    c.finish();
}

/// #6 と #7: ステートのキーの「上の N つ」(06 章)と「N 個」(12 章)、全体の探し。
/// `fragments` は 06 章の文の一部、`prefixes` は 12 章の「N 個」の前の語(rust、language-agnostic の順)。
fn check_state_keys(c: &mut Checker, expected: u32, fragments: [&str; 2], prefixes: [&str; 2]) {
    let docs = spec_docs();
    for (dir, fragment, prefix) in [
        (R, fragments[0], prefixes[0]),
        (L, fragments[1], prefixes[1]),
    ] {
        let doc = doc_named(&docs, &format!("{dir}/06-state-file.md"));
        for idx in c.anchors(doc, fragment, |l| l.contains(fragment)) {
            let n = first_number_after_prefix(&doc.lines[idx], "上の ", " つ");
            c.value(doc, idx, n, expected);
        }
        let doc = doc_named(&docs, &format!("{dir}/12-rebuild-guide.md"));
        let what = format!("{prefix}N 個");
        let pred = |l: &str| first_number_after_prefix(l, prefix, " 個").is_some();
        for idx in c.anchors(doc, &what, pred) {
            let n = first_number_after_prefix(&doc.lines[idx], prefix, " 個");
            c.value(doc, idx, n, expected);
        }
    }
    for doc in &docs {
        for (idx, line) in doc.lines.iter().enumerate() {
            if has_version_note(line) {
                continue;
            }
            for prefix in prefixes {
                for n in numbers_after_prefix(line, prefix, " 個") {
                    c.hit(doc, idx, format!("{n} 個"), format!("{expected} 個"));
                }
            }
        }
    }
}

#[test]
fn ステートのジョブのキーの数は_仕様書と実装で一致する() {
    let mut c = Checker::new("ステートのジョブのキー");
    let expected = array_strings(STATE_READER_SRC, "const JOB_KEYS");
    check_state_keys(
        &mut c,
        expected,
        ["(`JOB_KEYS`)だけ", "ジョブ状態の項目は、上の "],
        ["`JOB_KEYS` ", "ジョブの項目 "],
    );
    c.finish_with_search();
}

#[test]
fn 試行のキーの数は_仕様書と実装で一致する() {
    let mut c = Checker::new("試行のキー");
    let expected = array_strings(STATE_READER_SRC, "const ATTEMPT_KEYS");
    check_state_keys(
        &mut c,
        expected,
        ["(`ATTEMPT_KEYS`)だけ", "- 項目は上の "],
        ["`ATTEMPT_KEYS` ", "試行の項目 "],
    );
    c.finish_with_search();
}

#[test]
fn 試行の誤りの理由の数と名前は_仕様書と実装で一致する() {
    let mut c = Checker::new("試行の誤りの理由");
    let reasons = attempt_reasons();
    let expected = reasons.len() as u32;
    let quoted: Vec<String> = reasons.iter().map(|r| format!("`{r}`")).collect();
    let docs = spec_docs();
    let r10 = doc_named(&docs, &format!("{R}/10-errors-and-exit-codes.md"));
    let rows = r10
        .lines
        .iter()
        .filter(|l| l.starts_with('|') && quoted.iter().any(|q| l.contains(q.as_str())))
        .count();
    if rows as u32 != expected {
        c.fail(format!(
            "{}: 状態の文の表で理由の名前を含む行が {rows} 行、実装は {expected} 個({})",
            r10.rel,
            reasons.join("・")
        ));
    }
    let r12 = doc_named(&docs, &format!("{R}/12-rebuild-guide.md"));
    for doc in [r10, r12] {
        for q in &quoted {
            if !doc.lines.iter().any(|l| l.contains(q.as_str())) {
                c.fail(format!("{}: 実装の理由の名前 {q} が仕様書に無い", doc.rel));
            }
        }
    }
    let l12 = doc_named(&docs, &format!("{L}/12-rebuild-guide.md"));
    let pred = |l: &str| first_number_after_prefix(l, "試行の ", " つの理由").is_some();
    for idx in c.anchors(l12, "試行の N つの理由", pred) {
        let n = first_number_after_prefix(&l12.lines[idx], "試行の ", " つの理由");
        c.value(l12, idx, n, expected);
    }
    c.finish();
}

#[test]
fn watch_の表の列の数は_仕様書と実装で一致する() {
    let mut c = Checker::new("`watch` の表の列");
    let expected = watch_columns();
    let docs = spec_docs();
    let r08 = doc_named(&docs, &format!("{R}/08-output-and-i18n.md"));
    let pred = |l: &str| l.starts_with("JOB ") && l.contains("STATUS");
    for idx in c.anchors(r08, "JOB で始まる表の見出しの行", pred) {
        let n = r08.lines[idx].split_whitespace().count() as u32;
        c.value(r08, idx, Some(n), expected);
    }
    let l00 = doc_named(&docs, &format!("{L}/00-README.md"));
    let pred = |l: &str| l.contains("`watch` の表(");
    for idx in c.anchors(l00, "`watch` の表(N 列)", pred) {
        let n = first_number_before_unit(&l00.lines[idx], "列");
        c.value(l00, idx, n, expected);
    }
    for doc in &docs {
        for (idx, line) in doc.lines.iter().enumerate() {
            if has_version_note(line) || !line.contains("watch") {
                continue;
            }
            for (_, n, _) in numbers_before_unit(line, "列") {
                c.hit(doc, idx, format!("{n} 列"), format!("{expected} 列"));
            }
        }
    }
    c.finish_with_search();
}

/// `--interval` の範囲の全体の探しの手がかり(手がかりより後ろで最も近い範囲を見る)。
const INTERVAL_RANGE_CLUES: [&str; 3] = ["--interval", "seconds between redraws", "表示し直す間隔"];

/// `--interval` の既定の全体の探しの形(行に `--interval` か `--follow` があるときだけ見る)。
const INTERVAL_DEFAULT_PREFIXES: [&str; 3] = ["既定 ", "default: ", "省略時: "];

#[test]
fn interval_の範囲と既定は_仕様書と実装で一致する() {
    let mut c = Checker::new("`--interval` の範囲と既定");
    let min = cli_const("MIN_FOLLOW_INTERVAL_SECS");
    let max = cli_const("MAX_FOLLOW_INTERVAL_SECS");
    let default = cli_const("DEFAULT_FOLLOW_INTERVAL_SECS");
    let range = format!("{min}〜{max}");
    let docs = spec_docs();
    for dir in [R, L] {
        let doc = doc_named(&docs, &format!("{dir}/04-cli.md"));
        let pred = |l: &str| l.starts_with("| `--interval <");
        for idx in c.anchors(doc, "| `--interval <秒>` の行", pred) {
            let line = &doc.lines[idx];
            match ranges(line).first() {
                Some(&(_, a, b)) if (a, b) == (min, max) => {}
                Some(&(_, a, b)) => c.mismatch(doc, idx, &format!("{a}〜{b}"), &range),
                None => c.mismatch(doc, idx, "(範囲が取れない)", &range),
            }
            let last_cell = line.trim_end().trim_end_matches('|').rsplit('|').next();
            let n = last_cell.and_then(|cell| first_number_before_unit(cell, "秒"));
            c.value(doc, idx, n, default);
        }
    }
    for doc in &docs {
        for (idx, line) in doc.lines.iter().enumerate() {
            if has_version_note(line) {
                continue;
            }
            let found = ranges(line);
            for clue in INTERVAL_RANGE_CLUES {
                for (i, _) in line.match_indices(clue) {
                    let end = i + clue.len();
                    let Some(&(pos, a, b)) = found.iter().find(|&&(pos, _, _)| pos >= end) else {
                        continue;
                    };
                    if line[end..pos].chars().count() <= MAX_CLUE_DISTANCE {
                        c.hit(doc, idx, format!("{a}〜{b}"), range.clone());
                    }
                }
            }
            if !line.contains("--interval") && !line.contains("--follow") {
                continue;
            }
            for prefix in INTERVAL_DEFAULT_PREFIXES {
                for n in numbers_after_prefix(line, prefix, "") {
                    c.hit(doc, idx, format!("既定 {n}"), format!("既定 {default}"));
                }
            }
        }
    }
    c.finish_with_search();
}

#[test]
fn format_の値は_仕様書と実装で一致する() {
    let mut c = Checker::new("`--format` の値");
    let expected: BTreeSet<String> =
        enum_variants("crates/cli/src/types.rs", "pub enum GraphFormat {")
            .iter()
            .map(|v| v.to_lowercase())
            .collect();
    let show = |s: &BTreeSet<String>| s.iter().cloned().collect::<Vec<_>>().join("・");
    let docs = spec_docs();
    for dir in [R, L] {
        let doc = doc_named(&docs, &format!("{dir}/04-cli.md"));
        let pred = |l: &str| l.starts_with("| `--format`(");
        for idx in c.anchors(doc, "| `--format`( の行", pred) {
            let cell = doc.lines[idx].split('|').nth(3).unwrap_or("");
            let spec: BTreeSet<String> = cell
                .split('`')
                .skip(1)
                .step_by(2)
                .map(str::to_string)
                .collect();
            if spec != expected {
                c.mismatch(doc, idx, &show(&spec), &show(&expected));
            }
        }
    }
    c.finish();
}

#[test]
fn デモの場面の数は_仕様書と実装で一致する() {
    let mut c = Checker::new("デモの場面");
    let count = demo_scenes();
    let last = count - 1;
    let docs = spec_docs();
    for dir in [R, L] {
        let doc = doc_named(&docs, &format!("{dir}/00-README.md"));
        let pred = |l: &str| first_number_after_prefix(l, "場面 0〜", "").is_some();
        for idx in c.anchors(doc, "場面 0〜N", pred) {
            let n = first_number_after_prefix(&doc.lines[idx], "場面 0〜", "");
            c.value(doc, idx, n, last);
        }
    }
    let r11 = doc_named(&docs, &format!("{R}/11-quality-and-build.md"));
    for idx in c.anchors(r11, "### 9.4 場面", |l| l.trim() == "### 9.4 場面") {
        c.value(r11, idx, table_rows_after(&r11.lines, idx), count);
    }
    let l11 = doc_named(&docs, &format!("{L}/11-quality-and-build.md"));
    for idx in c.anchors(l11, "| AC-424 |", |l| l.starts_with("| AC-424 |")) {
        let n = numbers_after_prefix(&l11.lines[idx], "`=== ", ". `");
        c.value(l11, idx, n.into_iter().max(), last);
    }
    for doc in &docs {
        for (idx, line) in doc.lines.iter().enumerate() {
            if has_version_note(line) {
                continue;
            }
            for n in numbers_after_prefix(line, "場面 0〜", "") {
                c.hit(doc, idx, format!("場面 0〜{n}"), format!("場面 0〜{last}"));
            }
        }
    }
    c.finish_with_search();
}
