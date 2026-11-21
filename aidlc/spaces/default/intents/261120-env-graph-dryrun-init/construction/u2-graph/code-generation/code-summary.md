# コードの要約(Code Summary): U2 u2-graph(`conductor graph`)

## 作った・変えたファイル

- `conductor-core`: `flow/next_step.rs`(遷移の解決 `resolve_transition` と `TransitionResolution` を公開。`classify_and_decide` の結果は変えない)、新しいモジュール `graph/`(`mod.rs`: 型、`edges.rs`: 辺の取り出し、`overlay.rs`: 図の構造と状態の重ね合わせ、`layout.rs`: 「対応する形」の判定とレーンの配置)。
- `conductor-cli`: `args.rs`、`types.rs`(`graph`、`--format`、`--state`、`-v`、`--run-id` の文法)、`report/graph_text.rs`(text、辺の一覧、`-v`、状態)、`report/graph_mermaid.rs`、`report/messages.rs`、`report/mod.rs`、新しい `graph.rs`(流れ)、`aux_commands.rs`(`validate` の検査を共通の関数 `check_definition_and_env` に切り出し)、`ctx.rs`、`lib.rs`。
- `README.md`、`demo/`(場面 12)。
- 新しいテスト: `core` 6 ファイル(31 件。レイアウトの正解データ 537 例を含む)、`cli` 4 ファイル(22 件)。既存のテストの修正(`args_grammar.rs`、`report_messages.rs`、`unix_demo.rs`、`windows_demo.rs`)。
- 変えていない: `Cargo.toml`、`Cargo.lock`、CI の設定(新しい依存なし)。

## 主な実装の決定

- 辺は、実行のときに次のジョブを決める遷移の解決と、同じ関数から取り出す(`resolve_transition`)。テストで、すべての定義、すべての結果について、`classify_and_decide` と辺の行き先が一致することを確かめた(FR2.4)。
- 対応する形(各ジョブの飛び越しが高々 1 つ、行き先が重ならない、交差しない)は、git と同じ見た目のレーンで描き、`layout-oracle.md` の 537 個の例のすべてに、1 文字も違わず一致する。対応しない形は、text では辺の一覧で出す(ユーザーの選択)。Mermaid は常に出せる。
- `graph` は `validate` と同じ検査(定義、`command`、`.env`)を行う。`--state` は、完了したステートも含めて読み(`read_state_raw`)、書き込みも削除もしない。
- ラベルの位置と `\` の出方は、正解データから確かめた規則で実装した(規則は `layout.rs` のドキュメントコメントに書いてある)。

## テストの結果(macOS)

- `cargo fmt --all -- --check`: 通った。`cargo clippy --workspace --all-targets --locked -- -D warnings`: 警告なし。
- `cargo test --locked --workspace`: 922 件が通り、失敗 0 件(開発担当の報告)。
- `cargo llvm-cov --workspace --locked --ignore-filename-regex 'bin/conductor-(cli-)?test-helper' --fail-under-lines 80 --summary-only`: 行カバレッジ **97.96%**(8445 行のうち未通過 172 行)。承認された検証のコマンドを、オーケストレーターが再実行して、同じ数値を確かめた。しきい値は下げていない。
- 新しいコードの行カバレッジ: `graph/edges.rs` と `overlay.rs` 100%、`graph/layout.rs` 98.3%、`report/graph_text.rs` 99.4%、`report/graph_mermaid.rs` 95.9%、`cli/src/graph.rs` 94.2%。

## 書いたが、実行していないテスト(理由つき)

- `crates/cli/tests/windows_demo.rs`(場面 12 のテスト)と `demo/windows/run-demo.cmd` の場面 12: `#![cfg(windows)]` で、この macOS では実行も、コンパイルもされない。CI の `test`(windows)で確認する。
- ubuntu の `test` と、最小の Rust 1.100.0 の `msrv-test`: この環境では実行できない。CI で確認する。

## レビューの指摘への対応

- (Major) `demo/windows/run-demo.cmd` の改行が、CRLF から LF に変わっていた。CRLF に戻し(コミット済み)、変更前との差分が、場面 12 の 21 行の追加だけになったことを `git diff --stat` で確かめた。Windows の CI(`test`)で、場面 12 の `windows_demo.rs` が通ることは、この環境では確かめられない。マージ前に、CI で確認する。

## 既知の限界(レビューの助言。受け入れた)

- 性能のテスト(`graph_perf.rs`)は、辺の取り出しとレーンの配置だけを、1,000 ジョブで測る。text と Mermaid の整形(`-v` を含む)は、1,000 ジョブでは測っていない(整形は、行 × 列に比例する単純な処理)。
- text のジョブ ID とコマンドの制御文字、長い ID のテストは薄い。Mermaid のラベルのエスケープは、`char::is_control`(制御文字)までで、U+2028 や双方向の制御文字は逃がさない(text 側は逃がす)。
- レーンの配置の正解データの例は、成功の遷移だけで作られている。警告だけの飛び越しが同じ図になることは、辺のテストで間接的に確かめている。

## 計画からの逸脱

1. `\` の描き方: 設計の手順の説明だけでは、条件を決められなかったため、正解データで確かめた規則で実装した(設計の「例が正しい」の方針どおり)。
2. 計画の Assumptions のとおり、`GraphLayout`(`Lanes` か `EdgeList`)を足した。`LaneGlyph::cross` と `LaneRow::note_target` は作っていない。`DisplayEvent::Graph(&str)` を足した。
3. 計画 10.3 の「`ALL` の網羅の配列」は、`messages.rs` にはなく、網羅は `report_messages.rs` の `with_all_events` にあるので、そこに足した。
4. `--run-id` を `--state` なしで付けたときは、`OptionNotAllowed`(対象 `--run-id`)として終了コード 68。
5. 作業手順の違反(開発担当の報告): ファイルの作成・編集の一部を、シェル(`printf`、`python3`、`sed -i`)で行った。内容は、検査とテストで確認済み。
6. 14.3 の 3 ファイル(このファイル、`traceability.json`、`source-manifest.json`)は、オーケストレーターが作った。
