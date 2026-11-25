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

//! 現在の時刻(RuntimeClock。v0.10.0 の BR2.3、ADR-002、契約 C2)。
//!
//! 時刻の整形と読み取りは conductor-core の `time` が受け持ち、ここは時計から秒を取るだけ。
//! cli の流れは、試行の前に 1 回だけ `Runtime::now()`(本番はこの関数)を呼び、同じ値をログの
//! ファイル名と(U6 で)ステートの `started_at` に使う。

use std::time::{SystemTime, UNIX_EPOCH};

use conductor_core::Timestamp;

/// 現在の時刻(UTC の秒。BR2.3)。1970 年より前なら 0、9999 年を超えれば上限にする(防御)。
pub fn now() -> Timestamp {
    timestamp_of(SystemTime::now())
}

/// 時計の値を [`Timestamp`] にする(範囲の外は端に寄せる)。
fn timestamp_of(time: SystemTime) -> Timestamp {
    match time.duration_since(UNIX_EPOCH) {
        Ok(since) => Timestamp::from_unix_seconds(since.as_secs()).unwrap_or(Timestamp::MAX),
        Err(_) => Timestamp::EPOCH,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn 今の時刻は_2024_年より後() {
        // 2024-01-01T00:00:00Z より後(時計が正しく合っている前提の、ゆるい確認。NFR2.4)。
        assert!(now().unix_seconds() > 1_704_067_200);
    }

    #[test]
    fn 範囲の中の値はそのまま秒にする() {
        let t = UNIX_EPOCH + Duration::from_secs(1_795_597_968);
        assert_eq!(timestamp_of(t).unix_seconds(), 1_795_597_968);
        // 秒未満は切り捨てる。
        let t = UNIX_EPOCH + Duration::from_millis(1_500);
        assert_eq!(timestamp_of(t).unix_seconds(), 1);
    }

    #[test]
    fn 年が_1970_より前なら_0() {
        let before = UNIX_EPOCH - Duration::from_secs(1);
        assert_eq!(timestamp_of(before), Timestamp::EPOCH);
    }

    #[test]
    fn 上限を超える値は上限() {
        let after = UNIX_EPOCH + Duration::from_secs(Timestamp::MAX.unix_seconds() + 1);
        assert_eq!(timestamp_of(after), Timestamp::MAX);
    }
}
