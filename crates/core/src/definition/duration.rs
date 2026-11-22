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

//! リトライの間隔の文字列(`<整数><s|m|h>`)を秒に直す(BR1.10、FR1.4)。

/// 間隔の最小(1 秒)。
pub(super) const MIN_SECS: u64 = 1;
/// 間隔の最大(24 時間)。
pub(super) const MAX_SECS: u64 = 24 * 60 * 60;

/// 間隔の文字列を秒に直す。
///
/// 書式は、半角の数字だけの整数と、単位 `s`(秒)、`m`(分)、`h`(時間)のいずれか 1 文字。
/// 整数は 1 以上で、結果は 1 秒以上 24 時間以下。書式や範囲が合わなければ `None`(BR1.10)。
pub(crate) fn parse_duration_secs(text: &str) -> Option<u64> {
    let unit = text.chars().last()?;
    let factor = match unit {
        's' => 1,
        'm' => 60,
        'h' => 3600,
        _ => return None,
    };
    let digits = &text[..text.len() - unit.len_utf8()];
    if digits.is_empty() || !digits.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    let n: u64 = digits.parse().ok()?;
    let secs = n.checked_mul(factor)?;
    (n >= 1 && (MIN_SECS..=MAX_SECS).contains(&secs)).then_some(secs)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn 単位ごとに秒へ直す() {
        assert_eq!(parse_duration_secs("1s"), Some(1));
        assert_eq!(parse_duration_secs("30s"), Some(30));
        assert_eq!(parse_duration_secs("5m"), Some(300));
        assert_eq!(parse_duration_secs("2h"), Some(7200));
    }

    #[test]
    fn 範囲の境界() {
        assert_eq!(parse_duration_secs("24h"), Some(86_400));
        assert_eq!(parse_duration_secs("1440m"), Some(86_400));
        assert_eq!(parse_duration_secs("86400s"), Some(86_400));
        assert_eq!(parse_duration_secs("86401s"), None);
        assert_eq!(parse_duration_secs("25h"), None);
        assert_eq!(parse_duration_secs("0s"), None);
        assert_eq!(parse_duration_secs("0h"), None);
    }

    #[test]
    fn 書式が違えば_none() {
        for bad in [
            "", "s", "10", "1.5s", "-1s", "+1s", " 1s", "1s ", "1 s", "1d", "1S", "１s", "1ms",
            "0x10s", "1_0s",
        ] {
            assert_eq!(parse_duration_secs(bad), None, "{bad:?}");
        }
    }

    #[test]
    fn 桁あふれは_none() {
        assert_eq!(parse_duration_secs("99999999999999999999999s"), None);
        assert_eq!(parse_duration_secs("18446744073709551615h"), None);
    }
}
