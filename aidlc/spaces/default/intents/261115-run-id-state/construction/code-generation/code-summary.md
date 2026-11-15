# Code Summary(run-id-state)

zero-Unit(express、Minimal + NFR3 の境界のテスト)。承認済みの計画の 10 手順を、test-after で、層ごとに(core → runtime → cli → README → 版 → 確認)実施した。各区切りで、全体のビルドとテストが通ってからコミットした(4 コミット)。

## 変更したファイル

### conductor-core
- 新規 `crates/core/src/run_id.rs`: `RunId`(`parse`・`as_str`)、`RunIdError`(値を持たない)、`is_valid_env_name`、`peek_run_id_env`(ゆるい読み取り)、定数 `DEFAULT_RUN_ID_ENV`・`MAX_RUN_ID_LEN`。
- `lib.rs`(公開)、`types.rs`(`PipelineDefinition.run_id_env`、`PipelineState.run_id`)、`error.rs`(新しい `DefinitionErrorKind::InvalidEnvName`、`ALL` を 20 に)、`definition/walk.rs`(`PIPELINE_KEYS` に `run_id_env`、型と形の検査)、`flow/{decide,next_step,resume}.rs`(テストの構築の追従)。

### conductor-runtime
- 新規 `crates/runtime/src/state/reset_all.rs`: `reset_all_states`、`ResetAllReport { removed, kept }`。
- `types.rs`(`StateLocation::of(def, run_id)`)、`state/reader.rs`(`read_state(def, run_id)`、`run_id` の照合を `completed` の判定より前に、`TOP_KEYS` に `run_id`、`inspect_run_state`)、`state/writer.rs`(`write_state(def, run_id, state, dur)`、`reset_state(def, run_id)`、`run_id` はあるときだけ直列化)、`state/record.rs`(`initial_state(def, path, run_id)`)、`error.rs`(固定の理由 `RUN_ID_MISMATCH`)、`lib.rs`、`state/mod.rs`、`state/consistency.rs`(テストの追従)。

### conductor-cli
- `args.rs`(`--run-id`・`--all`、許可表、値の検査、`resolve_run_id`)、`types.rs`(`CliCommand.run_id`・`all`)、`deps.rs`(`Runtime` に `reset_all_states`・`run_id_env`、既存 3 メソッドの署名変更、`SystemRuntime`)、`ctx.rs`(`Ctx.env`、`prepare` に run ID の解決、`Session.run_id`、`aux_run_id`)、`run.rs`・`pipeline.rs`(run ID つきのステートの読み書き)、`aux_commands.rs`(`watch`・`reset`・`reset --all`)、`lib.rs`(`env` の受け渡し)、`report/mod.rs`(`StateContext::ReadRunId`、`DisplayEvent::ResetAllKept`、`ConsistencyMismatch.with_run_id`)、`report/messages.rs`(USAGE、警告、案内、`InvalidEnvName` の文)、`report/watch.rs`(`Run ID:` の行)。

### そのほか
- `README.md`(使い方、コマンドの表、オプションの表、定義ファイルの書き方、終了コードの表、「run ID でステートを分ける」の節、大文字小文字の注意)、4 つの `crates/*/Cargo.toml`(0.5.0)、`Cargo.lock`。

## 計画からの逸脱・判断

1. **`write_state` の引数の順**: `write_state(definition_path, run_id, state, durability)`(`run_id` を定義パスの直後に置き、`read_state`・`reset_state` と並びをそろえた)。
2. **`watch` の `Run ID:` の行の位置**: `Pipeline: <名前>` の行の次、表の見出しの前(1 行目は変えない。run ID なしの表示は不変)。`WatchTable` に別フィールドは足さず、`PipelineState.run_id` を使った(中身は、ファイル名と照合済み)。
3. **使い方の誤りの subject**: 値は決して入れない。`--run-id` の値の違反・重複・UTF-8 でない値は、subject が `--run-id`。環境変数の値の違反は、subject が `run ID (environment variable)`。`--all` と `--run-id` の併用は、`InvalidValue`(subject `--all`)。
4. **`reset --all` の「別の定義」の扱い**: 中身に `run_id` がない、または値が名前と違うファイルは、壊れていなくても消さず、警告にも出さない(別の定義 `x.toml.a` の run ID `b` の `x.toml.a.b.state` と、`x.toml` の run ID `a.b` が同じ名前になるため)。壊れている・読めない・大きすぎる・版が違うものは、`kept` に入れて警告する。長さ 0 のファイルは、持ち主が分からないので、消さず、警告もしない。
5. **`<定義ファイル名>.state`**(run ID なし)は、`reset --all` でも読まずに消すが、通常のファイルだけ(シンボリックリンクは消さない)。定義ファイル名が UTF-8 でないとき、run ID 付きの候補は探さない。
6. **`ConsistencyMismatch` と `Read` の案内**: run ID つきのステートの誤りの案内は `conductor reset --run-id <ID>`(新しい `StateContext::ReadRunId`、`with_run_id`)。run ID なしの文言は、そのまま(バイト単位で不変)。
7. **既存のテストの更新**: `flow_run.rs`・`flow_pipeline.rs` の「ロケールの判定が行われない」の確認を、`var_calls == 0` から `!locale_read()` に変えた(`CONDUCTOR_RUN_ID` の環境変数を読むのは、新しい仕様で、ロケールの遅延の性質は変わらないため。`FakeEnv` に読んだ名前の記録を足した)。ほかは、署名・フィールドの追従だけ。
8. **README の追加**: 計画の項目のほか、「(v0.5.0 以降)」の見出し、同じ run ID の同時起動の注意。

## 検出したこと・残る隙間(正直に)

- **ステートの誤りの表示に、ステートファイルのパスが出る**。パスには run ID が含まれる(`x.toml.<run-id>.state`)。これは従来の仕様(パスの表示)の延長で、run ID はすでに検査を通った値で、利用者自身が与えたもの。run ID の**値**を、独立して出す文(使い方の誤り)は、値を含めない。要件の「エラーの文に run ID の値を含めない」を厳密に読むと、ステートのパスの表示が該当する(要判断)。`reset --all` の警告のファイル名にも、同じ理由で run ID が入る(エスケープ済み)。
- `reset --all` は、`symlink_metadata` で通常のファイルと確認した後に、ファイルを開く(確認と使用のあいだの入れ替えの競合は、防いでいない。標準ライブラリだけでは、`O_NOFOLLOW` を使えない)。
- 大文字小文字を区別しないファイルシステムでの `A` と `a` は、README に書いた運用と、中身の照合(65)に任せている(専用のテストは、`A` と `a` の照合の不一致で代用)。
- Windows の実機のテストは、実行していない(`cargo check --target x86_64-pc-windows-msvc` のみ)。`reset --all` のシンボリックリンクのテストは、UNIX だけ。
- 実際のバイナリの確認で、スクラッチの下に、`printf` のリダイレクトで壊れた定義 1 つと、`touch` で空のフラグファイルと空のステートを作った(`target/runid-check/`、git の対象外)。ほかのファイルは、ファイル書き込みのツールで作った。

## テスト(新規 66 件。597 → 663 件)

| クレート | 新規 | 内容 |
|----------|------|------|
| core | 11 | `run_id.rs` の単体 8(境界の値、`peek_run_id_env`)、`definition_errors.rs` の 3(`run_id_env` の型・形・値を含めない)。`nfr_security.rs` の網羅に新しい種類を追加 |
| runtime | 20 | 単体 5(パス、`initial_state`、直列化、名前の判定 2)、`run_id_state.rs` の 15(往復、形の不変、照合 4、独立、`reset_all_states` の境界 7、シンボリックリンク、削除の失敗) |
| cli | 35 | `args_grammar.rs` の 7、`flow_run_id.rs` の 23(分離、優先順位、空の固定、`run_id_env`、違反の値を含めない、`watch`・`reset` の定義の読み方、`reset --all` の警告と 65、`validate`)、`binary_run_id.rs` の 5(並行する run ID と再開、環境変数と `run_id_env`、無効な値、`reset --all`) |
| converter | 0 | 変更なし(114 件) |

既存のテストの変更: 署名・フィールドの追従(runtime の統合テスト 6 ファイルの薄い包み、cli の `common/mod.rs` の偽 2 つ、構造体リテラル)と、7 の 2 か所だけ。

## 検証の結果

- `cargo fmt --all -- --check`: 成功。
- `cargo clippy --workspace --all-targets --locked -- -D warnings`: 警告なし。
- `cargo test --workspace --locked`: **663 件成功、0 失敗、1 無視**(基準 597 件、1 無視)。
- `cargo +1.85 build --workspace --locked --all-targets`: 成功(let チェーンなし)。
- `cargo check --workspace --tests --target x86_64-pc-windows-msvc --locked`: 成功。
- `cargo llvm-cov --workspace --locked --ignore-filename-regex 'bin/conductor-(cli-)?test-helper' --summary-only`: 行カバレッジ **97.72%**(基準 97.79%、下限 80%)。`--fail-under-lines 80` つきの CI 形式は、数値の確認のみで、同じコマンドの再実行はしていない。
- README の `docs-check` の語句(SHA-256、Gatekeeper、終了コードの表、ステートファイルの場所、at-least-once、シグナル、convert): すべて残っている。
- 実際のバイナリ(release): 2 つの run ID の `pipeline` を並行して動かすと、別々の `.state` ができ、互いの位置を変えない。止まった run は、同じ run ID で再開し、完了で消える。`--run-id ""` は、環境変数があっても、run ID なしに固定。オプションは環境変数に優先。定義の `run_id_env` は、`run`・`watch`・`reset` が使う。壊れた定義でも、`reset` は既定の環境変数の名前で動く。`reset --all` は、run ID なし・`opt`・`r1`・`r2` を消し、別の定義の `x.toml.q.state`・壊れた `x.toml.c.state`(警告つき、終了コード 0)・一時ファイルを残す。`a/b`・`..`・65 文字は 68。`validate --run-id x` は 68。`validate` は不正な `run_id_env` を 64 で拒否。`DEMO_SLEEP=0.1 sh demo/run-demo.sh` は 0 で終了。

## 実行していないもの

- 3 OS の CI(GitHub Actions)。Windows・Linux の実機でのテスト。
- `cargo +1.85 test`(1.85 は、ビルドのみ確認)。
- タグ付けと公開(範囲外)。
