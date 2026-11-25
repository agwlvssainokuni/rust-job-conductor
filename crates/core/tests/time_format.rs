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

//! 時刻の整形・詰めた形・読み取り・差(CoreTime。v0.10.0 の BR2.1、BR2.2、FR6.6、NFR2.2)のテスト。
//!
//! 境界(範囲の端、うるう年、各月末)を並べた表で、整形と読み取りの往復を確かめる。
//! U6 の所要時間(BR4.1 の表の各行、BR4.2 の書き方)も確かめる。

use conductor_core::{
    Attempt, JobDuration, JobState, JobStatus, Timestamp, duration_secs, format_compact,
    format_duration, format_rfc3339, job_duration, parse_rfc3339,
};

fn ts(secs: u64) -> Timestamp {
    Timestamp::from_unix_seconds(secs).expect("範囲の中")
}

/// (UNIX 時刻の秒, RFC 3339 の形, 詰めた形)の表。秒の値は別の手段(`date -u`)で求めた既知の値。
const TABLE: &[(u64, &str, &str)] = &[
    (0, "1970-01-01T00:00:00Z", "19700101T000000Z"),
    (59, "1970-01-01T00:00:59Z", "19700101T000059Z"),
    (86_399, "1970-01-01T23:59:59Z", "19700101T235959Z"),
    (86_400, "1970-01-02T00:00:00Z", "19700102T000000Z"),
    (951_782_400, "2000-02-29T00:00:00Z", "20000229T000000Z"),
    (951_868_800, "2000-03-01T00:00:00Z", "20000301T000000Z"),
    (978_307_199, "2000-12-31T23:59:59Z", "20001231T235959Z"),
    (1_709_164_800, "2024-02-29T00:00:00Z", "20240229T000000Z"),
    (1_709_251_199, "2024-02-29T23:59:59Z", "20240229T235959Z"),
    (1_677_628_799, "2023-02-28T23:59:59Z", "20230228T235959Z"),
    (1_677_628_800, "2023-03-01T00:00:00Z", "20230301T000000Z"),
    (4_107_456_000, "2100-02-28T00:00:00Z", "21000228T000000Z"),
    (4_107_542_400, "2100-03-01T00:00:00Z", "21000301T000000Z"),
    (1_795_597_968, "2026-11-25T09:12:48Z", "20261125T091248Z"),
    (253_402_300_799, "9999-12-31T23:59:59Z", "99991231T235959Z"),
];

#[test]
fn br2_1_境界の表で整形と詰めた形が一致する() {
    for &(secs, rfc, compact) in TABLE {
        assert_eq!(format_rfc3339(ts(secs)), rfc, "{secs}");
        assert_eq!(format_compact(ts(secs)), compact, "{secs}");
    }
}

#[test]
fn br2_2_境界の表で読み取りが整形と往復する() {
    for &(secs, rfc, _) in TABLE {
        assert_eq!(parse_rfc3339(rfc), Some(ts(secs)), "{rfc}");
    }
}

#[test]
fn br2_1_2024_年の各月末とその翌日() {
    // 2024 年(うるう年)の各月の最後の日と、翌月の 1 日。
    let ends = [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    let mut day_start = 1_704_067_200; // 2024-01-01T00:00:00Z
    for (i, &last) in ends.iter().enumerate() {
        let month = i + 1;
        let last_day = ts(day_start + (last - 1) * 86_400);
        assert_eq!(
            format_rfc3339(last_day),
            format!("2024-{month:02}-{last:02}T00:00:00Z")
        );
        day_start += last * 86_400;
        let (next_year, next_month) = if month == 12 {
            (2025, 1)
        } else {
            (2024, month + 1)
        };
        assert_eq!(
            format_rfc3339(ts(day_start)),
            format!("{next_year}-{next_month:02}-01T00:00:00Z")
        );
        // 月末の最後の秒も往復する。
        let text = format_rfc3339(ts(day_start - 1));
        assert_eq!(parse_rfc3339(&text), Some(ts(day_start - 1)), "{text}");
    }
}

#[test]
fn br2_1_日ごとに往復する() {
    // 1970 年から 400 年ぶん(すべての種類のうるう年を含む)を 1 日ずつ、時刻は日ごとに変える。
    for day in 0..146_097u64 {
        let secs = day * 86_400 + (day * 7919) % 86_400;
        let text = format_rfc3339(ts(secs));
        assert_eq!(parse_rfc3339(&text), Some(ts(secs)), "{text}");
        assert_eq!(text.len(), 20);
    }
}

#[test]
fn br2_2_形の違うものは読まない() {
    for bad in [
        "",
        "2026-11-25T09:12:48",       // Z が無い
        "2026-11-25T09:12:48.5Z",    // 小数の秒
        "2026-11-25T09:12:48+09:00", // 時差
        "2026-11-25T09:12:48+00:00", // 時差(0 でも)
        "2026-11-25t09:12:48Z",      // 小文字の t
        "2026-11-25T09:12:48z",      // 小文字の z
        "2026-11-25 09:12:48Z",      // 空白の区切り
        "20261125T091248Z",          // 詰めた形は読まない
        "2026-1-25T09:12:48Z",       // 桁の不足
        "2026-11-25T9:12:48Z",       // 桁の不足
        "226-11-25T09:12:48Z",       // 年の桁の不足
        "+2026-11-25T09:12:48Z",     // 符号
        "2026-11-25T09:12:48Z ",     // 後ろの空白
        " 2026-11-25T09:12:48Z",     // 前の空白
        "2026-1a-25T09:12:48Z",      // 数字でない
        "２０２６-11-25T09:12:48Z",  // 全角の数字
        "2026/11/25T09:12:48Z",      // 区切りの違い
    ] {
        assert_eq!(parse_rfc3339(bad), None, "{bad:?}");
    }
}

#[test]
fn br2_2_範囲の外の値は読まない() {
    for bad in [
        "2026-00-25T09:12:48Z",
        "2026-13-25T09:12:48Z",
        "2026-11-00T09:12:48Z",
        "2026-11-31T09:12:48Z",
        "2026-04-31T00:00:00Z",
        "2023-02-29T00:00:00Z", // 平年
        "2100-02-29T00:00:00Z", // 100 で割り切れる平年
        "2026-11-25T24:00:00Z",
        "2026-11-25T23:60:00Z",
        "2026-11-25T23:59:60Z", // うるう秒は扱わない
        "1969-12-31T23:59:59Z", // 範囲の前
        "0000-01-01T00:00:00Z",
    ] {
        assert_eq!(parse_rfc3339(bad), None, "{bad:?}");
    }
    // うるう年の 2 月 29 日は読める。
    assert!(parse_rfc3339("2000-02-29T00:00:00Z").is_some());
    assert!(parse_rfc3339("2400-02-29T12:00:00Z").is_some());
}

#[test]
fn br2_1_範囲は_1970_年から_9999_年まで() {
    assert_eq!(Timestamp::EPOCH.unix_seconds(), 0);
    assert_eq!(format_rfc3339(Timestamp::EPOCH), "1970-01-01T00:00:00Z");
    assert_eq!(Timestamp::MAX.unix_seconds(), 253_402_300_799);
    assert_eq!(Timestamp::from_unix_seconds(253_402_300_800), None);
    assert_eq!(Timestamp::from_unix_seconds(u64::MAX), None);
    assert_eq!(
        Timestamp::from_unix_seconds(253_402_300_799),
        Some(Timestamp::MAX)
    );
}

#[test]
fn nfr2_2_差は秒で_負にならない() {
    let a = ts(1_795_597_968);
    let b = ts(1_795_598_000);
    assert_eq!(duration_secs(a, b), 32);
    assert_eq!(duration_secs(a, a), 0);
    assert_eq!(duration_secs(b, a), 0, "後が前より小さければ 0");
    assert_eq!(
        duration_secs(Timestamp::EPOCH, Timestamp::MAX),
        253_402_300_799
    );
}

#[test]
fn 時刻は前後で比べられる() {
    assert!(ts(1) > Timestamp::EPOCH);
    assert!(Timestamp::MAX > ts(1_795_597_968));
}

#[test]
fn u6_br4_2_所要時間は単位つきの短い形() {
    for (secs, text) in [
        (0, "0s"),
        (17, "17s"),
        (59, "59s"),
        (60, "1m00s"),
        (65, "1m05s"),
        (3599, "59m59s"),
        (3600, "1h00m00s"),
        (3661, "1h01m01s"),
        (7384, "2h03m04s"),
        (100 * 3600, "100h00m00s"),
    ] {
        assert_eq!(format_duration(secs), text, "{secs}");
    }
}

/// `status` のジョブに、`(始まり, 終わり)` の試行を持たせる。
fn job_with(status: JobStatus, attempts: &[(u64, Option<u64>)]) -> JobState {
    let mut job = JobState::pending("a");
    job.status = status;
    job.attempts = attempts
        .iter()
        .map(|&(start, end)| Attempt {
            started_at: ts(start),
            ended_at: end.map(ts),
        })
        .collect();
    job
}

#[test]
fn u6_br4_1_試行が無ければ所要時間は空() {
    for status in [
        JobStatus::Pending,
        JobStatus::Success,
        JobStatus::Warning,
        JobStatus::Error,
        JobStatus::Skipped,
    ] {
        assert_eq!(job_duration(&job_with(status, &[]), ts(500)), None);
    }
}

#[test]
fn u6_br4_1_結果のあるジョブは最初の始まりから最後の終わりまで() {
    for status in [JobStatus::Success, JobStatus::Warning, JobStatus::Error] {
        let one = job_with(status, &[(100, Some(117))]);
        assert_eq!(
            job_duration(&one, ts(9_999)),
            Some(JobDuration::Finished(17))
        );
        // リトライや再開で試行が複数あれば、最初の始まりから最後の終わりまで(待ちを含む)。
        let many = job_with(status, &[(100, Some(110)), (130, Some(165))]);
        assert_eq!(
            job_duration(&many, ts(9_999)),
            Some(JobDuration::Finished(65))
        );
    }
}

#[test]
fn u6_br4_1_結果のあるジョブで最後の試行に終わりが無ければ空() {
    // conductor は書かない形(手で書き換えたなど)。
    let job = job_with(JobStatus::Success, &[(100, Some(110)), (130, None)]);
    assert_eq!(job_duration(&job, ts(9_999)), None);
}

#[test]
fn u6_br4_1_動いている最中の_pending_は今までの経過時間() {
    let job = job_with(JobStatus::Pending, &[(100, Some(110)), (130, None)]);
    let d = job_duration(&job, ts(165));
    assert_eq!(d, Some(JobDuration::Running(65)));
    assert_eq!(d.map(JobDuration::secs), Some(65));
}

#[test]
fn u6_br4_1_最後の試行が終わった_pending_と_skipped_は空() {
    let pending = job_with(JobStatus::Pending, &[(100, Some(110))]);
    assert_eq!(job_duration(&pending, ts(500)), None);
    let skipped = job_with(JobStatus::Skipped, &[(100, Some(110))]);
    assert_eq!(job_duration(&skipped, ts(500)), None);
    let skipped_open = job_with(JobStatus::Skipped, &[(100, None)]);
    assert_eq!(job_duration(&skipped_open, ts(500)), None);
}

#[test]
fn u6_br4_1_終わりが始まりより前なら_0_秒() {
    let finished = job_with(JobStatus::Success, &[(200, Some(150))]);
    assert_eq!(
        job_duration(&finished, ts(500)),
        Some(JobDuration::Finished(0))
    );
    // 今の時刻が始まりより前(時計の戻り)でも 0 秒。
    let running = job_with(JobStatus::Pending, &[(200, None)]);
    assert_eq!(
        job_duration(&running, ts(100)),
        Some(JobDuration::Running(0))
    );
}
