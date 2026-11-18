# Code Summary — u1-retry-config(リトライの設定と検証)

根拠: `code-generation-plan.md`(承認済み)、`unit-test-instructions.md`、設計 `functional-design/`(`rules.md`、`entities.md`、`functional-spec.md`)、`requirements.md`(FR1.1〜FR1.9、FR7.1、FR7.2)。

## 作成・変更したファイル

コミットは 4 件(`main` 上。push はしていない)。

| コミット | 内容 |
|---|---|
| d554ece | データの層: `RetryPolicy` と `JobDefinition::retry` の追加、既存の `JobDefinition { .. }` のリテラルに `retry: None` |
| c96cbdb | 誤りの種類 10 個と、英語・日本語の固定の文 |
| 445d478 | `[job.retry]` の読み取りと検証、core のテスト |
| 8ec98b5 | converter の往復変換と cli の `validate` のテスト |

### ソース

- `crates/core/src/types.rs`: `RetryPolicy { exit_codes, max, interval_secs, multiplier, max_interval_secs }` と、`JobDefinition::retry: Option<RetryPolicy>`(BR1.1)。`exit_codes` は昇順で重複なし(BR1.6)。値はすべて整数
- `crates/core/src/error.rs`: `DefinitionErrorKind` に 10 個の種類を追加
- `crates/core/src/definition/walk.rs`: `JOB_KEYS` に `retry` を追加。`[job.retry]` の読み取りと検査(BR1.2〜BR1.12)、`cross_check` で重なり(BR1.7)と上限の大小(BR1.13)、`build_retry` で `RetryPolicy` を組み立て
- `crates/core/src/definition/duration.rs`(新規): 間隔の文字列(`<整数><s|m|h>`)を秒に直す `parse_duration_secs`(BR1.10)
- `crates/core/src/definition/mod.rs`: 上記モジュールの宣言
- `crates/cli/src/report/messages.rs`: 10 個の誤りの種類の英語の固定の文(日本語は `Display` と共通)
- `JobDefinition { .. }` のリテラルへの `retry: None` の追加(機械的な変更): core の `flow/classify.rs`、`decide.rs`、`next_step.rs`、`resume.rs`、runtime の `runner.rs`、`state/consistency.rs`、`state/record.rs`、`tests/common/mod.rs`

追加した誤りの種類と `code`: `RetryNotTable`(`retryNotTable`)、`RetryMissingKey`(`retryMissingKey`)、`RetryExitCodesEmpty`(`retryExitCodesEmpty`)、`RetryExitCodeInvalid`(`retryExitCodeInvalid`)、`RetryExitCodesOverlap`(`retryExitCodesOverlap`)、`RetryMaxInvalid`(`retryMaxInvalid`)、`RetryDurationInvalid`(`retryDurationInvalid`)、`RetryBackoffIncomplete`(`retryBackoffIncomplete`)、`RetryMultiplierInvalid`(`retryMultiplierInvalid`)、`RetryMaxIntervalTooSmall`(`retryMaxIntervalTooSmall`)。許可されていないキー(BR1.4)は、既存の `UnknownKey` を使った。

### テスト(追加 51 件)

| ファイル | 件数 | 内容 |
|---|---|---|
| `crates/core/src/types.rs` | 3 | `RetryPolicy` の有無の作成と比較 |
| `crates/core/src/definition/duration.rs` | 4 | 単位、境界、書式違い、桁あふれ |
| `crates/core/tests/definition_retry.rs`(新規) | 36 | 受理(最小・最大の境界、重複、127、インライン表、`retry` なし)、10 種類の誤り、各境界の内側と外側、位置の順、入力の値が出ないこと |
| `crates/cli/tests/report_messages.rs` | 2 | 10 種類の英語と日本語の固定の文 |
| `crates/converter/tests/convert_roundtrip.rs` | 2 | TOML→YAML→TOML と YAML 起点の往復 |
| `crates/cli/tests/flow_validate.rs` | 4 | 正しい `retry` で成功、誤りで 64、複数の誤りが位置の順、未知のキーで 64 |

既存のテストの変更: `crates/core/tests/nfr_security.rs` の `cases()` に、10 種類の誤りの入力を足した(このテストは全種類の網羅を確かめるため、足さないと失敗する)。

## 主な実装上の判断

- `retry` の誤りは `Walker::job` の中で集める。誤りがあると、既存の仕組みどおり、定義が組み立てられない(BR1.14)
- 必須のキーがないときは、キーごとに `RetryMissingKey` を `[job.retry]` の行に出す(空の表なら 3 件)
- `multiplier` と `max_interval` は、書いてあるか(存在)で `RetryBackoffIncomplete` を判定する。値の正しさは、別の検査(BR1.10、BR1.12)で扱う
- `success_exit_codes` を書いて誤っていた場合は、成功の集合が決まらないため、重なりの検査(BR1.7)を省く。省略時の既定 `[0]` とは区別する
- 間隔は、半角の数字だけの整数と、`s`、`m`、`h` の 1 文字。符号、空白、小数、`_`、桁あふれは誤り
- converter のコードは変えていない。往復変換は、変更なしで通った(BR1.15)

## 計画からのずれ

- 既存のアサーションを 1 件、更新した。`crates/core/src/error.rs` のテスト `全種類に識別子と表示があり重複しない` が、種類の数を固定の `20` で確かめていたため、10 個足して `30` にした。計画では「種類の数を自動で拾う」と書いていたが、実際は固定の件数だった。これは、種類を足す変更に伴う、避けられない更新である

## 確認の結果(ローカル、macOS、rustc 1.100.0)

- `cargo test --workspace --locked`: 755 件が成功、失敗 0 件、無視 1 件(既存)。変更前は 704 件(今回の追加 51 件を足すと 755 件)
- `cargo fmt --all -- --check`: 問題なし
- `cargo clippy --workspace --all-targets --locked -- -D warnings`: 警告 0 件
- カバレッジ(`cargo llvm-cov ... --fail-under-lines 80`、開発担当の実測): 行 97.80%(`walk.rs` 98.75%、`duration.rs` と `error.rs` は 100%)。下限 80% を満たす
- `cargo tree`: `-sys` クレートと C のビルド依存はない。新しい外部クレートも追加していない
- 既存のアサーションの変更は、上の 1 件だけ

## 実行していない(書いたが実行していない)テストと検査

- Linux と Windows でのテスト: ローカルは macOS のため。CI で実行する
- CI での実行: この時点では、プッシュしていないため、CI は実行されていない

## この Unit の外

- 実行の流れ、ステートファイル、表示の種類、README は、触れていない(U2、U3 の担当)
