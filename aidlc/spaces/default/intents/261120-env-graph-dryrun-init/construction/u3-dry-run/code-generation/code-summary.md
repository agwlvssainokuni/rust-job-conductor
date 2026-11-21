# コードの要約(Code Summary): U3 u3-dry-run(`--dry-run` と `--assume`)

## 作った・変えたファイル

- `conductor-core`: 新しいモジュール `plan/`(`mod.rs`: 計画の型、`simulate.rs`: 模擬と計画の生成)、`flow/decide.rs`(下の「計画からの逸脱」の 2)、`lib.rs`。
- `conductor-cli`: `args.rs`、`types.rs`(`--dry-run`、`--assume` の文法と検査)、`ctx.rs`(`prepare` から、シグナルの準備を除いた検査を `prepare_checked` に切り出し)、`pipeline.rs`、`run.rs`(`--dry-run` の分岐)、新しい `dry_run.rs`(流れ)、`report/plan_text.rs`(計画の整形)、`report/messages.rs`、`report/mod.rs`、`lib.rs`。
- `README.md`、`demo/`(場面 13)。
- 新しいテスト: `core` 5 ファイル(23 件)、`cli` 4 ファイル(23 件)。既存のテストの修正(`args_grammar.rs`、`report_messages.rs`、`nfr_robust.rs`、`unix_demo.rs`、`windows_demo.rs`)。
- 変えていない: `Cargo.toml`、`Cargo.lock`、CI の設定(新しい依存なし)。

## 主な実装の決定

- 計画は、`pipeline` の流れと同じ判断(`decide_before_run`、`classify_and_decide`)を、実行の代わりに、仮定の終了コード(既定は 0)で進めて作る。ステートは、`read_state`(本番と同じ)で読むだけで、書き込みも削除もしない。シグナルの準備もしない。
- 定義と `command`、`.env`、ステートの誤りは、本番と同じ誤りと終了コード(64、65)で止まる。拒否は 66。`--assume` の誤りは 68 で、誤りの表示に、入力の値(ジョブ ID、終了コード)を含めない。
- 環境変数は、名前と出どころだけを計画に持たせ、値は、型にも、出力にも出ない。
- 出力の例 P1〜P8 と、英語の例 P2e、P4e は、テスト(`plan_report.rs`)で、1 文字も違わず固定した。README の出力例は、`docs_dry_run.rs` が実際の出力と照合する。

## テストの結果(macOS)

- `cargo fmt --all -- --check`: 通った。`cargo clippy --workspace --all-targets --locked -- -D warnings`: 警告なし。
- `cargo test --locked --workspace`: 972 件が通り、失敗 0 件(開発担当の報告)。
- `cargo llvm-cov --workspace --locked --ignore-filename-regex 'bin/conductor-(cli-)?test-helper' --fail-under-lines 80 --summary-only`: 行カバレッジ **97.99%**(9098 行のうち未通過 183 行)。承認された検証のコマンドを、オーケストレーターが再実行して、同じ数値を確かめた。しきい値は下げていない。
- 新しいコードの行カバレッジ: `core/src/plan/simulate.rs` 98.5%、`cli/src/report/plan_text.rs` 100%、`cli/src/dry_run.rs` 86.3%(未通過は、定義の検査で済んでいるため通常は起きない、`command` の分割の誤りの経路)。

## 書いたが、実行していないテスト(理由つき)

- `crates/cli/tests/windows_demo.rs`(場面 13 のテスト)と `demo/windows/run-demo.cmd` の場面 13: `#![cfg(windows)]` で、この macOS では実行も、コンパイルもされない。CRLF のまま編集したこと(`git ls-files --eol` が `i/crlf w/crlf`、差分が追加した行だけ)は確かめた。CI の `test`(windows)で確認する。
- ubuntu の `test` と、最小の Rust 1.100.0 の `msrv-test`: この環境では実行できない。CI で確認する。
- `pipeline` の模擬で `Reject`(66)が起きる分岐: 模擬は、前のジョブを必ず済んだ状態にして進めるため、公開の入力からは起こせない(本番の `pipeline` の流れも同じ)。防御の分岐として実装し、テストしていない。`run` の 66 は、テストした。

## 既知の限界(レビューの助言。受け入れた)

- `decide_before_run` の `status_at` の最適化は、整合性の検査を通ったステート(定義の順で、重複がない)では、元の判断と同じ結果になる。順序の違いや重複のあるステートでの同等性は、議論しただけで、テストしていない(実際の入力では、整合性の検査が除く)。
- `nfr_robust.rs` の上限(3,000 文字)は、固定の使い方の一覧の長さに対して、緩い。長い入力を、そのまま表示しないことの検査は、残っている。
- 「`pipeline` の模擬で `Reject` が起きない」のは、整合性の検査を通ったステートについての話で、記録が欠けたステートでは起きうる(その場合は、パニックせず、拒否 66 を返す)。
- `windows_demo.rs` と `demo/windows/run-demo.cmd` の場面 13 は、この環境では実行できていない。マージ前に、Windows の CI で確認する。

## 計画からの逸脱

1. `prepare_checked` の切り出し(計画 8.1): 計画は「切り出した関数を呼んだあとにシグナルを準備する」と読めたが、それでは、本番の順序(シグナルの準備が、定義の検査より前)が変わる。本番の `prepare` は、シグナルの準備のあとに、`prepare_checked` を呼ぶ形にして、順序を保った。コミットの順は、計画の「先に単独で」と違い、引数のコミットのあとになった。
2. `decide_before_run` の最適化(既存の `core` のコード。`flow/decide.rs`): 1,000 ジョブの計画が、debug ビルドで約 0.95 秒かかり、1 秒の上限に近かった(「前のジョブがすべて済んでいるか」の検査が、線形探索で、全体が 3 乗になるため)。記録が同じ位置にあれば直接引く小さな関数 `status_at` を足し、約 49ms になった。判断の結果は変えていない(既存の単体テストと流れのテストがすべて通った)。本番の実行にも使われる関数なので、レビューで確かめる。
3. `--assume` の誤りに、新しい `UsageErrorKind` は足していない(`InvalidValue`、`OptionNotAllowed`、`UnknownJob`(ジョブ ID なし)を使った)。
4. `PlanRejection` は `{ job_id, reason }` とした。`DisplayEvent::Plan(&str)` を足した。
5. 英語の語句で、設計にない `result (simulated): completed` を足した。使い方の表記から、省略記号 `...` を外した(既存のテストが、不正な run ID `..` が表示に含まれないことを確かめるため。複数回指定できることは、説明に書いた)。`nfr_robust.rs` の「長い入力は切って表示する」の上限を、使い方の一覧が長くなったため、2,000 から 3,000 文字にした(カバレッジのしきい値ではない)。
6. 作業手順の違反(開発担当の報告): ファイルの作成・編集の一部を、シェル(`python3`、`sed -i`)で行った。内容は、検査とテストで確認済み。`git stash` を一度試みたが、拒否された。
7. 11.3 の 3 ファイル(このファイル、`traceability.json`、`source-manifest.json`)は、オーケストレーターが作った。
