## Review

**Verdict:** READY
**Reviewer:** aidlc-architecture-reviewer-agent
**Date:** 2026-11-27T16:32:14Z
**Iteration:** 1
**Verdict Dictated:** no

### Findings

| ID | Severity | Location | Finding | Required action | Status |
|---|---|---|---|---|---|
| R-01 | Major | aidlc/spaces/default/intents/261127-v0102-spec-docs/construction/functional-design/functional-spec.md > 3.1 rust の書き換え、4 章の確かめのコマンド | 書き換えの一覧は rust だけで、language-agnostic の逆の記述は行ごとに挙がっていない。実際に残っている行は、`00-README.md:91`(環境変数の層)、`01-overview.md:67`(ジョブの出力の保存・`unset` はない)、`05-execution-flow.md:535`(ステートは最終の結果で 1 回だけ)、`05-execution-flow.md:646`(取り除く書き方はない)、`07-process-and-signals.md:243`(出力を読まない)、`07-process-and-signals.md:471`(「取り除かない」の規定の行)、`03-definition-format.md:688`(33 種)、`10-errors-and-exit-codes.md:139`(33 種)、`12-rebuild-guide.md:39・121・248`(33 種)、`00-README.md:24`(33 種・13 種)、`00-README.md:32` と `11-quality-and-build.md:11`(場面 0〜14)。4 章のコマンドも、33 種・13 種の `grep` と `log_dir` などの出現の確かめだけで、「足すだけ」「取り除かない」「読まない」「保存しない」「1 回だけ書」の逆の記述が language-agnostic に残っていないことは確かめられない。R-01 の受け入れた危険が、language-agnostic 側では行ごとの確かめになっていない。 | 3.1 に language-agnostic の表(上の行を出発点に、行・今の記述・書き換えた後)を足す。4 章に、`grep -n '足すだけ\|取り除かない\|取り除く書き方\|読まない・保存しない\|1 回だけ書\|0〜14' docs/spec/` が、履歴として残す行だけを返す確かめを足す。 | New |
| R-02 | Major | aidlc/spaces/default/intents/261127-v0102-spec-docs/construction/functional-design/functional-spec.md > 3.2 既存の AC の書き換え(と洗い出しの手順) | 一覧が実装と合わない AC を拾い切れていない。`11-quality-and-build.md` の AC-146(ステートの内容を `[[job]]` の項目で列挙。`[[job.attempt]]` と時刻が無い)と AC-147(キーの並び)、AC-241(ステートの形)、AC-305〜308(ステートの読み込みの誤り。`NO_ATTEMPT` などの新しい理由が 4.11 だけでなくここにも関わる)は、一覧に無い。洗い出しの語(`watch`、リトライ、ステート、`env`、`env_file`、`graph`、`--format`、デモ、終了コード)に、`出力`、`ログ`、`log_dir`、`unset_env`、`--follow`、`--interval`、`試行`、`時刻` が無く、ジョブの出力の扱い(AC-099 付近、07 章の AC)や `--follow` の AC を洗い出せない。 | 洗い出しの語に上の語を足し、AC-146・147・241・305〜308 を 3.2 の表に足すか、洗い出しの結果として確かめる対象に名指しで挙げる。 | New |
| R-03 | Minor | aidlc/spaces/default/intents/261127-v0102-spec-docs/construction/functional-design/functional-spec.md > 3.1 の 13 種の行 | 「02:106、03:391・563、10:121、11:213、12:321 ほか」は、実際の行(`02:298`、`11:291`、`12:161` が漏れ)と、`03:563` の記述(`13 種` の語の有無)を確かめた形になっていない。また「33 行」の表と、`rust/12:105`(履歴)や `rust/03:9` の 33 種の有無の仕分けが無い。 | 「ほか」をやめ、行を `grep` の結果から全部挙げて、履歴として残す行に印を付ける。 | New |
| R-04 | Minor | aidlc/spaces/default/intents/261127-v0102-spec-docs/construction/functional-design/functional-spec.md > 1 作業の順序、行 22(順 14) | 新しい AC(4.21 以降、AC-519〜)を「1 コミット(大きければ節ごと)」とする。v0.10 の機能は 7 つ(B1〜B7)あり、4.17〜4.20 のように機能ごとに節を分ける前例があるので、1 コミットでは小さな単位のレビューに合わない。また、順 12 の章のコミットが、順 14 で振る AC 番号を先に参照する並びになっている(AC の番号は後から入れる必要がある)。 | 順 14 を機能の節ごとのコミットと決める。章のコミットでの AC の参照は、順 14 のあとに番号を入れるか、先に番号の範囲を決めておくと書く。 | New |

### Validation Tool Results

| Tool | Result | Interpretation |
|---|---|---|
| 4 章の数の突き合わせ(実装を読んだ) | PASS | `PIPELINE_KEYS` 9、`JOB_KEYS` 9、`DefinitionErrorKind` 34、`EnvFileErrorKind` 14、ステートのキー 10・5・2(`reader.rs`)、試行の誤りの理由 3 つ、デモの場面 0〜15、`#[test]` 1,460、統合テストのファイル 36・16・26・2 がすべて一致。 |
| 3.1 の行番号の突き合わせ | PASS | `00:75`、`01:64`、`05:531・634・652`、`07:205・206`、`12:182`、`02:121`、`03` の 9.3 の見出しの記述が、今のファイルと合う(`rust` の 03 の表はすでに 34 種の見出し)。 |
| AC の番号の体系 | PASS | 既存の最大は AC-518 で、AC-519 からの続きは矛盾しない。`12-rebuild-guide.md` の AC の範囲の表は 13 行目(AC-503〜513、デモ 514〜518)で終わっており、15 番の作業で行を足す形で整合する。 |

### Summary

数と行番号は実装と一致し、AC-519 の続きの振り方と 12 章の扱いも今の体系と矛盾しない。ただし language-agnostic の逆の記述が行ごとの一覧と確かめになっておらず、既存の AC の洗い出しも語と一覧に漏れがあるため、設計の段階で上の 2 つを補うことを強く勧める(Major は 2 件で、READY の範囲内)。
