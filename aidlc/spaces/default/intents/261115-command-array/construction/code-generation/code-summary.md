# Code Summary(command-array)

zero-Unit・express・Minimal・test-after。承認済みの計画(13 手順)のとおり実装し、全手順を完了した。

## 作成・変更したファイル

新しいファイルは作っていない(ソースのヘッダの追加は不要)。

### conductor-core
- `crates/core/src/types.rs`: `enum Command { Line(String), Argv(Vec<String>) }`、`From<&str>`・`From<String>`、`program_name()` を追加。`JobDefinition.command` と `JobExecution.command` を `Command` に変更。
- `crates/core/src/definition/walk.rs`: `Walker::command` を追加(文字列と配列の両方を読む。要素の型の誤り、空の配列、先頭の要素が空は `EmptyField` または `TypeMismatch`。2 番目以降の空の文字列は有効)。`RawJob.command` を `Option<Command>` に。
- `crates/core/src/lib.rs`: doc コメントを新しい型に合わせた。

### conductor-runtime
- `crates/runtime/src/split.rs`: `command_line(&Command)`(`Line` は従来の `split`、`Argv` は要素をそのまま)を追加。`check_command` は `&Command` を受ける。
- `crates/runtime/src/runner.rs`: `run_job` が `command_line` を使う(引数の作り方以外は変更なし)。
- `crates/runtime/src/lib.rs`: doc の表を更新。

### conductor-cli
- `crates/cli/src/deps.rs`: `Runtime::check_command` が `&Command` を受ける。
- `crates/cli/src/ctx.rs`: 起動失敗の名前の代替を `program_name()` に。

### 文書・デモ・版
- `README.md`: 「command の書き方」に「配列で書く(v0.2.0 以降)」を追加。キー表の `command` の説明を更新。
- `demo/conductor.toml`: `notify` の `command` を `["sh", "scripts/notify.sh"]` に。`demo/README.md`: 1 行追加。
- 4 つの `crates/*/Cargo.toml` の `version` を 0.2.0 に。`Cargo.lock` が追従(`cargo update --workspace --offline`)。

### 既存テストの型の追従(挙動は変えていない)
- `crates/core/tests/definition_valid.rs`、`nfr_security.rs`: `command` の比較に `.into()`。
- `crates/runtime/tests/common/mod.rs`(`job` が `impl Into<Command>`)、`flow_with_state.rs`、`nfr_content.rs`、`nfr_robust.rs`。
- `crates/cli/tests/common/mod.rs`: 偽の `Runtime` の `check_command` の署名。

## 計画からの差異

- 実装の差異はない。`converter` は、計画の見込みどおり、実装の変更なしで往復できた。
- 起動失敗の名前: 計画は「`program_name()` を使う」。実装は、従来どおり `check_command` の結果の `program`(引用符を解いた名前)を優先し、失敗したときの代替を `program_name()` にした(`Line` の表示を変えないため。`Argv` は `check_command` が先頭の要素をそのまま返す)。
- Windows の `.cmd` のテスト(FR2.3)は `cfg(windows)` のため、この環境では実行していない(型検査のみ)。

## 追加したテスト(16 件 + Windows 専用 1 件)

| クレート | 件数 | 内容 |
|----------|------|------|
| core | 7 | `From`、`program_name`(`types.rs` 2 件)。配列が読める(`definition_valid.rs` 1)。型の誤り、空の配列と先頭の要素、`["echo",""]` が有効、文字列の空は誤り、メッセージに値を含まない(`definition_errors.rs` 4) |
| runtime | 5(+1) | `command_line` の単体 2(要素がそのまま、先頭が空は `Empty`)。統合 3(引数がそのまま渡る、先頭が空は起動失敗、存在しないプログラム)。Windows 専用 1(`cmd_への配列の危険な引数は注入にならない`) |
| cli | 2 | 配列のジョブの起動失敗で先頭の要素の名前が出る(`flow_run.rs`)。実バイナリで `run`・`pipeline` が文字列と同じ結果(`binary_basic.rs`) |
| converter | 2 | TOML → YAML → TOML、YAML → TOML → YAML |

## 確認の結果(この環境: macOS)

- `cargo fmt --all -- --check`: 成功。
- `cargo clippy --workspace --all-targets --locked -- -D warnings`: 警告なし。
- `cargo test --workspace --locked`: **580 passed、0 failed、1 ignored**(基準 564 件 + 新しい 16 件。既存は成功のまま)。
- `cargo +1.85 build --workspace --locked --all-targets`: 成功。
- `cargo check --workspace --tests --target x86_64-pc-windows-msvc --locked`: 成功(`cfg(windows)` のテストを含む)。
- `cargo llvm-cov ... --summary-only`: 行カバレッジ **97.79%**(基準 97.66%、下限 80%)。関数 97.97%、リージョン 97.61%。
- `cargo build --workspace --locked`(0.2.0 への版の更新のあと): 成功。
- デモ: `DEMO_SLEEP=0.1 sh demo/run-demo.sh`(0.2.0 のリリースバイナリで)すべての場面が完走し、終了コード 0。`notify`(配列)は、場面 1・2・4 で実行されて成功した。

## 注意・実行していないもの

- 最初のデモの実行は、古い 0.1.0 のリリースバイナリ(`target/` に残っていた)で動いたため、新しいコードの確認にならなかった。`cargo build --release --locked --bin conductor` で作り直し、0.2.0 のバイナリで再実行して確認した。
- Linux と Windows での実行は、CI に任せる(Windows 専用のテストは、この環境では型検査のみ)。
- 手順の逸脱: ファイルは Edit/Write で書く決まりだが、2 か所でシェルを使った。`walk.rs` の型の置き換えに `sed -i`(2 行)、テストの追記 2 件(`definition_errors.rs`、`definition_valid.rs`)にヒアドキュメント。内容は計画の範囲内で、`git diff` で確認できる。
- `source-manifest.json` は、zero-Unit のため作っていない。
- 変更していない既存の所有ファイル: `.gitignore`、`LICENSE`、`crates/cli/tests/WINDOWS_CTRL_C_CHECKLIST.md`、`local-docs/requirements.md`。`README.md`、`Cargo.lock`、`demo/README.md` は、計画が名前を挙げた変更だけ。
