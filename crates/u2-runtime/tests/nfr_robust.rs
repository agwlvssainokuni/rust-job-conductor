//! パニックしない、桁あふれしない(NFR4.7)。外部の入力(ステート、command、OS の結果)に対して、
//! 失敗は戻り値で返る。ランダムな入力は、固定の種から作る(再現できる)。

mod common;

use conductor_runtime::{InterruptSignal, check_command, read_state, run_job};

/// 小さな疑似乱数(xorshift64)。
struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }
    fn below(&mut self, n: usize) -> usize {
        (self.next() % n as u64) as usize
    }
}

const VALID: &str = r#"format_version = 1
definition_key = "c.toml"
definition_path = "c.toml"
pipeline_name = "p"
job_order = ["a", "b"]
end_reached = false
completed = false
resume_job_id = "b"

[[jobs]]
id = "a"
status = "success"
exit_code = 0

[[jobs]]
id = "b"
status = "pending"
"#;

fn read_bytes(bytes: &[u8]) -> bool {
    let dir = tempfile::tempdir().expect("一時ディレクトリ");
    let def = dir.path().join("c.toml");
    std::fs::write(dir.path().join("c.toml.state"), bytes).expect("書ける");
    // パニックしなければよい。結果は、読めても、エラーでもよい。
    read_state(&def).is_ok()
}

#[test]
fn 壊れた_toml_やランダムな入力でパニックしない() {
    let mut rng = Rng(0x9E37_79B9_7F4A_7C15);
    for _ in 0..200 {
        let len = rng.below(300);
        let bytes: Vec<u8> = (0..len).map(|_| rng.next() as u8).collect();
        let _ = read_bytes(&bytes);
    }
    for broken in [
        "format_version = 1\n[[jobs]\n",
        "format_version = 1\njobs = [[[[",
        "= = =",
        "format_version = 1\njob_order = [1, [2], {a = 1}]\n",
        "\u{feff}format_version = 1\n",
        "format_version = 1.5\n",
        "format_version = 9223372036854775807\n",
        "format_version = -9223372036854775808\n",
    ] {
        assert!(!read_bytes(broken.as_bytes()), "{broken}");
    }
}

#[test]
fn 正しいステートを壊した入力でパニックしない() {
    let mut rng = Rng(42);
    let base = VALID.as_bytes();
    for _ in 0..200 {
        let mut bytes = base.to_vec();
        match rng.below(3) {
            0 => {
                let cut = rng.below(bytes.len());
                bytes.truncate(cut);
            }
            1 => {
                for _ in 0..1 + rng.below(4) {
                    let i = rng.below(bytes.len());
                    bytes[i] = rng.next() as u8;
                }
            }
            _ => {
                let i = rng.below(bytes.len());
                bytes.remove(i);
            }
        }
        let _ = read_bytes(&bytes);
    }
    assert!(read_bytes(base), "もとの入力は読める");
}

#[test]
fn 巨大な値のステートでパニックしない() {
    // 範囲外の終了コード、巨大な文字列、巨大な配列。
    let huge = "x".repeat(900_000);
    let cases = [
        VALID.replace("exit_code = 0", "exit_code = 9223372036854775807"),
        VALID.replace("exit_code = 0", "exit_code = -9223372036854775808"),
        VALID.replace(
            "pipeline_name = \"p\"",
            &format!("pipeline_name = \"{huge}\""),
        ),
        VALID.replace(
            "job_order = [\"a\", \"b\"]",
            &format!("job_order = [{}]", "\"z\",".repeat(100_000)),
        ),
        VALID.replace(
            "[[jobs]]\nid = \"b\"",
            &"[[jobs]]\nid = \"b\"\nstatus = \"pending\"\n".repeat(5_000),
        ),
    ];
    for (i, text) in cases.iter().enumerate() {
        let accepted = read_bytes(text.as_bytes());
        assert!(!accepted || i == 2, "case {i}");
    }
}

#[test]
fn ランダムな_command_の文字列でパニックしない() {
    let alphabet: Vec<char> = "ab \t\n'\"\\;|&$*あ".chars().collect();
    let mut rng = Rng(7);
    for _ in 0..2000 {
        let len = rng.below(40);
        let s: String = (0..len)
            .map(|_| alphabet[rng.below(alphabet.len())])
            .collect();
        if let Ok(line) = check_command(&s) {
            assert!(!line.program.is_empty());
        }
    }
}

#[test]
fn 大きな入力の_command_でも分けられる() {
    // 入れ子はなく、再帰を使わないため、長さに対して、スタックを使わない。
    let long = format!("prog {}", "'a b' \"c\\\"d\" e ".repeat(100_000));
    let line = check_command(&long).expect("分けられる");
    assert_eq!(line.args.len(), 300_000);
    let unclosed = format!("prog {}'", "a ".repeat(100_000));
    assert!(check_command(&unclosed).is_err());
}

#[test]
fn 起動できないコマンドや範囲外の番号でパニックしない() {
    let dir = tempfile::tempdir().expect("一時ディレクトリ");
    let cases = [
        String::new(),
        "'".to_string(),
        "''".to_string(),
        "conductor-no-such-command-zzz".to_string(),
        // ディレクトリを実行ファイルとして指定する。
        format!("'{}'", dir.path().display()),
        "/".to_string(),
    ];
    for command in cases {
        let e = run_job(&common::job("j", command), &InterruptSignal::new());
        assert!(e.start_failure, "{e:?}");
    }
    for number in [i32::MAX, i32::MIN, 0, -1, 1_000_000] {
        let s = InterruptSignal::new();
        s.raise(number);
        let e = run_job(&common::job("j", "anything".into()), &s);
        assert!(!e.started && e.interrupted);
        assert!(e.signal_number.is_some());
    }
}
