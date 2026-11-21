# コード生成の計画(Code Generation Plan): U4 u4-init(`conductor init`)

## Summary

- Builds: `conductor init [--config <パス>] [--force] [<パイプライン名>]`(定義のひな形を作る。既にあれば何も変えず終了コード 69、`--force` で上書き、`--config -` で標準出力)。排他的なファイルの作成(`runtime`)、ひな形の生成(日本語と英語。必ず `validate` に通る)、引数、流れ、メッセージ
- Touches: `crates/runtime`(新しい `file_create` モジュール)、`crates/cli`(`args.rs`、`types.rs`、`deps.rs` の `Runtime` の trait、新しい `init.rs`、`report/`)、`README.md`、`demo/`
- Tests: 約 35 個(単体、ひな形が `validate` に通ること、偽の `Runtime` を使う流れのテスト、実バイナリのテスト)。既存の引数の文法とメッセージの網羅のテストを合わせて直す

## 前提と範囲

- 設計の原本: `construction/u4-init/functional-design/` の `entities.md`、`rules.md`(BR の番号)、`functional-spec.md`(WF1〜WF3、ひな形の原本(日本語と英語)、メッセージの表、契約への反映)。契約: `inception/contract-design/contract-summary.md` の C3、C4、C5(機能の設計の「契約への反映」で補正済み)。U1 の検査(`[pipeline.env]`)と、U3 の `--dry-run`(組み合わせの確認)を、テストで使う。
- 新しい外部ライブラリは足さない(`Cargo.toml` と `Cargo.lock` は変えない)。`unsafe` は書かない。`cfg(unix)`/`cfg(windows)` は `platform` の外に置かない(権限の誤りのテストは、実行時に効果を確かめ、効かない環境(root など)では、確かめずに通す)。ひな形の例の値は、固定の `hello` だけ(利用者の値は入らない)。
- 作らない: 複数のひな形、対話的な質問、親ディレクトリの作成、ひな形への追記。
- 範囲の外(別の依頼): 仕様書(`docs/spec/**`)の更新、版 0.9.0 への引き上げ、リリース。
- 変更を受ける、この作業の外の既存ファイル(名前を明記する): `README.md`(新しい節を足す。`docs-check` の必須語句は残す)、`demo/` の台本とデモのテスト。
- コミットは小さな単位で、直接 `main` に作る(`team.md`)。各ステップは、できるだけ 1 つのコミットにする。
- ファイルの改行を保つ: `demo/windows/run-demo.cmd` などの CRLF のファイルは、CRLF のまま編集する(編集のあと、`git ls-files --eol` が `w/crlf` のままで、`git diff --stat` が追加した行だけであること)。シェルで、ファイルを一括変換しない。

## Assumptions

なし(設計の `entities.md` にない、データモデルの要素は足さない)。

## Testing Contract

```json
{
  "version": 1,
  "methodology": "test-after",
  "source": "team",
  "ordering": "各テスト可能な層を実装してから、その層のテストを書いて実行する。",
  "scope": "feature",
  "test_strategy": "standard",
  "project_type": "brownfield",
  "applicable_notes": [
    {
      "layer": "org",
      "text": "We treat tests as a first-class deliverable in every Bolt. The specific\nmethodology (TDD, BDD, ATDD, or classic test-after) is affirmed at\npractices-discovery and recorded in `team.md` under this heading with explicit\n`Methodology` and `Ordering` fields; Code Generation resolves those fields\nindependently from coverage, tooling, and scope notes.\n\nWhen no posture has been affirmed, our default per scope is:\n- **Methodology**: test-after\n- **Ordering**: implement each applicable testable layer, then write and run\n  that layer's tests.\n- `mvp`, `enterprise`, `feature`, `infra` add an 80% line-coverage\n  floor and CI execution before merge.\n- `bugfix`, `security-patch` add a targeted regression for the specific\n  bug/vulnerability and require the existing suite to remain green.\n- `express` uses the Minimal strategy: requirement-driven unit tests (one per\n  requirement, with a happy-path floor per component); existing tests remain\n  green.\n- `poc`, `refactor`, `workshop`, `classic` add no extra new-test floor and require the\n  existing suite to remain green.\n\nThe active `Test Strategy` still applies in every scope and determines test\nvolume/types. Scope floors are additive; they never reduce or replace the\nselected strategy.\n\nBuild and Test verifies defined coverage floors and affirmed quality targets;\nthey may not be weakened to make a step pass.\n\nAffirm a stricter posture in `team.md` if the team commits to one."
    },
    {
      "layer": "team",
      "text": "- **Methodology**: test-after\n- **Methodology evidence**: 基準(前回のインタビュー Q3)でチームは test-after を選んだ。履歴も整合する。たとえば `[job.retry]` では、型・読み取りの実装(d554ece、445d478)の後に、テストを追加するコミット(8ec98b5)が続き、実行の流れ(19d7b1a)の後に README のテスト(de4e3ad)が続く。`code-quality-assessment.md` も「チームの方針(test-after、行カバレッジ 80%)と一致する」と記す。\n- **Ordering**: 各テスト可能な層を実装してから、その層のテストを書いて実行する。\n- テストの種類: 各クレートの `tests/` に統合テスト(core 10、runtime 13、cli 21、converter 5 のファイル)。流れのテストは偽の `Runtime`/`Sink`/`Env`/`Signals` を注入する。実バイナリのテストは `conductor-cli-test-helper` を使う。非機能のテスト(`nfr_*.rs`)、網羅のテスト(`ALL` 配列)、デモのテスト(`unix_demo.rs`、`windows_demo.rs`)を含む。テスト名は日本語。\n- 行カバレッジ 80% を下限とする。CI の `coverage` ジョブが `cargo llvm-cov --fail-under-lines 80` で強制する(テスト専用の補助バイナリは除外)。\n- ローカルでは、主な開発環境の OS でテストを実行する。それ以外の OS のテストは GitHub Actions の CI(`test`: ubuntu、macos、windows)で実行する。マージ前に CI を通す。\n- 最小の Rust(1.100.0)でもテストを実行する(`msrv-test`)。\n- 起動の性能(`nfr_perf.rs`)は `startup-bench` で測るが、警告のみで失敗にしない(`STARTUP_BENCH_ENFORCE=false`、`continue-on-error`)。\n- 今回の機能にも、test-after と 80% の下限、単体テストと実バイナリを動かす統合テストを続ける(インタビュー Q3)。\n- 主な開発環境の OS は macOS で、テストはローカルの macOS で実行する。他の OS は CI で確認する(インタビュー Q6)。"
    }
  ],
  "obligations": {
    "strategy": "standard",
    "strategy_volume": [
      "Five to eight tests per component.",
      "Unit tests plus integration tests for key boundaries.",
      "Add E2E, performance, or security tests when requirements demand them."
    ],
    "scope_floor": [
      "Meet an 80% line-coverage floor.",
      "Run the selected tests in CI before merge."
    ],
    "combination_rule": "Apply every selected-strategy obligation and every scope-floor obligation; neither replaces the other, and a targeted scope regression may add the narrowest necessary test type beyond the strategy default."
  },
  "plan_profile": {
    "methodology": "test-after",
    "runner_step": "Verify the existing test runner/configuration and record the exact unit-scoped command.",
    "runner_ready_before_first_test": true,
    "testable_layers": [
      "Data model / database behavior",
      "Repository / data access",
      "Business logic",
      "API / endpoint",
      "Frontend behavior"
    ],
    "steps": [
      "Project structure and production configuration skeleton.",
      "Verify the existing test runner/configuration and record the exact unit-scoped command.",
      "Data model / database behavior - implement.",
      "Data model / database behavior - write and run its tests after implementation.",
      "Repository / data access - implement.",
      "Repository / data access - write and run its tests after implementation.",
      "Business logic - implement.",
      "Business logic - write and run its tests after implementation.",
      "API / endpoint - implement.",
      "API / endpoint - write and run its tests after implementation.",
      "Frontend behavior - implement.",
      "Frontend behavior - write and run its tests after implementation.",
      "Environment/build configuration.",
      "Documentation and traceability."
    ]
  },
  "input_sha256": "sha256:826bac9fdf85fea4f3f7f79a7d3365e6615a41720c04ce8e867b1220ea1ec197",
  "contract_sha256": "sha256:0ca5a7b0baee94a8a8284941eb581edfac09ab7108c711149824fd0b57d005b4"
}
```

適用する層: 「データモデル」は `CreateOutcome`、`CreateError`(`runtime`)と `InitOptions`、`InitTarget`(`cli`)、「リポジトリ/データアクセス」は排他的なファイルの作成(`runtime::file_create`)、「ビジネスロジック」はひな形の生成と名前のエスケープ(`cli` の `report`。純粋な関数)、「API/エンドポイント」はコマンドの引数、流れ、メッセージ(`cli`)。「フロントエンド」は該当しない。

## 実装のステップ

### Step 1: 構造と設定の骨格(production configuration skeleton)

- [x] 1.1 新しいモジュールの空の枠を作る: `crates/runtime/src/file_create.rs`、`crates/cli/src/init.rs`、`crates/cli/src/report/init_template.rs`。`crates/runtime/src/lib.rs`、`crates/cli/src/lib.rs`、`crates/cli/src/report/mod.rs` に、`mod` と公開の `use` を足す。外部ライブラリは足さない(`Cargo.toml` は変えない)。`cargo build --locked --workspace` が通ることを確かめる。

### Step 2: テストの実行環境の確認(test runner readiness)

- [x] 2.1 既存のテストの実行環境(`cargo test --locked`)が動くことを確かめ、この Unit の、範囲を絞ったコマンドを `unit-test-instructions.md` に記録する(新しい実行環境や設定は作らない)。

### Step 3: データモデル — 実装(型)

- [x] 3.1 `crates/runtime/src/file_create.rs`: `CreateOutcome`(`Created`、`Replaced`)、`CreateError`(`AlreadyExists`、`ParentMissing`、`IsDirectory`、`PermissionDenied`、`OtherIo`)の型だけを定義する(`entities.md`)。`crates/cli/src/types.rs`: `InitTarget`(`File(path)`、`Stdout`)、`InitOptions`(`target`、`force`、`pipeline_name`。既定の名前 `my-pipeline`)を定義する。

### Step 4: データモデル — テストを書いて実行する

- [x] 4.1 `crates/cli/tests/init_types.rs`(新規、約 3 個): `InitOptions` の既定(名前 `my-pipeline`、`force = false`)、`InitTarget` の `-` の判定(`--config -` が `Stdout`、それ以外が `File`)。(BR1.1、BR1.5、BR2.2)

### Step 5: リポジトリ/データアクセス(排他的なファイルの作成。`runtime`)— 実装

- [x] 5.1 `crates/runtime/src/file_create.rs`: `create_file(path, bytes, force) -> Result<CreateOutcome, CreateError>` を、WF3 と BR1.3 のとおりに実装する。新規は `create_new`(排他的)で、失敗したら `symlink_metadata` で分類する(ディレクトリ → `IsDirectory`、ファイルまたはシンボリックリンク → `AlreadyExists`、`NotFound` → `ParentMissing`、権限 → `PermissionDenied`、そのほか → `OtherIo`)。書き込みに失敗したら、新規で作ったファイルを消す。`--force` は、同じディレクトリの一時ファイル(`state/writer.rs` の一時ファイルの名前の作り方を再利用)に書いて置き換える(失敗したら一時ファイルを消す。置き換え先がディレクトリなら `IsDirectory`。リンクは、リンクそのものを置き換える)。内容はそのまま(改行を変換しない)。`unsafe` は書かない。(BR1.2、BR1.3、BR1.6)

### Step 6: リポジトリ/データアクセス — テストを書いて実行する

- [x] 6.1 `crates/runtime/tests/file_create.rs`(新規、8 個、一時ディレクトリ): 新規の作成(内容が書いたとおり。改行が LF のまま)、既にあるファイルを壊さない(`AlreadyExists`。内容が変わらない)、`force` で置き換える(内容が新しくなる。一時ファイルが残らない)、親のディレクトリが無い(`ParentMissing`。ディレクトリを作らない)、作る先がディレクトリ(`IsDirectory`。`force` ありなしの両方)、宙に浮いたシンボリックリンクは `AlreadyExists`(`force` ならリンクそのものを置き換える。UNIX で作れる環境のみ。作れなければ通す)、書く権限がないディレクトリ(`PermissionDenied`。効かない環境(root など)では確かめずに通す)、同時に作る 2 つのスレッドのうち、ちょうど 1 つが `Created` になり、ほかは `AlreadyExists`(排他的)。(BR1.2、BR1.3、BR1.6)

### Step 7: ビジネスロジック(ひな形。`cli`)— 実装

- [x] 7.1 `crates/cli/src/report/init_template.rs`: `render_template(lang, name) -> String`(`functional-spec.md` の日本語と英語のひな形の原本。名前を `[pipeline].name` に、TOML の基本文字列として、`"`、`\`、制御文字(U+0000〜U+001F、U+007F は `\uXXXX`)をエスケープして入れる。名前はコメントには書かない。UTF-8、LF、末尾に改行 1 つ)。純粋な関数。(BR2.1〜BR2.3)

### Step 8: ビジネスロジック(ひな形)— テストを書いて実行する

- [x] 8.1 `crates/cli/tests/init_template.rs`(新規、7 個): 日本語と英語のひな形が、原本と 1 文字も違わず一致する(既定の名前)。名前のエスケープ(`"`、`\`、改行、U+007F、日本語)。どのひな形も、`parse_definition` で読めて、`check_command` で分けられ、U1 の検査(`[pipeline.env]` の名前の規則と予約名)と、リトライの検査を通る(日本語、英語、特殊な名前。BR2.4)。ひな形に、利用者の値が入らない(名前以外は、固定)。(BR2.1〜BR2.4)

### Step 9: API/エンドポイント(引数、流れ、メッセージ。`cli`)— 実装

- [x] 9.1 `crates/cli/src/args.rs`、`types.rs`: `init` サブコマンド、`--force`(`init` だけ)、位置引数 1 つ(パイプライン名。省略できる)、`--config -` で標準出力。名前が空または空白だけ、位置引数が 2 つ以上は 68。`init` では、`--config` と `--force` 以外のすべてのオプションは、許されないオプション(68)。(BR1.1、BR1.5、BR2.2、BR3.1)
- [x] 9.2 `crates/cli/src/deps.rs`: `Runtime` の trait に `create_file(path, bytes, force)` を足し、`SystemRuntime` で `conductor_runtime::create_file` を呼ぶ(偽の `Runtime` で、ファイルを使わずにテストできる)。
- [x] 9.3 `crates/cli/src/init.rs`: `init` の流れ(WF2): 引数の誤り(68)をファイルに触れる前に返す。ロケールと名前からひな形を作る。標準出力なら、1 回で書いて 0(書けなければ 65。成功のメッセージなし。`--force` は無視)。ファイルなら、`create_file` を呼び、`AlreadyExists` は標準エラーに「既にあります」を出して 69、そのほかの誤りは 65(固定の文とパス)、成功は標準エラーに 1 行(作成した、置き換えた)で 0。定義、`.env`、ステートを読まず、シグナルを準備しない。`crates/cli/src/exit.rs` に `EXIT_EXISTS = 69` を足す。`lib.rs` で振り分ける。(BR1.1〜BR1.6、BR3.1、BR3.2)
- [x] 9.4 `crates/cli/src/report/messages.rs` と `report/mod.rs`: 新しいメッセージ(作成した、置き換えた、既にある、作れなかった 4 種、標準出力に書けない。日本語と英語。`functional-spec.md` のメッセージの表のとおり)と、使い方を足す。パスは、制御文字をエスケープして、切り詰めて表示する。(BR4.1、BR3.2)

### Step 10: API/エンドポイント — テストを書いて実行する

- [x] 10.1 既存のテストの、引数の文法(`args_grammar.rs`)と、メッセージの網羅(`report_messages.rs`、`escape_table.rs`)を、新しいコマンドとメッセージに合わせて直す。
- [x] 10.2 `crates/cli/tests/flow_init.rs`(新規、9 個、偽の `Runtime`): 既定の名前とパスでひな形が書かれる(`create_file` の引数を確かめる)。`--force` が渡る。`AlreadyExists` で 69 とメッセージ。親が無い、ディレクトリ、権限、そのほかで 65 と、種類ごとの固定の文(OS の文を含まない)。`--config -` で標準出力にひな形だけ(`create_file` が呼ばれない。`--force` を無視。成功のメッセージなし)。標準出力に書けないと 65。名前が空、位置引数が 2 つ、許されないオプション(`-v`、`--dry-run` など)で 68(ファイルに触れる前。名前が空で、ファイルが既にあるときも 68)。定義、`.env`、ステートを読まず、シグナルを準備しない。(BR1.x、BR3.x)
- [x] 10.3 `crates/cli/tests/binary_init.rs`(新規、5 個、実バイナリ): 空のディレクトリで `conductor init` → `conductor validate` が通り(0)、`conductor pipeline my-pipeline --dry-run` が計画を出す(U3 との組み合わせ)。もう一度 `init` で 69 でファイルが変わらない。`init --force` で上書き(0)。`init --config -` の出力が、ファイルの内容と同じ。名前に特殊な文字を付けた `init` のファイルが、`validate` を通る。(FR4.x)

### Step 11: 環境/ビルドの設定、文書と追跡

- [x] 11.1 `Cargo.toml`、`Cargo.lock`、CI の設定は、変えない(新しい依存なし)。`crates/cli/tests/nfr_source.rs` が、新しいファイルについても、`unsafe`、`cfg`、新しい依存、ネットワークの検査を通ることを確かめる(必要なら対象の一覧を直す。しきい値は下げない)。
- [x] 11.2 `README.md`: `init` の節を足す(使い方、`--force`、`--config -`、ひな形の例(日本語)、終了コード(0、65、68、69)、`--force` の OS の差(Windows は読み取り専用の置き換え先を置き換えられない)、バージョン v0.9.0 以降)。`docs-check` の必須語句は残す。`crates/cli/tests/docs_init.rs`(新規、2 個): README に新しい語句(`conductor init`、`--force`)があり、README のひな形の例が、実際のひな形(日本語)と一致する。
- [x] 11.3 `demo/`: `init` の場面を足す(`demo/run-demo.sh` と `demo/windows/run-demo.cmd` に、一時ディレクトリで `init` → `validate` → 2 回目の `init`(69)の例を足す。`run-demo.cmd` は CRLF のまま編集する)。デモのテスト(`unix_demo.rs`、`windows_demo.rs`)を合わせる。(FR5.2)
- [ ] 11.4 `code-summary.md`、`traceability.json`(`aidlc engine sensor-traceability`)、`source-manifest.json` を作る。

## 要件・規則の対応(追跡)

| 要件 | 規則 | 実装のステップ | テスト |
|------|------|-----------------|--------|
| FR4.1 | BR1.1、BR2.2 | 3、7、9 | 4.1、8.1、10.2 |
| FR4.2 | BR1.2、BR1.4、BR3.1 | 5、9 | 6.1、10.2、10.3 |
| FR4.3 | BR1.3 | 5 | 6.1 |
| FR4.4 | BR1.5 | 9 | 10.2、10.3 |
| FR4.5 | BR2.1、BR2.3、BR2.4 | 7 | 8.1、10.3 |
| FR5.1 | BR4.1 | 9 | 10.1 |
| FR5.2 | BR4.2 | 11 | 11.2、11.3 |

## 順序の注意

- 各層は、「実装 → その層のテスト」の順(test-after)。
- 範囲を絞った実行のコマンドは、`unit-test-instructions.md` に書く。プロジェクト全体の行カバレッジ 80% は、Build and Test が、全体のテストで確かめる。
