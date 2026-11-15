# Code Summary(validate-command)

zero-Unit(express、Minimal、test-after)。承認済みの計画の 9 手順を、計画の順(改名 → ワークスペースが緑 → `validate`)に実施した。

## 変更したファイル

- **改名(FR2): `PipelineDefinition.jobs` → `job`**
  - core: `types.rs`、`definition/walk.rs`、`flow/{decide,next_step,resume}.rs`(単体テストを含む)、`tests/{definition_valid,flow_sequences,nfr_performance,nfr_security}.rs`
  - runtime: `state/{consistency,record}.rs`(単体テストを含む)、`tests/{flow_with_state,nfr_content,nfr_performance}.rs`
  - cli: `run.rs`、`pipeline.rs`
  - converter(サンプルのキーのみ): `yaml_write.rs`、`toml_write.rs`、`error.rs`、`tests/read_resolve.rs`
  - 変えていないもの: ローカル変数・引数名の `jobs`、`Kind::NoJobs`、英語の文面、`[[jobs]]` を拒否するテスト、README の旧キーの注意書き。
- **`validate`(FR1)**: cli の `types.rs`(`CommandKind::Validate`)、`args.rs`(コマンド名、`-v` の許可、位置引数 0 か 1)、`ctx.rs`(`Ctx::check_definition` を切り出し。`prepare` が呼ぶ。順序と動作は不変)、`aux_commands.rs`(`run_validate`)、`lib.rs`(振り分け)、`report/mod.rs`(`DisplayEvent::DefinitionValid { job_count }`、水準 Verbose)、`report/messages.rs`(使い方の文言、`definition_valid`)。
- **文書(FR3)**: `README.md`(1 行目、使い方、コマンドの表、オプションの表、例、終了コードの表)。`docs-check` の語句 7 つは、すべて残っている(手元で grep を実行して確認)。
- **版(NFR4)**: 4 つの `Cargo.toml` を 0.4.0 に、`Cargo.lock` が追従(`cargo update --workspace --offline`)。
- **テスト**: `crates/cli/tests/flow_validate.rs`(新規。13 行のヘッダ付き)、`args_grammar.rs`、`report_messages.rs`、`binary_basic.rs`。

## 動作の要点

- `validate` は、シグナルの準備、ステートの読み書きと初期化、ジョブの実行をしない。`load_definition`(名前があれば照合)と全ジョブの `check_command` だけを行う。
- 成功は何も表示しない。`-v` で標準エラーに 1 行(`[conductor] definition is valid (N jobs)` / `[conductor] 定義は正しいです(ジョブ N 個)`。英語は N が 1 のとき `job`)。
- 定義の誤り・`command` の誤り・名前の不一致は、既存の表示で 64。使い方の誤り(名前 2 個以上、`--to` など)は 68。

## テスト(追加 12 件、期待値の更新 2 件)

- 追加: `args_grammar.rs` に 2 件、`report_messages.rs` に 1 件、`flow_validate.rs` に 8 件、`binary_basic.rs` に 1 件(合計 12)。
- 更新(件数は変えず): `args_grammar.rs` の `コマンドの名前`、`report_messages.rs` の使い方の一覧(`validate` を含める)。

## 確認の結果

- `cargo fmt --all -- --check`: 成功
- `cargo clippy --workspace --all-targets --locked -- -D warnings`: 警告なし
- `cargo test --workspace --locked`: 597 件成功、0 件失敗、1 件 ignored(基準は 585 件、1 ignored。差の 12 件が追加分)
- `cargo +1.85 build --workspace --locked --all-targets`: 成功(let チェーンなし)
- `cargo check --workspace --tests --target x86_64-pc-windows-msvc --locked`: 成功
- `cargo llvm-cov`(補助の実行ファイルを除く): 行 97.79%(基準 97.80%、下限 80%)、関数 97.99%、領域 97.60%
- 実際のバイナリ(`target/validate-check/`): 正しい定義 0(出力なし)、`-v` で 1 行(英語・日本語)、閉じていない引用符 64、遷移先の誤り 64、名前の一致 0、不一致 64、位置引数 2 個 68、`--help` に `validate`、`.state` は作られない、壊れた `.state` が隣にあっても 0(同じ状態で `run` は 65)。
- `DEMO_SLEEP=0.1 sh demo/run-demo.sh`: 終了コード 0。

## 逸脱とギャップ

- 逸脱: 計画は「新しいソースファイルは作らない」としたが、`validate` の流れのテストは、既存のテストに入れるより明確なため、新しいテストファイル `crates/cli/tests/flow_validate.rs` を作った(指示で許可された範囲)。
- ギャップ: `-v` の英語の文言は、1 件のとき `job`(それ以外は `jobs`)にした。計画の例にはない小さな追加。
- 未実行: CI(GitHub Actions)の 3 OS での実行(ローカルは macOS のみ。Windows は `cargo check --target` で型の確認まで)。`windows_demo` などの `cfg(windows)` のテストは動かしていない。
- 作業のフォルダ外に作ったもの: `target/validate-check/`(git で無視されるフォルダ)の確認用の定義 3 つと、壊れた `.state` 1 つ。
