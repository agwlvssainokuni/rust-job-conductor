## Developer Code Scan Results

対象: rust-job-conductor(HEAD `5bdcbbfc66a4af65db045373b01d134b91655fc9`、版 0.6.0)。意図: `readme-reorg`(README.md の構成の整理。意味は変えない)。深さ: Minimal。

### Scan Coverage
- **Analyzed deeply**(全文または公開 API・契約・分岐を読んだ):
  - `README.md`(全 283 行)、`demo/README.md`(全 59 行)
  - `.github/workflows/ci.yml`、`release.yml`、`audit.yml`(全文)
  - `Cargo.toml`(ワークスペース)、`crates/*/Cargo.toml`、`Cargo.lock` の要点、`.gitignore`
  - `crates/cli/src/`: `lib.rs`、`main.rs`、`exit.rs`、`args.rs`、`types.rs`、`ctx.rs`、`run.rs`、`pipeline.rs`、`aux_commands.rs`、`platform/*`、`report/messages.rs`(USAGE ほか)、`report/watch.rs`、`report/escape.rs`、`report/mod.rs`(言語・色・表示の経路)
  - `crates/core/src/`: `lib.rs`、`run_id.rs`、`definition/mod.rs`、`definition/version.rs`、`definition/intake.rs`(定数)、`definition/walk.rs`(キー・規則の部分)、`flow/classify.rs`、`flow/decide.rs`、`error.rs`(種類の一覧)
  - `crates/runtime/src/`: `lib.rs`、`types.rs`、`state/reset_all.rs`、`state/writer.rs`(直列化・一時ファイル名)、`state/mod.rs`、`runner.rs`(待ちの部分)、`split.rs`(規則)、`platform/unix.rs`
  - `crates/converter/src/lib.rs`(契約・上限)、`intake.rs`(上限定数)
  - `demo/run-demo.sh`(場面の見出し)、`demo/windows/run-demo.cmd`(要点)、`demo/conductor.toml`、`demo/windows/conductor.toml`、`demo/scripts/extract.sh`
- **Skimmed only**:
  - `crates/*/tests/`(計 41 ファイル。cli 20、runtime 11、core 6、converter 4)。ファイル名と、README・USAGE・デモを固定するテスト(`unix_demo.rs`、`windows_demo.rs`、`report_messages.rs` の見出し)だけ
  - `crates/core/src/definition/{parse,depth}.rs`、`flow/{next_step,resume,mod}.rs`、`types.rs`(型の一覧のみ)
  - `crates/runtime/src/state/{reader,record,consistency}.rs`、`error.rs`、`platform/{mod,windows}.rs`、`bin/conductor-test-helper.rs`
  - `crates/converter/src/{resolve,scalar,raw,yaml_read,yaml_write,toml_read,toml_write,error,types}.rs`(関数・型の一覧のみ)
  - `crates/cli/src/deps.rs`、`panic_hook.rs`、`bin/conductor-cli-test-helper.rs`
  - `demo/scripts/*`、`demo/windows/scripts/*`、`demo/run-demo.sh` の場面 0〜6・8〜9 の本体
- **Left out**: `.claude/`、`aidlc/`(作業の領域)、`.idea/`、`target/`、`local-docs/`、`.gitignore` で除外されるファイル。`LICENSE`(本文は読まず)。

### Packages Found
- `conductor-core`(`crates/core`)— lib — Rust — 定義の読み込み・検証と、フローの判断(純粋)。run ID の規則もここ
- `conductor-runtime`(`crates/runtime`)— lib(+ テスト用 bin `conductor-test-helper`)— Rust — ジョブの直接起動、シグナルの転送、ステートの保持・照合・初期化
- `conductor-converter`(`crates/converter`)— lib — Rust — 定義ファイルの YAML と TOML のあいだの変換
- `conductor-cli`(`crates/cli`)— lib `conductor_cli` + bin `conductor`(配布)+ テスト用 bin `conductor-cli-test-helper` — Rust — コマンド、流れ、シグナルの受け取り、表示、終了コード

### Build System
- **Type**: Cargo ワークスペース(`resolver = "3"`、edition 2024、`rust-version = "1.85"`、license Apache-2.0)。リリースのプロファイル: `opt-level=3`、`lto="thin"`、`codegen-units=1`、`strip=true`(`Cargo.toml`)
- **Config Files**: `Cargo.toml`、`Cargo.lock`(46 パッケージ)、`crates/*/Cargo.toml`。`rustfmt.toml`・`clippy.toml` はない(既定の設定)
- **Build Dependencies**: `conductor-cli` → `core`・`runtime`・`converter`、`runtime` → `core`、`converter` は `core` に依存しない、`core` は葉。全クレート `publish = false`、版は 0.6.0 で同じ

### APIs Discovered
- CLI(`conductor`)— `crates/cli/src/args.rs` — コマンド 6(run・pipeline・watch・reset・validate・convert)、オプション `--config`・`-v/--verbose`・`--run-id`・`--all`・`--to`・`-h/--help`・`-V/--version`
- ライブラリ契約 C2(core)— `crates/core/src/lib.rs` — 公開関数 5(`load_definition`、`parse_definition`、`decide_before_run`、`classify_and_decide`、`resume_point_on_interrupt`)
- ライブラリ契約 C3(runtime)— `crates/runtime/src/lib.rs` — 公開関数 8 + `apply_*` 3
- ライブラリ契約 C4(converter)— `crates/converter/src/lib.rs` — `convert`、`convert_text`
- 差し替え口(cli)— `crates/cli/src/deps.rs` — trait 4(`Runtime`・`Sink`・`Env`・`Signals`)
- ファイル形式: 定義(TOML。`format_version = 1`)、ステート(TOML。`format_version = 1`)

### Frameworks & Libraries
- `toml` 1.2.1(core・runtime・converter)— 定義とステートの読み書き
- `thiserror` 2.0.21(core・runtime・converter)— エラー型
- `saphyr-parser` 0.3.4(converter)— YAML の読み取り
- `lexopt` 0.3.2(cli)— 引数の字句への分割
- `signal-hook` 0.4.6(cli、UNIX。`default-features = false`、`iterator`)、`ctrlc` 3.5.2(cli、Windows)— シグナル
- `libc` 0.2(runtime、UNIX)— `kill`(`unsafe` はここだけ)
- `tempfile` 3.27.0(dev)

### Test Coverage
- **Test Directories**: `crates/{core,runtime,converter,cli}/tests/`(統合テスト 41 ファイル)、各 `src/` 内の `#[cfg(test)]`。`#[test]` はおよそ 681 個
- **Test Frameworks**: 標準の `cargo test`。テスト用の補助の実行ファイル 2 つ(`conductor-test-helper`、`conductor-cli-test-helper`)
- **Coverage Config**: CI の `coverage` ジョブ(`cargo-llvm-cov`、行 80% 未満で失敗。補助の bin は除外)。ほかに MSRV(1.85)のテスト、起動のオーバーヘッドの測定(`startup-bench`。警告のみ)

### Code Quality Indicators
- **Linting**: `cargo fmt --all -- --check`、`cargo clippy --workspace --all-targets --locked -- -D warnings`(`ci.yml` の `lint`)。`cargo audit`(`audit.yml`。警告のみ)
- **CI/CD**: `ci.yml`(lint・build-check・test・msrv-test・coverage・docs-check・startup-bench)、`release.yml`(タグ `v*` で下書きのリリース)、`audit.yml`
- **Documentation**: README と demo/README は日本語。ライブラリの doc コメントは契約・順序・保証を詳しく書く

### Technical Debt Signals
- 同じ規則のエスケープ関数が 4 クレートに重複(`crates/cli/src/report/escape.rs` の doc コメントが明記。`tests/escape_table.rs` で揃えを確認)
- README とヘルプ(`messages.rs` の USAGE)の二重管理(下の 6)
- doc コメントの内部の ID(`BR*`、`NFR*`、`FR*`、`W*`)は、リポジトリ内に対応の文書がなく(要件は外部)、追えない

## Handoff Summary
- **Intent-relevant finding**: README の整理に必要な事実は、下の (1)〜(6) にまとめた。README の主張の大半はコードと一致する。ただし、下の (2) の「不一致・あいまい」の 4 件は、整理のときに直すか、意味を変えずに言い換えるかの判断が要る。最も重要なのは、**展開の手順(README 47〜50 行)が、アーカイブの実際の構造と合わない**こと(`release.yml:77-84`)。
- **Risks / follow-up**: docs-check の 7 語を失わないこと((3))。run ID の記述は 9 か所に散っており、整理で表の記述を短くすると、規則(値・優先順位・`--run-id ""`・`reset --all`)の欠落が起きやすい((1))。

### (1) README の現在の節と、話題の所在

| 行 | 節 |
|----|----|
| 1-11 | 題名と概要(単一バイナリ、ネットワークなし、ロケールで英日) |
| 12-71 | インストール: 14-23 ダウンロード(OS 別ファイル表)/ 25-41 SHA-256 の確認 / 43-51 展開と PATH / 53-62 macOS Gatekeeper / 64-71 ソースからのビルド |
| 73-122 | 使い方: 75-83 構文 / 85-92 コマンド表 / 94-102 オプション表 / 104-105 注記 / 107-120 例 / 122 デモへのリンク |
| 124-196 | 定義ファイルの書き方: 128-161 例 / 163-174 キー表 / 176-178 注記 / 180-182 command の書き方 / 184-196 配列(v0.2.0 以降) |
| 198-216 | 終了コードの表(表 200-212、注記 214-216) |
| 218-237 | ステートファイルの場所: 220-225 / 227-237 run ID でステートを分ける(v0.5.0 以降) |
| 239-247 | 保証の範囲(at-least-once) |
| 249-256 | シグナルの扱い |
| 258-270 | convert の使い方 |
| 272-279 | ロケールと言語 |
| 281-283 | ライセンス |

run ID の所在(散在 9 か所): 構文 76-79(`--run-id`・`--all`)/ コマンド表 89(watch の `Run ID:` 行)・90(reset)/ オプション表 98(`--run-id`)・99(`--all`。末尾の「(68)」は唐突)/ 例 115-116 / 定義の例 134 と キー表 167(`run_id_env`)/ 終了コード表 205(64)・206(65)・209(68)/ 注記 225(同時実行)/ 本体 227-237(与え方・値の規則・中身の `run_id`・watch/reset・reset --all・大文字小文字・同時実行)。

版の注記: 184「(v0.2.0 以降)」、195「v0.1.x」、227「(v0.5.0 以降)」。`validate` と `--all` に注記はない(各 README の存在: `validate` は v0.4.0 の README に既にある。`--all` は v0.5.0 の README にある)。

### (2) README の主張とコードの照合

| README の主張(行) | 結果 | 根拠 |
|---|---|---|
| 6 コマンドと構文(75-83)は USAGE と同じ | 一致 | `crates/cli/src/report/messages.rs:59-100`、`args.rs:23-30` |
| `--config` の既定は `conductor.toml`(96)。`convert` では使えない | 一致 | `ctx.rs:26,51-56`、`args.rs:218,223` |
| `-v` は run・pipeline・validate だけ、1 段階(97)。2 回目は 68 | 一致(2 回は README に未記載) | `args.rs:214-221,240` |
| `--run-id`: run/pipeline/watch/reset だけ。validate・convert では 68(98、209) | 一致 | `args.rs:229-235` |
| `--all` は reset だけ。`--run-id` と併用不可 → 68(99、209) | 一致 | `args.rs:236-238,256-258`(`invalidValue`) |
| `--` で先頭 `-` のジョブIDを書ける(104) | 一致 | `lexopt` の標準の動作。`args.rs:35` |
| 既定では conductor の出力なし。エラーだけ標準エラー(105) | 一致 | `report/mod.rs:225-240`(Level)、`lib.rs` |
| watch は 3 列の表を 1 回。run ID があれば次の行に `Run ID: <ID>`(89) | 一致 | `report/watch.rs:86-98` |
| validate: ジョブを実行せず、ステートに触れず、成功は何も出さない。名前は照合、`-v` で 1 行(91、97) | 一致 | `aux_commands.rs:100-109`、`ctx.rs:136-165`、`messages.rs:103-111` |
| convert: 入力は `--to` の反対とみなし、拡張子を見ない。標準出力に 1 回(260、267) | 一致 | `converter/src/lib.rs:8-12`、`aux_commands.rs:112-135` |
| 入力ファイルは 1 MiB まで(270) | 一致 | `converter/src/intake.rs:29` |
| 定義のキー・規則(163-174): `id` は空でなく重複なし `@` で始めない、遷移先は後ろか `@end`、0 は警告に不可、重なり不可 | 一致 | `walk.rs:33-45,350-420`、`error.rs:31-71` |
| `success_exit_codes` 省略で `[0]`(171) | 一致 | `classify.rs`(`effective_success_codes`) |
| `[[jobs]]` は定義の誤り(178) | 一致(`unknownKey`) | `walk.rs:33,88-94` |
| `run_id_env` の形、型の誤りは定義の誤り(167) | 一致 | `walk.rs:236-243`、`run_id.rs:is_valid_env_name` |
| command の分割規則(182) | 一致 | `runtime/src/split.rs:15-50` |
| 終了コード 0/ジョブの値/1/64/65/66/67/68/70/127/128+n(200-212) | 一致 | `exit.rs:17-31`、`classify.rs:17-25,70-110`、`aux_commands.rs`、`ctx.rs:75-103` |
| 範囲外は下位 8 ビット、0 になれば 1(214)。`-v` に元の値(214) | 一致 | `exit.rs:35-42`、`report/mod.rs:474-482` |
| 書き込み失敗は 65 で、「ジョブは実行されました」(216、244) | 一致 | `run.rs`、`messages.rs:235` |
| ステートの場所と名前(220、229) | 一致 | `runtime/src/types.rs:StateLocation::of` |
| 完了・`@end`・スキップ完了でステートを消す(222) | 一致 | `run.rs`、`pipeline.rs:finish` |
| 自動の再試行なし。エラーのジョブから再開(223) | 一致 | `pipeline.rs:117-120`、`decide.rs:4` |
| 排他制御なし(225、237) | 一致 | `runtime/src/lib.rs`(ロックは提供しない) |
| run ID の与え方・優先順位(231) | 一致 | `args.rs:327-342`、`ctx.rs:93-103` |
| run ID の値の規則(232) | 一致 | `run_id.rs:35-53`、`args.rs:309-318` |
| 中身の `run_id` と名前の食い違いは壊れたステート(233) | 一致 | `reader.rs:233-244` |
| watch/reset は `--run-id` があれば定義を読まず、なければ `run_id_env` をゆるく読む(234) | 一致 | `ctx.rs:285-303`、`run_id.rs:peek_run_id_env` |
| reset --all の対象と残すもの(235) | 一致(ただし下の不一致 ③) | `reset_all.rs:1-12,48-98` |
| シグナル: UNIX は 4 種、1 回だけ転送、128+n、最初の番号の規則(253) | 一致 | `platform/unix.rs`、`runtime/src/platform/unix.rs:34-50`、`runtime/src/lib.rs:59-61` |
| Windows は Ctrl+C のみ、130(254) | 一致 | `platform/windows.rs:20-31` |
| 起動前・ジョブのあいだの中断は再開位置を記録(255) | 一致 | `pipeline.rs:131-148` |
| ロケール: `LC_ALL`→`LC_MESSAGES`→`LANG`、最初の空でない値が `ja` で始まれば日本語(274) | 一致 | `report/mod.rs:371-385` |
| 色: 標準出力が端末、UNIX 系、`NO_COLOR` 未設定または空(278) | 一致 | `report/mod.rs:206-212`、`platform/mod.rs:60` |
| エスケープ: ジョブID・キー名は 64 文字まで、パスは切らない(279) | 一致 | `report/escape.rs:19-48` |
| ソースのビルドは Rust 1.85 以降、`cargo build --release --locked --bin conductor`、`target/release/conductor`(66-70) | 一致 | `Cargo.toml:5`、`release.yml:63` |
| アーカイブにはバイナリと `LICENSE` が入る(45) | 一致(ただし下の不一致 ①) | `release.yml:73-77` |
| リリースは署名しない(41、55) | 一致 | `release.yml`(署名の手順なし) |

不一致・あいまい・確認できないもの:
- **① 不正確(47-50 行)**: `tar xzf conductor-<版>-<対象>.tar.gz` のあと `install -m 755 conductor /usr/local/bin/conductor` と書くが、アーカイブは最上位のディレクトリ `conductor-<版>-<対象>/` を含み、その中に `conductor` と `LICENSE` がある(`release.yml:73-81`。スモークテストも `smoke/*/<bin>` で読む: `release.yml:90-94`)。README 45 の「展開すると `conductor` と `LICENSE` がある」も、ディレクトリの一段を言っていない。コマンドは `conductor-<版>-<対象>/conductor` を指すべき。Windows の zip も同じ構造。
- **② 不完全(251)**: 「`watch`・`reset`・`convert` は既定の動作」と並べるが、`validate` も、シグナルの準備をしない(`aux_commands.rs:3` の冒頭の注記。`run_validate` は `prepare` を呼ばない: `ctx.rs:67-80` は run・pipeline だけ)。
- **③ あいまい(235)**: 「`<名>.state` と `<名>.<run-id>.state` のうち、…中身の `run_id` が一致するものをすべて消す」は、run ID なしの `<名>.state` にも中身の検査があるように読める。実際は、`<名>.state` は読まずに、通常のファイルなら消す(`reset_all.rs:36-40`)。シンボリックリンクなら消さない。また、消すのに失敗したときは 65(`reset_all.rs:remove`)だが、README にない。
- **④ 記述の断片(99)**: 「`--run-id` とは、一緒に使えない(68)」の「(68)」は終了コードの意味だが、ほかの箇所と書き方が違う。
- **⑤ 確認できない**: `shasum -a 256 --check --ignore-missing`(32-33)は、OS の `shasum` の版に依存する(コードでは検証できない)。Gatekeeper の手順(55-62)も、macOS の動作で、コードの外。
- **⑥ README にない事実**: Intel macOS(`x86_64-apple-darwin`)は、クロスビルドで、スモークテストをしない(`release.yml:84`)。リリースの説明文に「動作の確認なし」と書く(`release.yml:131`)。README の OS 表(18-23)は、これを言っていない。Linux のバイナリは、musl の静的リンク(`release.yml:92-94`)。

### (3) docs-check と、README / デモを読むほかのもの
- `.github/workflows/ci.yml:114-127` の `docs-check`(`ubuntu-24.04`): `for pat in 'SHA-256' 'Gatekeeper' '終了コードの表' 'ステートファイルの場所' 'at-least-once' 'シグナル' 'convert'`(124 行)を、`grep -q -- "$pat" README.md`(125 行)で、固定の文字列として探す。無ければ失敗。現在の所在: `SHA-256` 25 行(見出し)、`Gatekeeper` 53・55 行、`終了コードの表` 198 行(見出し。この語は見出しだけ)、`ステートファイルの場所` 218 行(見出し。この語は見出しだけ)、`at-least-once` 239 行(見出し。この語は見出しだけ)、`シグナル` 210・212・249 ほか、`convert` 多数。**`SHA-256`・`終了コードの表`・`ステートファイルの場所`・`at-least-once` は、見出しにだけ現れる**ので、見出しの語を変える整理は CI を壊す。
- この `docs-check` は `release.yml` の `checks`(`workflow_call`)にも含まれ、リリースの前に必ず通る。
- `README.md` を読むほかのコード・テスト・ワークフローはない(`crates`・`.github`・`demo` を `README` で検索: 一致は `ci.yml` だけ)。
- `demo/README.md` を読むものもない。ただし、デモの動作はテストが固定する: `crates/cli/tests/unix_demo.rs`(`sh demo/run-demo.sh`)、`windows_demo.rs`(`demo/windows`)。README のデモの記述(場面の内容・`DEMO_*` の名前と既定値)を、スクリプトと合わせる必要がある。
- demo/README の細かい点: 場面の表(14-24 行)は 1〜9 だが、Windows 版の記述(35 行)は「場面 0〜9」と書く。場面 0(定義の表示。`run-demo.sh:67`)が、UNIX 版の表にない。`DEMO_SLEEP` の既定は UNIX 0.5 秒(`extract.sh`)、Windows 1 秒(demo/README 52 行の記述。スクリプトは未確認)。

### (4) ソースからのビルドの事実
- ワークスペース: `Cargo.toml` の `members = ["crates/*"]`(core・runtime・converter・cli)。edition 2024、`rust-version = "1.85"`、`resolver = "3"`。
- 配布する実行ファイルは `conductor` だけ(`crates/cli` の `[[bin]] name = "conductor"`)。`conductor-cli-test-helper`(cli)と `conductor-test-helper`(runtime)は、テスト専用で、配布しない。`cargo build --workspace` はこれらも作る。
- ビルド: `cargo build --release --locked --bin conductor` → `target/release/conductor`(Windows は `conductor.exe`)。CI のリリースは `--target <triple>` を付け、`target/<triple>/release/` に出る(`release.yml:63,72`)。
- テスト: `cargo test --workspace --locked`(`ci.yml:63,72`)。起動の測定: `cargo test -p conductor-cli --release --locked --test nfr_perf -- --nocapture`。
- 品質の検査: `cargo fmt --all -- --check`、`cargo clippy --workspace --all-targets --locked -- -D warnings`。カバレッジ: `cargo llvm-cov --workspace --locked --ignore-filename-regex 'bin/conductor-(cli-)?test-helper' --fail-under-lines 80 --summary-only`(`cargo-llvm-cov` と `llvm-tools-preview`)。脆弱性: `cargo audit`(`cargo-audit`。警告のみ)。
- ビルドの制約: 依存に C のビルド(`cc`・`cmake`・`pkg-config`・`bindgen`、`-sys` クレート。ただし `linux-raw-sys`・`windows-sys` を除く)を入れない(`ci.yml:46-54`)。最小の Rust 1.85.0 で、ビルドとテストの両方を行う(`ci.yml:36,67`)。
- 外部の実行環境は不要(単一バイナリ)。Windows の MSRV のビルドは `build-check` だけ(テストは stable)。

### (5) インストールの事実(`release.yml`)
- トリガー: タグ `v*`(`release.yml:17-19`)。`verify-tag`: タグが `main` の祖先で、`crates/cli/Cargo.toml` の版と一致すること。版に `-` を含めば、プレリリース。
- `checks`: `ci.yml` を `workflow_call` で全部流す(リリースの前の関門)。
- `build`(`fail-fast: true`)の 4 ターゲット: `x86_64-unknown-linux-musl`(ubuntu-24.04、tar.gz)、`aarch64-apple-darwin`(macos-latest、tar.gz)、`x86_64-apple-darwin`(macos-latest、tar.gz。クロスビルド)、`x86_64-pc-windows-msvc`(windows-2022、zip。`conductor.exe`)。
- アーカイブ名: `conductor-<版>-<ターゲット>.tar.gz` または `.zip`(`<版>` は `v` を除く)。中身: 最上位のディレクトリ `conductor-<版>-<ターゲット>/` に、バイナリと `LICENSE`。テスト用の補助は含めない。
- スモークテスト: 展開して `--version`。Linux は静的リンクも確認。Intel macOS は実行しない。
- `draft-release`: アーティファクトを集めて `SHA256SUMS`(`sha256sum *` の形式: 「ハッシュ  ファイル名」)を作り、`gh release create --draft --generate-notes`(プレリリースなら `--prerelease`)で、**下書き**を作る。公開は人が確認して行う。説明文に、Intel macOS は動作の確認なし(クロスビルド)と、`SHA256SUMS` で確かめることを書く。
- 署名・公証・Windows の署名はしない(README の 41・55 行と一致)。`permissions` は、`draft-release` だけ `contents: write`。

### (6) README とヘルプ(USAGE)の重複と、ずれやすい点
- `messages.rs:59-100` の USAGE(英語・日本語の 2 本)が持つ: 構文 7 行、オプションの説明 7 行(`-v`、`--config`、`--run-id`、`--all`、`--to`、`-h`、`-V`)。README の 75-83 行は、日本語版の構文と同じ記号(`<パス>`・`<ID>`・`<ジョブID>`・`<パイプライン名>`・`<ファイル>`)で、同一。94-102 行のオプション表は、同じ内容を、より詳しく書く。
- ずれやすい箇所: `--run-id` の既定の説明(USAGE 73-74 と README 98・231)、`-v` の対象(USAGE 71・README 97)、`--all`(USAGE 75・README 99・235)。コマンドの追加やオプションの変更があると、USAGE と README の両方を直す必要がある。USAGE の内容を固定するテストは `crates/cli/tests/report_messages.rs`(`使い方は種類ごとの固定の文と使い方の一覧`、711 行付近)。
- USAGE にあって README にない: `-v` の validate の説明の細部(「validate は 1 行」)は両方にある。`--help` の出力先と、`--version` の形(`conductor <版>`、`report/mod.rs:239-241`)は README にない(README は `conductor --version` を例に使うだけ)。
