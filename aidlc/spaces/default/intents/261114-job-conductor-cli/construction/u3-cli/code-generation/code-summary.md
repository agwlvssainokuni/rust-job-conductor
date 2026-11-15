# Code Summary(u3-cli)

18 ステップをすべて完了した。`cargo fmt --all -- --check`、`cargo clippy --workspace --all-targets --locked -- -D warnings`、`cargo test --workspace --locked`(ワークスペース全体で 564 件成功、失敗 0、無視 1(u1 の以前からのもの))が通った。この Unit は 185 件(計画の約 130 件より多い。レビュー 1 回目の修正で 3 件増えた)。u1-core・u2-runtime・u4-converter のテストは、キー名の変更(Step 2)の前と同数(115・152・112)で、すべて成功した。

## 作成・変更したファイル

- **新規 `crates/u3-cli/`(パッケージ `conductor-cli`、実行ファイル `conductor`):** `Cargo.toml`、`src/{lib,main,types,args,exit,deps,ctx,run,pipeline,aux,panic_hook}.rs`、`src/platform/{mod,unix,windows}.rs`、`src/report/{mod,escape,messages,watch}.rs`、`src/bin/conductor-cli-test-helper.rs`(テスト専用)。
- **テスト:** `tests/{args_grammar,exit_codes,report_messages,escape_table,flow_run,flow_pipeline,flow_aux,panic_hook,signals_real,binary_basic,binary_signal,nfr_source,nfr_robust,nfr_perf}.rs`、`tests/common/mod.rs`、`tests/bin_support/mod.rs`、`tests/WINDOWS_CTRL_C_CHECKLIST.md`。
- **新規 `README.md`(ルート)。** 概要、インストール(SHA-256 の確認、macOS の警告の回避)、使い方、定義ファイルの書き方、終了コードの表、ステートファイルの場所、保証の範囲(at-least-once)、シグナル、convert、言語。
- **ワークスペース:** `Cargo.toml`(`[profile.release]`)、`Cargo.lock`。
- **u1-core(Step 2。契約 E2 のキー名の変更): ** `src/definition/{walk,depth}.rs`、`tests/*.rs`(6 ファイル)、fixtures(2 つ)。定義の表のキーが `[[job]]` / `[[job.transitions]]` になった。
- **u4-converter:** `tests/` の定義の例(fixtures 4 つ、`convert_ac.rs`、`convert_roundtrip.rs`)だけ。`src/` は変えていない。**u2-runtime は変えていない**(ステートファイルの `[[jobs]]` は、定義のキーとは別の形式)。

## Step 1 の確認結果

- 使った版: `lexopt 0.3.2`、`signal-hook 0.4.6`(`default-features = false`、feature `iterator`)、`ctrlc 3.5.2`、`tempfile 3.27.0`。`cargo metadata` で、依存の `rust-version` が、すべて 1.85 以下であることを確認した(ただし `lexopt` は `rust-version` 未宣言(edition 2018))。**最小の 1.85 のツールチェインでの実ビルドは、できていない**(この環境には、安定版のみ)。CI の build-check に委ねる。
- `signal-hook` の `Signals` が、HUP/INT/QUIT/TERM を、番号 1/2/3/15 で返すことを、使い捨てのテストと、本番の `SystemSignals`(`tests/signals_real.rs`、実際のシグナル)で確認した。
- `ctrlc` の Windows: `cargo check` と `cargo clippy --target x86_64-pc-windows-msvc`(全ターゲット)が通る(型検査まで)。実行は CI。
- `cargo tree -p conductor-cli -e normal,build` に、`cc`・`cmake`・`pkg-config`・`bindgen`・`-sys` はない(`tests/nfr_source.rs` も確認する)。

## 性能の実測(macOS、debug ビルド。警告のみで、失敗にしていない)

| 測定 | 結果 | 基準 |
|------|------|------|
| `run`(ジョブ 10 個、20 回)のオーバーヘッド p95 | 21.4 ms | 100 ms |
| 同上 起動前 / ジョブのあと | 11.6 ms / 14.6 ms | 50 / 50 ms(配分) |
| `pipeline` のジョブ 1 つあたり p95 | 24.9 ms | 50 ms(目標) |
| ジョブ 1,000 個の定義の `run` | 151 ms | 1 秒 |
| 1,000 ジョブの `pipeline` の流れ(メモリの Runtime) | 1.17 秒 | (記録のみ) |
| ジョブ実行中(約 3 秒)の conductor の CPU 時間 | 0.01 秒 | 0.1 秒 |

- 超過時は `STARTUP_BENCH_ENFORCE=true` のときだけ失敗にする。リリースのビルドは `cargo build --release --locked -p conductor-cli` が成功し、`conductor` は約 971 KB。Linux と Windows の実測は CI。

## 設計との差(計画からの逸脱と、設計の成果物への反映)

1. **`Signals::start` の引数:** `start(&Arc<InterruptSignal>)`(計画は `&InterruptSignal`)。監視のスレッドが、合図を `'static` で持つ必要があるため。
2. **`run_cli` の形:** `run_cli(args, env, sink)`(本番の `Runtime` と `Signals` は、内部で組み立てる)。`run_cli_with(args, &mut Deps)` が、テスト用。`Runtime` trait は 1 つ(7 メソッド)。
3. **Windows の Ctrl+C の自動テストは作らなかった:** CI に対話的なコンソールがなく、`GenerateConsoleCtrlEvent` はテスト自身にも届き、`CREATE_NEW_PROCESS_GROUP` では Ctrl+C が無効になる。計画が許す、リリース前の手動確認のチェックリスト(`tests/WINDOWS_CTRL_C_CHECKLIST.md`)に置き換えた。これに伴い、開発用の `windows-sys` は足していない。**Windows の Ctrl+C の経路は、自動のテストで確かめられていない。** `ctrlc` が Ctrl+Break も受け取るかは、Windows の実機で未確認(チェックリストの項目)。
4. **補助の実行ファイルの時刻:** `Instant` はプロセスをまたいで比べられず、`unsafe` なしでは取り出せないため、`SystemTime`(UNIX 時刻、ナノ秒)で出している(単調な時計ではない)。
5. **パニックのフックの差し替え:** `on_panic(err, exit)` と `install_panic_hook_with(fn(i32))` を公開した。`tests/panic_hook.rs` で単体テストしている。
6. **閉じたパイプ:** バイナリを起動するテストは入れていない(決定的にできず、不安定になるため。Rust は EBADF を無視する)。偽の `Sink` の流れのテスト(watch 65、convert 67)で確かめた。
7. **ジョブのあとの書き込みの失敗の文言:** 「ジョブは実行されました」(JobExecuted)と、「ジョブは起動していません」(JobNotStarted)で区別した(ジョブのあいだの中断と、スキップの書き込みの失敗は後者)。
8. **進め方の逸脱(開発担当のエージェント):** `Cargo.toml` の `[profile.release]` の追記と、`tests/common/mod.rs` への `MemRuntime` の追記を、シェルで行った(ファイルは Write/Edit で書く、という規則に反する)。内容は、意図どおり。
9. **契約の差分(書き換えていない設計の成果物への反映):** functional-spec の「契約への反映が必要な点」(C2・C3・E1・E2・E3)と、nfr-requirements・nfr-design の差分の表のとおり実装した。E2 の `job` は、u1-core の実装を直した(Step 2)。E3 のステートファイルの `[[jobs]]` は、変わらない(定義の `job` とは別の形式)。u1-core の functional-design の成果物は、`jobs` のままで、書き換えていない。

## レビュー(1 回目、READY)で直した点

- **Major:** ジョブの起動の前にシグナルが届いて(`started == false`)結果の書き込みが失敗したとき、「ジョブは実行されました」と誤って表示していた(`run.rs`、`pipeline.rs`)。`JobNotStarted` で報告するようにし、run と pipeline に、それぞれ、テストを足した。
- **Minor:** `main.rs` と補助の実行ファイルに `#![forbid(unsafe_code)]` を足した。README のビルド手順を `cargo build --release --locked --bin conductor` にした。ほぼ同時に届いたシグナルは、`signal-hook` が番号の小さい順に返すため、「最初の番号」は、小さい番号になりうる(時間を空けて届けば、到着順)。README と doc に正確に書き、INT のあとに HUP を送るテスト(`tests/signals_order.rs`)を足した。同時到着は、不安定になるため、確かめていない。
- **Minor(記録のずれ):** traceability.json の一部の対応先を、実際の実装(`ctx.rs`、`pipeline.rs`)に直した。Windows の経路(`platform/windows.rs`)は、型検査までで、実行は CI と手動のチェックリスト。

## 設計の隙間(後のステージ・Unit で解決する)

- **上流の理由文が日本語のみ:** u1-core・u2-runtime・u4-converter の `reason` は、日本語の固定文で、英訳がない。英語のロケールでは、種類の文だけを英語にして、`reason` は、そのまま出す(定義の誤りは「kind(英語): reason(日本語)」になる)。README に注記した。完全な英訳には、上流 Unit の変更が要る。
- **補助の実行ファイルが、通常のビルドに含まれる:** `cargo build -p conductor-cli` で、`conductor-cli-test-helper` もビルドされる(`test = false` を付けても、通常のビルドでは作られる。`required-features` にすると、`--features` なしのテストが動かなくなるため、見送った)。**cicd-pipeline の「`-p conductor-cli` のビルドに含まれない」は、不正確。リリースのワークフローでは、`conductor` だけを、まとめる**(CI Pipeline・Deployment Pipeline のステージで反映)。
- **`escape_display` の複製(4 つ目):** u1-core の `sanitize` とは、直接比べるテストを置き、u2・u4 は、同じ期待値の表で揃えた。共通のクレートへのまとめは、将来の検討。
- **`pipeline` 内の `Reject`:** 通常の入力では到達できず、流れのテストは作れていない(u1-core の `decide_before_run` 側のテストの範囲)。
- **`--config=` の空文字:** 空のパスとして扱われる(定義の誤り 64)。
- **`main.rs`、標準出力・標準エラーの本番の実装、環境の本番の実装:** バイナリを起動するテストでだけ通る。カバレッジは、`cargo llvm-cov` の導入後に、このバイナリも含めて確かめる。
- **IDE の設定ファイル(`.idea/rust-job-conductor.iml`):** 変更が、作業ツリーにあるが、この Unit の変更ではない(IDE の自動更新)。

## テスト(182 件)

| 範囲 | 件数 |
|------|------|
| 引数 | 24 |
| 終了コード | 7 |
| 表示、escape | 22 + 4 |
| 個別実行 | 45 |
| 一括実行 | 21 |
| 補助のコマンドと入口 | 25 |
| パニックのフック | 3 |
| 実シグナル | 1 |
| バイナリ起動 | 13 + 2 |
| 横断(ソース検索 6、パニックしない 4、性能・同時実行 5) | 15 |

## 実行していないテスト・測定

- **行カバレッジ:** `cargo llvm-cov` が未導入のため、測っていない。80% の判定は Build and Test が行う(`--ignore-filename-regex` で、補助の実行ファイルを除く)。`cfg(windows)` のコードは、Linux の測定に入らない。
- **`cargo +1.85 build`:** 1.85 のツールチェインがなく、実行できなかった。CI の build-check に委ねる。
- **Windows の実行(Ctrl+C を含む)、Linux の実行:** macOS でだけ実行した。CI(CI Pipeline のステージで作成)で確かめる。CI のワークフローは、まだない。
