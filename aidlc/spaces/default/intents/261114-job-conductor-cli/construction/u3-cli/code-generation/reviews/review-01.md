## Review

**Verdict:** READY
**Reviewer:** aidlc-architecture-reviewer-agent
**Date:** 2026-11-15T00:57:19Z
**Iteration:** 1
**Verdict Dictated:** no

### Findings

| ID | Severity | Location | Finding | Required action | Status |
|---|---|---|---|---|---|
| R-01 | Major | crates/u3-cli/src/run.rs:96-101; crates/u3-cli/src/pipeline.rs:101-107 | `run_job` が返したあとの結果の書き込みは、いつも `StateContext::JobExecuted` で行う。シグナルが起動より前に届いて `execution.started == false` のとき(BR4.2a。ジョブは起動していない)に、この書き込みが失敗すると、標準エラーに「ジョブは実行されました」(`the job has already been executed`)と出る。実際にはジョブは動いていない。at-least-once の文言(「起動していません」と「実行されました」の使い分け)が、この経路では誤っている。この経路を確かめるテストもない(`tests/flow_run.rs` の 346 行目以降と `tests/flow_pipeline.rs` の 147 行目以降の書き込み失敗のテストは、ジョブが起動した場合だけ) | `execution.started` が false のときは、`StateContext::JobNotStarted` で書き込みを報告する(`run.rs` と `pipeline.rs` の両方)。起動前の中断の結果の書き込みが失敗する流れのテストを、`run` と `pipeline` に 1 つずつ追加し、文言が「起動していません」になることを確かめる | New |
| R-02 | Minor | crates/u3-cli/src/platform/unix.rs:16-20; crates/u3-cli/tests/binary_signal.rs:116-117 | 監視は `signal-hook` の `Signals::forever()` で、2 つのシグナルが同じ起床の間に届いて保留されると、到着順ではなく番号の小さい順に返る(保留の走査が番号順)。「最初の番号が残る」は、番号の小さい方が残る場合がある。テストは HUP(1)のあとに INT(2)を送る順番なので、この差を見分けられない(どちらの実装でも通る) | 文書(README のシグナルの節、`InterruptSignal` の説明)を「ほぼ同時に届いた場合は、番号の小さい方」と正確にするか、INT のあとに HUP を送る順番も確かめて、挙動を明記する | New |
| R-03 | Minor | crates/u3-cli/src/main.rs:1-12; crates/u3-cli/src/bin/conductor-cli-test-helper.rs:1 | `#![forbid(unsafe_code)]` は `lib.rs` にだけあり、実行ファイルのクレートの根(`main.rs` と補助の実行ファイル)にはない(別のクレートなので、`lib.rs` の指定は及ばない)。`tests/nfr_source.rs` の文字列検索が、`unsafe` を入れないことを守っているが、コンパイラによる保証ではない | `main.rs` と補助の実行ファイルにも `#![forbid(unsafe_code)]` を足すか、ワークスペースの `[lints]` で一括して指定する | New |
| R-04 | Minor | README.md:62-64(ソースからのビルド); crates/u3-cli/Cargo.toml:20-24 | ビルドの手順 `cargo build --release --locked -p conductor-cli` は、テスト専用の `conductor-cli-test-helper` も `target/release/` に作る(BR1.5 の「テスト専用の補助の実行ファイルは含めない」。code-summary 自身が、cicd-pipeline の「含まれない」の記述が不正確だと記録している)。配布物には入らないが、利用者の手順で余分な実行ファイルができる | README のビルド手順を `cargo build --release --locked --bin conductor` にする。cicd-pipeline の記述の訂正は、CI のステージで確実に行う | New |
| R-05 | Minor | aidlc/spaces/default/intents/261114-job-conductor-cli/construction/u3-cli/code-generation/traceability.json(AC1.3.3、BR4.2、BR4.3、AC2.5.6、BR3.5) | 対応先のファイルが実際の実装と食い違うものがある。AC1.3.3(定義の誤りで 64)は `run.rs` になっているが、実装は `ctx.rs` の `prepare`。BR4.2・BR4.3 は `platform/unix.rs` になっているが、判断は `pipeline.rs` と `ctx.rs`(unix.rs は監視だけ)。Windows の経路(`platform/windows.rs`)はどこにも対応づけられていない | 実際に実装しているファイル(`ctx.rs`、`pipeline.rs`、`platform/windows.rs`)に、対応先を直す | New |
| R-06 | Minor | aidlc/spaces/default/intents/261114-job-conductor-cli/construction/u3-cli/code-generation/code-summary.md(実行していないテスト・測定) | 行カバレッジ 80% の下限は未測定(`cargo llvm-cov` 未導入)で、`main.rs` と標準出力・標準エラー・環境の本番の実装は、バイナリを起動するテストでだけ通る。Windows の Ctrl+C は手動のチェックリストだけ。いずれも code-summary に正直に書かれている。Build and Test に確実に引き継ぐこと | Build and Test で、`cargo llvm-cov` による測定と、リリース前のチェックリストの実施を確かめる | New |

### Validation Tool Results

| Tool | Result | Interpretation |
|---|---|---|
| `cargo fmt --all -- --check` | PASS | 整形は一貫している |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | PASS(警告なし) | 静的検査に問題なし |
| `cargo test -p conductor-cli --locked` | PASS(182 件、失敗 0、無視 0。性能のテストを含めて約 7 秒) | code-summary の件数と一致。実シグナルのテストも 1 回の実行では安定していた |

実行順序(検査、unknown job id の 68、ステートの照合、判断、初期ステートは Run のあとだけ、`Durable` の書き込み、完了で初期化)、終了コードの規則(0〜255、エラーが 0 にならない)、引数の文法(`--config=`、`--`、`-vv`、help の優先)、シグナルの準備が検査より前であること、`job` キーの置き換え(u1-core の `walk.rs` の `ROOT_KEYS`、fixtures、u4 の fixtures、README)、u2-runtime と u4 の `src/` が変わっていないことは、コードとテストで確かめた。

### Summary

流れ、終了コード、引数、シグナル、表示は、設計どおりに実装されており、テストも実質的である。見つかった主な問題は、起動前の中断のあとの書き込みの失敗で、「ジョブは実行されました」と誤って表示する経路が 1 つあること(Major 1 件。修正は小さい)。そのほかは Minor で、READY とするが、Major の修正を推奨する。
