# 11 品質・ビルド・配布(非機能要件、テスト、CI、リリース、デモ)

対象: `conductor` v0.9.0 の機能を含む現在の実装(Rust 実装)。根拠は `Cargo.toml`(ルートと各クレート)、`Cargo.lock`、`.github/workflows/{ci,audit,release}.yml`、
各クレートの `tests/*.rs` と `src/bin/*.rs`、`demo/*`、`README.md`。この文書の基準を満たせば、同じ品質ゲートを再現できる。

---

## 1. リポジトリとクレートの構成

```text
Cargo.toml                 ワークスペース(members = ["crates/*"])
Cargo.lock                 コミットする。CI とリリースは --locked で使う
crates/core/               conductor-core      定義の読み込み・検証と、フロー(分類・次の動き・リトライ)の判断。入出力なし
crates/runtime/            conductor-runtime   ジョブの起動・待ち・シグナルの転送、ステートの保持・照合・初期化・記録
crates/converter/          conductor-converter 定義ファイルの YAML と TOML の相互変換
crates/cli/                conductor-cli       実行ファイル conductor(引数、流れの組み立て、シグナルの監視、表示、終了コード)
.github/workflows/         ci.yml, audit.yml, release.yml
demo/                      デモ(UNIX 版と demo/windows/ の Windows 版)
```

| クレート | 種別 | 依存する内部クレート | 配布物 |
|----------|------|----------------------|--------|
| `conductor-core`(0.9.6) | lib | なし | 含まれる(`conductor` に静的に入る) |
| `conductor-runtime`(0.9.6) | lib + テスト補助 bin | core | 同上。補助 bin は配布しない |
| `conductor-converter`(0.9.6) | lib | なし | 同上 |
| `conductor-cli`(0.9.6) | lib + bin `conductor` + テスト補助 bin | core、runtime、converter | bin `conductor` だけ |

4 つのクレートの版は同じ 0.9.6。版の上げ方は手動(すべての `Cargo.toml` を書き換えるコミット)。リリースの検査が見るのは `crates/cli/Cargo.toml` の版だけ(7 章)。
`publish = false`(crates.io には公開しない)。

### 1.1 ワークスペース共通の設定(ルートの `Cargo.toml`)

| 項目 | 値 |
|------|----|
| `resolver` | `"3"` |
| `[workspace.package] edition` | `"2024"`(各クレートが `edition.workspace = true`) |
| `[workspace.package] rust-version`(MSRV) | `"1.100"` |
| `[workspace.package] license` | `"Apache-2.0"` |
| `[profile.release]` | `opt-level = 3`、`lto = "thin"`、`codegen-units = 1`、`strip = true` |

各ソースファイルの先頭に Apache License 2.0 のヘッダ(`Copyright 2026 agwlvssainokuni`)を置く。

### 1.2 MSRV と edition

- MSRV は **1.100**。edition は **2024**(let チェーン `if let ... && let ...` を使う)。
- MSRV の記載は 3 か所で一致させる(テスト `crates/cli/tests/msrv_consistency.rs` が強制):
  1. `Cargo.toml` の `rust-version = "1.100"`。
  2. `README.md` に文字列 `Rust(1.100 以降)` が含まれる。
  3. `ci.yml` の、版を固定したツールチェーン指定(`dtolnay/rust-toolchain@<版>`、`stable` 以外のすべて)が `1.100.0`(= `<MSRV>.0`)。
     さらに `ci.yml` に、コメントとして `rust-version = 1.100)` と `最小の Rust 1.100 でも` の 2 つの文字列が含まれる。
- MSRV を上げるときは、この 3 か所とコメントを同時に更新する。履歴: 1.85 から 1.100 へ引き上げ(コミット `f724dec`)。

### 1.3 リンタ・フォーマッタの設定

| 項目 | 内容 |
|------|------|
| フォーマッタ | `rustfmt`(`cargo fmt --all -- --check`)。設定ファイル(`rustfmt.toml`)はない。既定の規則 |
| リンタ | `clippy`(`cargo clippy --workspace --all-targets --locked -- -D warnings`)。警告はすべて失敗。設定ファイル(`clippy.toml`)も `[lints]` の設定もない |
| 属性での許可 | 製品のコードに `#[allow(...)]` はない。`#![allow(unsafe_code)]` が `crates/runtime/src/platform/unix.rs` に 1 つだけ(`libc::kill` のため) |
| `unsafe` | runtime は `#![deny(unsafe_code)]`、cli と `main.rs` とテスト補助は `#![forbid(unsafe_code)]` |
| 命名 | Rust 標準(関数・変数は snake_case、型は PascalCase、定数は SCREAMING_SNAKE_CASE)。テスト関数名は日本語の名前を使う |

---

## 2. 依存クレート

方針: 依存は最小にする。C のコンパイルを要する依存と、ネットワークのライブラリを持たない(CI とテストで検査する。6.1 節の `build-check`)。
バージョンは `Cargo.toml` の指定(Cargo の既定の互換範囲)と、`Cargo.lock` で固定した版。

### 2.1 通常の依存

| クレート | 版(`Cargo.lock`) | 使うクレート | 機能・用途 |
|----------|-------------------|--------------|------------|
| `toml` | 1.2.1(指定 `1.2.1`) | core(`parse`、`std`)、runtime と converter(`parse`、`std`、`display`、`serde`) | `default-features = false`。定義とステートの TOML の読み書き。core は `DeTable` で要素ごとのバイト範囲(行番号)を得る |
| `thiserror` | 2.0.21 | core、runtime、converter | エラー型の `Display` の導出 |
| `saphyr-parser` | 0.3.4 | converter | YAML の構文解析(YAML の読み取り) |
| `lexopt` | 0.3.2 | cli | コマンドラインの引数の解析 |
| `libc` | 0.2.191(指定 `0.2`) | runtime(`cfg(unix)` のみ) | `libc::kill` を 1 か所だけ呼ぶ |
| `signal-hook` | 0.4.6 | cli(`cfg(unix)` のみ。`default-features = false`、`features = ["iterator"]`) | SIGHUP・SIGINT・SIGQUIT・SIGTERM の受信(`Signals` のイテレータ) |
| `ctrlc` | 3.5.2 | cli(`cfg(windows)` のみ) | Ctrl+C の受信 |

### 2.2 開発用の依存

| クレート | 版 | 用途 |
|----------|----|------|
| `tempfile` | 3.27.0 | 全クレートのテストで、一時ディレクトリ・一時ファイルを作る |

### 2.3 禁止する依存(テストと CI が検査)

- 依存ツリー(`normal,build`)に、`cc`、`cmake`、`pkg-config`、`bindgen`、`openssl`、`reqwest`、`hyper`、`ureq`、`tokio` がない。
- 名前が `-sys` で終わる依存は、`windows-sys` と `linux-raw-sys` だけ許す。
- `src/` に、`std::net`、ネットワークのライブラリ、`unsafe`(許可した 1 か所を除く)、`low_level` がない。

---

## 3. 非機能要件(NFR)

要件番号(NFR*)は、テストのコメントやドキュメントで使う識別子。ここでは、テストが確かめる基準をカテゴリ別に示す。
「強制」は基準を超えると失敗するもの、「計測のみ」は超えても警告にとどまるもの。

### 3.1 性能(NFR1)

| 項目 | 基準 | 扱い | 根拠のテスト |
|------|------|------|--------------|
| 定義(ジョブ 10 個)の読み込み(取り込みから結果まで)を 100 回 | p95 20 ms | 計測のみ(標準エラーと `GITHUB_STEP_SUMMARY` に出力) | `core/tests/nfr_performance.rs` |
| 起動前の処理(`command` の検査、ステートの読み、照合、`Durability::Fast` の書き込み)を 100 回、ジョブ 10 個の定義 | p95 20 ms(仮)。`Durable` の書き込みは基準なし(記録のみ) | 計測のみ | `runtime/tests/nfr_performance.rs` |
| `run` の自身のオーバーヘッド(起動前 + ジョブのあと。ジョブ 10 個の定義、20 回、ジョブは時刻を出す補助の `timing`) | p95 100 ms | 計測のみ(`STARTUP_BENCH_ENFORCE=true` で強制) | `cli/tests/nfr_perf.rs` |
| `pipeline` のジョブ 1 つあたり(ジョブ 10 個、5 回。ジョブ自身の起動を含む) | p95 50 ms | 同上 | 同上 |
| ジョブ 1,000 個の定義で `run job0` が終わるまで | 1,000 ms(仮) | 同上 | 同上 |
| ジョブ 1,000 個の `pipeline` の流れ(メモリ上の偽の Runtime。ファイルとプロセスなし) | 10,000 ms | 同上。完了のステートが残ることは常に検査 | 同上 |
| ジョブの実行中(約 3 秒)の `conductor` の CPU 時間(`ps -o time=`、UNIX のみ) | 100 ms 以下 | **強制**(`ps` で読めなければスキップ) | 同上 |
| 同じ定義への 2 つの `run` を同時に | 両方が実行され(ジョブが 2 回始まる)、どちらもジョブの終了コード(99)で終わり、ステートは読み戻せる(排他制御はしない) | **強制** | 同上 |
| 待ちの間隔 | `try_wait` を 1・2・4・8・10・10... ms(上限 10 ms)。リトライの待機は 50 ms 以下の間隔で合図を確かめ、合図から 1 秒以内に返る | 強制 | `runner.rs` の単体、`wait_interruptible.rs` |
| 図の生成(v0.9.0 以降)。ジョブ 1,000 個で、構造の組み立てと配置(`build_graph` + `layout_graph`)。最悪の 4 つの形(入れ子が最大の形 = 前半の各ジョブ `i` が `N-1-i` へ飛ぶ。交差する形 = 全ジョブ `j(i)` → `j(i+2)`。幅の広い交差の形 = 前半の各ジョブ `i` が `i+N/2` へ飛ぶ。同じ行き先が離れて重なる形 = 前半の各ジョブ `i` が `N/2 + i % 250` へ飛ぶ)と、「両方の飛び越し」と分岐を含む 2 つの形(入れ子 = 前半の偶数のジョブ `i` が `N-1-i` へ両方の飛び越し、奇数のジョブが `N-2-i` と `N-1-i` へ分岐。幅広い交差 = 前半のジョブ `i` を 3 つおきに、`i+N/2` と `i+N/2+1` への分岐、`i+N/2` への両方の飛び越し、通常の飛び越しにする)。すべてレーンの図になる | 1 秒以内 | **強制**(`elapsed < 1s`) | `core/tests/graph_perf.rs` |
| 計画の生成(v0.9.0 以降)。ジョブ 1,000 個がすべて `env` と `env_file` を持ち、すべて実行する計画(`plan_pipeline`) | 1 秒以内 | **強制** | `core/tests/plan_perf.rs` |
| `@end` に着いたときの残りのジョブの列挙。ジョブ 1,000 個、先頭のジョブの成功で `@end`(`classify_and_decide`、`RunMode::Pipeline`)を 100 回 | 5 回の中央値が 50 ms 以内 | **強制** | `core/tests/flow_perf.rs` |
| `.env` の層の統合(`merge_env`)。1 ファイル 10,000 項目(すべて別の名前)/ 2 ファイル 10,000 項目ずつ(同じ名前。後ろが勝つ) | 5 回の中央値が 50 ms / 100 ms 以内 | **強制** | `core/tests/env_merge_perf.rs` |

測定のしくみ: p95 は、昇順に並べた測定値の `ceil(n × 0.95)` 番目。測定値は `[bench] <行>` として標準エラーに出し、環境変数 `GITHUB_STEP_SUMMARY`
があれば、そのファイルに `- <行>` として追記する。基準を超えたときは `警告: ...` を出す。環境変数 `STARTUP_BENCH_ENFORCE` が文字列 `true` のときだけ、
`cli/tests/nfr_perf.rs` の基準超えを失敗(panic)にする(CI の既定は `false`)。測定の行には「debug ビルド」と書かれるが、CI の `startup-bench` は `--release` で実行する(文言と実態の差)。

### 3.2 堅牢性(NFR4.7 ほか。パニックしない・桁あふれしない)

外部の入力(コマンドライン、`command` の文字列、ステートファイル、OS の結果)に対して、失敗は戻り値で返し、パニックしない。

| 項目 | 検査の内容 | テスト |
|------|-----------|--------|
| ランダムな引数 | 固定の種のランダムな引数列(語の表から 0〜6 個を選ぶ)を 500 回。終了コードは 0〜255 の範囲、標準エラーに制御文字(改行を除く)と U+202E がない | `cli/tests/nfr_robust.rs` |
| UTF-8 でない引数(UNIX) | `0xff 0xfe` を含む `OsString` を、ジョブID・`--config`・オプション・`convert` の引数に渡してもパニックしない | 同上 |
| 壊れたステート | 状態・再開位置・フラグを乱したステートを返す偽の Runtime でもパニックしない | 同上 |
| 空・制御文字を含むジョブID | パニックしない | 同上 |
| 壊れた TOML・ランダムなバイト列のステート | `read_state` が失敗を返し、パニックしない。正しいステートを壊した入力、巨大な値のステートも同様 | `runtime/tests/nfr_robust.rs` |
| ランダムな `command` | 文字集合 `ab ⇥⏎'"\;|&$*あ` から長さ 0〜39 の文字列を 2,000 件。分割が成功したら `program` は空でない | 同上 |
| 大きな `command` | 約 30 万引数(`prog ` + `'a b' "c\"d" e ` を 10 万回)が分割できる。閉じない引用符は誤り。再帰がないのでスタックを使わない | 同上 |
| 起動できない `command` | 分けられない空文字列、`'`、`''` は `check_command` の誤り(パニックしない)。存在しない名前、ディレクトリのパス、`/` は、すべて起動失敗(`StartFailed`)。範囲外のシグナル番号(`i32::MAX`、`i32::MIN`、0、-1、1,000,000)の合図でも、起動しない(`NotStarted`) | 同上 |
| 入れ子の深い TOML | 深さ 1,000 の入力で異常終了せず失敗を返す(テストのスレッドの標準のスタック約 2 MiB で) | `core/tests/library_capability.rs`、`runtime/tests/toml_depth.rs` |
| 製品のコードの書き方 | runtime の製品のコードに `.unwrap()`、`.expect(`、`panic!(`、`unreachable!(`、`todo!(`、`unimplemented!(` がない。converter も入力経路で `unwrap`/`panic` を使わない | `runtime/tests/nfr_source.rs`、`converter/tests/nfr_security.rs` |
| 内部の誤り | パニックのフックが、固定の英語 `conductor: error: internal error` を標準エラーに書き、終了コード 70。標準エラーが閉じていても 70 | `cli/tests/panic_hook.rs` |
| 変換の入力 | 範囲外・空・壊れた入力、ランダム・切り詰めた入力でパニックしない。成功したランダムな構造は往復で等しい | `converter/tests/nfr_security.rs` |
| 時間の上限 | プロセスを起動するテストは、すべて 30 秒の上限つき(超えたら子プロセスを強制終了して失敗) | 各 `common/mod.rs`、`bin_support/mod.rs` |

### 3.3 セキュリティ(NFR2、NFR4.1・4.3・4.5・4.6・4.8・4.9)

| 項目 | 基準 | テスト |
|------|------|--------|
| シェルを呼ばない | 起動は、`Command::new(&line.program)` だけ。`"sh"`、`"cmd"`、`"/bin/sh"`、`"cmd.exe"`、`"bash"`、`"powershell"`、`"-c"`、`"/C"` の文字列がソースにない | `runtime/tests/nfr_source.rs` |
| 引数にメタ文字が入っても注入にならない | `a;b`、`&&`、`|`、`$HOME`、`*`、`>x` などが、そのまま引数として渡る。Windows の `.cmd` への危険な引数(引用符を壊して `& echo injected > file`)で、続くコマンドが実行されない(文字列形式と配列形式) | `runtime/tests/runner_basic.rs` |
| 検証は副作用を持たない | 定義の検証は、コマンドを解釈・展開・起動しない。ファイルから読んでも定義のディレクトリは変わらない | `core/tests/nfr_security.rs` |
| 入力の値を表示に出さない | すべての種類のエラーの表示に、入力の値(秘密のつもりの文字列、`command`、ライブラリのエラー文)が現れない。キー名は制御文字をエスケープして表示 | `core/tests/nfr_security.rs`、`runtime/tests/nfr_content.rs`、`converter/tests/nfr_security.rs` |
| ステートに機微な値を書かない | ステートファイルに、`command`、引数、環境変数の値、標準出力の内容が現れない | `runtime/tests/nfr_content.rs` |
| 環境変数の値を出さない(v0.9.0 以降) | `env`・`.env` の値(秘密のつもりの文字列)が、標準出力、標準エラー(`-v` を含む)、ステートファイルに現れない。`graph`・`--dry-run` の出力にも現れず、名前だけが出る。`reservedEnvName`・`envNameCaseConflict`・`invalidEnvValue` の表示にも値が現れない。値を持つ型の `Debug` は値を `"***"` にする | `cli/tests/binary_env.rs`、`binary_graph.rs`、`binary_dry_run.rs`、`core/tests/nfr_security.rs`、`env_types.rs` |
| 表示のエスケープの表 | `escape_display`(本体は core)が期待値の表のとおりで、core の `sanitize`・converter の写しと同じ入力に同じ結果を返す(制御文字・方向制御文字の無効化) | `cli/tests/escape_table.rs` |
| ネットワークに接続しない | `src/` に `std::net` などがない。依存に HTTP・TLS のライブラリがない | `cli/tests/nfr_source.rs` |
| `unsafe` | runtime は `platform/unix.rs` の 1 回(`libc::kill`)だけ。cli は禁止(`#![forbid(unsafe_code)]`)。`low_level` を使わない | `runtime/tests/nfr_source.rs`、`cli/tests/nfr_source.rs` |
| シグナルの送信 | ジョブの子プロセスの pid だけ(0 以下にならない)、許可した 4 つのシグナルだけ、1 回だけ | `runtime/src/platform/unix.rs` の単体、`runner_signal.rs` |
| テスト専用の経路を製品に持たない | `main.rs` に `panic!` がない。`src/bin/` 以外に `panic!(` がない。配布は `conductor` だけ | `cli/tests/nfr_source.rs`、`release.yml` |
| converter の副作用 | ソースにプロセス・ネットワーク・環境変数のアクセスがない。静的な初期化がない。`convert` は入力以外のファイルを作らず、カレントディレクトリの変更と環境変数に左右されない | `converter/tests/nfr_security.rs` |
| 既知の脆弱性 | `cargo audit`(6 章。警告のみ) | `audit.yml` |
| 配布物の完全性 | `SHA256SUMS` を付ける(署名はしない。転送の破損の検知まで)。macOS の署名・公証はしない | `release.yml`、README |

### 3.4 保守性(OS 固有コードの隔離。NFR3.1)

`cfg(unix)`・`cfg(windows)`・`target_os`・`libc::` などは、runtime と cli のそれぞれの `src/platform/` の中にだけ置く(ソース検索のテストが強制。詳細は 07 の 8 章)。

### 3.5 ソースの検査を行うテスト一覧

`runtime/tests/nfr_source.rs`(`unsafe` の個所、`unsafe` の許可の場所、OS 分岐の場所、シェルを呼ばない、パニックしうる書き方がない)、
`cli/tests/nfr_source.rs`(`forbid(unsafe_code)`、`unsafe`/`low_level` なし、ネットワークなし、OS 分岐の場所、テスト専用の経路なし、依存の検査。
最後は `cargo tree -p conductor-cli -e normal,build --prefix none --offline --locked` を実行し、`cargo` が使えない・失敗する場合はスキップする)、
`converter/tests/nfr_security.rs`(ソース検査を含む)。

---

## 4. テスト構成

### 4.1 方針

- 方法論は test-after(実装してから、層ごとにテストを書いて実行する)。単体テスト(各 `src/*.rs` の `#[cfg(test)]`)と、バイナリを実行する統合テスト(`tests/`)を持つ。
- 行カバレッジの下限は **80%**(`cargo-llvm-cov`。5 章)。
- プロセスを起動するテストは、30 秒の上限(`LIMIT`)で、ハングだけを検知する。失敗した場合は、ジョブの子プロセスを強制終了して後始末する。
- テストは日本語の関数名を使い、OS 依存のテストは `#[cfg(unix)]` / `#[cfg(windows)]` をファイル全体またはテスト単位に付ける。
- 実行: ローカルは `cargo test --workspace --locked`。補助の実行ファイルは `cargo test` が自動でビルドする(`env!("CARGO_BIN_EXE_<名前>")`)。

### 4.2 テストファイルの目的

ファイルごとのテストの件数は書かない(機能を足すたびに変わり、OS によっても変わる。件数は `cargo test` の出力で分かる)。

#### conductor-core(`crates/core/tests/`)

| ファイル | 目的 |
|----------|------|
| `definition_valid.rs` | 正しい定義の受理と、`load_definition`(ファイルから読む入口) |
| `definition_errors.rs` | 定義の検証の誤りの網羅(違反の例と受理の例) |
| `definition_retry.rs` | `[job.retry]` の読み取りと検証 |
| `flow_sequences.rs` | フローの判断の一連の流れ(`parse_definition` と手元のステートだけ) |
| `retry_flow.rs` | `retry_decision` の判断 |
| `library_capability.rs` | `toml` の `DeTable` の確認(要素ごとのバイト範囲、深さ 1,000 の入力で異常終了しない) |
| `nfr_performance.rs` | NFR1.1 の測定(3.1) |
| `nfr_security.rs` | NFR4.5〜4.7(3.2、3.3)。v0.9.0 以降は、新しい 3 種(`reservedEnvName`・`envNameCaseConflict`・`invalidEnvValue`)の表示に値が出ないことも確かめる |
| `env_definition.rs` | v0.9.0 以降。`env`・`env_file` の読み取りと検査(03 の 6.6。各誤り、予約名の大文字小文字、NUL、順序) |
| `env_file_parse.rs` | `parse_env_file`(03 の 6.7。各引用符、エスケープ、BOM、CRLF、`export`、重複、名前の誤り、上限) |
| `env_merge.rs` | `merge_env`(05 の 14 章。優先順位、配列の順、完全一致、大文字小文字だけが違う名前の順、自動の変数、run ID の有無、試行) |
| `env_types.rs` | 環境変数の型(値を持つ型の `Debug` の伏せ字、`DefinitionErrorKind` が 33 種、`EnvFileErrorKind` が 13 種、`EnvFileError` の表示、`summarize_errors` の 20 件の打ち切り) |
| `graph_types.rs`、`graph_edges.rs`、`graph_overlay.rs`、`graph_drawable.rs`、`graph_layout_oracle.rs`、`graph_shape_skip.rs`、`graph_shape_same_target.rs`、`graph_shape_both.rs`、`graph_shape_branch.rs`、`graph_perf.rs` | v0.9.0 以降。図の型、辺の取り出し、状態の重ね合わせ、「対応する形」の判定(4 つの種類。定義から作れる形はすべて対応する形で、範囲の外の形は図の構造を直接作って確かめる)、配置を正解データと 1 文字単位で照合(git の出力: `fixtures/graph_layout_oracle.md`、537 例、同じ行き先を含む `fixtures/graph_layout_oracle_same_target.md`、1,517 例、交差する形の `fixtures/graph_layout_oracle_cross.md`、940 例、交差と同じ行き先が混ざる形の `fixtures/graph_layout_oracle_mixed.md`、487 例。規則から導いた図で、git の出力ではない: 分岐を含む `fixtures/graph_layout_oracle_branch.md`、871 例(主線の列 0 の規則で作り直した第 2 版)、「両方の飛び越し」か分岐を含むジョブ 3〜5 個の全形 `fixtures/graph_layout_oracle_offmain.md`、2,748 例、ジョブ 6〜7 個の `fixtures/graph_layout_oracle_offmain_67.md`、300 例)、形ごとの図の全文: 通常の飛び越しと交差の例 C1〜C3(`graph_shape_skip.rs`)、同じ行き先と交差に同じ行き先が混ざる例 M2〜M4(`graph_shape_same_target.rs`)、「両方の飛び越し」の例 D1〜D7 と組み合わせ N1、N3、N4(`graph_shape_both.rs`)、分岐の例 B1〜B4 と組み合わせ N2、N5(`graph_shape_branch.rs`)、辺の一覧に残る形と主線の列 0 の記号(`graph_drawable.rs`)、1,000 ジョブの性能(幅の広い交差、「両方の飛び越し」と分岐の入れ子・交差を含む。`graph_perf.rs`。05 の 15 章)。`graph_common/mod.rs` は補助(`definition`、`letters`、`letter_definition`、`lanes`、`both` など) |
| `plan_types.rs`、`plan_pipeline.rs`、`plan_run.rs`、`plan_assume.rs`、`plan_perf.rs` | v0.9.0 以降。計画の型、`plan_pipeline`・`plan_run`、`--assume`(使われなかった仮定、既定の 0)、1,000 ジョブの性能(05 の 16 章) |
| `flow_perf.rs`、`env_merge_perf.rs` | `@end` に着いたときの残りのジョブの列挙(ジョブ 1,000 個)と、`merge_env`(`.env` 10,000 項目)の性能(3.1 節の表)。結果の中身(並び、出どころ)も確かめる |
| `fixtures/valid_pipeline.toml`、`fixtures/multiple_errors.toml`、`fixtures/graph_layout_oracle*.md` | 入力例、配置の正解データ(後者は v0.9.0 以降。7 つ。上の `graph_layout_oracle.rs` の行) |
| `common/mod.rs` | 補助(`SECRET`、`run`・`kinds`(定義の検査の結果)、`median`、`state(jobs, resume, completed)`、`job(id)`、`exited(job_id, code)`) |

#### conductor-runtime(`crates/runtime/tests/`)

| ファイル | 目的 |
|----------|------|
| `runner_basic.rs` | `run_job` の統合テスト(補助の実行ファイルを実際に起動)。メタ文字、配列、環境変数と作業ディレクトリの引き継ぎ、起動失敗(空の `program` を含む)、渡した分割済みのコマンドで起動し `command` を分け直さないこと、終了コード、起動前の合図。Windows 専用の `cmd` 注入 3 件 |
| `runner_env.rs` | v0.9.0 以降。`run_job` の `env` 引数(足した値が子に見える、引き継いだ環境が残る、同じ名前の上書き、並びの後ろが勝つ) |
| `env_loader.rs` | v0.9.0 以降。`load_env_files`(無い、読めない、大きすぎる、UTF-8 でない、相対と絶対のパス、同じパスの 1 回だけの読み取り、すべての誤りを集める) |
| `file_create.rs` | v0.9.0 以降。`create_file`(新規で内容がそのまま、既存は壊さない、`--force` の置き換えと一時ファイルが残らない、親が無い、ディレクトリ、宙に浮いたシンボリックリンク、権限がない、同時に作る 2 つのうち 1 つだけが `Created`) |
| `runner_signal.rs`(UNIX) | SIGTERM・SIGINT の転送、許可外の番号、新しいプロセスグループ、転送は 1 回、先に終わったジョブ |
| `wait_interruptible.rs` | `wait_for`(経過、0 秒、合図、1 秒以内の打ち切り、最初の番号) |
| `state_read.rs`、`state_read_raw.rs`、`state_write.rs` | ステートの読み書き・削除(実ファイル。書き込みの失敗は、置き換え先をディレクトリにして起こす) |
| `run_id_state.rs` | run ID ごとのステートと `reset --all` |
| `flow_with_state.rs` | core の判断と runtime の関数を組み合わせた、ジョブの一連の流れ(今の cli と同じ順で: スキップは変わったときだけ書く、削除は `remove_on_complete` のときだけ、`pipeline` は記述順に決める。完了のステートは既定で残る) |
| `toml_depth.rs` | 深い入れ子の TOML |
| `nfr_content.rs`、`nfr_performance.rs`、`nfr_robust.rs`、`nfr_source.rs` | 3 章 |
| `common/mod.rs` | 補助(`helper_path`、`helper_command`、`job`、`definition(name, jobs)`、`without_run_id`(run ID なしの `read_state`・`reset_state`・`initial_state`・`write_state`)、`lines`、`wait_for_line`、`pid_in`、`force_kill`、`is_root`、`run_with_limit`) |

#### conductor-converter(`crates/converter/tests/`)

| ファイル | 目的 |
|----------|------|
| `convert_ac.rs` | 受け入れ基準と、`convert`(ファイルを読む入口) |
| `convert_roundtrip.rs` | 往復(変換して読み戻した木が入力の木と等しい) |
| `read_resolve.rs` | 文書の確認と展開、失敗の順序、暴走する入力 |
| `nfr_security.rs` | 3 章 |
| `fixtures/{pipeline,broken}.{toml,yaml}` | 入力例 |

#### conductor-cli(`crates/cli/tests/`)

| ファイル | 目的 |
|----------|------|
| `args_grammar.rs` | 全コマンドの引数の文法の表(正しい引数と誤り。v0.9.0 以降は `graph`・`init`・`--dry-run`・`--assume`・`--format`・`--state`・`--force` を含む) |
| `exit_codes.rs` | 終了コードの決定(純粋な関数の全項目) |
| `run_id_arg.rs` | `--run-id` の指定の型 `RunIdArg`(空は固定、規則どおりは値、規則違反は値を含めない `invalidValue`) |
| `escape_table.rs` | `escape_display` の入力と期待値の表(本体は core。converter の写しは converter の単体テストの同じ表で揃える) |
| `report_messages.rs` | 表示の全種類の文言(英語・日本語。v0.9.0 以降は、`.env` の誤り 13 種と 20 件の打ち切り、重複キーの警告、`init` の全メッセージ、`CreateError` の全種類を含む)、ロケール、遅延、色 |
| `graph_report.rs` | v0.9.0 以降。`render_text`・`render_mermaid` の出力が、例 G1〜G4、G6、E1(レーンの図)、E1・E2 の辺の一覧(整形の関数に辺の一覧の配置を直接渡す)、S1、D1、B1、M2、M4(交差に同じ行き先が混ざる図。`-v`、`--state` を含む)、N1(全文と `--state`)、N1・N2・N5 の `-v`(詳細の行の列 0 が主線の列 0 の規則に従う)と 1 文字単位で一致、`--state`、`-v`、Mermaid のエスケープ(08 の 11 章) |
| `graph_long_id.rs` | `graph` の text で、65 文字以上のジョブ ID を切らずに出す(レーンの図と辺の一覧。`escape_path` と遷移の表示の `escape_name` の使い分け) |
| `plan_report.rs` | v0.9.0 以降。`render_plan` の出力が、例 P1〜P8、P2e、P4e と 1 文字単位で一致、エスケープ、環境変数の名前の順(値は出ない) |
| `init_template.rs`、`init_types.rs` | v0.9.0 以降。ひな形(日本語・英語、名前のエスケープ、`validate` に通る)、`InitOptions`・`InitTarget` |
| `flow_env.rs`、`flow_graph.rs`、`flow_dry_run.rs`、`flow_init.rs` | v0.9.0 以降。偽の `Runtime` を使った、`.env` の誤りで 64(ジョブ 0 個、ステートに触れない)と `CONDUCTOR_ATTEMPT` の増加と 20 件の打ち切り、`graph`、`--dry-run`、`init` の流れ |
| `flow_run.rs`、`flow_pipeline.rs`、`flow_retry.rs`、`flow_run_id.rs`、`flow_aux.rs`、`flow_validate.rs` | 偽の `Runtime`・`Sink`・`Env`・`Signals` を使った、各コマンドの流れ(個別実行、一括実行、リトライ、run ID、`watch`/`reset`/`convert`、`validate`) |
| `binary_basic.rs`、`binary_run_id.rs` | 実際のバイナリを起動し、終了コード・標準出力と標準エラーの分け方を確かめる |
| `binary_env.rs`、`binary_graph.rs`、`binary_dry_run.rs`、`binary_init.rs` | v0.9.0 以降。実際のバイナリで: `env`・`.env` の値と優先順位が子の環境に見える(補助の `env <NAME>...` モードで確かめる)、自動の変数、`.env` の誤りで 64、重複キーの警告、値が出力・ステートに無い / `graph` の出力と `--state` / `--dry-run` の出力と、何も実行されない・ステートのバイト列が変わらない / `init`(空のディレクトリで、2 回目は 69、`--force`、`--config -`、特殊な文字の名前) |
| `docs_env.rs`、`docs_graph.rs`、`docs_dry_run.rs`、`docs_init.rs` | v0.9.0 以降。README の説明の検査(必須の語句、版の表記 `v0.9.0 以降`、例が実際の読み取り・出力と一致) |
| `binary_signal.rs`(UNIX) | 実際の SIGTERM → 143 と再開位置、続けた HUP・INT → 129 |
| `signals_real.rs`、`signals_order.rs`(UNIX) | 本番のシグナル監視(最初の番号が残る、2 回目の登録は `AlreadyRegistered`、時間を空けた INT のあとの HUP) |
| `panic_hook.rs` | パニックのフック |
| `nfr_perf.rs`、`nfr_robust.rs`、`nfr_source.rs` | 3 章 |
| `msrv_consistency.rs` | MSRV の記載の一致(1.2 節) |
| `docs_retry.rs` | README のリトライの説明の検査(見出し、5 つのキー名、TOML の例が実際の読み取りに通ること) |
| `unix_demo.rs`(UNIX)、`windows_demo.rs`(Windows) | デモの検査(9.5 節) |
| `WINDOWS_CTRL_C_CHECKLIST.md` | Windows の Ctrl+C の手動確認の手順と記録欄(自動テストにできないため) |
| `common/mod.rs` | 偽の `Runtime`(`MemRuntime`:メモリ上のステート)・`Sink`・`Env`・`Signals`、定義の組み立て、`run_mem`、ステートの中の状態 `status_in`・`status_of` |
| `bin_support/mod.rs` | バイナリを起動するテストの補助(`conductor_path`、`helper_path`、`helper_command`、`job_block`、`definition`、`conductor(args)`、`run_limited`、`Project`) |
| `docs_support/mod.rs` | README を検査するテストの補助(`readme`、`DOCS_CHECK_WORDS`、`block_after_heading`、`assert_readme_has`、`assert_docs_check_words`) |
| `signal_support/mod.rs` | 実際のシグナルを送るテストの補助(UNIX。`send(signal, pid)`、`send_self(signal)`) |

`bin_support::conductor(args)` は、表示を安定させるため、`LC_ALL` と `LC_MESSAGES` を外し、`LANG=C`(英語)にし、`NO_COLOR` を外した `Command` を返す。

### 4.3 テスト補助の実行ファイル(配布しない)

どちらも `[[bin]]` で `test = false`、`doc = false`。リリースは `--bin conductor` だけをビルドするため、配布物に入らない。
共通: 引数の先頭が `--out <ファイル>` なら、記録の行をそのファイルに**追記**する(指定がなければ標準出力)。テストは、ジョブの標準出力を
`conductor` 越しに読めないことがあるため、ファイルで結果を受け取る。値の文字列は `{:?}`(Debug 形式)で出す。未知のモードの終了コードは 2。

#### `conductor-test-helper`(`crates/runtime/src/bin/conductor-test-helper.rs`)

`conductor-test-helper [--out <file>] <モード> [引数...]`

| モード | 動作 |
|--------|------|
| `echo <args...>` | 各引数を `arg[i]="..."` の 1 行ずつ出力して、終了コード 0 |
| `run <label> <n>` | `ran=<label>` を出力し、終了コード n(省略・不正は 0) |
| `exit <n>` | 終了コード n(省略・不正は 2) |
| `env <NAME>...` | 各環境変数を `env[NAME]=Some("...")` / `env[NAME]=None` で出力 |
| `cwd` | `cwd=Some("...")` の形で作業ディレクトリを出力 |
| `pid` | `pid=<N>` を出力 |
| `sleep [秒]` | `pid=<N>` と `ready` を出力し、眠る。秒があれば、その秒の後に終了コード 99。なければ眠り続ける。シグナルのハンドラは持たず、既定の動作で終わる(シグナルの受信を `terminated_by_signal` で確かめるため) |

#### `conductor-cli-test-helper`(`crates/cli/src/bin/conductor-cli-test-helper.rs`)

`conductor-cli-test-helper [--out <file>] <モード> [引数...]`。`#![forbid(unsafe_code)]`。

| モード | 動作 |
|--------|------|
| `exit <n>` | 終了コード n(省略・不正は 2) |
| `run <label> <n>` | `ran=<label>` を記録(`--out` に従う)して、終了コード n(省略は 0) |
| `echo <args...>` | `arg[i]="..."` を、**標準出力**に書く(`--out` に従わない) |
| `env <NAME>...` | `env[NAME]=Some("...")` / `None` を、**標準出力**に書く |
| `say <out> <err> [n]` | `<out>` を標準出力、`<err>` を標準エラーに、改行なしでそのまま書き、終了コード n(省略は 0) |
| `sleep [秒]` | `pid=<N>` と `ready` を記録(`--out` に従う)し、眠る。秒があれば終了コード 99。なければ眠り続ける。ハンドラなし |
| `timing` | 起動の直後に取った時刻と終了の直前の時刻を、UNIX 時刻のナノ秒で `start_ns=`・`end_ns=` として標準出力に書く(性能の測定用。プロセスをまたいで比べられる) |
| `burn <ms>` | 指定のミリ秒(既定 100)、CPU を使い続けて終わる(CPU 時間の測定用) |

---

## 5. カバレッジ

| 項目 | 内容 |
|------|------|
| ツール | `cargo-llvm-cov`(`taiki-e/install-action@v2` で導入)。ツールチェーンは `stable` に `llvm-tools-preview` |
| コマンド | `cargo llvm-cov --workspace --locked --ignore-filename-regex 'bin/conductor-(cli-)?test-helper' --fail-under-lines 80 --summary-only` |
| 基準 | 行カバレッジが **80% 未満なら失敗**。ワークスペース全体の合計で判定 |
| 除外 | 2 つのテスト補助の実行ファイル(正規表現 `bin/conductor-(cli-)?test-helper`) |
| 実行する OS | Ubuntu(`ubuntu-24.04`)のみ。したがって `cfg(windows)` のコードと Windows 専用のテストは、計測の対象に入らない |
| 範囲 | 単体テストと統合テストの全体(`cargo test` と同じ) |

---

## 6. CI(`.github/workflows/`)

### 6.1 `ci.yml`(名前 `CI`)

| 項目 | 内容 |
|------|------|
| 起動 | `push`(`main`)、`pull_request`(対象 `main`)、`workflow_call`(リリースから呼ばれる) |
| 権限 | `contents: read` |
| 同時実行 | `group: ci-${{ github.workflow }}-${{ github.ref }}`、`cancel-in-progress: true` |
| 環境変数 | `CARGO_TERM_COLOR: always` |
| チェックアウト | `actions/checkout@v6` |

| ジョブ | 目的 | OS | ツールチェーン | コマンド | 失敗の条件 |
|--------|------|----|----------------|----------|------------|
| `lint` | 書式と静的検査 | `ubuntu-24.04`(タイムアウト 10 分) | `dtolnay/rust-toolchain@stable`(`rustfmt`、`clippy`) | `cargo fmt --all -- --check`、`cargo clippy --workspace --all-targets --locked -- -D warnings` | 書式の差分が 1 つでもある、または clippy の警告が 1 つでもある |
| `build-check` | 最小の Rust(MSRV)で全ターゲット(テストを含む)がビルドでき、依存が C のビルドを要しないこと | `ubuntu-24.04`、`windows-2022`(`fail-fast: false`。15 分) | `dtolnay/rust-toolchain@1.100.0` | `cargo build --workspace --locked --all-targets`。Linux のみ追加で、bash で次を検査: ① `cargo tree -p conductor-cli -e normal,build --locked` の出力に `cc`・`cmake`・`pkg-config`・`bindgen` の `v<版>` がない ② `cargo tree --workspace -e normal --locked` の `-sys v` のうち、`linux-raw-sys`・`windows-sys` 以外がない | ビルドの失敗、または検査で該当する依存が見つかる(`exit 1`) |
| `test` | 3 つの OS でテストが通ること | `ubuntu-24.04`、`macos-latest`、`windows-2022`(`fail-fast: false`。15 分) | `stable` | `cargo test --workspace --locked` | テストの失敗(30 秒の上限を含む) |
| `msrv-test` | MSRV でも、ビルドだけでなくテストが通ること | `ubuntu-24.04`(15 分) | `@1.100.0` | `cargo test --workspace --locked` | テストの失敗 |
| `coverage` | 行カバレッジ 80% の維持(要件 NFR5) | `ubuntu-24.04`(20 分) | `stable`(`llvm-tools-preview`)+ `cargo-llvm-cov` | 5 章のコマンド | 行カバレッジが 80% 未満 |
| `docs-check` | README に必要な節・語が残っていること | `ubuntu-24.04`(5 分) | なし | bash で、`README.md` に次の 7 つの文字列がすべて含まれるかを `grep -q --` で確かめる: `SHA-256`、`Gatekeeper`、`終了コードの表`、`ステートファイルの場所`、`at-least-once`、`シグナル`、`convert` | 1 つでも欠けると `README.md is missing: <語>` を出して失敗 |
| `startup-bench` | 起動のオーバーヘッドの測定(NFR1。3.1 節) | `ubuntu-24.04`、`macos-latest`、`windows-2022`(`fail-fast: false`。20 分) | `stable` | `cargo test -p conductor-cli --release --locked --test nfr_perf -- --nocapture`(環境変数 `STARTUP_BENCH_ENFORCE: "false"`) | `continue-on-error: true` のため、ジョブが失敗しても全体は失敗にならない。基準超えは警告のみ(強制するには環境変数を `true` にする) |

補足:

- `msrv_consistency.rs` が `ci.yml` を読むため、`ci.yml` の書き方(`dtolnay/rust-toolchain@<版>` の形、コメントの文言)を変えるときは、そのテストも見る。
- マージの関門に、`lint`・`build-check`・`test`・`msrv-test`・`coverage`・`docs-check` を使う想定(ブランチ保護の設定そのものはリポジトリの外にあり、未確認)。

### 6.2 `audit.yml`(名前 `Audit`)

| 項目 | 内容 |
|------|------|
| 目的 | 既知の脆弱性の検査。結果は警告として出し、マージはブロックしない |
| 起動 | `pull_request`(対象 `main`)、`schedule`(cron `0 3 * * 1` = 毎週月曜 03:00 UTC) |
| 権限 | `contents: read` |
| ジョブ `audit`(名前 `cargo audit`) | `ubuntu-24.04`、10 分、**`continue-on-error: true`**。`actions/checkout@v6` → `dtolnay/rust-toolchain@stable` → `taiki-e/install-action@v2`(`tool: cargo-audit`)→ `cargo audit` |
| 失敗の条件 | `cargo audit` が脆弱性を報告すると、ジョブは失敗するが、`continue-on-error` のため、ワークフロー全体は失敗にならない |

---

## 7. リリース(`release.yml`)

### 7.1 手順(人の操作)

1. すべての `Cargo.toml`(4 クレート)の版を新しい版に上げて、`main` にマージする(例: `0.9.0`)。`Cargo.lock` も更新する。
2. `main` のそのコミットに、タグ `v<版>`(例: `v0.9.0`)を付けて push する。タグは `main` の上にあること。
3. ワークフローが、下書き(draft)のリリースを作る。人が内容を確認して、公開する(自動では公開しない)。

タグの規則: パターン `v*`。`v` を除いた文字列が `crates/cli/Cargo.toml` の版と**完全に一致**すること。版に `-` を含む(例: `0.9.0-rc.1`)と、プレリリース。

### 7.2 ジョブ

| ジョブ | 依存 | OS | 内容 | 失敗の条件 |
|--------|------|----|------|------------|
| `verify-tag` | なし | `ubuntu-24.04`(5 分) | `actions/checkout@v6`(`fetch-depth: 0`)。bash: タグ名 `${GITHUB_REF_NAME}` から `v` を除いて `version`。① `git merge-base --is-ancestor "$GITHUB_SHA" origin/main`(タグのコミットが `main` の祖先)② `crates/cli/Cargo.toml` の最初の `^version` の行の値 `crate` が `version` と等しい。出力: `version`、`prerelease`(`version` に `-` を含めば `true`、含まなければ `false`) | タグが `main` 上にない(`tag is not on main`)、または版の不一致(`tag X != crate version Y`) |
| `checks` | `verify-tag` | — | `uses: ./.github/workflows/ci.yml`(6.1 節の全ジョブを再利用。マージの関門と同じ検査) | ci の `startup-bench` は `continue-on-error` のため影響しない。それ以外のジョブの失敗で、以降が止まる |
| `build` | `verify-tag`、`checks` | 表のとおり(30 分、`fail-fast: true`) | 下の表の 4 つの対象。`dtolnay/rust-toolchain@stable`(`targets: <対象>`)。musl は `sudo apt-get update && sudo apt-get install -y musl-tools`。`cargo build --release --locked --bin conductor --target <対象>`(**`conductor` だけ**。テスト補助は含めない)。パッケージ(下記)。展開した動作確認(下記)。`actions/upload-artifact@v7`(名前 = 対象、`if-no-files-found: error`) | ビルド・パッケージ・動作確認・アップロードのどれかの失敗 |
| `draft-release` | `verify-tag`、`build` | `ubuntu-24.04`(10 分。`permissions: contents: write`) | `actions/download-artifact@v8`(`path: dist`、`merge-multiple: true`)。`cd dist && sha256sum * > SHA256SUMS`。`gh release create "$TAG" dist/* --repo "$GITHUB_REPOSITORY" --title "$TAG" --draft --generate-notes [--prerelease] --notes "<注記>"`(トークンは `github.token`) | `gh` の失敗 |

ビルドの対象:

| 対象(`target`) | OS | 形式 | 実行ファイル |
|------------------|----|------|--------------|
| `x86_64-unknown-linux-musl` | `ubuntu-24.04` | `tar.gz` | `conductor` |
| `aarch64-apple-darwin` | `macos-latest` | `tar.gz` | `conductor` |
| `x86_64-apple-darwin` | `macos-latest`(クロスビルド) | `tar.gz` | `conductor` |
| `x86_64-pc-windows-msvc` | `windows-2022` | `zip` | `conductor.exe` |

パッケージ(bash): `name="conductor-<版>-<対象>"`、ディレクトリ `name/` を作り、`target/<対象>/release/<実行ファイル>` と `LICENSE` をコピーする。
`zip` は `7z a "$name.zip" "$name"`、`tar.gz` は `tar czf "$name.tar.gz" "$name"`。成果物の名前は `conductor-<版>-<対象>.<拡張子>`
(`<版>` は `v` なし)。展開すると、最上位に `conductor-<版>-<対象>/` のディレクトリがあり、その中に実行ファイルと `LICENSE`。

動作確認(`x86_64-apple-darwin` 以外。実行できない対象を除く): 成果物を `smoke/` に展開し、`<実行ファイル> --version` を実行する。Linux の musl は、
さらに `file` の出力が `statically linked` または `static-pie linked` に一致することを確かめる(静的リンク)。

成果物(リリースに添付): 上の 4 つのファイルと `SHA256SUMS`(`sha256sum` の形式)。リリースの注記は `--generate-notes` の自動生成に、固定の 1 文を加える:
`Intel macOS (x86_64-apple-darwin): 動作の確認なし(クロスビルド)。SHA256SUMS で、ダウンロードしたファイルを確かめてください。`
署名・公証はしない(macOS は Gatekeeper の警告の回避手順を README に書く)。

---

## 8. 起動の性能計測(`startup-bench`)と `STARTUP_BENCH_ENFORCE`

- 実体は `cli/tests/nfr_perf.rs`(3.1 節の表)。CI では `--release` で 3 OS で実行し、測定値をジョブの Summary(`GITHUB_STEP_SUMMARY`)に出す。
- 既定は計測のみ。基準を超えた場合も、警告を出すだけ(`警告: <名前>: <値> ms が基準 <限度> ms を超えた`)。
- 強制にするには、`ci.yml` の `STARTUP_BENCH_ENFORCE` を `"true"` に変える。強制にすると、基準超えの測定が panic になる。
- 強制の対象外(常に強制): ジョブ実行中の CPU 時間 100 ms 以下(UNIX)、同時の 2 つの `run` の整合。

---

## 9. デモ(`demo/`)

### 9.1 構成

```text
demo/README.md                 使い方と各場面の説明
demo/conductor.toml            UNIX 版の定義(パイプライン nightly)
demo/retry.toml                UNIX 版のリトライの定義(パイプライン retry-demo)。場面 10 だけが使う
demo/env.toml, demo/demo.env   UNIX 版の環境変数の定義(パイプライン env-demo)と `.env`。場面 11 だけが使う(v0.9.0 以降)
demo/run-demo.sh               UNIX 版のスクリプト(sh)
demo/scripts/{extract,transform,load,notify,flaky,show-env}.sh   UNIX 版のジョブ(show-env は v0.9.0 以降)
demo/windows/conductor.toml, retry.toml, env.toml, demo.env, run-demo.cmd   Windows 版(cmd)
demo/windows/scripts/{extract,transform,load,notify,flaky,show-env}.cmd   Windows 版のジョブ
```

### 9.2 実行の仕方

| 項目 | UNIX 版 | Windows 版 |
|------|---------|------------|
| 実行 | `demo/run-demo.sh`(リポジトリの根から) | `demo\windows\run-demo.cmd` |
| 実行ファイルの選択 | 環境変数 `CONDUCTOR`。なければ `target/release/conductor`。無ければ `cargo build --release --locked --bin conductor` | `CONDUCTOR`。なければ `target\release\conductor.exe`。無ければ同じビルド |
| 表示の言語 | `DEMO_LANG`(既定 `ja_JP.UTF-8`。英語は `en_US.UTF-8`)を `LANG` に設定。`LC_ALL`、`LC_MESSAGES` は外す | `DEMO_LANG`(既定 `ja_JP.UTF-8`。`en_US` のようにも書ける)。`LC_ALL`、`LC_MESSAGES` は空に。コードページを実行中だけ 65001 にして、終わりに戻す |
| 一時停止 | `DEMO_PAUSE=1` のときだけ、各場面のあとで Enter を待つ | 同じ |
| ジョブの待ち時間 | `DEMO_SLEEP`(既定 0.5 秒。`sleep` に渡すので小数可) | `DEMO_SLEEP`(整数の秒。既定 1。`0` で待たない。内部は `ping -n <秒+1> 127.0.0.1`) |
| ジョブの終了コード | 環境変数 `DEMO_<ジョブ名の大文字>_EXIT`(既定 0) | 同じ |
| 作業場所 | `mktemp -d` の一時ディレクトリに定義とスクリプトを写して実行し、終了時に `trap` で消す(リポジトリに何も残らない) | `%TEMP%\conductor-demo-<乱数>` に写して実行し、終わりに `rmdir /s /q` |
| 事前の環境変数の処理 | `DEMO_RUN_ID`、`CONDUCTOR_RUN_ID` を消す(利用者の環境の影響を除く) | 同じ(空にする) |
| 色 | 見出し・コマンド・終了コードの行に ANSI の色。`conductor` の `watch` なども色あり | 画面に色なし |
| 各コマンドの出力 | `$ <コマンド>` の行(色つき)、実行、`→ 終了コード <N>` | `$ conductor <引数>`、実行、`→ 終了コード <N>` |
| ステートの一覧 | `ls conductor.toml*.state`(なければ `(なし)`) | `dir /b conductor.toml*.state`(なければ `(なし)`) |

### 9.3 定義

`demo/conductor.toml`(Windows 版は、コマンドが `scripts\<名>.cmd` で、`notify` は `["cmd", "/c", "scripts\\notify.cmd"]`):

- `[pipeline]`: `name = "nightly"`、`run_id_env = "DEMO_RUN_ID"`。
- `extract`: `command = "sh scripts/extract.sh"`、`warning_exit_codes = [1]`、`[[job.transitions]] on = "warning", next = "notify"`。
- `transform`: `sh scripts/transform.sh`。`load`: `sh scripts/load.sh`。
- `notify`: `command = ["sh", "scripts/notify.sh"]`(配列形式の例)。

`demo/retry.toml`: `[pipeline] name = "retry-demo"`、ジョブ `flaky`(`sh scripts/flaky.sh`)、`[job.retry] exit_codes = [75]`、`max = 3`、`interval = "1s"`。

ジョブのスクリプト: `echo "  [<名>] 実行中..."` → 待つ → `DEMO_<名>_EXIT` の終了コードを表示して、その値で終了。
`flaky` は、カレントディレクトリの `flaky.count` に実行回数を覚え、`DEMO_FLAKY_FAILS`(既定 2)回までは終了コード 75 で失敗し、それ以降は 0 で終わる。

`demo/env.toml`(v0.9.0 以降。場面 11): `[pipeline] name = "env-demo"`、`description = "環境変数のデモ"`、`env_file = "demo.env"`、`[pipeline.env]` に `DEMO_GREETING = "hello from [pipeline.env]"` と `DEMO_STAGE = "pipeline"`、ジョブ `show-env`(`sh scripts/show-env.sh`)の `[job.env]` に `DEMO_STAGE = "job"`。`demo/demo.env`: `DEMO_FROM_FILE=from demo.env`、`export DEMO_QUOTED="two words"`(コメントつき)、`DEMO_STAGE=dotenv`(コメントで、優先順位の例を説明)。`show-env.sh` は、`DEMO_GREETING`、`DEMO_FROM_FILE`、`DEMO_QUOTED`、`DEMO_STAGE`、`CONDUCTOR_PIPELINE`、`CONDUCTOR_JOB_ID`、`CONDUCTOR_RUN_ID`(なければ `(なし)`)、`CONDUCTOR_ATTEMPT` を `  [show-env] <名前>=<値>` の形で 1 行ずつ表示する。

### 9.4 場面

| 場面 | 内容(実行するコマンド) | 見せること |
|------|--------------------------|-----------|
| 0 | `cat conductor.toml` | 定義の中身 |
| 1 | `pipeline -v nightly` | 全部成功。書いた順に 1 つずつ |
| 2 | `DEMO_EXTRACT_EXIT=1` で `pipeline -v nightly` | `extract` が警告 → `transform`・`load` をスキップして `notify` へ |
| 3 | `DEMO_LOAD_EXIT=2` で `pipeline -v nightly`、続けて `watch` | `load` のエラーで止まり、終了コード 2。`watch` で止まった位置 |
| 4 | `pipeline -v nightly`(再実行)、`watch`、ステートの一覧、`pipeline -v --remove-on-complete nightly`、一覧、`watch` | `load` から再開(`transform` は再実行しない)。完了してもステートは残り、`watch` が完了の行を出す。`--remove-on-complete` で完了時にステートを消す |
| 5 | `run -v extract`、`reset` | 個別実行と初期化 |
| 6 | `convert conductor.toml --to yaml` | TOML から YAML への変換 |
| 7 | `validate -v`、誤りの定義 `bad.toml`(`command = "sh -c 'echo hi"`)を作って `validate --config bad.toml` | 正しい定義は 0(`definition is valid`)、引用符が閉じていない定義は 64(`command has an unclosed quote`)。ジョブは実行されず、ステートも作られない |
| 8 | `pipeline -v --run-id build-1 nightly`(`load` で失敗)、環境変数 `DEMO_RUN_ID=build-2` で `pipeline -v nightly`、一覧、`watch --run-id build-1`、`pipeline -v --run-id build-1 nightly`(再開)、一覧、`watch --run-id build-1`、`reset --run-id build-1`、`reset --run-id build-2`、一覧 | run ID ごとに別のステートファイル(`conductor.toml.<run ID>.state`)。同じ run ID で再開。完了後もステートは残り、`reset --run-id` で片づける |
| 9 | `DEMO_RUN_ID=env-1` で `pipeline`(`load` で失敗)、`DEMO_RUN_ID=env-1` と `--run-id opt-1`、`--run-id ""`、`--run-id opt-2`、一覧、`reset --run-id opt-1`、一覧、`reset --all`、一覧、`watch` | `--run-id` は環境変数に勝つ。`--run-id ""` は run ID なしに固定。`reset --run-id` は 1 つ、`reset --all` は全部。最後の `watch` は `no state` |
| 10 | `rm -f flaky.count`、`pipeline -v --config retry.toml retry-demo` | `flaky` が 2 回、終了コード 75 で失敗。`conductor` は 1 秒待つたびに標準エラーへ `conductor: retry: job 'flaky' exited with code 75; retry <n> of 3 in 1 seconds` を出して実行し直し、3 回目で成功(`job=flaky result=success`) |
| 11(v0.9.0 以降) | `cat env.toml`、`cat demo.env`、`pipeline -v --config env.toml env-demo`。続けて、`env_file = "missing.env"` の定義 `bad-env.toml` で `pipeline`、`A=1`・`A=2` の `dup.env` を指す定義 `dup-env.toml` で `validate` | `show-env` が、`.env` < `[pipeline.env]` < `[job.env]` の優先順位で足された環境変数(`DEMO_STAGE=job`)と、自動の `CONDUCTOR_*`(run ID がなければ `CONDUCTOR_RUN_ID=(なし)`、`CONDUCTOR_ATTEMPT=1`)を表示する。`.env` が無いと、ジョブを動かさずに 64。`validate` は、重複キーを警告して 0 |
| 12(同) | `graph`、`graph -v`、`graph --format mermaid`、`DEMO_LOAD_EXIT=2` で `pipeline nightly`(`load` で止める)、`graph --state`、`reset` | `extract` が警告なら `notify` へ飛ぶ遷移の図(右の線が飛び越し)、`-v` の詳細、Mermaid、`--state` の `[OK]`・`[NG]`・`[  ]` と再開位置(`[NG] load  <- resume here`)。`graph` はステートを読むだけ |
| 13(同) | `pipeline nightly --dry-run`、`--assume extract=1`、`--assume load=2`、`run extract --dry-run`、ステートの一覧 | 計画(ジョブは実行されない)、`extract` が警告で `transform`・`load` が遷移でスキップされ `notify` へ、`load` で停止(`result (simulated): stopped (error: load)`)、`run` の次の動き(`next step: next job transform`)。ステートは作られない |
| 14(同) | `init --config new.toml starter`、`validate --config new.toml starter`、もう一度 `init`、`init --config new.toml --force starter`、`init --config - starter` | `created new.toml`、ひな形が `validate` に通る、2 回目は終了コード 69(`already exists (use --force to overwrite)`)、`--force` で `replaced new.toml`、`--config -` は標準出力にひな形 |

最後に `デモは以上です。`(Windows 版も同じ)を出す。

### 9.5 デモのテスト

| テスト | OS | 内容 |
|--------|----|------|
| `cli/tests/unix_demo.rs` | UNIX | `sh demo/run-demo.sh` を、`CONDUCTOR=<テスト用のバイナリ>`、`DEMO_LANG=en_US.UTF-8`、`DEMO_SLEEP=0`、`TMPDIR=<一時ディレクトリ>` で実行。終了コード 0、見出し `=== 0. ` から `=== 14. `、場面 7・4・8・9・10 の要点の行(`definition is valid`、`command has an unclosed quote`、`→ 終了コード 64`、`completed`、`conductor.toml.build-1.state`、`skipped (already executed)`、`conductor.toml.opt-1.state`、`no state`、リトライの知らせ 2 行、`[flaky] 3 回目の実行`、`job=flaky result=success`)と、場面 11〜14 の要点の行(`[show-env] DEMO_GREETING=hello from [pipeline.env]`、`DEMO_FROM_FILE=from demo.env`、`DEMO_QUOTED=two words`、`DEMO_STAGE=job`、`CONDUCTOR_PIPELINE=env-demo`、`CONDUCTOR_JOB_ID=show-env`、`CONDUCTOR_RUN_ID=(なし)`、`CONDUCTOR_ATTEMPT=1`、`missing.env`、``duplicate key `A` ``。`*   extract` から `o @end` までの図、`command: sh scripts/extract.sh`、`success -> transform`、`flowchart TD`、`[NG] load  <- resume here`、`[OK] extract`。`plan (--dry-run): pipeline nightly` の計画、`assume: extract=1`、`[skip (transition)]`、`stopped (error: load)`、`target job: extract`、`next step: next job transform`。`created new.toml`、`name = "starter"`、`already exists (use --force to overwrite)`、`→ 終了コード 69`、`replaced new.toml`)が出る。利用者の `CONDUCTOR_RUN_ID`・`DEMO_RUN_ID`・`LC_ALL`・`LC_MESSAGES`・`NO_COLOR` に左右されない(`zzz` が出力に現れない)。終了後に `TMPDIR` に何も残らない |
| `cli/tests/windows_demo.rs` | Windows | 定義とスクリプトを一時ディレクトリへ写し、本物の `conductor` で、場面 1〜14 の終了コードと結果を確かめる(定義が 4 ジョブと `extract` の警告の遷移を持つこと、場面 1 は 0、場面 2 は `notify` へ進み 0、場面 3・4 は 2 のあと `load` から再開、`run`/`reset`/`convert`、`validate` が 0 と 64、run ID ごとにステートが分かれる、`--run-id` が環境変数に勝つ、`reset --run-id` と `--all`、リトライで 2 回失敗してから成功、場面 11(環境変数が子に渡り優先順位は `[job.env]` が最も強い。`.env` が読めないと 64、`validate` は重複キーを警告して 0)、場面 12(`graph` の図と `--state`)、場面 13(`--dry-run` は何も実行せずステートも作らない)、場面 14(`init` は 2 回目に 69、`--force` で上書き)、利用者の run ID に影響されない) |

---

## 10. 品質ゲートの再現手順(まとめ)

ローカルで、マージ前に次をすべて通す。

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo build --workspace --locked --all-targets          # MSRV の 1.100.0 でも通すこと
cargo test --workspace --locked                           # MSRV の 1.100.0 でも通すこと
cargo llvm-cov --workspace --locked \
  --ignore-filename-regex 'bin/conductor-(cli-)?test-helper' \
  --fail-under-lines 80 --summary-only
cargo audit                                               # 警告のみ(マージはブロックしない)
```

加えて、CI で次を確かめる: Windows(`windows-2022`)と macOS(`macos-latest`)での `cargo test`、依存に C のビルドがないこと、README の必須の 7 語、MSRV の記載の一致。
リリース前に、Windows の実機で Ctrl+C のチェックリスト(`crates/cli/tests/WINDOWS_CTRL_C_CHECKLIST.md`)を確かめ、結果を記録する。

---

## 11. 実装と README の差

| 項目 | README | 実装・CI |
|------|--------|----------|
| テストの案内 | 「ビルドしたあとの確認には、テストが使える」として `cargo test --workspace --locked` | 一致。ただし CI は 3 OS + MSRV。README は CI、カバレッジ、clippy、`startup-bench` を説明しない(開発者向けの情報は README にない) |
| リリース | 成果物の名前・`SHA256SUMS`・Intel macOS はクロスビルドで動作未確認 | 一致(`release.yml`)。README の `<版>` の例が `0.7.0`(現行は 0.9.6。例なので誤りではない) |
| MSRV | `Rust(1.100 以降)` | 一致(`msrv_consistency.rs` が強制) |
| 性能の測定の表示 | 記載なし | `nfr_perf.rs` の出力文言は「debug ビルド」だが、CI の `startup-bench` は `--release`(文言と実態の差。基準の判定には影響しない) |
| カバレッジ・clippy・audit | 記載なし | CI で強制/警告(6 章)。README に書かれていない |
| デモ | `demo/README.md` に説明 | 実装(スクリプトとテスト)と一致。Windows 版は色なし・`DEMO_SLEEP` が整数・既定 1 秒、という差が README に書かれている |

## 12. 未確認の事項

- ブランチ保護(どのジョブを必須にするか)は、リポジトリの設定であり、リポジトリ内のファイルからは確認できない。
- 現在の行カバレッジの実測値、各 NFR の計測値は、この調査では実行していないため不明(基準だけを記した)。
- `cargo audit` の現在の結果、`macos-latest`・`windows-2022` のイメージの将来の変更による影響は未確認。
- Windows のチェックリストの記録欄は空で、手動確認の実施の有無は不明。
- `startup-bench` を強制(`STARTUP_BENCH_ENFORCE=true`)に切り替える予定の有無は、コードのコメント(「配分の確定のあと、CI が切り替える」)以上の情報がない。
