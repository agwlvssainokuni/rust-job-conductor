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

//! `.env` の解析(WF2。BR1.3〜BR1.5、BR2.1〜BR2.8、BR3.2)。純粋な関数。

use super::{
    EnvEntry, EnvFileContent, EnvFileError, EnvFileErrorKind as Kind, EnvWarning, EnvWarningKind,
    MAX_ENV_ENTRIES, is_reserved_env_name,
};
use crate::run_id::is_valid_env_name;

/// `.env` の内容(UTF-8 の文字列)を解析する。
///
/// `file` は、誤りと警告に書く表示用のファイル名(解決済みのパス)。行ごとの誤りは止めずに集め、
/// 1 つでもあれば、項目は返さず誤りの並びを返す。誤りに値は含めない。
pub fn parse_env_file(text: &str, file: &str) -> Result<EnvFileContent, Vec<EnvFileError>> {
    let text = text.strip_prefix('\u{FEFF}').unwrap_or(text);
    let mut entries: Vec<EnvEntry> = Vec::new();
    let mut warnings: Vec<EnvWarning> = Vec::new();
    let mut errors: Vec<EnvFileError> = Vec::new();
    let mut taken = 0usize;
    // 小文字にした名前 → 綴り(大文字小文字だけが違う名前と、完全な重複を見分ける)。
    let mut seen: std::collections::HashMap<String, String> = std::collections::HashMap::new();

    for (i, raw) in text.split('\n').enumerate() {
        let line_no = i + 1;
        let line = raw.strip_suffix('\r').unwrap_or(raw);
        let trimmed = line.trim_start();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let err = |kind: Kind| EnvFileError::new(file, Some(line_no), kind);

        let body = strip_export(trimmed);
        let Some((key, rest)) = body.split_once('=') else {
            errors.push(err(Kind::MissingEquals));
            continue;
        };
        let key = key.trim();
        if !is_valid_env_name(key) {
            errors.push(err(Kind::InvalidName));
            continue;
        }
        if is_reserved_env_name(key) {
            errors.push(err(Kind::ReservedName));
            continue;
        }
        let value = match parse_value(rest) {
            Ok(v) => v,
            Err(kind) => {
                errors.push(err(kind));
                continue;
            }
        };
        if value.contains('\0') {
            errors.push(err(Kind::NulInValue));
            continue;
        }

        taken += 1;
        if taken > MAX_ENV_ENTRIES {
            errors.push(err(Kind::TooManyEntries));
            break;
        }

        match seen.get(&key.to_ascii_lowercase()) {
            Some(exact) if exact == key => {
                if let Some(pos) = entries.iter().position(|e| e.name == key) {
                    entries.remove(pos);
                }
                warnings.push(EnvWarning {
                    file: file.to_string(),
                    line: line_no,
                    name: key.to_string(),
                    kind: EnvWarningKind::DuplicateKey,
                });
            }
            Some(_) => {
                errors.push(err(Kind::CaseConflict));
                continue;
            }
            None => {
                seen.insert(key.to_ascii_lowercase(), key.to_string());
            }
        }
        entries.push(EnvEntry {
            name: key.to_string(),
            value,
            line: line_no,
        });
    }

    if errors.is_empty() {
        Ok(EnvFileContent {
            path: file.to_string(),
            entries,
            warnings,
        })
    } else {
        Err(errors)
    }
}

/// 先頭の `export `(空白 1 つ以上)を取り除く(BR2.1)。
fn strip_export(line: &str) -> &str {
    match line.strip_prefix("export") {
        Some(rest) if rest.starts_with([' ', '\t']) => rest.trim_start(),
        _ => line,
    }
}

/// `=` の右側から値を取り出す(BR2.2〜BR2.5)。
fn parse_value(rest: &str) -> Result<String, Kind> {
    let value = rest.trim_start();
    match value.chars().next() {
        Some('\'') => parse_single(&value[1..]),
        Some('"') => parse_double(&value[1..]),
        _ => Ok(parse_plain(rest)),
    }
}

/// 引用符なし: 空白に続く `#` の手前まで。前後の空白を除く(BR2.2)。
fn parse_plain(rest: &str) -> String {
    let mut prev_space = false;
    for (idx, c) in rest.char_indices() {
        if c == '#' && prev_space {
            return rest[..idx].trim().to_string();
        }
        prev_space = c.is_whitespace();
    }
    rest.trim().to_string()
}

/// 単一引用符: 次の `'` まで、そのまま(BR2.3)。
fn parse_single(after_open: &str) -> Result<String, Kind> {
    let Some(end) = after_open.find('\'') else {
        return Err(Kind::UnterminatedQuote);
    };
    check_tail(&after_open[end + 1..])?;
    Ok(after_open[..end].to_string())
}

/// 二重引用符: 5 種のエスケープだけを解く(BR2.4)。
fn parse_double(after_open: &str) -> Result<String, Kind> {
    let mut out = String::new();
    let mut chars = after_open.char_indices();
    while let Some((idx, c)) = chars.next() {
        match c {
            '"' => {
                check_tail(&after_open[idx + 1..])?;
                return Ok(out);
            }
            '\\' => match chars.next() {
                Some((_, 'n')) => out.push('\n'),
                Some((_, 'r')) => out.push('\r'),
                Some((_, 't')) => out.push('\t'),
                Some((_, '"')) => out.push('"'),
                Some((_, '\\')) => out.push('\\'),
                Some(_) => return Err(Kind::InvalidEscape),
                None => return Err(Kind::UnterminatedQuote),
            },
            c => out.push(c),
        }
    }
    Err(Kind::UnterminatedQuote)
}

/// 閉じ引用符のあとは、空白とコメントだけ(BR2.3、BR2.4)。
fn check_tail(tail: &str) -> Result<(), Kind> {
    let t = tail.trim_start();
    if t.is_empty() || t.starts_with('#') {
        Ok(())
    } else {
        Err(Kind::TrailingGarbage)
    }
}
