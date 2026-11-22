# 機能の仕様(Functional Spec): args.rs のオプションの表化

意図: `ideation/intent-capture/intent-statement.md`(Q1: B、Q2: A、Q3: A)。HEAD 59ca1af の `crates/cli/src/args.rs`(538 行)をもとに書く。

## 1. 方針

- 利用者から見える変更は、`convert --to` の値の誤りの対象だけ(値 → `--to`)。ほかの出力、終了コード、誤りの種類と対象、誤りの報告の順は変えない。
- オプションの性質を 1 つの表に置き、字句の段階と組み立ての段階は、その表を読んで検査する。
- 範囲の外: `CliCommand` の型の形、`resolve_run_id`、`--to` 以外の誤りの対象(知らないコマンド名、余分な位置引数、知らないオプションの綴り、UTF-8 でない位置引数)、凍結した 5 つの仕様書。

## 2. 今の形(置き換える場所の一覧)

| 場所 | 今 | 何に置き換わるか |
|------|----|------------------|
| `args.rs:65-88` | `Token` の 13 個の変種(オプションごとに 1 つ) | `Token::Opt(Opt, Option<OsString>)`、`Token::Unknown`、`Token::Positional` |
| `args.rs:133-177` | 値を取るオプションの `match` の腕 5 つと、値を取らないオプションの腕 6 つ | 表を名前で引いて、`takes_value` で分ける 1 つの腕(`-v` は短い形として残す) |
| `args.rs:201-210` | `map_lex_error` の「値を取らないオプション」の綴りの並び | 表の `takes_value == false` の行から作る |
| `args.rs:229-260` | オプションごとの数と値の変数 11 個と、それを数える `match` | オプションごとの値の並び 1 つ(`Opt` で引く)。数は並びの長さ |
| `args.rs:284-348` | 「そのコマンドで使えない」の `if` 13 個(`--force` と `--dry-run` の重複を途中に含む) | 検査の順の表 `CHECKS` の `NotAllowed` と `Duplicate` |
| `args.rs:349-376` | 重複の `if` 8 個 | 同じ `CHECKS` の `Duplicate` |
| `args.rs:529-538` | `parse_target` の誤りの対象が値 | 対象を `--to` に(Q1) |
| `dry_run.rs:32`(doc コメント)、`dry_run.rs:40-44`(分岐) | `enter_flow` の位置引数なしの分岐(doc コメントは「`parse_args` が先に止めるので来ないが、動きを保つ」) | 分岐は残し、コメントを「公開の型 `CliCommand` から来たときの守り」に(Q3) |

## 3. 表の形

### 3.1 オプションの表 `OPTIONS`

オプション 1 つにつき 1 行。列は次の 4 つ。

- `opt`: 識別子(`Opt::Verbose`、`Config`、`To`、`RunId`、`All`、`RemoveOnComplete`、`DryRun`、`Force`、`Assume`、`Format`、`State`)
- `long`: 長い名前(`verbose`、`config` …)。誤りの対象の綴りは、`Verbose` だけ `-v`、ほかは `--<long>`(今と同じ)
- `takes_value`: 値を取るか(`config`、`run-id`、`to`、`format`、`assume` が真)
- `allowed`: 使える場所。`Commands(&[CommandKind])` か、`WithDryRun`(`--assume` だけ。`--dry-run` があるときだけ使える)

`allowed` の中身は、仕様書の 04 章の 3.1 の表と `args.rs` の今の条件のとおり。

| オプション | 使えるコマンド |
|------------|----------------|
| `-v` | run、pipeline、validate、graph |
| `--config` | convert 以外 |
| `--to` | convert |
| `--run-id` | run、pipeline、watch、reset、graph(graph は、`--state` なしを後の 8a で拒否する。今と同じ) |
| `--all` | reset |
| `--remove-on-complete`、`--dry-run` | run、pipeline |
| `--force` | init |
| `--assume` | `--dry-run` があるとき |
| `--format`、`--state` | graph |

`-h`/`--help`、`-V`/`--version` は表に入れない(コマンドの解釈より先に処理する特別なもの。今と同じ)。

### 3.2 検査の順の表 `CHECKS`

仕様書の 04 章の 4.2 の 6 と 7 の順を、そのまま 1 つの並びにする。誤りの報告の順は、この並びの順で決まる。

```
NotAllowed(Verbose), NotAllowed(Config), NotAllowed(To), NotAllowed(RunId), NotAllowed(All),
NotAllowed(RemoveOnComplete), NotAllowed(DryRun), NotAllowed(Force), Duplicate(Force),
NotAllowed(Assume), Duplicate(DryRun), NotAllowed(Format), NotAllowed(State),
Duplicate(Format), Duplicate(State), Duplicate(Verbose), Duplicate(Config), Duplicate(To),
Duplicate(RunId), Duplicate(All), Duplicate(RemoveOnComplete)
```

- `NotAllowed(o)`: `o` が 1 回以上あり、`allowed` に当てはまらない → `OptionNotAllowed`(対象は `o` の綴り)。
- `Duplicate(o)`: `o` が 2 回以上 → `InvalidValue`(対象は `o` の綴り)。
- `--assume` は重複を許す(同じジョブID の重複は、後の値の検査で `--assume` の誤り。今と同じ)。

`CHECKS` のあとの検査(`--all` と `--run-id` の併用、graph の `--state` なしの `--run-id`、`--assume` と `--format` と `--run-id` の値、位置引数の過不足、convert の `--to`)は、今の順のまま、表を読む形に書き換えるだけにする。

### 3.3 字句の段階

- `Short('h')`・`Long("help")`・`Short('V')`・`Long("version")` は、`OPTIONS` を引く前に判定する(今と同じ。`special` にする)。
- `Long(name)` は `OPTIONS` を `long` で引く。見つかって `takes_value` なら `parser.value()` で値を取り、無ければ `MissingValue`(対象は綴り)で打ち切る。値を取らなければ値なしで記録する。見つからなければ `Unknown("--<name>")`(今と同じ)。
- `Short('v')` は `Verbose`、ほかの短い形は `Unknown("-<c>")`(今と同じ)。
- `map_lex_error` の「知っている、値を取らないオプション」は、`OPTIONS` の `takes_value == false` の行の `--<long>` と、`-v` から決める(今の並び `--verbose`、`-v`、`--all`、`--remove-on-complete`、`--state`、`--dry-run`、`--force` と同じ集合になる)。

## 4. `--to` の誤りの対象(Q1)

- `--to` の値が `yaml`/`toml` 以外のとき(UTF-8 でない値を含む)、`InvalidValue` の対象を `--to` にする。入力の値は含めない。
- 直す仕様書の行:
  - `docs/spec/rust/04-cli.md`: 73 行(表)、90 行(非 UTF-8 の行から `--to` の値を外し、「`--to` の値が非 UTF-8 なら対象 `--to`」に)、111 行、291 行(例の出力)
  - `docs/spec/rust/10-errors-and-exit-codes.md`: 233 行(対象を `--to` に)。234 行(非 UTF-8 の位置引数)は位置引数の話で `--to` の値を含まないので、そのまま
  - `docs/spec/language-agnostic/10-errors-and-exit-codes.md`: 241 行(対象を `--to` に。値は含めない)。243 行(非 UTF-8 の位置引数)はそのまま
  - `docs/spec/language-agnostic/04-cli.md`: 79 行、134 行、327 行(例の出力)
  - `docs/spec/language-agnostic/11-quality-and-build.md`: 611 行(AC-237)、752 行(AC-337)の期待を `invalid value: '--to'` にし、テストの列に新しいテストの名前を足す
- 凍結した 5 つの仕様書(go、python、typescript、csharp、powershell)の「対象は値」は直さない。v0.8.0 時点の凍結版と明記済みで、中身を変えない方針(前の intent で決めたこと)なので、この差は意図したもの。
- テスト: 今の `to_の値は_yaml_か_toml_だけ` は種類しか見ていないので、変えずに通る。新しく、対象が `--to` で値を含まないことを確かめるテスト(名前の例: `to_の値の誤りの対象は_to_で値を含まない`)を足す(`json`、`YAML`、空文字)。UNIX では UTF-8 でない値も確かめるテストを、`args_grammar.rs` のほかの `#[cfg(unix)]` のテストと同じ置き方で足す(`cfg` を足さない制約は `src/` の話で、`nfr_source.rs` も `src/` だけを見る。テストの `#[cfg(unix)]` は既存の置き方に倣う)。
- 版の更新の候補として記録する(版を上げるかは、リリースのときにユーザーが決める)。

## 5. テストと確かめ方

- 既存のテストは 1 行も変えない。`args_grammar.rs`(106 件)ほか、全体が通る。
- 足すテスト(`args_grammar.rs` の末尾に):
  1. 4 の `--to` の対象のテスト。
  2. 誤りの報告の順のうち、`CHECKS` で並びが入り組んだところを、今のテストが確かめていなければ足す: `--force` の重複が `--assume` の不可より先、`--assume` の不可が `--dry-run` の重複より先、「使えない」がすべての「重複」より先(例: `watch -v -v` は `-v` の不可、`graph --format a --format b -v -v` は `--format` の重複)。どれも今の動きを固定するテストで、実装の前の HEAD でも通ることを確かめてから表化する。
- 表化の確かめ: 2 の表の「今」の場所に、オプションごとの `match` の腕と `if` が残っていないこと。
- 合格の条件: fmt、clippy、全テスト、行カバレッジ 80% 以上、MSRV 1.100。新しい依存、`unsafe`、`cfg` を足さない。

## 6. コミットの分け方

1. 順のテストを足す(今の動きのまま通る)
2. `OPTIONS` と字句の段階
3. `CHECKS` と組み立ての段階
4. `--to` の誤りの対象、そのテスト、仕様書
5. `enter_flow` のコメント
6. rust の仕様書のうち `args.rs` の内部の形(`Token` など)を書いている箇所があれば合わせる

## 7. 成功の指標との対応

`traceability.json`。SM1(表化)→ BR1.1〜BR1.3、SM2(順と既存のテスト)→ BR2.1、SM3(`--to`)→ BR3.1、BR3.2、SM4(`enter_flow`)→ BR4.1、SM5(合格の条件)→ BR5.1。
