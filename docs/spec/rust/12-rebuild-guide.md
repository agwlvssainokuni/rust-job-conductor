# 12. 再構築の手引き(ゼロから実装する順序・落とし穴・差と未確認の一覧)

対象: `conductor` v0.10.0 の機能と v0.10.1 の直しを含む現在の実装(Rust 実装)。この文書は、03〜11 章の仕様に従って、ゼロから同じ挙動のツールを実装するための順序と、確認の観点をまとめる。仕様の本体は 03〜11 章にあり、この文書は指針と集約である。用語は [00-README.md](00-README.md) の用語集に従う。

## 1. 実装の順序

依存の少ない順(純粋な判断 → 入出力 → 流れ → 実行ファイル)に作る。各段の「完了条件」は、その段のテストが通ることである。段の順序は、クレートの依存方向([02-architecture.md](02-architecture.md) の 1 章)に沿っている。

### 段 0. ワークスペースの土台

| 項目 | 内容 |
|------|------|
| 作るもの | ルートの `Cargo.toml`(`members = ["crates/*"]`、`resolver = "3"`、edition 2024、`rust-version = "1.100"`、Apache-2.0、`[profile.release]` は `opt-level = 3`・`lto = "thin"`・`codegen-units = 1`・`strip = true`)。4 つのクレートの雛形(`conductor-core`、`conductor-runtime`、`conductor-converter`、`conductor-cli`。すべて 0.10.1、`publish = false`)。`Cargo.lock` をコミットする |
| 依存 | 11 の 2 章の表のとおり(`toml` 1.2.1 は `default-features = false`)。禁止の依存を入れない |
| 完了条件 | `cargo build --workspace --locked` が通る。`cargo fmt --all -- --check` と `cargo clippy --workspace --all-targets --locked -- -D warnings` が通る |
| 参照 | 11 の 1〜2 章 |

### 段 1. `conductor-core` の型・run ID・エラー

| 項目 | 内容 |
|------|------|
| 作るもの | `types.rs`(02 の 2.1 の型)、`run_id.rs`(`RunId::parse` は、空・長すぎ・不正な文字・先頭 `.` の 4 つの条件のどれかで誤り。`is_valid_env_name`、`peek_run_id_env`)、`error.rs`(`DefinitionErrorKind` 34 種〔v0.9.0 の 3 種と v0.10.0 の 1 種を含む〕と識別子・日本語の文、`DefinitionError`、`escape_display`(上限つきと上限なし)、`quote_key`、`Locate`) |
| 完了条件 | run ID の有効・無効の例(05 の 6.1)が通る。`Locate::line_of` が、行頭・行の途中・範囲外・CRLF で、03 の 10 章のとおりの行を返す。`escape_display`(上限つき)が制御文字・書式文字をエスケープし、64 文字(キー名)で切る。ファイル名は、ライブラリ側の `Display` だけ 256 文字で切り、CLI の表示は切らない |
| 確認の観点 | 識別子は 34 種すべてが変種と 1 対 1(変種の名前の先頭を小文字にしたもの)。表示は `<file>:<line>: <種類の文>: <reason>` |
| 参照 | 03 の 9・10・12 章、05 の 6 章 |

### 段 2. `conductor-core` の定義の読み込み

| 項目 | 内容 |
|------|------|
| 作るもの | `definition/` の 5 段階: Intake(1 MiB 上限、UTF-8、場所の決定)→ Parse(`toml` の `DeTable::parse`、固定の文法の誤り)→ Depth(32 段)→ VersionCheck → Walk(構造・型・遷移・終了コード・リトライ・ジョブ間の検査) |
| 完了条件 | 03 の 13 章の受理の 2 例と、拒否の表の全行が、同じ種類・同じ行番号になる。誤りは位置の昇順で、複数返る(03 の 7 章の `multiple_errors.toml`)。段階 1〜4 の誤りは 1 件で打ち切る |
| 確認の観点 | `versionMissing` の定義は、ほかの誤りがあっても 1 件だけ。`[job.retry]` の位置が `[[job.transitions]]` の前後どちらでも受理。`i64` を超える整数は読み取りでは拒否されない。入力の値が誤りの文に出ない。1 MiB ちょうど受理・超過で `tooLarge` |
| 参照 | 03 の 1〜8 章(`env`・`env_file` は 6.6。段 10 で足す) |

### 段 3. `conductor-core` の判断(`flow`)

| 項目 | 内容 |
|------|------|
| 作るもの | `classify`(05 の 2.3 の 9 行の優先順)、`classify_and_decide`(3.2 の判定表)、`decide_from_state`(4.2 の判定表。判定の本体 `decide_before_run` は `flow/decide.rs` の中だけ)、`resume_point_on_interrupt`、`retry_decision`(待ち時間の列) |
| 完了条件 | 05 の各表の例がすべて通る: 分類の全組み合わせ、起動前の判断の 15 例、リトライの待ち時間の 6 例(`10,20,40,60,60` など)。関数は純粋(ファイル・プロセスに触れず、失敗しない) |
| 確認の観点 | 分類なし(起動前の中断)は `None`・`128 + 番号`・再開位置はそのジョブ。成功コードに無い 0 は `conductor_error`。`is_retry_target` は分類を見ず、conductor が付けた起動失敗の 127 を対象にしない |
| 参照 | 05 の 2〜4・7・9 章 |

### 段 4. `conductor-runtime` の `command` の分割とジョブの起動

| 項目 | 内容 |
|------|------|
| 作るもの | `split.rs`(状態機械。区切りはスペース・タブ・LF・CR の 4 つ、引用符、二重引用符の中のバックスラッシュ)、`check_command`、`runner.rs`(`run_job(line, env, signal)`(v0.9.6 までは `run_job(job, line, env, signal)`): 分割済みの `line` を受け取る(分け直さない。空の `program` は起動失敗)→ 起動前の合図の確認 → spawn → 1・2・4・8・10・10… ms の `try_wait` → 合図の転送は 1 回 → 終了の観測の直後の合図の確認)、`InterruptSignal`、`platform/`(UNIX: `process_group(0)`、`libc::kill` で 4 つのシグナルだけ、`ExitStatus::signal`。Windows: 何もしない) |
| 完了条件 | 分割の受理・拒否の表(07 の 3.3・3.4)が通る。メタ文字が引数にそのまま渡る。起動失敗・存在しない実行ファイルは `StartFailed`。UNIX で SIGTERM の転送が 1 回で、ジョブは新しいプロセスグループにいる |
| 確認の観点 | シェルを呼ばない(`sh`・`cmd`・`-c` の文字列がソースにない)。`unsafe` は `libc::kill` の 1 か所。子孫には送らない |
| 参照 | 07 の 1〜8 章 |

### 段 5. `conductor-runtime` のステートと待機

| 項目 | 内容 |
|------|------|
| 作るもの | `wait_for`(50 ms 以下の間隔)、`state/reader.rs`(15 項目の検査順(パスの種別を先に見て、通常のファイルだけを開く)、1 MiB、未知のキーの拒否、`format_version` の先行検査、完了は空)、`state/writer.rs`(一時ファイル → 書き込み → Durable のとき同期 → 閉じる → rename → ディレクトリの同期 → 24 時間以上古い一時ファイルの掃除)、`flow/record.rs`(core に置く。`initial_state`・`apply_*`。`apply_skip` は状態を定義の位置で引き、1 回がジョブの数に比例する時間。runtime の `lib.rs` は `conductor_core` から同じ名前で再公開する)、照合(core の `consistency.rs` に `check_consistency`・`ConsistencyResult`・`MismatchKind` を置き、runtime は再公開する。単体テストも core の `consistency.rs`)、`state/reset_all.rs`、`StateLocation`。上限つきの読み取りは `bounded_read.rs` の `read_regular_file` の 1 か所(`.env` と共有)、状態の語は `JobStatus`・`SkipReason` の `as_str`・`parse` |
| 完了条件 | 書いたステートを読み戻すと等しい。06 の 2.4 の矛盾がすべて `corrupt`(65 になる)。書き込みの失敗でステートファイルが直前のまま。`reset --all` の実測の例(06 の 7.2)が再現する。`apply_*` の純粋な更新が 05 の 3.3〜3.5 のとおり |
| 確認の観点 | 理由文は日本語の固定文(22 種)。ステートに `command`・環境変数・出力が書かれない。キーは辞書順で、`[[job]]` が最後。`reset`(個別)はリンクそのものを消し、`reset --all` はリンクを消さない |
| 参照 | 06 章、05 の 3 章、10 の 7.2 |

### 段 6. `conductor-converter`

| 項目 | 内容 |
|------|------|
| 作るもの | Intake、TOML の読み取り(`toml`)、YAML の読み取り(`saphyr-parser` のイベント列を、再帰なしの明示のスタックで素の木へ)、スカラーの解決(YAML 1.2 コアスキーマ)、RootCheck、展開(アンカー・マージキー、上限: 要素数 100,000、文字 16 MiB、深さ 32)、表せない値の検出、TOML と YAML の書き出し(辞書順) |
| 完了条件 | 09 の 11 章の例が同じ出力になる。往復(TOML → YAML → TOML)で値が等しい。失敗の順序(09 の 2 章の 9 段)が守られる。1 MiB 超は、中身が壊れていても `tooLarge` |
| 確認の観点 | 副作用がない(ファイルは入力 1 つを読むだけ)。入力の値をエラーに出さない。ランダム・壊れた入力でパニックしない |
| 参照 | 09 章 |

### 段 7. `conductor-cli` の引数・表示・終了コード

| 項目 | 内容 |
|------|------|
| 作るもの | `types.rs`、`exit.rs`(定数と `exit_code_for`)、`args.rs`(`lexopt`、解釈の順序 11 段)、`run_id.rs`(`resolve_run_id`、`aux_run_id`)、`report/`(`Reporter`、メッセージカタログ英日、`watch` の表、エスケープ、遅延評価のロケール・色)、`deps.rs`(4 つの trait と本番の実装)、`panic_hook.rs` |
| 完了条件 | 04 の 9 章の実行例が同じ結果。解釈の優先順位(04 の 4.1)のとおりに、同時に複数の誤りがあるとき最初のものが返る。10 の 4 章の終了コードの整形表のとおり。08 の各文言が、英語・日本語で一致する |
| 確認の観点 | `-h`/`-V` はほかの誤りより先で、先に現れた方が勝つ。`--` 以降は位置引数。使い方の誤りは「1 行 + 空行 + 使い方全文」で 68。入力由来の文字列は、エスケープ・切り詰めして出す |
| 参照 | 04、08、10 章 |

### 段 8. `conductor-cli` の流れ

| 項目 | 内容 |
|------|------|
| 作るもの | `ctx.rs`(`prepare_session`: シグナル → 渡された検査の並び `check_job`・`check_pipeline`: 定義(64)→ `command`(64。分割したコマンドを `Checked.base` が持ち、`Session { checked, signal }` が包む)→ ジョブID(68。`check_job` は位置も返す)→ run ID(68)→ ステートの読み込み(65)→ 照合(65)→ 削除の実効値。run ID の決定は `definition_run_id`、ステートの読み込みと照合は `read_checked_state`)、`step.rs`(1 つのジョブの共通の手順と、リトライを含む実行 `run_job_with_retry`)、`run.rs`、`pipeline.rs`、`aux_commands.rs`(`watch`・`reset`・`validate`・`convert`)、`platform/`(UNIX: `signal-hook` の専用スレッド。Windows: `ctrlc`)、`main.rs`、`lib.rs`(`run_cli`・`run_cli_with`) |
| 完了条件 | 偽の `Runtime`・`Env`・`Sink`・`Signals` で、05 の状態遷移の例(`a → b → c`)、`@end` の例、リトライの例、中断の表(05 の 8 章)が再現する。実ファイルと実プロセスで、`run`・`pipeline`・`watch`・`reset`・`validate`・`convert` が、04・05・06 の挙動と終了コード(10 の 1 章の全件)を返す |
| 確認の観点 | 検査の順序(10 の 2 章の 13 段)。ステートの書き込みの失敗は、ジョブの終了コードより優先して 65 で、「ジョブは実行されました」を表示。完了の判定は、中断の合図の判定より先。リトライの知らせは `-v` によらず常に標準エラー |
| 参照 | 04、05、06、07 の 7 章、10 章 |

### 段 9. 品質ゲート・CI・リリース・デモ

| 項目 | 内容 |
|------|------|
| 作るもの | 非機能要件のテスト(11 の 3 章: 性能の計測、ランダム入力、ソース検索、セキュリティ)、テスト補助の実行ファイル 2 つ、`ci.yml`・`audit.yml`・`release.yml`、`startup-bench`、`demo/` |
| 完了条件 | 11 の 10 章の品質ゲート(fmt、clippy、build、test、行カバレッジ 80% 以上)が、ローカルと CI で通る。MSRV の記載 3 か所が一致する(`msrv_consistency.rs`)。3 OS(`ubuntu-24.04`、`macos-latest`、`windows-2022`)でテストが通る |
| 確認の観点 | リリースは `conductor` だけを配布(テスト補助を含めない)。`SHA256SUMS` を付ける。README に必須の 7 語が残る |
| 参照 | 11 章 |

### 段 10. v0.9.0 の機能 1: 環境変数の注入(`env`、`env_file`、`.env`)

段 0〜9 までで v0.8.0 の挙動が揃ったあとに、機能を 1 つずつ、層の順(core → runtime → cli)に足す(各機能の中で、実装してから、その層のテストを書く)。

| 項目 | 内容 |
|------|------|
| 作るもの(core) | `types.rs` に `EnvVar`、`PipelineDefinition`・`JobDefinition` の `env`・`env_file`(`Debug` は値を `"***"`)。`error.rs` に `ReservedEnvName`・`EnvNameCaseConflict`・`InvalidEnvValue`(33 種)。`definition/walk/mod.rs` の `PIPELINE_KEYS`・`JOB_KEYS` に `env`・`env_file` を足し、`definition/walk/env.rs` の `env_table`・`env_file` を、03 の 7 章の順序(`remove_on_complete` のあと、`retry` のあと)で呼ぶ。`envfile/{mod,parse,merge}.rs`(型、`parse_env_file`、`merge_env`、`env_file_refs`、`is_reserved_env_name`、`summarize_errors`、定数)。`lib.rs` の再公開 |
| 作るもの(runtime) | `envfile.rs`(`load_env_files`)、`runner.rs` の `run_job(line, env, signal)`(`Command::env` を並びの順に。v0.9.6 までは `run_job(job, line, env, signal)`) |
| 作るもの(cli) | `deps.rs` の `Runtime::load_env_files`・`run_job` の `line` と `env`、`ctx.rs` の `load_env_files`(前処理の run ID のあと)、`step.rs` の `run_job_with_retry`(試行ごとに `merge_env`、`attempt = retries_done + 1`)、`ctx.rs` の `check_definition_and_env`(`validate`。重複キーの警告)、`report`(`EnvFileErrors`・`EnvWarnings`、日本語・英語の 13 種の文と理由、「ほか N 件」) |
| 完了条件 | 03 の 6.6 の受理・拒否の例、6.7 の受理・拒否の例が同じ結果になる。`merge_env` が 05 の 14.3 の優先順位と自動の変数のとおり。`.env` の誤りで、ジョブが 1 つも起動されず、ステートに触れず 64。`run` は `[pipeline]` とそのジョブの分だけ、`pipeline` は全ジョブ分を読む。リトライで `CONDUCTOR_ATTEMPT` が 1、2、3 と増え、再開で 1 に戻る。実バイナリで、`env`・`.env` の値が子の環境に見え、標準出力・標準エラー・ステートのどこにも値が出ない |
| 確認の観点 | 値の展開をしない。予約名は大文字小文字を区別せず拒否(`env` は `reservedEnvName`、`.env` は `reservedName`)。同じ表の中の大文字小文字だけが違う名前は拒否。`.env` の相対パスは、**定義ファイルのあるディレクトリ**が基準。同じ解決済みのパスは 1 回だけ読む。誤りは全ファイル分を集め、20 件で打ち切る。使い方の誤り(68)が `.env` の誤り(64)より先。新しい外部の依存を足さない |
| 参照 | 03 の 6.6・6.7、05 の 14 章、07 の 4 章、08 の 4.12 |

### 段 11. v0.9.0 の機能 2: `graph`

| 項目 | 内容 |
|------|------|
| 作るもの(core) | `flow/next_step.rs` から `resolve_transition` を切り出し(`classify_and_decide` の出力は変えない)、`graph/{mod,edges,layout,overlay}.rs`(型、`flow_edges`、`skip_targets`、`layout_graph`・`layout_lanes`(唯一の配置。主線の列 0 の記号 `main_line` を含む)、`build_graph`・`overlay_from_state`・`pending_overlay`・`node_state_of`) |
| 作るもの(cli) | `types.rs` の `CommandKind::Graph`・`GraphFormat`・`GraphOptions`、`args.rs`(`--format`・`--state`、`graph` の `--run-id`)、`aux_commands.rs` の `run_graph`、`report/graph_text.rs`・`graph_mermaid.rs`、メッセージ(辺の一覧の語、`-v` の語、再開位置の注記) |
| 完了条件 | 08 の 11 章の例 G1〜G4、G6、E1、E2、N1〜N5、`--state`、`-v`、Mermaid が 1 文字も違わず一致する。配置が、正解データ(`crates/core/tests/fixtures/graph_layout_oracle*.md` の 7 つ。git の出力の 4 つと、規則から導いた 3 つ)の全例と一致する(配置は 1 つの仕組みで、交差する飛び越し、同じ行き先、「両方の飛び越し」、分岐と、その組み合わせを、交差の行、寄せの行、横線 `_` を使って、レーンの図にする。主線の列 0 は、05 の 15.5 の規則)。1,000 ジョブの最悪の 6 つの形が 1 秒以内。`graph` が、ジョブを起動せず、ステートを書かず、標準出力に 1 回で書く。`validate` が 64 になる定義は `graph` も 64 |
| 確認の観点 | 辺の取り出しは `resolve_transition` を呼ぶ(判断を二重に書かない)。完了したステートは状態の記号を重ねるが、再開位置は示さない。エラーに辺はない。「対応する形」の 3 条件。`-v` の詳細は、分岐の行のあと(合流の行の前)に置く。`#` を最初に置き換える Mermaid のエスケープ |
| 参照 | 04 の 7.7、05 の 15 章、08 の 11 章 |

### 段 12. v0.9.0 の機能 3: `--dry-run` と `--assume`

| 項目 | 内容 |
|------|------|
| 作るもの(core) | `plan/{mod,simulate}.rs`(型、`plan_pipeline`、`plan_run`、`simulate_job`。`plan_pipeline` は `PipelineState` を 1 つ持ち、本番と同じ `apply_skip`・`apply_outcome` で更新する。更新の写しを作らない)。`decide_before_run` の探索を、位置の一致を先に見る `status_at` にする(挙動は同じ) |
| 作るもの(cli) | `args.rs`(`--dry-run`、`--assume` の検査)、`ctx.rs` の `prepare_checked`(本番の前処理の、シグナルを除く部分。対象は `FlowTarget` で、`check_job`・`check_pipeline` に `--assume` のジョブIDを渡し、`require_known` で検査する)、`dry_run.rs`(`run_dry_run`、入口 `enter_flow`。計画には定義の検査で分割したコマンドを渡す)、`report/plan_text.rs`、メッセージ |
| 完了条件 | 08 の 11 章の例 P1〜P8、P2e、P4e が 1 文字も違わず一致する。ジョブを起動せず、ステートを書かず・消さず、シグナルを準備しない。計画の「実行する」ジョブが、本番の流れが起動するジョブと一致する(新規と再開の両方)。1,000 ジョブの計画が 1 秒以内 |
| 確認の観点 | 完了したステートは「ステートなし」。使われなかった仮定は誤りにしない。定義にないジョブIDの仮定は 68(ジョブIDを表示しない)。`.env` は本番と同じ範囲で読み、64。環境変数は名前だけを計画に持つ(値を持たない) |
| 参照 | 04 の 7.9、05 の 16 章、08 の 11 章 |

### 段 13. v0.9.0 の機能 4: `init`

| 項目 | 内容 |
|------|------|
| 作るもの(runtime) | `file_create.rs`(`create_file`、`CreateOutcome`、`CreateError`) |
| 作るもの(cli) | `exit.rs` の `EXIT_EXISTS = 69`、`types.rs` の `CommandKind::Init`・`InitOptions`・`InitTarget`、`args.rs`(`--force`、`init` の名前の検査)、`aux_commands.rs` の `run_init`、`report/init_template.rs`(`render_template`)、メッセージ(`DisplayEvent::Init*`) |
| 完了条件 | 08 の 12 章のひな形が 1 文字も違わず一致する(日本語・英語)。ひな形が `validate` に通り、`--dry-run` の計画が出る。2 回目の `init` が、何も変えず 69。`--force` で置き換わり、`--config -` は標準出力だけ。特殊な文字の名前も TOML として正しくエスケープされ、`validate` に通る。作れないときは、種類ごとの固定の文で 65 |
| 確認の観点 | 親のディレクトリを作らない。新規は排他的に作る(既存のファイル・宙に浮いたシンボリックリンクを壊さない)。`--force` は一時ファイルから置き換える。OS のエラーの文を表示しない。ひな形に、名前以外の利用者の値を入れない |
| 参照 | 04 の 7.8、05 の 17 章、06 の 13 章、08 の 4.13・12 章 |

デモ(`demo/`)に場面 11〜14 を足し、README に v0.9.0 の節(環境変数、`graph`、`--dry-run`、`init`)を、版(`v0.9.0 以降`)つきで書く(cli の単体テスト `src/tests/docs.rs` が、必須の語句と例を検査する。11 の 9 章)。

### 段 14. v0.10.0 の機能 1: 既定のリトライ(`[pipeline.retry]`)と `retry = false`

段 13 までで v0.9.x の挙動が揃ったあとに、v0.10.0 の機能を 1 つずつ、層の順(core → runtime → cli)に足す。定義の書き方(段 14〜16)、図(段 17)、出力とステート(段 18〜20)の順。各機能の最後に README とデモを直す。

| 項目 | 内容 |
|------|------|
| 作るもの(core) | `definition/walk/mod.rs` の `PIPELINE_KEYS` に `retry`。`definition/walk/retry.rs` に、表 1 つの読み取りの共有(`retry_table`)、`[pipeline.retry]` の読み取り(`pipeline_retry`、`PipelineRetryDefault`。上限と間隔の大小はここで 1 回)、ジョブの `retry` の 3 つの書き方(`job_retry`、`JobRetrySetting { Absent, Disabled, Table }`)、既定との重なり(`check_retry`。ジョブの表の先頭の行)。定義の組み立てで、書かないジョブに既定を複製して入れる(`build_jobs`)。理由の文 `` `retry` はテーブルか `false` で書きます ``、`` `[pipeline.retry]` はテーブルで書きます `` |
| 完了条件 | 03 の 6.8 の表と誤りの表が通る。既定を当てはめたジョブは、同じ値の表を書いたジョブと同じ定義になる。`retry = false` は既定が無くても誤りでない。`[pipeline]` の `name` に誤りがあっても、既定の誤りと既定との重なりを報告する |
| 確認の観点 | 既定と自分の表を混ぜない。既定の `exit_codes` に誤りがあれば重なりを調べない。`JobDefinition.retry` は既定から来たかを持たない(05 の 7 章の判断と実行ループは変えない) |
| 参照 | 03 の 6.5・6.8・7 章、05 の 7 章 |

### 段 15. v0.10.0 の機能 2: `env_file` の表(無くてもよい `.env`)

| 項目 | 内容 |
|------|------|
| 作るもの(core) | `types.rs` の `EnvFileRef { path, required }`、`PipelineDefinition`・`JobDefinition` の `env_file: Vec<EnvFileRef>`(v0.9.x の `Vec<String>` から)。`definition/walk/env.rs` の `env_file` に表 `{ path, required }`(`env_file_table`、`ENV_FILE_TABLE_KEYS`)と新しい理由の文。`envfile/mod.rs` の `env_file_refs` の戻り値を `Vec<EnvFileRef>` にし、同じパスの文字列の `required` を論理和にする |
| 作るもの(runtime・cli) | `envfile.rs` の `load_env_files(definition_path, refs: &[EnvFileRef])`(解決済みのパスごとに `required` を論理和、必ず要らないパスが「存在しない」ときだけ読み飛ばして引き当てに載せない)。cli の `Runtime::load_env_files` の引数 |
| 完了条件 | 03 の 6.6 の表の誤りと、6.7 の読み飛ばしが通る。読み飛ばしたファイルは、何も足さない(`merge_env` は引けない記述を無視する) |
| 確認の観点 | あって読めない・大きすぎる・書式の誤りは、`required` によらず誤り。無いことの判断は「存在しない」だけ |
| 参照 | 03 の 6.6・6.7、05 の 14.5 |

### 段 16. v0.10.0 の機能 3: `unset_env`

| 項目 | 内容 |
|------|------|
| 作るもの(core) | `error.rs` に `UnsetEnvConflict`(34 種)、`envfile/mod.rs` の `EnvFileErrorKind` に `UnsetConflict`(14 種)と日本語の文。`types.rs` の `unset_env: Vec<String>`(パイプラインとジョブ)。`PIPELINE_KEYS`・`JOB_KEYS` に `unset_env`。`definition/walk/env.rs` の `unset_env`(要素の検査、完全に同じ名前は最初の 1 つ)と `check_unset_env`(全ジョブのあと。ASCII の大文字小文字を区別せず比べる)。`EnvEntry.line`・`EnvFileContent.file`。`check_unset_conflicts`。`EffectiveEnv.removals` と `merge_unset_names`。計画の `PlanEntry.unset_names` |
| 作るもの(runtime・cli) | `runner.rs` の `run_job` に `unset`(`env_remove` で消してから足す)。cli の `Runtime::run_job` の引数、`ctx.rs` の `load_env_files` の後の `check_unset_conflicts`、`step.rs` で `env.removals` を渡す。メッセージ(`unsetEnvConflict`・`unsetConflict` の英語、計画の `unset env:`) |
| 完了条件 | 03 の 6.10 の表、`.env` の `unsetConflict`(03 の 6.7)、05 の 14 章の層(引き継いだ環境から消してから足す)が通る。実バイナリで、消した名前が子の環境に無い |
| 確認の観点 | 比べ方は、定義と `.env` の検査ではどの OS でも大文字小文字を区別しないが、`env_remove` の比べ方は OS に任せる(07 の 8.1)。ジョブの `unset_env` は、ほかのジョブの `env` と比べない。誤りに値も名前も入れない |
| 参照 | 03 の 6.7・6.10、05 の 14 章、07 の 4.1・4.2、08 の 4.3・4.12・11.6 |

### 段 17. v0.10.0 の機能 4: `graph --format dot`

| 項目 | 内容 |
|------|------|
| 作るもの(cli) | `types.rs` の `GraphFormat::Dot`、`args.rs` の `--format` の値 `dot`、`report/graph_dot.rs`(`render_dot`、`escape_dot_label`)、`aux_commands.rs` の `run_graph` の切り替え、`--help` の行 |
| 完了条件 | 08 の 11.8 の G2 の全文が 1 文字も違わず一致する。`--state` の記号と再開位置の `penwidth=3`。節と辺の数が Mermaid と同じ。1 万ジョブの描画が 1 秒以内 |
| 確認の観点 | 節の ID は連番で、ジョブ ID はラベルにだけ書く。`escape_full` のあとに `\` と `"` をエスケープする(DOT の `\n`・`\l` を働かせない)。`-v` は影響しない |
| 参照 | 04 の 7.7、05 の 15.8、08 の 11.8 |

### 段 18. v0.10.0 の機能 5: 時刻とログのファイル(`log_dir`)

| 項目 | 内容 |
|------|------|
| 作るもの(core) | `time.rs`(`Timestamp`、`format_rfc3339`、`format_compact`、`parse_rfc3339`、`duration_secs`。年月日の変換を自前で書く)。`PIPELINE_KEYS` に `log_dir`、`PipelineDefinition.log_dir`(文字列でなければ `typeMismatch`、空なら `emptyField`) |
| 作るもの(runtime) | `clock.rs`(`now`)、`log.rs`(`LogTarget`、`LogOutcome`、`safe_log_stem`、`open_log_file`)、`platform` の `restrict_to_owner`(UNIX 系は 0600)、`runner.rs` の `run_job` の `log` と戻り値の `LogOutcome`(パイプ、読み取りのスレッド、2 秒の待ち、「止める」印) |
| 作るもの(cli) | `Runtime::now`、`step.rs` の `resolve_log_dir` と試行ごとの `LogTarget`、`report_execution` のログを作れなかった 1 行、`report_log_warnings`、メッセージの 3 つの文(08 の 4.14) |
| 完了条件 | 07 の 4.5 の名前・番号・作れないときのパス・安全な ID の表が通る。端末に出る内容が `log_dir` のあり・なしで同じ。16 MiB の出力でも端末とファイルが一致する。作れなければ起動せず 127 と 1 行。`validate`・`graph`・`--dry-run` はディレクトリを作らない |
| 確認の観点 | ファイルは「無いときだけ作る」で、上書きも追記もしない。`conductor` はログを消さない。待ち切れたあとに書き足さない。新しい外部の依存を足さない(時刻の書式は自前) |
| 参照 | 03 の 6.9、05 の 18.1・19 章、07 の 4.5、08 の 4.14 |

### 段 19. v0.10.0 の機能 6: 試行の時刻(`[[job.attempt]]`)と `watch` の所要時間

| 項目 | 内容 |
|------|------|
| 作るもの(core) | `types.rs` の `Attempt`、`JobState.attempts`。`flow/attempt.rs`(`start_attempt`、`end_attempt`、`cancel_attempt`)。`time.rs` の `job_duration`、`JobDuration`、`format_duration` |
| 作るもの(runtime) | `state/writer.rs` の `[[job.attempt]]` の書き出し(試行があるときだけ)、`state/reader.rs` の `JOB_KEYS` 5 個・`ATTEMPT_KEYS` 2 個と `walk_attempts`、理由の文 `NO_ATTEMPT`・`BAD_TIME`・`UNFINISHED_ATTEMPT` |
| 作るもの(cli) | `step.rs` の `run_job_with_retry`(試行ごとに時刻を取り、`start_attempt` して `Fast` で書いてから起動、`cancel_attempt`・`end_attempt`、リトライの待ちの `pending_end`)。`run_step` は空のステートを書かない(最初の試行の書き込みが初期化を兼ねる)。`report/watch.rs` の所要時間の列(`now` を 1 回だけ取る) |
| 完了条件 | 06 の 2.3a・2.4・3 章のとおりに書き、読む。v0.9.9 までのステート(動いたジョブに試行が無い)は 65 で `reset` を案内する。08 の 6 章の 4 列の表。05 の 18.3 の所要時間の表 |
| 確認の観点 | ステートに書くのは開始と終了の時刻だけ(`nfr_content.rs` を、時刻だけを許すように直す)。起動しなかった試行は残さない。強制終了の後の開いた試行は、自身の `started_at` で閉じる。`--dry-run` は試行を記録せず、時刻も取らない |
| 参照 | 05 の 18 章、06 の 2.3a・2.4・3 章、08 の 6 章、10 の 7.2 |

### 段 20. v0.10.0 の機能 7: `watch --follow`

| 項目 | 内容 |
|------|------|
| 作るもの(core) | `flow/activity.rs`(`pipeline_activity`、`PipelineActivity`) |
| 作るもの(cli) | `types.rs` の `follow`・`interval` と間隔の定数、`args.rs` の `--follow`・`--interval`(検査の順は `--follow` が先)、`aux_commands.rs` の `run_watch` のシグナルの準備と `follow_watch`・`FollowEnd`、`platform` の `clear_screen_supported`、`report` の `FramePrefix`・`WatchFrame`、`--help` の行 |
| 完了条件 | 05 の 20 章の判定と読み直しの表、04 の 7.3 の終了コード、08 の 6.4 の描き方が通る。リトライの待ちのあいだは `Running` で続ける。1,000 回の繰り返しで、ステートに書き込まない |
| 確認の観点 | 画面を消せるかは初めに 1 回だけ決める。パイプ・ファイルに制御の文字を出さない。読めないのは 1 秒おきに 3 回で 65、消えたのは 2 回続けて無いとき |
| 参照 | 04 の 7.3、05 の 20 章、07 の 7.1・7.6、08 の 6.4、10 の 1・2 章 |

デモに場面 4 の `watch --follow` と場面 15(ログのファイル。`demo/log.toml`)を足し、README に v0.10.0 の節を書き、版を 0.10.0 にする(11 の 9 章)。

### 段 21. v0.10.1 の直し

| 項目 | 内容 |
|------|------|
| 作るもの | `log.rs` の `safe_log_stem` で、予約名と比べる前に、最初の `.` より前の部分から前後の ASCII の空白を除く(名前そのものからは除かない)。`runner.rs` の `copy_stream` で、読む前にも「止める」印を見る。`flow` の計画の模擬の内部の入口(`decide_at`、`classify_and_decide_at`、目印つきの `apply_skip_in`・`apply_outcome_in`、`record_at`、`position_of`、`SimulationMarks`)と、それを使う `plan_pipeline`。デモの `DEMO_PAUSE=1` の場面 4 の `watch --follow` の前の待ち(UNIX 版) |
| 完了条件 | 07 の 4.5 の安全な ID の例(`con ` → `_con_`、`nul .tar.gz` → `_nul .tar.gz`)。計画の出力は v0.10.0 と 1 文字も変わらず、1,000 ジョブの P1・P2 が 150 ms 以内 |
| 確認の観点 | 内部の入口は `pub(crate)` で、公開の口を変えない。判断と記録の規則を模擬に書き写さない(05 の 16.2) |
| 参照 | 05 の 16.2、07 の 4.5、11 の 3.1・9 章 |

## 2. 実装時の落とし穴

03〜11 章の「実装と README の差」「未確認」から、実装を間違えやすい点を集約する。

### 2.1 定義の読み込み(03)

- 段階 1〜4 の誤りは 1 件で打ち切る。Walk だけが複数返す。`format_version` が無いと、未知のキーや型の誤りが隠れる(`versionMissing` の 1 件だけ)。
- 誤りは**位置の昇順**に並べて返す(安定な並べ替え)。検査の順に返さない。
- 必須のキーの欠落の行は、「それを含むテーブルの先頭」。`pipeline`・`job` 自体が無いときは 1 行目(位置 0)。
- 終了コードの重なり(`overlappingExitCodes`、`zeroAsWarning`、`retryExitCodesOverlap`)の行は、**キー**の行(値ではない)。0 が成功と警告の両方に書かれたら `zeroAsWarning` の 1 件だけ。
- `success_exit_codes` を省略すると `[0]`。書いたら**書いた値だけ**が成功(0 を書かなければ 0 は成功でない)。空配列も受理する。
- `i64` に収まらない整数は、読み取りでは `syntax` にならない。`format_version` なら `versionUnsupported`、終了コードの要素なら `typeMismatch`。
- 理由の文に入力の値を入れない。キー名だけを `` `キー名` `` の形で、エスケープして 64 文字で切って入れる。
- `[job.retry]` は `[[job.transitions]]` の前後どちらでもよい(README の書き方は推奨であり、制約ではない)。
- 数値の範囲: `retry.exit_codes` は 1〜255(127 も可)、`max` は 1〜100、`multiplier` は 2〜10、間隔は 1 秒〜24 時間。`multiplier` と `max_interval` は両方書くか両方書かない。
- 文法の誤りの表示は固定で、読み取り器のエラー文を使わない。
- (v0.9.0 以降)`env` の検査は、キーの位置の順に、名前の形 → 予約名 → 大文字小文字の衝突 → 値の型 → NUL。予約名は ASCII の大文字小文字を区別しない(`conductor_x` も拒否)。同じ綴りの重複は TOML の文法の誤り(`syntax`)で、`envNameCaseConflict` は綴りが違うものだけ。`env_file = []` は誤りでなく、空の文字列は `emptyField`。
- (v0.9.0 以降)`.env` の誤りは `DefinitionError` ではなく `EnvFileError`(14 種。v0.10.0 で `unsetConflict` の 1 種を足した)で、理由は言語に従い、値を含めない。`env` の誤りの `quote_key` と違い、`.env` の表示は名前も出さない。
- (v0.10.0 以降)`[pipeline]` の `retry` は表だけ(`false` も `retryNotTable`)で、ジョブの `retry` は表か `false`。既定との重なりの誤りの行は、`exit_codes` のキーではなく、そのジョブの表の先頭。`log_dir` は `[pipeline]` だけのキー。`unset_env` と `env` の重なりは、全ジョブの検査のあとに、名前の要素の行で報告する。`env_file` の表の `required` を省くと `true`。

### 2.2 CLI(04)

- オプションはコマンド名の前にも置ける。値を取るオプションは、次の引数を `-` 始まりでも値にする。
- 重複指定の誤りは、すべて `invalid value`(対象はオプション名)。README は「使い方の誤り」とだけ書く。
- `watch` と `reset` は定義を検証しない。`--run-id` があれば定義を読まず、なければ `run_id_env` だけをゆるく読む(壊れた定義でも動く)。
- `reset --all` は環境変数も `--run-id` も見ない(値の誤りも 68 にならない)。
- `NO_COLOR` は、**空でない値が設定されていれば**色なし(空は未設定と同じ)。
- (v0.9.0 以降)サブコマンドは 8 つ。`init` は `--config` と `--force` 以外のオプションを取らない(68)。`--config -` は標準出力。`graph` は `-v`・`--format`・`--state` を取り、`--run-id` は `--state` と一緒のときだけ。`--assume` は `--dry-run` と一緒のときだけ。`--assume` の誤りの表示に値(ジョブIDを含む)を入れない。解釈の順序は 04 の 4.1(使えないオプション → 重複 → 値の検査 → 位置引数)。
- (v0.9.0 以降)終了コード 69 は `init` の「既にある」だけ。`init` の作れない誤りは 65、使い方の誤りは 68。

### 2.3 実行の意味論(05)

- 分類なし(起動前・待機中の中断)は、ジョブの状態を変えない。`error` のジョブの再実行が起動前に中断されたときは、`status` は `error` のまま `exit_code` が消える。
- `pipeline` は、次のジョブを `NextStep.next_job_id` で決めず、記述順に全ジョブを見る。
- 完了の判定は、ジョブ実行後の中断の合図の判定より**先**。
- 全ジョブを通過しても `completed` が真にならないと、65(`cannot determine whether the pipeline is complete`)。
- 起動前の中断の終了コードは `is_error = false` で整形される(`128 + n` の下位 8 ビットが 0 なら 0 になりうる。実在のシグナル番号では起きない)。
- リトライの知らせは「待機の前」に、常に標準エラーへ。`1 seconds`(単数形の処理なし)。リトライの回数はステートに書かず、再開では最初から数え直す。
- 遷移でスキップしたあとのステートには、成功直後でも `resume_job_id` が入りうる(`apply_skip` が最初の `pending` を入れる)。
- (v0.9.0 以降)環境変数は、引き継いだ環境を残して、定義の層と自動の変数を**足す**。(v0.10.0 以降)`unset_env` を書いたときだけ、足す前に、その名前を引き継いだ環境から**消す**(`env_remove`。書かなければ何も消さない。消す名前と足す名前は検証で重ならない)。完全に同じ名前は後ろの層が勝ち、大文字小文字だけが違う名前は別の名前(Windows だけ、後ろが前を置き換える。許容)。`CONDUCTOR_RUN_ID` は、run ID があるときだけ足す(無いとき、引き継いだ値は消さない)。`CONDUCTOR_ATTEMPT` は試行ごとに作り直し、再開で 1 に戻る。`.env` は、ステートを読む**前**に(run ID の決定のあと)読み、再開でも毎回読み直す。
- (v0.9.0 以降)`graph` の辺は、実行と同じ `resolve_transition` から取り出す。`--dry-run` は完了したステートを「ステートなし」として扱い(本番と同じ)、`graph --state` は完了したステートも読む(意図した違い)。`--dry-run` はシグナルを準備しない。`--dry-run` でも、`.env` は本番と同じ範囲を読み、誤りは 64。
- (v0.10.0 以降)ステートは、試行の開始ごとに `Fast` で書く(リトライの各試行も)。リトライの待ちのあいだは、前の試行を閉じずに書かない(`pending_end`)。起動しなかった試行は取り除く。`apply_outcome` は試行を変えない。ログのファイルを作れないときは、起動の失敗(127)と同じで、リトライしない。
- (v0.10.0 以降)`watch --follow` は、止まった(エラー、または再開位置のジョブが試行を持つ)と判定すると、その表で終わる。リトライの待ちのあいだは試行が開いているので続ける。

### 2.4 ステート(06)

- `definition_key` は書くが、読んでも存在と型を確かめるだけで、型に持たず、照合に使わない(`definition_path` と同じ値)。`job_order` も型に持たない(書き出しは `job_states` のジョブIDの並び。読み込みは並びの一致と再開位置の検査だけに使う)。
- 完了のステートは `read_state` で「空」。次の `pipeline` は先頭から始まり、初期ステートで**上書き**する。先頭以外の `run` は順序外(66)。
- 大文字小文字を区別しないファイルシステムでは、run ID `A` と `a` が同じファイルになる(中身の `run_id` との照合で 65)。
- `reset --all` は、長さ 0 のファイルを消さず、警告にも出さない。別の `run_id` を持つファイルは黙って残す。
- 理由文(`TOML として読めません` など)は、ロケールによらず日本語。種類の文と案内だけがロケールに従う。
- 排他制御はしない。書き込みは原子的だが、同時実行では後の書き込みが勝つ。
- (v0.10.0 以降)`[[job.attempt]]` のキーはアルファベット順(`ended_at`、`started_at`)に書き出す。時刻の読み取りは `YYYY-MM-DDTHH:MM:SSZ` の形だけ。動いたジョブに試行が無いステート(v0.9.9 までに書いたもの)は 65。

### 2.5 プロセスとシグナル(07)

- 区切りの文字はスペース・タブ・LF・CR の 4 つだけ(U+00A0 などは普通の文字)。二重引用符の中の `\\` は 1 つに縮まらない。
- シグナルの転送先は、ジョブの子プロセスの pid だけ(許可は 4 つ、1 回だけ)。孫には送らない。
- 終了の観測の直後に合図が立っていたら、ジョブの終了コードが 0 でも、エラー(`128 + 番号`)。
- 登録はプロセスで 1 回だけ呼ぶ(`prepare_session` の 1 か所)。2 回呼んだときの動きは契約の外。
- ほぼ同時に複数のシグナルが届くと、番号の小さい方が「最初」になりうる(`signal-hook` の返す順)。
- `try_wait` の失敗も起動失敗(127)として扱う。
- (v0.10.0 以降)`log_dir` があるときだけ、標準出力と標準エラーをパイプにする(標準入力は引き継ぐ)。子の終了の後の読み取りの待ちは、合わせて 2 秒。子の子がパイプを開いたままでも、そこで打ち切る。
- (v0.10.0 以降)`watch --follow` は、`run`・`pipeline` と同じくシグナルを準備する(ほかの `watch` は準備しない)。

### 2.6 出力(08)

- 日本語の補足の区切りは ASCII の `, `。`Run ID` 見出しは日本語でも `Run ID`。`reset --all` の警告は `conductor: warning:`(終了コード 0)。
- 定義・ステート・変換の誤りの原因は、英語ロケールでも日本語の固定文。
- `--version`、パニックの文、シグナルの準備の失敗の文は、ロケールを判定しない(英語のみ)。
- ロケールは `LC_ALL`、`LC_MESSAGES`、`LANG` の最初の空でない値だけで決める(大文字小文字を区別する)。
- (v0.9.0 以降)`.env` の誤りの種類の文と理由は、言語に従う(定義の誤りの理由は日本語のまま)。図・計画・ひな形は、組み立て済みの文字列を標準出力に 1 回で書き、各行の末尾の空白を取り除く。計画の「仮定の補足」は、日本語は語のすぐあとに括弧(`実行する(仮定: …)`)、英語は空白を置く(`run (assumed: …)`)。レーンの図のラベルの位置は、分岐のあとの列の数 × 2。
- (v0.10.0 以降)`watch` の表は 4 列(所要時間の列は、見出しの幅と値の幅の大きい方)。`watch --follow` は、UNIX 系の端末だけ `ESC[H ESC[2J` で画面を消し、ほかは 2 回目から空の 1 行を置いて追記する。ログの 2 つの警告は `conductor: warning: `、作れないのは `conductor: error: `。

### 2.7 変換(09)

- 出力のキーは**辞書順**(README の「記述順は保持されない」の実態は、整列)。
- 入力の形式は `--to` の反対。拡張子は見ない。
- `null`・日時・タグつきの値・文字列でないキー・複数の文書・範囲外の数値・マージキーの値がマップでない、はすべて失敗(67)。

### 2.8 品質・ビルド(11)

- MSRV の記載は 3 か所(`Cargo.toml`、README の `Rust(1.100 以降)`、`ci.yml` の `dtolnay/rust-toolchain@1.100.0`)を同時に更新する。
- カバレッジは Ubuntu だけで測る(`cfg(windows)` は計測対象外)。テスト補助の実行ファイルは除外する。
- 性能の計測は、既定では警告だけ(`STARTUP_BENCH_ENFORCE=true` で強制)。測定の行は「debug ビルド」と書くが、CI は `--release`。

## 3. 実装と README の差の一覧

各文書の末尾の表を集約する。README は利用者向けの文書で、実装が正である。ここに無い項目は、確認した範囲で README と実装が一致する。

| 文書 | 項目 | README | 実装 |
|------|------|--------|------|
| 03 | `[job.retry]` の位置 | `[[job.transitions]]` の前に書く | 後に書いても受理(TOML の意味どおり。推奨であり制約ではない) |
| 03 | 上限・形式 | 定義ファイルの大きさ(1 MiB)、入れ子の深さ(32 段)、整数のリテラルの形式を述べない | 1 MiB、32 段を課す。16 進・8 進・2 進・`_` 区切りも値で判定して受理 |
| 03 | 終了コードの値の範囲 | 負の数や大きな数に触れない | `i32` の範囲(負を含む)を受理し、範囲外は `typeMismatch` |
| 03 | 重なりの検査 | 「重ならず、0 を含まない」 | 0 が両方にあるとき `zeroAsWarning` だけを報告 |
| 04 | 引数の置き場所 | 記述なし | オプションはコマンド名の前にも置ける。値を取るオプションは `-` 始まりでも値にする |
| 04 | `--version` | 形式の記述なし | `conductor <版>`(標準出力。ロケールを判定しない) |
| 04 | `-h`/`-V` の優先 | 記述なし | ほかの誤りより先。先に現れた方が勝つ。`--` の後ろでは効かない |
| 04 | 短縮形の連結 | 記述なし | `-vh`、`-vv`(後者は 68)が `lexopt` の規則で解釈される |
| 04 | 重複指定の誤り種別 | 「使い方の誤り」(68) | すべて `invalid value`(対象はオプション名) |
| 04 | `--config=` の空値 | 記述なし | 空パスとして受理され、後段で `definition file does not exist` 等(64) |
| 05 | リトライの知らせの単数形 | — | 待ちが 1 秒のとき `in 1 seconds`(単数形の処理なし) |
| 05 | エラーのジョブの再実行が起動前・待機中に中断 | — | `status` は `error` のまま、`exit_code` が消える |
| 05 | 成功直後の `resume_job_id` | — | 遷移で飛ばしたあとのステートには、成功直後でも `resume_job_id` が入りうる |
| 05 | 分類なしの終了コードの整形 | — | `is_error = false` で整形(下位 8 ビットが 0 の番号では 0 になりうる。実在のシグナルでは起きない) |
| 05 | `pipeline` の完了を判定できない | 表のみ(65) | 全ジョブを通過しても `completed` が偽なら 65。ステートは初期化しない |
| 05 | ステートの誤りの原因の文 | — | 種類の文はロケールに従い、`reason` は日本語の固定文 |
| 06 | `reset --all` と長さ 0 のファイル | 「読めない・壊れている」ものを警告 | 長さ 0 は消さず、警告にも出さない |
| 06 | `reset`(個別)とシンボリックリンク | 「リンクは消さない」(`--all` の説明) | 個別の `reset` は `remove_file` で、リンク自体を削除 |
| 06 | 書き込みの手順の細部 | 「一時ファイルに書いて置き換える」まで | 一時ファイルの名前、新規作成、Fast/Durable、24 時間以上古い一時ファイルの掃除 |
| 06 | 読み込みの検査の順序と上限 | 「読めない・壊れている・版の不一致」だけ | 1 MiB の上限、未知のキーの拒否、`format_version` の先行検査 |
| 06 | 理由文の言語 | — | 日本語の固定文 |
| 06 | `definition_key` | — | 書式に残るが `definition_path` と同じ値で、照合に使わない |
| 07 | 区切りの文字 | 「空白」 | スペース、タブ、LF、CR の 4 つだけ |
| 07 | 二重引用符の中のバックスラッシュ | 中の `\"` だけが `"` になる | 左から読む状態機械。`\\` の連続は縮まらない。引用符の外・一重引用符の中は普通の文字 |
| 07 | 配列の先頭の要素 | 空・空白だけは定義の誤り | 起動の側も二重に検査。空白を含む先頭(`"my prog"`)はそのままファイル名。文字列の先頭の語が空白だけ(`'  '`)も同じく `Empty`(v0.9.7 までは 127) |
| 07 | シグナルの転送先 | ジョブ(の子プロセス) | 子プロセスの pid だけ。孫・プロセスグループ全体には送らない |
| 07 | 127 の範囲 | 「起動できなかった」 | `spawn` の失敗に加え、`try_wait` の失敗も 127 |
| 07 | 許可外の番号 | 記述なし | `run_job` は送信しない(通常は起こらない) |
| 07 | ジョブが 0 で終わったが合図が立っていた | エラーとして記録 | 終了の観測の直後に立っていれば、0 でもエラー(`128 + 番号`)。完了済みの最後のジョブの後に立った合図は無視 |
| 07 | ジョブの出力 | そのまま流れる | 標準入出力・環境変数・作業ディレクトリをすべて引き継ぐ |
| 08 | 日本語の補足の区切り | 記述なし | ASCII の `, ` |
| 08 | リトライの知らせ | 概要 | `conductor: retry: ` の行。常に標準エラー |
| 08 | `Run ID` 見出し | `Run ID: <ID>` | 日本語でも `Run ID` |
| 08 | `reset --all` の警告の接頭辞 | 「警告」 | `conductor: warning: `(終了コード 0) |
| 08 | 定義・ステート・変換の誤りの原因 | 「一部は日本語」 | 英語ロケールでも日本語の固定文 |
| 09 | キーの順序 | 「記述順は保持されない」 | 辞書順に整列 |
| 09 | 表せない値 | `null`・日時など | さらに、タグつきの値、範囲外の数値、文字列でないキー、複数の文書、マージキーの値がマップでないとき |
| 09 | 展開の上限 | 触れない | 要素数 100,000、文字 16 MiB、深さ 32 |
| 09 | TOML の入力 | 「`--to` の反対」 | TOML 1.1 の構文の一部も受理(読み取りクレートの仕様) |
| 10 | 終了コード 1 | 成功コードに含まれない 0 | 加えて、終了コードのないまま終わったジョブと、エラーの終了コードの下位 8 ビットが 0 の場合 |
| 10 | 127 | 起動できなかった | 加えて、起動していないのに中断でもない組み合わせ、終了の状態を取得できない場合 |
| 10 | 65 | 一覧あり | 加えて、`pipeline` が完了を判定できないとき |
| 11 | テスト・CI | `cargo test --workspace --locked` のみ | CI(3 OS + MSRV)、カバレッジ、clippy、`startup-bench` は README にない |
| 11 | リリース | 例の版が `0.7.0` | クレートの版は 0.10.1(例なので誤りではない) |
| 11 | 性能の測定の文言 | — | 「debug ビルド」の文言だが、CI は `--release` |
| 04 | `--dry-run` と `graph --state` の、完了したステートの扱い(v0.9.0 以降) | 計画は完了したステートをステートなしとして扱い、`graph --state` は完了したものも読む(意図した違い) | 一致(`read_state` と `read_state_raw`) |
| 04 | `init` が作ったファイル(v0.9.0 以降) | 新規の作成の短い間と強制終了のあとに、空のファイルが見えることがある | 一致(`create_new` で作ってから書く。`--force` で直せる) |
| 08 | `.env` の誤りの理由の言語(v0.9.0 以降) | — | 種類の文と理由が言語に従う(定義の誤りの理由は日本語のまま) |
| 07 | 環境変数(v0.9.0 以降) | README の「環境変数の設定」の節 | 引き継いだ環境に、`env`・`.env`・自動の変数を足す(v0.8.0 以前は何も足さなかった)。v0.10.0 以降は、足す前に `unset_env` の名前を消す |
| 06 | `[[job.attempt]]` のキーの順(v0.10.0 以降) | 例は `started_at`、`ended_at` の順 | 書き出しはアルファベット順(`ended_at`、`started_at`)。読み込みは順に依存しない |
| 10 | `watch --follow` の 65(v0.10.0 以降) | 65 の行は `watch` の標準出力に書けないときだけ | 加えて、読めないのが 3 回続いたとき、壊れている・大きすぎる・版の違い |

## 4. 未確認事項の一覧

各文書の「未確認」を集約する。実装者は、これらを仕様として断定せず、必要なら実機・実装で確かめる。

| 文書 | 未確認の事項 |
|------|--------------|
| 03 | 深さ 33 段〜100,000 段の入力で、拒否の理由文が `入れ子が深すぎます(上限は 32 段)` か `TOML の文法に誤りがあります` のどちらになるか(種類は常に `syntax`) |
| 03 | TOML の読み取りで受理される構文の全範囲(`toml` 1.2.1 の仕様に従う。TOML 1.1 の構文は変換器経由で確認) |
| 03 | 暗黙に作られるテーブル(ドットキー `retry.exit_codes = ...` など)でキーが欠けたときの誤りの行 |
| 03 | Windows など他の OS での、ファイルの権限・シンボリックリンクに関わる `unreadable` の判定の細部 |
| 04 | `reset --all` の対象の詳細は、runtime クレートの実装と 06 の 7.2 節に従う(04 では README の記述を根拠とした) |
| 05 | Windows での実行中ジョブへのシグナル転送の細部と、`exit_code` が負になる場合の実機の値 |
| 05 | ジョブの標準入出力の扱い(子プロセスの継承の設定)の確認(07 の 11 章に記述あり) |
| 05 | ほぼ同時に複数のシグナルが届いたときの再現(ライブラリの返す順序は番号の小さい順とするが、同時到着そのものの再現は未確認) |
| 06 | `toml` クレートが、特殊な文字(バックスラッシュ、引用符、制御文字)を含む文字列を、どの表記で書き出すか(読み込みは TOML として正しければよい) |
| 06 | Windows での `rename` による置き換えの細部、ディレクトリを `File::open` したときの `unreadable` の判定、ファイルの権限(Windows の ACL) |
| 06 | 一時ファイルの `<時刻>` の、クロックが戻った場合の挙動(名前の衝突の再試行 16 回に任せている) |
| 07 | Windows の実機の動作(Ctrl+C の終了コード 130、`.cmd` の起動)。チェックリストの記録欄が空 |
| 07 | `ctrlc` が Windows で Ctrl+Break を受け取るか |
| 07 | UNIX で、ジョブが新しいプロセスグループにあるため、端末から読む場合に背景グループとして停止するか |
| 07 | Windows の `Command::new` のプログラム探索(カレントディレクトリを含むか等) |
| 07 | この調査ではテスト・ビルドを実行していない(ソースとテストの読み取りに基づく) |
| 09 | YAML の受理する文法の全範囲(`saphyr-parser` 0.3.4 の仕様による) |
| 09 | 定義されていないアンカーの参照が、専用の理由(`定義されていないアンカーを参照しています`)になる経路の有無 |
| 09 | 深さが 33 を大きく超える TOML の入力で、理由が `入れ子が深すぎます(上限 32)` か `TOML として読めません`(v0.9.6 までは `文法の誤りです`)か |
| 09 | YAML の `---` だけの文書などの `rootNotMap` の行番号の厳密な規則(観測: `---\n` は 2 行目) |
| 09 | TOML の書き出しの整形規則の完全な記述(`toml` クレートの標準の整形に従う) |
| 09 | 非 ASCII のキー・値の YAML での扱い(実装は、非 ASCII を含む文字列を常に二重引用符で囲む。実行で確認) |
| 10 | `DefinitionError.reason` の全件の列挙(03 の各表が原文)。converter の `kind` と `reason` の完全な対応 |
| 11 | ブランチ保護(必須のジョブ)は、リポジトリの設定でファイルから確認できない |
| 11 | 現在の行カバレッジの実測値、各 NFR の計測値(実行していない) |
| 11 | `cargo audit` の現在の結果、CI のイメージの将来の変更の影響 |
| 11 | Windows のチェックリストの手動確認の実施の有無 |
| 11 | `startup-bench` を強制に切り替える予定の有無 |
| 05 | (v0.9.0 以降)統合した環境変数の全体の大きさの上限は設けておらず、OS の上限を超えたときの起動の失敗(127)の実機の挙動は、未確認。層をまたぐ、大文字小文字だけが違う環境変数の名前の Windows と UNIX 系の違いは、許容として README に書かれている(実機は未確認) |
| 15 | (v0.9.0 以降)`graph` のレーンの配置の規則は、正解データ(`graph_layout_oracle.md`、`graph_layout_oracle_cross.md` など)の例から導いたもので、正解データが原本(git のソースは読まない)。配置は 1 つの仕組みで、交差と同じ行き先は、「両方の飛び越し」や分岐と組み合わさっても、レーンの図で描く。git では作れない形(次のジョブへの辺がないジョブを含む形)の正解データは、設計の参照実装から導いた図で、git の出力ではない。4 つの種類(05 の 15.4)に当てはまらない図の構造では、「対応する形」の判定が、辺の一覧へ切り替える |

## 5. 受け入れの観点(まとめ)

実装が仕様どおりかを、次の順で確かめる。

1. **ユニット**: core の判断(05 の各表)、`split`(07 の 3 章)、`apply_*`(05 の 3 章)、`wait_loop`(間隔と転送)が、表の例のとおり。
2. **定義の検証**: 03 の受理・拒否の例と、34 種の識別子、行番号、位置の昇順。`.env` の 14 種(03 の 6.7)。
3. **ステート**: 書き込みの原子性、14 項目の読み込みの異常系(06 の 5 章)、`reset --all` の実測の例。
4. **流れ**: 偽の `Runtime` で、`a → b → c` の再開、`@end`、リトライの待ち時間の列、中断の 4 つの契機。
5. **実バイナリ**: 終了コードの全件(10 の 1 章)、SIGTERM → 143 と再開位置、`watch` の表、`convert` の往復、`--version`。
6. **品質ゲート**: fmt、clippy、test(3 OS + MSRV)、行カバレッジ 80% 以上、禁止の依存がないこと、シェルを呼ばないこと、`unsafe` が 1 か所。
7. **v0.9.0 の機能**: `env`・`.env` の値が子の環境に見え、値がどの出力にも出ない。`graph` の例(08 の 11 章)と配置の正解データ。`--dry-run` の計画の例と「本番と同じジョブ」。`init` の 69・`--force`・`--config -`、ひな形が `validate` に通る。
8. **v0.10.0 の機能**: 既定のリトライと `retry = false`(03 の 6.8)、無くてもよい `.env`、`unset_env` で消した名前が子の環境に無い、DOT の例(08 の 11.8)、試行ごとのログのファイル(名前、0600、端末と同じ中身)、`[[job.attempt]]` の読み書きと v0.9.9 までのステートの 65、`watch` の 4 列、`watch --follow` の終わりの判定と終了コード、デモの場面 0〜15。
