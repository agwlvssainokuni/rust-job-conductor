# Code Generation Plan — 最新の Rust に合わせたリファクタリング

## Summary

- Builds: MSRV を 1.85 から最新の安定版 1.100.0 に引き上げ、それに伴う clippy の指摘と、ネストした `if let` の let チェーン化(候補は 14 か所。条件を満たす箇所だけ)
- Touches: `Cargo.toml`、`README.md`、`.github/workflows/ci.yml`、`crates/{core,runtime,converter,cli}/src/` の該当 14 か所、`crates/cli/tests/msrv_consistency.rs`(新規)
- Tests: 既存の約 716 件(変更なし)に、MSRV の記載が 4 種類の場所で一致することを確かめる統合テスト 1 件を追加

## 背景と決定事項

- 「LTS」= Rust ツールチェーンの最新の安定版。[要件 Q1: A]
- 計画の作成時点(2026-11-17)で `rustup check` が示す最新の安定版は **1.100.0**(2026-11-09 リリース)。MSRV はこの版に固定する。(要件 FR1.1 のレビュー指摘 R-01 への対応: 版は計画の承認時に確定し、`code-summary.md` に記録する)
- 現状(1.100.0 で `cargo clippy --workspace --all-targets --locked -- -D warnings`)の警告は 0 件。`is_some_and` や `is_none_or` など、新しめの API はすでに使われている。
- MSRV が 1.85 のままだと、clippy は let チェーンを使う提案(`collapsible_if`)を出さない。`rust-version` を 1.88 以上にすると、ネストした `if let` に対する指摘が新たに出る見込み。この指摘が FR2.1 の主な対象になる。
- let チェーン(`if let ... && ...`)は edition 2024 かつ Rust 1.88 以降で使える。edition は 2024 のままなので、edition の変更は不要。(要件 C3)

## 書き換えの範囲(FR2.3 の確定)

書き換えるのは、ネストした `if let` の入れ子を let チェーン 1 つにまとめる箇所だけにする。評価の順序と結果は変わらない機械的な変更で、次の 14 か所を候補とする。

| crate | ファイル:行 | 内容 |
|---|---|---|
| core | `definition/walk.rs:256` | `if let Some(..) = &run_id_env { if !is_valid_env_name(..)` |
| core | `definition/walk.rs:497` | `if let Some(head) = pipeline { if let Some(jobs) = build_jobs(..)` |
| runtime | `state/record.rs:85` | `if let Some(job) = find_mut(..) { if let Some(classification) = ..` |
| runtime | `state/reader.rs:304` | `if let Some(resume) = &resume_job_id { if !job_order.contains(..)` |
| converter | `resolve.rs:352` | `if let Some(k) = key { if seen.insert(..)` |
| converter | `resolve.rs:515` | `if let Some(node) = enter.take() { if let Entered::Leaf(..) = ..` |
| converter | `resolve.rs:520` | `if let Some(v) = ret.take() { if self.stack.is_empty()` |
| converter | `scalar.rs:64` | `if let Some(rest) = text.strip_prefix("0x") { if ..` |
| converter | `scalar.rs:70` | `if let Some(rest) = text.strip_prefix("0o") { if ..` |
| converter | `yaml_read.rs:130` | `if let Some(top) = .. { .. if let Some(id) = id { if !top.seen.insert(..)` |
| converter | `yaml_read.rs:267` | `if let Some(open) = doc.stack.pop() { if open.anchor > 0` |
| cli | `ctx.rs:101` | `if let Some(job_id) = require_job { if definition.job_index(..).is_none()` |
| cli | `ctx.rs:127` | `if let Some(state) = &state { if let ConsistencyResult::Mismatch(..)` |
| cli | `ctx.rs:270` | `if let Some(on) = on { if let Some(transition) = ..` |

規則(レビュー指摘 R-02 への対応):

- 実際に書き換えるのは、上の表のうち、clippy が MSRV 1.100 で指摘した箇所、または入れ子が `else` を持たないことを確かめた箇所だけにする。`else` を持つ入れ子は、let チェーンにすると動作が変わるため、書き換えない。
- 書き換える箇所は、それを通る既存のテストが 1 件以上あることを確かめてから変更する。通るテストがない箇所は、先にテストを足すのではなく、書き換えない。除外した箇所は `code-summary.md` に理由とともに記録する。
- 上の表にない箇所は書き換えない。clippy が新しく出した指摘で、表にないものは、その修正が必要最小限であれば FR2.1 として直し、`code-summary.md` に記録する。

## 実装ステップ(test-after: 各層を実装してから、その層のテストを書いて実行する)

### 設定と文書の層

- [x] Step 1: 現状の土台を確かめる。テスト実行環境を確認し、変更前のツリーで `cargo test --workspace --locked` を実行して、すべて通ることを記録する(基準)。実行コマンドは `unit-test-instructions.md` に記載する。(NFR1、FR3.3)
- [x] Step 2: MSRV の記載を 1.100 に揃える。(FR1.1、FR1.2、FR1.3)
  - `Cargo.toml:7` の `rust-version = "1.85"` → `"1.100"`
  - `README.md:84` の「Rust(1.85 以降)」→「Rust(1.100 以降)」
  - `.github/workflows/ci.yml:57` と `:84` のコメントの 1.85 → 1.100
  - `.github/workflows/ci.yml:58` と `:91` の `dtolnay/rust-toolchain@1.85.0` → `@1.100.0`
  - `@stable` を使うジョブ、runner、アクションの版は変えない。`Cargo.lock` は変えない(FR1.4: 変更が必要になった場合だけ、最小限に更新する)。
- [x] Step 3: Step 2 のテストを書いて実行する。MSRV の記載が `Cargo.toml`、`README.md`、`ci.yml`(`@x.y.z` とコメント)で一致することを確かめる統合テスト `crates/cli/tests/msrv_consistency.rs` を追加し、`cargo test -p conductor-cli --test msrv_consistency --locked` を実行する。README の必須語(`docs-check`)は Step 2 で変えないため、影響しないことも確認する。(FR1.2、要件 R-01 の再発防止)

### コードの層(crate ごとに小さな単位で進める)

- [x] Step 4: `cargo clippy --workspace --all-targets --locked -- -D warnings` を 1.100 で実行し、MSRV を上げたことで新たに出る指摘の一覧を記録する。(FR2.1)
- [x] Step 5: conductor-core の書き換え。上の表の `definition/walk.rs` の 2 か所を、規則に従って let チェーンにする。(FR2.1、FR2.3)
- [x] Step 6: Step 5 のテストを書いて実行する。既存のテストを `cargo test -p conductor-core --locked` で実行する。書き換えた箇所を通るテストの名前を `code-summary.md` に記録する。(FR3.1、FR3.2)
- [x] Step 7: conductor-runtime の書き換え。`state/record.rs` と `state/reader.rs` の 2 か所を同じ規則で書き換える。(FR2.1、FR2.3)
- [x] Step 8: Step 7 のテストを書いて実行する。`cargo test -p conductor-runtime --locked`。(FR3.1、FR3.2)
- [x] Step 9: conductor-converter の書き換え。`resolve.rs`(3 か所)、`scalar.rs`(2 か所)、`yaml_read.rs`(2 か所)を同じ規則で書き換える。(FR2.1、FR2.3)
- [x] Step 10: Step 9 のテストを書いて実行する。`cargo test -p conductor-converter --locked`(fixtures を使う往復変換のテストを含む)。(FR3.2)
- [x] Step 11: conductor-cli の書き換え。`ctx.rs` の 3 か所を同じ規則で書き換える。(FR2.1、FR2.3)
- [x] Step 12: Step 11 のテストを書いて実行する。`cargo test -p conductor-cli --locked`(バイナリを実行する統合テストを含む)。(FR3.1)

### 仕上げ

- [x] Step 13: 全体の検証。次をすべて実行して通ることを確かめる。(NFR1、NFR3、NFR4)
  - `cargo fmt --all -- --check`
  - `cargo clippy --workspace --all-targets --locked -- -D warnings`(CI の `lint` と同じコマンド)
  - `cargo build --workspace --all-targets --locked`
  - `cargo test --workspace --locked`
  - `-sys` クレートと C のビルド依存が増えていないこと(`cargo tree` で `-sys` と `cc`、`cmake`、`pkg-config`、`bindgen` を確認)
- [x] Step 14: テストコードの差分を確認する。`git diff` で、`crates/*/tests/` のアサーションが変わっていないことを確かめる。変わっていた場合は理由を記録する。(FR3.3)
- [ ] Step 15: 文書とトレーサビリティ。`code-summary.md` に、確定した MSRV(1.100.0)とその日付、書き換えた箇所と除外した箇所(理由つき)、実行しなかった検査(下記)を記録し、`traceability.json` を作る。

## 実行できない・ローカルでは実行しない検査

- カバレッジ(`cargo llvm-cov --fail-under-lines 80`、NFR2)と、Linux・Windows 上のテストは、このローカルの環境(macOS)では実行しない。CI で実行する。`cargo-llvm-cov` が手元に入っている場合だけ、ローカルでも実行して値を記録する。
- CI のワークフロー(`ci.yml`)の変更は、プッシュ後の実行でしか確かめられない。`@1.100.0` のジョブ(`build-check`、`msrv-test`)の結果は、CI で確認する。

## 要件との対応

| 要件 | ステップ |
|---|---|
| FR1.1 | 2 |
| FR1.2 | 2、3 |
| FR1.3 | 2 |
| FR1.4 | 2(`Cargo.lock` は変えない) |
| FR2.1 | 4、5、7、9、11、13 |
| FR2.2 | 4(非推奨の警告が出ないこと。いまは 0 件) |
| FR2.3 | 5、7、9、11(範囲は上の表と規則で確定) |
| FR2.4 | 5〜12(crate ごとに小さな単位。単位ごとにテストを通し、コミットする) |
| FR3.1 | 6、8、12 |
| FR3.2 | 6、8、10 |
| FR3.3 | 1、14 |
| NFR1 | 1、13 |
| NFR2 | CI で確認(上記) |
| NFR3 | 13 |
| NFR4 | 2、13、CI の `build-check` と `msrv-test` |

## Testing Contract

```json
{
  "version": 1,
  "methodology": "test-after",
  "source": "team",
  "ordering": "各テスト可能な層を実装してから、その層のテストを書いて実行する。",
  "scope": "refactor",
  "test_strategy": "minimal",
  "project_type": "brownfield",
  "applicable_notes": [
    {
      "layer": "org",
      "text": "We treat tests as a first-class deliverable in every Bolt. The specific\nmethodology (TDD, BDD, ATDD, or classic test-after) is affirmed at\npractices-discovery and recorded in `team.md` under this heading with explicit\n`Methodology` and `Ordering` fields; Code Generation resolves those fields\nindependently from coverage, tooling, and scope notes.\n\nWhen no posture has been affirmed, our default per scope is:\n- **Methodology**: test-after\n- **Ordering**: implement each applicable testable layer, then write and run\n  that layer's tests.\n- `mvp`, `enterprise`, `feature`, `infra` add an 80% line-coverage\n  floor and CI execution before merge.\n- `bugfix`, `security-patch` add a targeted regression for the specific\n  bug/vulnerability and require the existing suite to remain green.\n- `express` uses the Minimal strategy: requirement-driven unit tests (one per\n  requirement, with a happy-path floor per component); existing tests remain\n  green.\n- `poc`, `refactor`, `workshop`, `classic` add no extra new-test floor and require the\n  existing suite to remain green.\n\nThe active `Test Strategy` still applies in every scope and determines test\nvolume/types. Scope floors are additive; they never reduce or replace the\nselected strategy.\n\nBuild and Test verifies defined coverage floors and affirmed quality targets;\nthey may not be weakened to make a step pass.\n\nAffirm a stricter posture in `team.md` if the team commits to one."
    },
    {
      "layer": "team",
      "text": "- **Methodology**: test-after\n- **Methodology evidence**: インタビュー Q3 で、チームは test-after(実装してからテストを書く)を選んだ。\n- **Ordering**: 各テスト可能な層を実装してから、その層のテストを書いて実行する。\n- `cargo test` による単体テストと、バイナリを実行する統合テストを含める。\n- 行カバレッジ 80% を下限とする(インタビュー Q4: A)。\n- 主な開発環境の OS では、ローカルでテストを実行する。それ以外の OS(Linux・macOS・Windows のうち該当するもの。制約 C3)のテストは、GitHub Actions の CI で実行する。マージ前に CI を通す。"
    }
  ],
  "obligations": {
    "strategy": "minimal",
    "strategy_volume": [
      "One verifiable test per requirement at the narrowest effective level.",
      "At least one happy-path unit test per component.",
      "Unit tests are the default; a bugfix/security scope floor may require an integration or E2E regression when that is the narrowest level that reproduces the defect."
    ],
    "scope_floor": [
      "Keep the existing test suite green.",
      "This scope adds no extra new-test floor beyond the selected test strategy."
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
  "input_sha256": "sha256:d900e03360550bf47f051236f3df6a05c9187f6a8e013be65811af5ef32d1e8d",
  "contract_sha256": "sha256:af4c0e3cdf95a14f6563a4adc13e41addbf92e80bf61c33c217e0245d725ffa0"
}
```
