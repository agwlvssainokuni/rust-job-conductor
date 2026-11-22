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

//! リトライの設定 `[job.retry]` の読み取りと検証(U1。BR1.1〜BR1.16、FR1.1〜FR1.9、FR7.1)。

mod common;

use common::{kinds, run};
use conductor_core::{
    DefinitionErrorKind as K, JobDefinition, PipelineDefinition, RetryPolicy, parse_definition,
};

/// 先頭に版とパイプライン(3 行)と、ジョブ `a`(4〜6 行目)を付ける。`job_extra` は 7 行目から。
fn job_with(job_extra: &str) -> String {
    format!(
        "format_version = 1\n[pipeline]\nname = \"p\"\n[[job]]\nid = \"a\"\ncommand = \"c\"\n{job_extra}"
    )
}

/// `[job.retry]`(7 行目)と、`body`(8 行目から)を持つ定義。
fn retry_def(body: &str) -> String {
    job_with(&format!("[job.retry]\n{body}"))
}

/// 必須の 3 つを正しく書いた本体に、`extra` を足したもの(`extra` は 11 行目から)。
fn base(extra: &str) -> String {
    retry_def(&format!(
        "exit_codes = [75]\nmax = 3\ninterval = \"30s\"\n{extra}"
    ))
}

fn parse_ok(text: &str) -> PipelineDefinition {
    parse_definition(text, "t.toml", None).unwrap_or_else(|e| panic!("受理されること: {e:?}"))
}

fn policy(text: &str) -> RetryPolicy {
    parse_ok(text).job[0].retry.clone().expect("retry がある")
}

// ---- 正しい定義 ----

#[test]
fn 固定の間隔の設定を読める() {
    let p = policy(&base(""));
    assert_eq!(
        p,
        RetryPolicy {
            exit_codes: vec![75],
            max: 3,
            interval_secs: 30,
            multiplier: None,
            max_interval_secs: None,
        }
    );
}

#[test]
fn 倍率と上限つきの設定を読める() {
    let p = policy(&base("multiplier = 2\nmax_interval = \"10m\"\n"));
    assert_eq!(p.multiplier, Some(2));
    assert_eq!(p.max_interval_secs, Some(600));
    assert_eq!(p.interval_secs, 30);
}

#[test]
fn 最小値の境界を受理する() {
    let t = retry_def(
        "exit_codes = [1]\nmax = 1\ninterval = \"1s\"\nmultiplier = 2\nmax_interval = \"1s\"\n",
    );
    let p = policy(&t);
    assert_eq!(p.exit_codes, vec![1]);
    assert_eq!(p.max, 1);
    assert_eq!(p.interval_secs, 1);
    assert_eq!(p.multiplier, Some(2));
    assert_eq!(p.max_interval_secs, Some(1));
}

#[test]
fn 最大値の境界を受理する() {
    let t = retry_def(
        "exit_codes = [255]\nmax = 100\ninterval = \"24h\"\nmultiplier = 10\nmax_interval = \"24h\"\n",
    );
    let p = policy(&t);
    assert_eq!(p.exit_codes, vec![255]);
    assert_eq!(p.max, 100);
    assert_eq!(p.interval_secs, 86_400);
    assert_eq!(p.multiplier, Some(10));
    assert_eq!(p.max_interval_secs, Some(86_400));
}

#[test]
fn br1_6_終了コードの重複は集合にまとめられ昇順になる() {
    let t = retry_def("exit_codes = [75, 3, 75, 3, 12]\nmax = 2\ninterval = \"1m\"\n");
    assert_eq!(policy(&t).exit_codes, vec![3, 12, 75]);
}

#[test]
fn br1_8_終了コード_127_を受理する() {
    let t = retry_def("exit_codes = [127, 1]\nmax = 2\ninterval = \"1m\"\n");
    assert_eq!(policy(&t).exit_codes, vec![1, 127]);
}

#[test]
fn 間隔の単位_s_m_h_を秒に直す() {
    for (text, secs) in [("45s", 45), ("5m", 300), ("2h", 7200)] {
        let t = retry_def(&format!(
            "exit_codes = [1]\nmax = 1\ninterval = \"{text}\"\n"
        ));
        assert_eq!(policy(&t).interval_secs, secs, "{text}");
    }
}

#[test]
fn インラインテーブルでも書ける() {
    let t = job_with("retry = { exit_codes = [75], max = 3, interval = \"30s\" }\n");
    assert_eq!(policy(&t).max, 3);
}

#[test]
fn ジョブごとに別の設定を持てる() {
    let t = format!(
        "{}[[job]]\nid = \"b\"\ncommand = \"c\"\n[job.retry]\nexit_codes = [9]\nmax = 5\ninterval = \"2s\"\n",
        base("")
    );
    let def = parse_ok(&t);
    assert_eq!(def.job[0].retry.as_ref().map(|r| r.max), Some(3));
    assert_eq!(def.job[1].retry.as_ref().map(|r| r.max), Some(5));
}

#[test]
fn br1_1_retry_がなければ_none() {
    let t = job_with("");
    assert_eq!(parse_ok(&t).job[0].retry, None);
}

#[test]
fn br1_16_retry_の有無でそれ以外の読み取りの結果は変わらない() {
    let extra = "warning_exit_codes = [4]\nsuccess_exit_codes = [0, 5]\n[[job.transitions]]\non = \"success\"\nnext = \"@end\"\n";
    let without = parse_ok(&job_with(extra));
    // `[job.retry]` は、キーの後ろ(`transitions` の前)に書く。
    let with = parse_ok(&job_with(
        "warning_exit_codes = [4]\nsuccess_exit_codes = [0, 5]\nretry = { exit_codes = [75], max = 3, interval = \"30s\" }\n[[job.transitions]]\non = \"success\"\nnext = \"@end\"\n",
    ));
    let strip = |j: &JobDefinition| JobDefinition {
        retry: None,
        ..j.clone()
    };
    assert_eq!(strip(&with.job[0]), without.job[0]);
    assert!(with.job[0].retry.is_some());
}

#[test]
fn 成功の終了コードを書いても_retry_の終了コードが別なら受理する() {
    let t = job_with(
        "success_exit_codes = [0, 5]\nwarning_exit_codes = [4]\n[job.retry]\nexit_codes = [75]\nmax = 3\ninterval = \"30s\"\n",
    );
    assert!(run(&t).is_empty());
}

// ---- BR1.2 / BR1.3 / BR1.4 ----

#[test]
fn br1_2_retry_が表でない() {
    assert_eq!(
        run(&job_with("retry = 3\n")),
        vec![(K::RetryNotTable, Some(7))]
    );
    assert_eq!(
        run(&job_with("retry = \"x\"\n")),
        vec![(K::RetryNotTable, Some(7))]
    );
    // テーブルの配列
    let arr = job_with("[[job.retry]]\nexit_codes = [1]\nmax = 1\ninterval = \"1s\"\n");
    assert_eq!(kinds(&arr), vec![K::RetryNotTable]);
}

#[test]
fn br1_3_必須のキーがない() {
    assert_eq!(
        run(&retry_def("max = 3\ninterval = \"30s\"\n")),
        vec![(K::RetryMissingKey, Some(7))]
    );
    assert_eq!(
        run(&retry_def("exit_codes = [75]\ninterval = \"30s\"\n")),
        vec![(K::RetryMissingKey, Some(7))]
    );
    assert_eq!(
        run(&retry_def("exit_codes = [75]\nmax = 3\n")),
        vec![(K::RetryMissingKey, Some(7))]
    );
}

#[test]
fn br1_3_空の表は_3_つの必須のキーすべてを報告する() {
    let t = job_with("[job.retry]\n");
    assert_eq!(
        kinds(&t),
        vec![K::RetryMissingKey, K::RetryMissingKey, K::RetryMissingKey]
    );
}

#[test]
fn br1_4_許可されていないキーは未知のキーの誤り() {
    assert_eq!(run(&base("retries = 3\n")), vec![(K::UnknownKey, Some(11))]);
}

// ---- BR1.5 / BR1.7 ----

#[test]
fn br1_5_終了コードの一覧が空() {
    assert_eq!(
        run(&retry_def("exit_codes = []\nmax = 3\ninterval = \"30s\"\n")),
        vec![(K::RetryExitCodesEmpty, Some(8))]
    );
}

#[test]
fn br1_5_終了コードの範囲の内側と外側() {
    let with = |codes: &str| {
        retry_def(&format!(
            "exit_codes = {codes}\nmax = 3\ninterval = \"30s\"\n"
        ))
    };
    assert!(run(&with("[1]")).is_empty());
    assert!(run(&with("[255]")).is_empty());
    for bad in ["[0]", "[256]", "[-1]", "[1, 0]", "[75, 256]"] {
        assert_eq!(kinds(&with(bad)), vec![K::RetryExitCodeInvalid], "{bad}");
    }
}

#[test]
fn br1_5_終了コードの型が違う() {
    let with = |codes: &str| {
        retry_def(&format!(
            "exit_codes = {codes}\nmax = 3\ninterval = \"30s\"\n"
        ))
    };
    for bad in [
        "75",
        "\"75\"",
        "[\"75\"]",
        "[1.5]",
        "[true]",
        "[99999999999]",
    ] {
        assert_eq!(kinds(&with(bad)), vec![K::RetryExitCodeInvalid], "{bad}");
    }
}

#[test]
fn br1_5_複数の不正な要素は要素ごとに報告する() {
    let t = retry_def("exit_codes = [0, 75, 256]\nmax = 3\ninterval = \"30s\"\n");
    assert_eq!(
        kinds(&t),
        vec![K::RetryExitCodeInvalid, K::RetryExitCodeInvalid]
    );
}

#[test]
fn br1_7_成功の終了コードと重なる() {
    let t = job_with(
        "success_exit_codes = [0, 5]\n[job.retry]\nexit_codes = [5, 75]\nmax = 3\ninterval = \"30s\"\n",
    );
    assert_eq!(run(&t), vec![(K::RetryExitCodesOverlap, Some(9))]);
}

#[test]
fn br1_7_警告の終了コードと重なる() {
    let t = job_with(
        "warning_exit_codes = [4]\n[job.retry]\nexit_codes = [4]\nmax = 3\ninterval = \"30s\"\n",
    );
    assert_eq!(run(&t), vec![(K::RetryExitCodesOverlap, Some(9))]);
}

#[test]
fn br1_7_成功の終了コードを書いたときは_0_以外も成功なので重なりを調べる() {
    // 成功の集合を `[3]` と書けば、既定の `[0]` ではなくなる。`[3]` と重なれば誤り。
    let t = job_with(
        "success_exit_codes = [3]\n[job.retry]\nexit_codes = [3]\nmax = 3\ninterval = \"30s\"\n",
    );
    assert_eq!(kinds(&t), vec![K::RetryExitCodesOverlap]);
}

#[test]
fn br1_7_成功の終了コードの記述が誤りなら重なりは調べない() {
    let t = job_with(
        "success_exit_codes = [\"x\"]\n[job.retry]\nexit_codes = [75]\nmax = 3\ninterval = \"30s\"\n",
    );
    assert_eq!(kinds(&t), vec![K::TypeMismatch]);
}

// ---- BR1.9 ----

#[test]
fn br1_9_最大回数の境界の内側と外側() {
    let with = |max: &str| {
        retry_def(&format!(
            "exit_codes = [75]\nmax = {max}\ninterval = \"30s\"\n"
        ))
    };
    assert!(run(&with("1")).is_empty());
    assert!(run(&with("100")).is_empty());
    for bad in ["0", "101", "-1", "1.5", "\"3\"", "true"] {
        assert_eq!(
            run(&with(bad)),
            vec![(K::RetryMaxInvalid, Some(9))],
            "{bad}"
        );
    }
}

// ---- BR1.10 ----

#[test]
fn br1_10_間隔の境界の内側と外側() {
    let with = |v: &str| retry_def(&format!("exit_codes = [75]\nmax = 3\ninterval = {v}\n"));
    assert!(run(&with("\"1s\"")).is_empty());
    assert!(run(&with("\"24h\"")).is_empty());
    for bad in [
        "\"0s\"",
        "\"25h\"",
        "\"86401s\"",
        "\"1.5s\"",
        "\"10\"",
        "\"1d\"",
        "\"\"",
        "\"s\"",
        "30",
        "true",
    ] {
        assert_eq!(
            run(&with(bad)),
            vec![(K::RetryDurationInvalid, Some(10))],
            "{bad}"
        );
    }
}

#[test]
fn br1_10_間隔の上限にも同じ書式の規則を使う() {
    let t = base("multiplier = 2\nmax_interval = \"25h\"\n");
    assert_eq!(run(&t), vec![(K::RetryDurationInvalid, Some(12))]);
}

// ---- BR1.11 / BR1.12 / BR1.13 ----

#[test]
fn br1_11_倍率だけを書く() {
    assert_eq!(
        run(&base("multiplier = 2\n")),
        vec![(K::RetryBackoffIncomplete, Some(7))]
    );
}

#[test]
fn br1_11_間隔の上限だけを書く() {
    assert_eq!(
        run(&base("max_interval = \"10m\"\n")),
        vec![(K::RetryBackoffIncomplete, Some(7))]
    );
}

#[test]
fn br1_12_倍率の境界の内側と外側() {
    let with = |m: &str| base(&format!("multiplier = {m}\nmax_interval = \"10m\"\n"));
    assert!(run(&with("2")).is_empty());
    assert!(run(&with("10")).is_empty());
    for bad in ["1", "11", "0", "-2", "2.0", "\"2\"", "true"] {
        assert_eq!(
            run(&with(bad)),
            vec![(K::RetryMultiplierInvalid, Some(11))],
            "{bad}"
        );
    }
}

#[test]
fn br1_13_上限が間隔より小さい() {
    let t = retry_def(
        "exit_codes = [75]\nmax = 3\ninterval = \"10m\"\nmultiplier = 2\nmax_interval = \"5m\"\n",
    );
    assert_eq!(run(&t), vec![(K::RetryMaxIntervalTooSmall, Some(12))]);
}

#[test]
fn br1_13_上限が間隔と同じなら受理する() {
    let t = retry_def(
        "exit_codes = [75]\nmax = 3\ninterval = \"10m\"\nmultiplier = 2\nmax_interval = \"600s\"\n",
    );
    assert!(run(&t).is_empty());
}

// ---- BR1.14 ----

#[test]
fn br1_14_誤りは位置の順に集められる() {
    // 3 行目までが見出し。ジョブ a の retry に、複数の誤りを入れる。
    let t = retry_def(
        "exit_codes = [0]\nmax = 0\ninterval = \"0s\"\nmultiplier = 1\nmax_interval = \"0s\"\nunknown = 1\n",
    );
    let got = run(&t);
    let lines: Vec<usize> = got.iter().filter_map(|(_, l)| *l).collect();
    let mut sorted = lines.clone();
    sorted.sort_unstable();
    assert_eq!(lines, sorted, "位置の順: {got:?}");
    assert_eq!(
        got,
        vec![
            (K::RetryExitCodeInvalid, Some(8)),
            (K::RetryMaxInvalid, Some(9)),
            (K::RetryDurationInvalid, Some(10)),
            (K::RetryMultiplierInvalid, Some(11)),
            (K::RetryDurationInvalid, Some(12)),
            (K::UnknownKey, Some(13)),
        ]
    );
}

#[test]
fn br1_14_ほかの定義の誤りと一緒に位置の順に集められる() {
    let t = "format_version = 1\n[pipeline]\nname = \"p\"\n[[job]]\nid = \"a\"\ncommand = \"\"\n[job.retry]\nexit_codes = [75]\nmax = 0\ninterval = \"30s\"\n[[job]]\nid = \"a\"\ncommand = \"c\"\n[job.retry]\nexit_codes = [75]\nmax = 3\n";
    assert_eq!(
        run(t),
        vec![
            (K::EmptyField, Some(6)),
            (K::RetryMaxInvalid, Some(9)),
            (K::DuplicateJobId, Some(12)),
            (K::RetryMissingKey, Some(14)),
        ]
    );
}

#[test]
fn br1_14_誤りがあれば定義は組み立てられない() {
    assert!(parse_definition(&base("multiplier = 2\n"), "t.toml", None).is_err());
}

#[test]
fn 誤りの表示に入力の値は出ない() {
    const S: &str = "SENTINEL_VALUE_retry";
    let t = retry_def(&format!(
        "exit_codes = [\"{S}\"]\nmax = \"{S}\"\ninterval = \"{S}\"\nmultiplier = \"{S}\"\nmax_interval = \"{S}\"\n"
    ));
    let errs = parse_definition(&t, "t.toml", None).unwrap_err();
    assert!(errs.len() >= 5);
    for e in errs {
        assert!(!e.to_string().contains(S), "{e}");
    }
}
