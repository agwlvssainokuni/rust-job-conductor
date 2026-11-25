# 信頼性の設計: u5-log-dir

出典: 信頼性の要件(`reliability-requirements.md` の NFR1.1、NFR1.2、NFR1.10、NFR1.11、NFR2.1〜NFR2.4、NFR3.1、NFR3.2)、技術の選択(`tech-stack-decisions.md`)、機能の仕様(`functional-spec.md` の 2〜5)、契約(`contract-summary.md` の C2、C6)。

## 1. ログの失敗を流れへ伝える(NFR1.2)

機能の設計のレビューの指摘(`StartFailed` に理由が無く、ログの失敗を見分けられない)に応えて、結果の形を決める。

- runtime の `run_job` の戻り値を、今の `JobExecution` と LogOutcome の組にする。LogOutcome は次の 3 つの値を持つ:
  - `log_not_created: Option<パス>`: 起動の前にログを作れなかった(このとき `JobExecution` は今の `StartFailed`)。
  - `write_failed: bool`: 実行中にファイルへの書き込みに失敗した。
  - `output_cut_short: bool`: 出力を待ち切れなかった。
- cli の流れは、`log_not_created` があれば BR5.1 の 1 行だけを出し、今の起動の失敗の表示(`StartFailure`)は出さない。終了コードとリトライの扱いは今の `StartFailed`(127)と同じ。
- `log_dir` が無いとき、LogOutcome はすべて空で、今と同じ。
- 確かめ方: 流れのテスト(偽の `Runtime` が `log_not_created` を返す)で、標準エラーが BR5.1 の 1 行だけで、終了コードが 127 であること。

## 2. 読み取りのスレッドの止め方(NFR1.10)

- 各スレッドは「止める」印(`Arc<AtomicBool>`)と、ファイルへの書き込みの印を見る。
- 端末への書き込みに失敗したら、そのスレッドは端末に書くのをやめるが、パイプは読み続ける(ファイルには書き続ける)。子は止めない。
- ファイルへの書き込みに失敗したら、`write_failed` を立て、以後ファイルには書かない(端末には書き続ける)。
- 待ち切れたら、流れに戻る前に「止める」印を立てる。スレッドは次の読み取りの後で印を見て、どこにも書かずに読み捨てて終わる。これで次のジョブの出力に混ざらない。
- 確かめ方: 端末を閉じた子プロセスとして conductor を動かすテスト(UNIX 系)、孫が残るテスト(UNIX 系)、ファイルへの書き込みの失敗(読み取り専用にした後の書き込みは OS で揺れるため、書き込みの失敗を起こす差し替えの `Write` を runtime の内部の単体テストで渡す)。

## 3. シグナル(NFR1.11)

- 待ちのくり返しとシグナルの転送は今の関数をそのまま使う。読み取りのスレッドは、待ちのくり返しとは別に動く。
- 確かめ方: 今の `signals_real.rs` に、`log_dir` を書いた定義での中断を 1 件足し、終了コードが今と同じこと。

## 4. 後方互換(NFR1.1)

- `log_dir` が無いときは、今の `run_job` の経路をそのまま通る。今のテストを変えない。

## 5. テストの配置(NFR2.1〜NFR2.4、NFR3.1、NFR3.2)

| テストの場所 | 確かめること |
|---|---|
| core の `time.rs` の `mod tests` と `crates/core/tests/time_format.rs`(新規) | 整形・詰めた形・読み取り・差、うるう年と境界の表 |
| `crates/core/tests/definition_valid.rs`、`definition_errors.rs` | `log_dir` の検証 |
| runtime の `log.rs` の `mod tests` と `crates/runtime/tests/log_file.rs`(新規) | 安全な名前の表、衝突の番号と上限、ディレクトリの作成、0600(UNIX 系)、作れないとき |
| runtime の `runner.rs` の `mod tests` と `crates/runtime/tests/runner_log.rs`(新規) | 出力の複製(中身、順、両方の流れ)、大量の出力、書き込みの失敗、待ち切り、止める印 |
| `crates/cli/tests/flow_run.rs`・`flow_pipeline.rs`・`flow_retry.rs` | 偽の `Runtime` で、試行ごとの時刻とログの指定、127 の表示、警告 |
| `crates/cli/tests/binary_log.rs`(新規) | 実バイナリで、ファイルの中身、名前、リトライで 3 つ、作れないときの 127 |
| `signals_real.rs` | `log_dir` ありの中断 |
| `unix_demo.rs`、`windows_demo.rs` | デモの場面 |

- 時計は cli の `Runtime::now()` を偽にして決める。runtime の時計の本物の関数は 1 行で、テストは「1970 年より後の値を返す」だけを確かめる(NFR2.4)。
- 行カバレッジ 80% は確認のコマンド、3 つの OS と MSRV は CI。
