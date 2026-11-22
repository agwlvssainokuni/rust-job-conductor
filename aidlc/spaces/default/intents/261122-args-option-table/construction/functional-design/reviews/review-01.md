## Review

**Verdict:** READY
**Reviewer:** aidlc-architecture-reviewer-agent
**Iteration:** 1
**Verdict Dictated:** no

### 確かめた結果(4 つの観点)

1. 誤りの報告の順: `CHECKS` の 21 項目を、`args.rs` 284-376 行の `if` の順と 1 つずつ突き合わせた。完全に一致する(`--force` の不可 → `--force` の重複 → `--assume` の不可 → `--dry-run` の重複 → `--format`/`--state` の不可 → 重複 8 個の順。`--dry-run` の不可が `--assume` より先なのも同じ)。`--assume` の `WithDryRun` は、今の `assume 非空 && dry_run_count == 0` と同じで、`watch --dry-run --assume a=1` は今も表も `--dry-run` の不可が先。graph の `--run-id` は、`NotAllowed(RunId)` では graph を許し、後段(8a)で `--state` なしを拒むので今と同じ。`--all` と `--run-id` の併用は `CHECKS` の外に残す設計で、今(377 行)と同じ位置。順が変わる入力は見つからなかった。
2. 字句の段階: `-h`/`--help`/`-V`/`--version` を `OPTIONS` の引きより先に処理する限り、今と同じ。値を取る 5 つの打ち切り(`MissingValue` の対象の綴りと `break`)、`--name=value`(`parser.value()` が受ける)、値を取らないものへの `=x`(次の `next()` が `UnexpectedValue`)、`Short(c)` の `-c` 表記は、設計の記述で今と同じ挙動になる。`map_lex_error` の集合は `takes_value == false` の 6 行(`--verbose`、`--all`、`--remove-on-complete`、`--state`、`--dry-run`、`--force`)と `-v` で、今の 7 個と同じ集合。
3. `--to` の変更箇所: 下の R-1(漏れ 1 件)と R-4 を参照。README(229、242、256、515 行)、`messages.rs` の使い方の文、cli の `tests/`(`args_grammar.rs`、`flow_aux.rs`、`nfr_robust.rs`、`binary_basic.rs` など)は、`--to` の値が対象に出ることを前提にしていない。`to_の値は_yaml_か_toml_だけ` は種類だけを見るので、変えずに通る。
4. 制約: 新しい依存なし、`unsafe` なし、`src/` に `cfg` なし、エラーに入力の値を含めない(対象を `--to` の固定の語にする)、テスト名は日本語、の各点と矛盾しない。ただし R-2 の言い回しを参照。

### Findings

| ID | Severity | Location | Finding | Required action | Status |
|---|---|---|---|---|---|
| R-01 | Major | aidlc/spaces/default/intents/261122-args-option-table/construction/functional-design/functional-spec.md > 4 直す仕様書の行 | 直す行の一覧に `docs/spec/language-agnostic/10-errors-and-exit-codes.md` の 241 行(表「`--to` の値が `yaml`/`toml` 以外 \| `InvalidValue` \| その値」)が無い。rust 側の 233 行は入っている。このまま実装すると、language-agnostic の 04 章と 11 章は「対象は `--to`」、10 章は「その値」と食い違う。同じ表の 243 行(非 UTF-8 の位置引数)は、`--to` の値を含まなくなるので文言の確認も要る。rust/10 の 234 行も同じ。 | 一覧に language-agnostic/10 の 241 行を足す。243 行と rust/10 の 234 行は「`--to` の値を除く」旨を確認して直すか、直さない理由を書く。BR3.2 の `applies_to` にも 10 章(language-agnostic)を足す。 | New |
| R-02 | Minor | aidlc/spaces/default/intents/261122-args-option-table/construction/functional-design/rules.md > BR5.1、functional-spec.md > 5 合格の条件 | 「`cfg` を足さない」と、4 の「UNIX では UTF-8 でない値の確認を、ほかの `cfg(unix)` のテストと同じ置き方で足す」が、字面で食い違う。`nfr_source.rs` が見るのは `src/` だけで、テストの `#[cfg(unix)]` は検査の対象外(既存のテストも使っている)ので、実害はない。 | 「`src/` に `cfg` を足さない(テストの `#[cfg(unix)]` は除く)」と書き分ける。 | New |
| R-03 | Minor | aidlc/spaces/default/intents/261122-args-option-table/construction/functional-design/functional-spec.md > 2 の表(`dry_run.rs:40-44`)、4 | 「コメント」は 40-44 行の分岐ではなく、32 行の doc コメント(「`parse_args` が先に止めるので来ないが」)にある。行の指定だと、コメントの場所を取り違える。 | 場所を「`dry_run.rs:32`(doc コメント)と 36-41 行の分岐」と直す。 | New |
| R-04 | Minor | aidlc/spaces/default/intents/261122-args-option-table/construction/functional-design/functional-spec.md > 4 直す仕様書の行 | language-agnostic/11 の AC-237(611 行)と AC-337(752 行)は、期待の `invalid value: 'json'`/`'YAML'`/`'Toml'` に加えて、「テスト」の列が `args_grammar::to の値は yaml か toml だけ` を指す。足す新しいテストを、この列に結びつける記述が無い。 | 新しいテストの名前を決め、AC-237 と AC-337 のテストの列に足す。 | New |
| R-05 | Minor | aidlc/spaces/default/intents/261122-args-option-table/construction/functional-design/functional-spec.md > 1 方針 | go、python、typescript、csharp、powershell の仕様書(凍結)は「対象は値」のまま残る(例: go/04-cli.md の 83、164、194 行)。language-agnostic が先に変わると、5 つの言語別の仕様書と食い違うが、一覧のどこにもその旨が無い。 | 「凍結した 5 つは変えず、language-agnostic との差は意図したもの」と 1 行記す(版の更新の候補の記録に含めてもよい)。 | New |
| R-06 | Minor | aidlc/spaces/default/intents/261122-args-option-table/construction/functional-design/functional-spec.md > 3.3 | 字句の段階で、`-h`/`--help`/`-V`/`--version` の処理を `OPTIONS` の引きより前に置くことが明記されていない(今は `match` の先頭 2 腕)。表の引きを先にすると、`--help` が `Unknown` になりうる。 | 「特別なオプションを先に判定し、そのあとで `OPTIONS` を引く」と明記する。 | New |

### Validation Tool Results

| Tool | Result | Interpretation |
|---|---|---|
| 目視の突き合わせ(`CHECKS` と `args.rs` の `if` の順) | 一致 | 順が変わる入力は見つからなかった |
| `grep` による `--to` の参照の洗い出し(README、docs、crates) | language-agnostic/10 の 241 行が一覧から漏れ | 上の Major |

### Summary

設計は、今の報告の順と字句の段階の挙動を、そのまま写せている。`--to` の対象の変更について、language-agnostic/10 の 241 行が直す一覧から漏れている点が 1 件(Major)あるが、Major が 2 件以下なので判定は READY。残りは Minor で、一覧と言い回しの整理で足りる。
