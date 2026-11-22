## Review

**Verdict:** READY
**Reviewer:** aidlc-architecture-reviewer-agent
**Iteration:** 1
**Verdict Dictated:** no

### Findings

| ID | Severity | Location | Finding | Required action | Status |
|---|---|---|---|---|---|
| R-01 | Minor | crates/cli/tests/args_grammar.rs:1122 > `utf8_でない_to_の値は_invalid_value_で対象は_to` | `#[cfg(unix)]` を付けたテストで、非 UTF-8 の `--to` の確かめは Windows の CI では動かない。ただし `unix_demo.rs`、`nfr_perf.rs` にも同じ前例があり、「cfg を platform の外に置かない」の対象は src/ なので、規則違反ではない。 | 対応は任意。Windows でも確かめたいなら、`std::os::windows::ffi::OsStringExt::from_wide` で不正な UTF-16 の値を組むテストを足す。 | New |
| R-02 | Minor | crates/cli/src/args.rs > `CHECKS` | 検査の順は、旧コードの `if` の並びと 1 件ずつ照合して一致した(使えない 8 件、`--force` の重複、`--assume`、`--dry-run` の重複、format/state、重複 7 件)。ただし、`CHECKS` の並びと仕様書 04 章 4.2 の 6・7 の順が合うことを機械的に守る仕組みは、報告の順のテストだけである。表に行を足して順を誤ったときは、そのテストが落ちることに頼っている。 | 現状は許容できる。表に行を足すときは、報告の順のテストにも対を足すことを、メモに残す。 | New |

### 確認した内容

**観点 1(利用者から見える変更)**
- `git diff ba1cdfb..HEAD -- crates/cli/src/args.rs` を旧コードと突き合わせた。差が出るのは `parse_target` だけ。誤りの対象が値(`to_string_lossy`)から `--to` に変わり、非 UTF-8 の値でも `--to` になる。
- 字句の段階は旧コードと同じ。
  - `--help`/`--version` は表を引く前に先に取る。
  - 値の欠落は `break` で打ち切り、対象は `spec.opt.subject()` で旧来の綴りと同じ(`-v` だけ短い形)。
  - `--name=value` は lexopt が処理する。値を取らないオプションに `=値` を付けたときの判定(`map_lex_error` の `known`)は、旧来の固定一覧(`--verbose`、`-v`、`--all`、`--remove-on-complete`、`--state`、`--dry-run`、`--force`)と、表の `takes_value` が偽の行と `-v` が同じ集合になる。
  - 短い形は `-v` だけが `Opt::Verbose`。ほかの短い形は旧来どおり `Unknown`。
- 組み立ての段階も同じ。
  - 検査の順は旧コードの `if` の並びと一致する。
  - `--all` と `--run-id` の併用、`graph` の `--run-id` の条件、値の検査の順(`--assume`、`--format`、`--run-id`、位置引数、`--to`)も変わらない。
- 差が出る入力は、`convert --to` の誤りの対象のほかに見つからなかった。

**観点 2(テスト)**
- `git diff` で、既存のテストファイルの削除行は 0 件だった。`args_grammar.rs` への追加(270 行)だけである。
- 追加したテストは、誤りの種類と対象を `assert_error` で固定していて、常に通るものではない。
  - 報告の順を 8 本で確かめる。
  - 値の欠落の対象、`=値` の対象、`--to` の値の誤りの対象(`json`、`YAML`、空、`--to=` の形)を確かめる。
  - UTF-8 でない値は unix のときだけ確かめる。
- 名前はすべて日本語。
- `cargo test --locked --workspace` は 1128 件が通り、失敗は 0 件。`cargo clippy --workspace --all-targets --locked -- -D warnings` も警告なしで通った。

**観点 3(表の設計)**
- `OPTIONS`(11 行)と `CHECKS`(21 行)が表になっていて、オプションごとの `match` の腕や `if` の並びは残っていない(`build_command` は表をたどる 1 つのループ)。
- 表の行の順の検査は、const の `assert!`(行数が `Opt::State as usize + 1` と等しいこと、各行の `opt as usize == i`)で妥当。`spec` が添字で引くので、順がずれるとコンパイルが通らない。

**観点 4(仕様書)**
- rust の 04・10 と language-agnostic の 04・10・11 の変更は、実装と合う(対象は `--to` で、値は含めない。非 UTF-8 を含む。AC-237、AC-337 の例も更新済み)。
- 凍結した 5 つの仕様書(go、python、typescript、csharp、powershell)は変更なし。

**観点 5(制約)**
- `Cargo.toml` と `Cargo.lock` は変更なし(新しい依存なし)。
- `args.rs` に `unsafe` も `cfg` もない。
- 公開の型(`CliCommand`、`ParsedArgs`、`UsageError`、`UsageErrorKind`)は変更なし。`dry_run.rs` の差分はコメントだけ。
- 誤りに入力の値を含めない性質は、`parse_target` の修正でより強くなった。
