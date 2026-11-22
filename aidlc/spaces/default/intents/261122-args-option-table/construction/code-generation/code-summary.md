# コードの要約(Code Summary): args.rs のオプションの表化

計画: `code-generation-plan.md`(Step 1〜10)。設計: `construction/functional-design/functional-spec.md`、`rules.md`、`entities.md`。

## コミット

| 手順 | コミット | 内容 |
|------|----------|------|
| Step 2 | `a175253` | 誤りの報告の順を固定するテストを足す(表化の前のコードで通ることを確かめてからコミット) |
| Step 3 | `0300b32` | `Opt`、`Allowed`、`OptSpec`、`OPTIONS` を足し、字句の段階を表から行う。`map_lex_error` の集合と、使える場所の条件も表から読む |
| Step 4、5 | `bee8f58` | `Check`、`CHECKS`、`Given` を足し、「使えない」と「重複」の検査を表から行う。Step 5 の確かめもここで済ませた |
| Step 6 | `61690c3` | `convert --to` の誤りの対象を `--to` に。テストと仕様書 |
| Step 7 | `68747c0` | `enter_flow` の doc コメント |
| Step 8〜10 | (この文書のコミット) | Step 8 は直す箇所なし。Step 9 の全体の確認。この要約と `traceability.json` |

各コミットで `cargo build`、`cargo clippy --workspace --all-targets --locked -- -D warnings`、`cargo test --locked --workspace` が通った。`cargo fmt --all -- --check` も通る。ローカルの `rustc` は 1.100.0 で、MSRV と同じ版。

## テストの件数の移り変わり

| 時点 | 全体(通過 / 無視) | `args_grammar` |
|------|----------------------|----------------|
| 基準(Step 1、`ba1cdfb`) | 1,117 / 1 | 49 |
| Step 2 のあと | 1,126 / 1 | 58(+9) |
| Step 3、4 のあと | 1,126 / 1 | 58 |
| Step 6 のあと | 1,128 / 1 | 60(+2) |
| Step 7 以降 | 1,128 / 1 | 60 |

失敗は、どの時点でも 0。既存のテストの行は 1 行も変えていない(`git diff ba1cdfb -- crates/cli/tests/args_grammar.rs` の削除行は 0)。

注: `functional-spec.md` 5 の「`args_grammar.rs`(106 件)」は、実際の件数(49)と合わない。設計の時点の数え方の違いと見られる。計画の基準(全体 1,117)とは一致した。

## 足したテスト(すべて `crates/cli/tests/args_grammar.rs` の末尾)

Step 2(報告の順。表化の前の動きを固定。補助関数 `error_of`、`assert_error` を足した):

1. `報告の順_使えないの検査は仕様の順に並ぶ`: `-v`→`--to`→`--run-id`→`--all`→`--remove-on-complete`→`--dry-run` の隣り合う組と、`--format`→`--state`
2. `報告の順_force_の使えないは重複より先で_force_の重複は_assume_の不可より先`
3. `報告の順_assume_の不可と_dry_run_の重複は_format_と_state_の不可より先`(`--dry-run` の重複が `-v` の重複より先も)
4. `報告の順_使えないはすべての重複より先`(`watch -v -v` は `-v` の不可、`graph --format a --format b -v -v` は `--format` の重複ほか)
5. `報告の順_重複の検査は仕様の順に並ぶ`(`--format`→`--state`→`-v`→`--config`→`--run-id`、`--force` の重複が `--config` の重複より先)
6. `報告の順_重複は_all_と_run_id_の併用と値の検査より先`
7. `報告の順_字句の誤りは知らないオプションより先で_最初のものを報告する`
8. `値の欠落の対象はオプションの綴り`(値を取る 5 つ)
9. `値を取らないオプションに値を付けると対象はその綴り`(値を取らない 6 つ)

7〜9 は、字句の段階を表から行う形に書き換えるため、その段階の対象の綴りと順も固定した(計画の Step 2 の例より広い。下の「計画からのずれ」)。

Step 6(`--to`):

10. `to_の値の誤りの対象は_to_で値を含まない`(`json`、`YAML`、空文字。`--to v` と `--to=v` の両方の形)
11. `utf8_でない_to_の値は_invalid_value_で対象は_to`(`#[cfg(unix)]`。既存のテストと同じ置き方)

## 表の形(`crates/cli/src/args.rs`)

- `enum Opt`: `Verbose`、`Config`、`To`、`RunId`、`All`、`RemoveOnComplete`、`DryRun`、`Force`、`Assume`、`Format`、`State`。
- `enum Allowed`: `Commands(&'static [CommandKind])` か `WithDryRun`(`--assume` だけ)。
- `struct OptSpec { opt, long, takes_value, allowed }`、`const OPTIONS: &[OptSpec]`(11 行。`Opt` の並びと同じ順に置き、`const _: () = { ... }` でコンパイル時に順を検査する。`spec(opt)` は添字で引く)。
- `OptSpec::allows(kind, dry_run)`、`Opt::subject()`(`Verbose` だけ `-v`、ほかは `--<long>`)、`spec_by_long(name)`。
- `enum Check { NotAllowed(Opt), Duplicate(Opt) }`、`const CHECKS: &[Check]`(21 個。仕様書 04 章 4.2 の 6・7 の順)。
- `struct Given(Vec<Vec<Option<OsString>>>)`: オプションごとの指定の並び(`Opt` で引く)。`count`、`once`、`take`。
- `enum Token { Opt(Opt, Option<OsString>), Unknown(String), Positional(OsString) }`。
- 字句の段階: `-h`/`--help`/`-V`/`--version` を先に判定し、`Short('v')` は `Verbose`、ほかの短い形は `Unknown`、`Long(name)` は `spec_by_long` で引いて `takes_value` で分ける(1 つの腕)。
- 組み立ての段階: `CHECKS` を順に回して最初の誤りを返す。そのあとの `--all` と `--run-id` の併用、`graph` の `--state` なしの `--run-id`、値の検査、位置引数、`convert` の `--to` は今の順のまま。
- `functional-spec.md` 2 の「今」の場所(オプションごとの `match` の腕、`map_lex_error` の綴りの並び、数と値の変数 11 個、`if` 21 個)は残っていない。残る `"--to"` などの文字列は、`CHECKS` のあとの値の検査の関数(`parse_assumptions`、`parse_graph_format`、`parse_run_id_option`)と `resolve_run_id`、`convert` の `--to` なしの誤りで、範囲の外か、今の順のままにする部分。

公開の型(`CliCommand`、`ParsedArgs`、`UsageError`、`UsageErrorKind`)は変えていない。依存(`Cargo.toml`、`Cargo.lock`)の差分なし。`unsafe`、`cfg` は `src/` に足していない。

## 仕様書の変更

| ファイル | 箇所 | 変更 |
|----------|------|------|
| `docs/spec/rust/04-cli.md` | 3 節の表の `--to` の行 | 対象を「値そのもの」→「`--to`。値は含めない」 |
| 同 | 4 節の「非 UTF-8」の行 | 位置引数の並びから `--to` の値を外し、「`--to` の値が非 UTF-8 なら `invalid value`(対象 `--to`)」を足す |
| 同 | 4.1 の 11 | 対象を `--to` に(非 UTF-8 を含む、値は含めない) |
| 同 | 例の出力 | `invalid value: 'json'` → `invalid value: '--to'` |
| `docs/spec/rust/10-errors-and-exit-codes.md` | 対象の表の `--to` の行 | 「その値」→「`--to`(値は含めない)」。非 UTF-8 を含むと明記 |
| `docs/spec/language-agnostic/04-cli.md` | 3 節の表の `--to` の行、4.2 の 11、例の出力 | rust と同じ |
| `docs/spec/language-agnostic/10-errors-and-exit-codes.md` | 対象の表の `--to` の行 | rust と同じ |
| `docs/spec/language-agnostic/11-quality-and-build.md` | AC-237、AC-337 | 期待を `invalid value: '--to'` にし、テストの列に新しいテストの名前を足す |

凍結した 5 つの仕様書(go、python、typescript、csharp、powershell)は変えていない。非 UTF-8 の位置引数の行(rust 10 章、language-agnostic 10 章)は位置引数の話なので、そのまま。

## 版の更新の候補

`convert --to` の値の誤りの表示が `invalid value: '<値>'` から `invalid value: '--to'` に変わる。利用者から見える唯一の変更なので、版の更新(リリースの注記)の候補として記録する。版を上げるかは、リリースのときにユーザーが決める。この作業では `crates/cli/Cargo.toml` の版を変えていない。

## 計画からのずれ

1. Step 3 と Step 4 の境目: 計画の Step 3 は「`OPTIONS`(`Opt`、`OptSpec`、`Allowed`)を足し、字句の段階を表から行う」。`allowed` の列は Step 4 まで読まれず、`clippy -D warnings` の dead code で止まるため、Step 3 で「使える場所」の条件(`takes_verbose` などの `matches!`)も `OptSpec::allows` で表から読む形にした。`if` の並びそのものの置き換えは計画どおり Step 4。
2. Step 4 と Step 5 を 1 つのコミットにした(Step 5 は確かめだけで、変更が無いため)。
3. Step 2 で足した順のテストは、計画の例より広い(字句の段階の対象の綴りと、字句の誤りと知らないオプションの順も固定した)。どれも表化の前のコードで通ることを確かめた。
4. Step 2 の「`--assume` の不可が `--dry-run` の重複より先」は、テストにできない(`--assume` の不可は `--dry-run` が無いときだけで、`--dry-run` の重複とは同時に起きない)。代わりに、`--assume` の不可と `--dry-run` の重複が、それぞれ `--format`・`--state` の不可より先であることを固定した。
5. Step 8: rust の仕様書には `args.rs` の内部の形(`Token` の変種、`build_command` の `if` の並び)を書いた箇所が無かったので、変更なし(計画どおり)。
6. `args.rs` の行数は 538 → 627 に増えた。表の行を 1 項目 1 行で書く `rustfmt` の形のため。オプションごとの分岐は無くなった。
