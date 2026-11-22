# 機能の仕様(Functional Spec): 動きを変えない整理のリファクタリング

意図: `ideation/intent-capture/intent-statement.md`(Q1: A、Q2: A、Q3: A)。**変える場所の正確な一覧(ファイルと行、目標の形、テストへの影響)は `inventory.md`**(この記録。HEAD 38a8c97 で作った)。この文書は、方針と、決めたことと、確かめ方を書く。

## 1. 方針

- 利用者から見える動き(標準出力と標準エラーの文、終了コード、ステートファイル、`--dry-run` の出力)を変えない。
- 4 つの段を順に進める。段 1(cli)、段 2(型)、段 3(core)、段 4(テストと文書)。各段は小さなコミットの積み重ねで、どのコミットでもビルドとテストが通る。
- 範囲の外: 引数の読み取りの表(`args.rs`)、`Runtime` トレイトの向こうの分割(`runner.rs`)、converter と core の重複(converter は core に依存しない)、5 つの仕様書の中身。

## 2. 段ごとの変更(WF1。詳細は `inventory.md`)

| 段 | 項目 | 要点 |
|----|------|------|
| 1 | 1a | run ID の決定(`definition_run_id`)と、ステートの読み取りと照合(`read_checked_state`、`StateRead::{Live, Raw}`)を `ctx.rs` の 2 つの関数に。prepare、graph、watch がそれを使う。**prepare と graph の、検査の順序を変えない**(別々の関数のまま呼ぶ) |
| 1 | 1b | 定義の検査が分割したコマンドを返し(`CheckedDefinition.commands`)、`Checked` と `Session` が持つ。dry_run と `report_execution` は使い回す。通らない枝を消す。誤りは全件を集めて表示する(今のとおり) |
| 1 | 1c | `check_definition_and_env` を `ctx.rs` へ |
| 1 | 1d | run と pipeline の入口の共通化。prepare を部品に分け、run は prepare からジョブの位置を受け取る(通らない else が消える)。**prepare の順序「シグナル → 定義 → command → ジョブ → `--assume` → run ID → `.env` → ステート」を保つ** |
| 2 | 2a | 分類と遷移を、文字列でなく `Option<Classification>` と `TransitionOn` で `DisplayEvent` に渡す。文字列にするのは表示の側だけ。分類 → 遷移の写しを core の 1 か所に |
| 2 | 2b | `JobStatus` と `SkipReason` の文字列の対応(`as_str`、`parse`)を core の型に。writer、reader、watch が使う |
| 2 | 2c | `check_consistency` と型を core へ。runtime は再エクスポート。**runtime の単体テスト 4 件は、その場に 1 文字も変えずに残す** |
| 2 | 2d | ステートの上限つきの読み取りを `read_bounded` の 1 か所に(今の誤りの写像を保つ) |
| 3 | 3a | `walk.rs` を `walk/{mod,retry,env}.rs` に分ける(移すだけ) |
| 3 | 3b〜3e | 既定の成功コード、終了コードの読み取り、入れ子の上限と表示の上限、空の次の動き、の重複をなくす(文は 1 文字も変えない) |
| 3 | 3f | `JobExecution::exited(job, code)` を足し、本番(simulate)で使う |
| 4 | 4a | graph のテストを形ごとのファイルに組み直し、補助を `graph_common` に。入力と期待の全文が同じ 5 組と、同じ性能のテスト 1 件をまとめる |
| 4 | 4b | converter のテスト名(117 件)を日本語に。`env_probe_child` の起動の文字列と、language-agnostic の参照(24 個)も合わせる |
| 4 | 4c | テストの偽物の `wait` を共通の関数に。`run_job` の土台に `JobExecution::exited` |
| 4 | 4d | README の終了コード 68 の行を、実装に合わせる(代表を挙げ、詳しくは本文の節を指す) |
| 4 | 4e | 5 つの仕様書の `00-README.md` の見出しの直後に、凍結の明記 |

段 1〜3 のあと、`docs/spec/rust` の内部の名前(02・04・05・06・07・10・12 章)を新しい置き場所と名前に合わせる。段 4 のあと、テストのファイル名と件数を参照する仕様書(`docs/spec/rust/11`、`05:804`、`docs/spec/language-agnostic/11`)を直す。

## 3. 段 1〜3 で変えざるを得ないテスト(ユーザーの承認が要る)

一覧を作った結果、**変える必要があるのは 1 か所だけ**:

- `crates/cli/tests/report_messages.rs` の `with_all_events`(`:84〜`。3 つのテストに波及)と `v_の文言`(`:524-566`)。
- 理由: 段 2a で、公開の型 `DisplayEvent` のフィールドを文字列から型(`Option<Classification>`、`TransitionOn`)に変える。このテストは、そのフィールドに文字列の値を入れて組み立てている。
- 変えるのは、組み立てに入れる値だけ(`"warning"` → `Some(Classification::Warning)`、`"interrupted"` → `None`、`on: "warning"` → `TransitionOn::Warning` など)と import。**期待する出力の文字列は 1 文字も変えない**。
- 代案(テストを変えない): フィールドを文字列のまま残し、core の `as_str` で作る。ただし表示の側に文字列の比較が残り、2a の目的(型で渡す)を満たさない。
- **ユーザーは、このテストの変更を認めて、設計を承認した**(2026-11-22)。

ほかの段 1〜3 の変更は、非公開のモジュールの中か、再エクスポートで道を保つので、既存のテストは 1 行も変えずに通る(`inventory.md` の段ごとの「影響するテスト」)。

## 4. 確かめ方(Q3: A)

- 段の前と後で、`cargo test --locked --workspace` の件数を記録する(基準: 1,123 通過、1 無視)。段 1〜3 は件数が変わらない(新しいテストを足すときは、その分だけ)。
- 段 1〜3 のコミットでは、テストのファイルを変えない(§3 の 1 か所を除く)。`git diff` で確かめる。
- 「1 か所にまとまる」「重複がない」は、`inventory.md` の「今」の場所に、実装の後で重複が残っていないことを確かめる。
- 段 4 は、まとめたテスト(§2 の 4a の 6 件)の一覧を記録する。ほかのテストは、確かめている入力と期待を落とさずに移す(件数は 64 から 58 へ。converter は改名だけで件数は変わらない)。
- 合格の条件: `cargo fmt --check`、`clippy -D warnings`、全テスト、行カバレッジ 80% 以上、MSRV 1.100。新しい依存、`unsafe`、`cfg` を足さない。

## 4a. 設計の確認の指摘の反映(すべて軽いもの)

1. 段 4 のあとに直す仕様書に、`docs/spec/language-agnostic/11-quality-and-build.md` の AC-476c、AC-476d、AC-476e、AC-475 も含める(graph のテストのファイル名を参照している)。
2. 4a の graph のテストは `crates/core/tests/` にある(cli ではない)。
3. 2c の runtime の `consistency.rs` は、`mod tests` が `super::*` で使う名前(`check_consistency`、`ConsistencyResult`、`MismatchKind`、テストが組み立てに使う `PipelineDefinition`、`PipelineState` など、実際にテストが使うものすべて)を、その薄いファイルで use する。実装の最初に、テストが使う名前を洗い出す。
4. 3d: `crate::raw::MAX_DEPTH` が定数として const の assert に使えることを確かめる。converter の文「(上限 32)」と core の文「(上限は 32 段)」は別の文で、どちらも変えない。
5. 4d の README の変更は、`docs_*.rs` と CI の `docs-check` で確かめる。
6. 2d: `inspect_run_state` は、ファイルが無いとき(`Ok(None)`)も `Broken` にする(今のとおり)。

## 5. 成功の指標との対応(traceability.json)

| ID | 成功の指標(`intent-statement.md`) | 対応する規則 |
|----|------------------------------------|--------------|
| SM1 | 段 1(cli の整理) | BR1.1〜BR1.4 |
| SM2 | 段 2(型の整理) | BR2.1〜BR2.4 |
| SM3 | 段 3(core の整理) | BR3.1〜BR3.3 |
| SM4 | 段 4(テストと文書) | BR4.1〜BR4.5 |
| SM5 | 動きを変えない、テストの扱い | BR5.1、BR5.2 |
| SM6 | 合格の条件 | BR5.3 |
