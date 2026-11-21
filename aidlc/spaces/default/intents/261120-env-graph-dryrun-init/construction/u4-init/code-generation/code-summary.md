# コードの要約(Code Summary): U4 u4-init(`conductor init`)

## 作った・変えたファイル

- `conductor-runtime`: 新しい `file_create.rs`(`create_file`、`CreateOutcome`、`CreateError`)、`lib.rs`。
- `conductor-cli`: 新しい `init.rs`(流れ)、`report/init_template.rs`(ひな形の生成。日本語と英語)、`args.rs`、`types.rs`(`init`、`--force`、位置引数の名前、`InitTarget`、`InitOptions`)、`deps.rs`(`Runtime` の trait に `create_file`)、`exit.rs`(`EXIT_EXISTS = 69`)、`report/messages.rs`、`report/mod.rs`、`lib.rs`。
- `README.md`、`demo/`(場面 14。`demo/README.md` の場面の表も更新)。
- 新しいテスト: `runtime` 1 ファイル(8 件)、`cli` 5 ファイル(26 件)。既存のテストの修正(`args_grammar.rs`、`report_messages.rs`、`common/mod.rs`、`unix_demo.rs`、`windows_demo.rs`)。
- 変えていない: `Cargo.toml`、`Cargo.lock`、CI の設定(新しい依存なし)。

## 主な実装の決定

- ファイルの作成: 新規は `create_new`(排他的)で、失敗したら `symlink_metadata` で分類する(ディレクトリ、既にある、親が無い、権限、そのほか)。`--force` は、同じディレクトリの一時ファイル(`state/writer.rs` の名前の作り方を再利用)に書いて置き換える。内容は UTF-8、LF のまま。
- `init` は、定義、`.env`、ステートを読まず、シグナルを準備しない。引数の誤り(68)は、ファイルに触れる前に返す。既にあれば 69、作れなければ 65、標準出力(`--config -`)のときは、ひな形だけを出す(`--force` は無視)。
- ひな形は、`functional-spec.md` の原本と、日本語と英語のどちらも、1 文字も違わず一致することを、テストで確かめた(期待値は原本から書いた)。`parse_definition`、`check_command`、U1 の検査を通ることも、名前を変えて(エスケープが要る文字を含めて)確かめた。U3 の `--dry-run` と組み合わせた実バイナリのテストもある。

## テストの結果(macOS)

- `cargo fmt --all -- --check`: 通った。`cargo clippy --workspace --all-targets --locked -- -D warnings`: 警告なし。
- `cargo test --locked --workspace`: 1017 件が通り、失敗 0 件(開発担当の報告)。
- `cargo llvm-cov --workspace --locked --ignore-filename-regex 'bin/conductor-(cli-)?test-helper' --fail-under-lines 80 --summary-only`: 行カバレッジ **97.94%**(9308 行のうち未通過 192 行)。承認された検証のコマンドを、オーケストレーターが再実行して、同じ数値を確かめた。しきい値は下げていない。
- 新しいコードの行カバレッジ: `file_create.rs` 88.0%、`init_template.rs` 100%、`types.rs` 100%。

## 書いたが、実行していないテスト(理由つき)

- `crates/cli/tests/windows_demo.rs`(場面 14 のテスト)と `demo/windows/run-demo.cmd` の場面 14: `#![cfg(windows)]` で、この macOS では実行も、コンパイルもされない。CRLF のまま編集したこと(`git ls-files --eol` が `i/crlf w/crlf`、差分が追加した行だけ)は確かめた。CI の `test`(windows)で確認する。
- ubuntu の `test` と、最小の Rust 1.100.0 の `msrv-test`: この環境では実行できない。CI で確認する。
- 権限(`0o555`)とシンボリックリンクのテストは、`#[cfg(unix)]` の中で、実行時に効果を確かめ、効かない環境(root など)では確かめずに通す。Windows では、この 2 つの確認は、実質、行われない。
- (開発担当の報告)計測つきの実行で、`unix_demo` が 30 秒の上限を超える場合がある(計測のため遅い)。通常ビルドでは通る。オーケストレーターの検証のコマンドの再実行では、通った。

## 既知の限界(レビューの助言。受け入れた)

- パスの末尾が `/` のとき(例: `conductor init --config new.toml/`)の分類は、OS による。macOS では、存在しないファイル名に `/` を付けると `parent_missing`(設計の BR1.3 は `other_io`)になる。このケースを固定するテストは無い。
- 設計の文書(`functional-spec.md` の「パスは、切り詰めて」)と、実装(エスケープのみ。切り詰めない)が食い違う。実装は、「定義ファイルのパスと同じ扱い」に合わせた。非常に長い `--config` のパスは、全体が表示される。
- 部分的に書いたファイルと一時ファイルの後始末は、実装されているが、書き込みの失敗を起こすテストは無い(移植性のある起こし方が難しい)。レビューで、手で確かめ、残りは無かった。
- 権限とシンボリックリンクのテストは、実行時に効果を確かめ、Windows と root では、確かめずに通る。
- マージ前に、GitHub Actions の `test`(windows と ubuntu)、`msrv-test`、`coverage` で確認する。

## 計画からの逸脱

1. `InitOptions` を `CliCommand` の欄にしなかった: 欄を足すと、`ParsedArgs` に clippy の `large_enum_variant` が出たため、`config_path`、`pipeline_name` と新しい `force` から、`CliCommand::init_options()` で取り出す形にした(lint は許可していない)。
2. パスの表示: 計画の 9.4 は「切り詰めて」と書いたが、機能の仕様は「定義ファイルのパスと同じ扱い」(エスケープのみ)なので、仕様に合わせて、切り詰めていない。
3. 名前が空のときの誤りの対象は、固定の語 `pipeline name`(名前の値を含めない)。`--force` を `init` 以外に付けたときは、使い方の誤り 68(対象は `--force`)。
4. `demo/README.md` は、開発担当が更新しなかったので、オーケストレーターが、場面 14 の行と、Windows 版の「場面 0〜14」を足した。
5. 作業手順の違反(開発担当の報告): 本ユニットでの有無は、報告に無い。
6. 11.4 の 3 ファイル(このファイル、`traceability.json`、`source-manifest.json`)は、オーケストレーターが作った。
