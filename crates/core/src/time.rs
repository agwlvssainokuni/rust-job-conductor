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

//! 時刻の整形・読み取り・差(CoreTime。v0.10.0 の契約 C2、BR2.1、BR2.2、FR6.6)。
//!
//! 時刻は UTC の UNIX 時刻の秒([`Timestamp`])で持つ。範囲は 1970-01-01T00:00:00Z から
//! 9999-12-31T23:59:59Z まで。日数と年月日の変換(グレゴリオ暦。うるう年を含み、うるう秒は扱わない)は
//! 自前で書き、新しい依存を使わない。どの関数も純粋で、時計にも環境にも触れない
//! (現在の時刻は runtime の時計が取る。ADR-002)。
//!
//! | 関数 | 形 | 例 |
//! |------|----|----|
//! | [`format_rfc3339`] | `YYYY-MM-DDTHH:MM:SSZ`(ステート用) | `2026-11-25T09:12:48Z` |
//! | [`format_compact`] | `YYYYMMDDTHHMMSSZ`(ログのファイル名用) | `20261125T091248Z` |
//! | [`parse_rfc3339`] | [`format_rfc3339`] の形だけを読む | 小数の秒・時差・小文字は受け付けない |
//! | [`duration_secs`] | 2 つの時刻の差(秒。負にならない) | |

/// 1 日の秒数。
const SECS_PER_DAY: u64 = 86_400;

/// 扱う年の下限と上限(BR2.1)。
const MIN_YEAR: u64 = 1970;
const MAX_YEAR: u64 = 9999;

/// 1970-01-01 から、暦の計算の起点(0000-03-01)までの日数の差。
const DAYS_FROM_ERA_START_TO_EPOCH: u64 = 719_468;

/// 400 年(1 つの周期)の日数。
const DAYS_PER_ERA: u64 = 146_097;

/// UTC の時刻(UNIX 時刻の秒。契約 C2、`entities.md` の Timestamp)。
///
/// 値は 0(1970-01-01T00:00:00Z)から [`Timestamp::MAX`](9999-12-31T23:59:59Z)まで。
/// 範囲の外の値は作れない。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Timestamp(u64);

impl Timestamp {
    /// 範囲の始まり(1970-01-01T00:00:00Z)。
    pub const EPOCH: Timestamp = Timestamp(0);

    /// 範囲の終わり(9999-12-31T23:59:59Z)。
    pub const MAX: Timestamp = Timestamp(253_402_300_799);

    /// UNIX 時刻の秒から作る。[`Timestamp::MAX`] を超えれば `None`。
    pub fn from_unix_seconds(secs: u64) -> Option<Timestamp> {
        (secs <= Self::MAX.0).then_some(Timestamp(secs))
    }

    /// UNIX 時刻の秒。
    pub fn unix_seconds(self) -> u64 {
        self.0
    }
}

/// 年月日と時分秒(内部の計算用)。
struct Civil {
    year: u64,
    month: u64,
    day: u64,
    hour: u64,
    minute: u64,
    second: u64,
}

impl Civil {
    fn of(ts: Timestamp) -> Civil {
        let days = ts.0 / SECS_PER_DAY;
        let rest = ts.0 % SECS_PER_DAY;
        let (year, month, day) = civil_from_days(days);
        Civil {
            year,
            month,
            day,
            hour: rest / 3600,
            minute: rest % 3600 / 60,
            second: rest % 60,
        }
    }
}

/// 1970-01-01 からの日数を、年月日にする(グレゴリオ暦。日数は 0 以上)。
///
/// 3 月始まりの年で 400 年の周期に分けて求める、よく知られた方法。
fn civil_from_days(days: u64) -> (u64, u64, u64) {
    let z = days + DAYS_FROM_ERA_START_TO_EPOCH;
    let era = z / DAYS_PER_ERA;
    let doe = z % DAYS_PER_ERA; // 周期の中の日(0〜146096)
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365; // 周期の中の年(0〜399)
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100); // 3 月 1 日からの日(0〜365)
    let mp = (5 * doy + 2) / 153; // 3 月を 0 とする月(0〜11)
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = yoe + era * 400 + u64::from(month <= 2);
    (year, month, day)
}

/// 年月日を、1970-01-01 からの日数にする([`civil_from_days`] の逆。年は 1970 以上)。
fn days_from_civil(year: u64, month: u64, day: u64) -> u64 {
    let y = if month <= 2 { year - 1 } else { year };
    let era = y / 400;
    let yoe = y % 400;
    let mp = if month > 2 { month - 3 } else { month + 9 };
    let doy = (153 * mp + 2) / 5 + day - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * DAYS_PER_ERA + doe - DAYS_FROM_ERA_START_TO_EPOCH
}

/// うるう年か(4 で割り切れ、100 で割り切れないか、400 で割り切れる)。
fn is_leap_year(year: u64) -> bool {
    (year.is_multiple_of(4) && !year.is_multiple_of(100)) || year.is_multiple_of(400)
}

/// その月の日数。
fn days_in_month(year: u64, month: u64) -> u64 {
    match month {
        2 if is_leap_year(year) => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    }
}

/// `YYYY-MM-DDTHH:MM:SSZ` の形にする(ステート用。BR2.1)。
pub fn format_rfc3339(ts: Timestamp) -> String {
    let c = Civil::of(ts);
    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
        c.year, c.month, c.day, c.hour, c.minute, c.second
    )
}

/// `YYYYMMDDTHHMMSSZ` の形にする(ログのファイル名用。BR2.1、契約 C6)。
pub fn format_compact(ts: Timestamp) -> String {
    let c = Civil::of(ts);
    format!(
        "{:04}{:02}{:02}T{:02}{:02}{:02}Z",
        c.year, c.month, c.day, c.hour, c.minute, c.second
    )
}

/// [`format_rfc3339`] の形(`YYYY-MM-DDTHH:MM:SSZ` の 20 文字)だけを読む(BR2.2)。
///
/// 月・日・時・分・秒の範囲(日は月とうるう年に従う)と、年の範囲(1970〜9999)を検査する。
/// 小数の秒、`+09:00` などの時差、小文字の `t`・`z`、桁の不足は受け付けず `None`(使う側が扱う)。
pub fn parse_rfc3339(text: &str) -> Option<Timestamp> {
    let b = text.as_bytes();
    if b.len() != 20
        || b[4] != b'-'
        || b[7] != b'-'
        || b[10] != b'T'
        || b[13] != b':'
        || b[16] != b':'
        || b[19] != b'Z'
    {
        return None;
    }
    let number = |range: std::ops::Range<usize>| -> Option<u64> {
        b[range].iter().try_fold(0u64, |acc, &d| {
            d.is_ascii_digit().then(|| acc * 10 + u64::from(d - b'0'))
        })
    };
    let year = number(0..4)?;
    let month = number(5..7)?;
    let day = number(8..10)?;
    let hour = number(11..13)?;
    let minute = number(14..16)?;
    let second = number(17..19)?;
    if !(MIN_YEAR..=MAX_YEAR).contains(&year)
        || !(1..=12).contains(&month)
        || day == 0
        || day > days_in_month(year, month)
        || hour > 23
        || minute > 59
        || second > 59
    {
        return None;
    }
    let days = days_from_civil(year, month, day);
    Timestamp::from_unix_seconds(days * SECS_PER_DAY + hour * 3600 + minute * 60 + second)
}

/// 2 つの時刻の差(秒)。`end` が `start` より前なら 0(負にしない。NFR2.2)。
pub fn duration_secs(start: Timestamp, end: Timestamp) -> u64 {
    end.0.saturating_sub(start.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn 日数と年月日の変換は往復する() {
        // 範囲の端と、うるう年の前後を含む日を、日ごとに往復させる。
        for days in (0..800).chain(10_950..11_400).chain(2_932_500..2_932_897) {
            let (y, m, d) = civil_from_days(days);
            assert_eq!(days_from_civil(y, m, d), days, "{days}");
            assert!((1..=12).contains(&m) && (1..=days_in_month(y, m)).contains(&d));
        }
    }

    #[test]
    fn 上限の時刻は_9999_年の最後の秒() {
        assert_eq!(format_rfc3339(Timestamp::MAX), "9999-12-31T23:59:59Z");
        assert_eq!(
            Timestamp::from_unix_seconds(Timestamp::MAX.unix_seconds() + 1),
            None
        );
    }

    #[test]
    fn うるう年の判定() {
        assert!(is_leap_year(2000) && is_leap_year(2024));
        assert!(!is_leap_year(2100) && !is_leap_year(2023) && !is_leap_year(1900));
    }
}
