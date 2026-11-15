//! Step 1 の確認(c): `toml` が、深く入れ子になった入力で、異常終了せず失敗を返すこと。
//! conductor-core と同じ確認(NFR4.7)。テストのスレッドの標準のスタック(約 2 MiB)で行う。

fn nested_arrays(depth: usize) -> String {
    format!("a = {}{}\n", "[".repeat(depth), "]".repeat(depth))
}

fn nested_inline_tables(depth: usize) -> String {
    format!("a = {}1{}\n", "{a=".repeat(depth), "}".repeat(depth))
}

fn parse(text: &str) -> Result<toml::Table, toml::de::Error> {
    text.parse::<toml::Table>()
}

#[test]
fn 深さ_1000_と_100000_の入れ子の配列は失敗を返す() {
    for depth in [1000, 100_000] {
        assert!(parse(&nested_arrays(depth)).is_err(), "depth {depth}");
    }
}

#[test]
fn 深さ_1000_と_100000_の入れ子の表は失敗を返す() {
    for depth in [1000, 100_000] {
        assert!(
            parse(&nested_inline_tables(depth)).is_err(),
            "depth {depth}"
        );
    }
}

#[test]
fn 状態の読み書きに必要な型を扱える() {
    let text =
        "n = 1\nb = true\ns = \"x\"\nl = [\"a\", \"b\"]\n[[jobs]]\nid = \"a\"\nexit_code = -3\n";
    let table = parse(text).expect("読める");
    assert_eq!(table["n"].as_integer(), Some(1));
    assert_eq!(table["b"].as_bool(), Some(true));
    assert_eq!(table["l"].as_array().map(|a| a.len()), Some(2));
    let jobs = table["jobs"].as_array().expect("配列");
    assert_eq!(jobs[0]["exit_code"].as_integer(), Some(-3));
    let written = toml::to_string(&table).expect("書ける");
    assert_eq!(parse(&written).expect("読み戻せる"), table);
}
