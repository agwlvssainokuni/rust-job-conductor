# Code Quality Assessment

## テスト

| 項目 | 状況 |
|------|------|
| 構成 | 各 crate に、`src/` 内の単体テスト(`#[cfg(test)]`、約 200)と `tests/` の統合テスト(約 480) |
| core | `definition_valid`(9)、`definition_errors`(28)、`flow_sequences`(9)、`library_capability`(3)、`nfr_security`(5)、`nfr_performance`(1)。定義の誤りは種類ごと・複数同時を網羅 |
| runtime | `runner_basic`(14)、`runner_signal`(6)、`state_read`(14)、`state_write`(10)、`state_read_raw`(4)、`run_id_state`(15)、`flow_with_state`(20。cli の流れを小さな関数で再現して core + runtime を組み合わせる)、`nfr_*`、`toml_depth` |
| converter | `convert_ac`(8)、`convert_roundtrip`(12)、`read_resolve`(29)、`nfr_security`(13)。往復で同じ木になることが合否の基準 |
| cli | `flow_run`(51)、`flow_pipeline`(31)、`flow_run_id`(24)、`flow_aux`(33)、`flow_validate`(8)、`args_grammar`(35)、`report_messages`(23)、`binary_basic`(19)、`exit_codes`(7)、`signals_*`・`binary_signal`(本物のシグナル)、`unix_demo`・`windows_demo`(デモの再現)、`nfr_*`、`msrv_consistency`、`panic_hook`、`escape_table` |
| テストの方式 | 偽の `Runtime`・`Sink`・`Env`・`Signals`(`ScriptedRuntime` など)+ 本物のバイナリを起動するテスト(30 秒の上限つき)。テスト名は日本語 |
| 非機能のテスト | ソースの検索で、`unsafe`・ネットワーク・`cfg(os)` の位置・シェルの呼び出し・パニックしうる書き方がないことを検査(`nfr_source`)。性能(起動時間、CPU 時間)は `nfr_perf`/`nfr_performance` |
| カバレッジ | CI が `cargo llvm-cov --fail-under-lines 80`(テスト用バイナリは除外)。手元の数値は未取得(この調査はテストを実行しない) |
| 弱い点 | 待ち(スリープ)を差し替える仕組みがなく、時間に依存する新機能は実時間のテストになる。Windows の Ctrl+C は手動のチェックリスト(`WINDOWS_CTRL_C_CHECKLIST.md`) |

## リント・フォーマット・型

- `cargo fmt --check` と `cargo clippy ... -D warnings` が CI で必須。設定ファイルなし(既定値)。
- `#![forbid(unsafe_code)]`(cli)、`#![deny(unsafe_code)]`(runtime。許可は `platform/unix.rs` の 1 か所、SAFETY コメントつき)。core・converter は `unsafe` を使わない(`nfr_source` で検査)。
- 外部の入力(定義・ステート・引数・環境・シグナル・OS の結果)でパニックしない設計。パニックのフックは 70 で終わる固定の文。

## CI/CD

- `ci.yml`: lint、build-check(MSRV 1.100、ubuntu・windows、C 依存の検査)、test(3 OS)、msrv-test、coverage、docs-check、startup-bench。`release.yml`: タグで 3 OS の成果物を作り、下書きで公開。`audit.yml`: 脆弱性の検査(警告のみ)。
- チームの規則(`aidlc/active-memory/team.md`): 主な開発環境の OS ではローカルでテストし、ほかの OS は CI。マージ前に CI を通す。

## ドキュメント

- README.md(日本語、440 行): インストール、使い方、定義の書き方(表)、終了コードの表、at-least-once、シグナルの扱い、convert、ロケール。CI の `docs-check` が必須の見出しを検査する。
- ソースの doc コメントが詳細(要件 ID 付きの契約、呼び出しの順序、保証の範囲、守る性質)。`lib.rs` が各 crate の公開関数の表を持つ。
- 設計文書(`entities.md`、`logical-components` など)はソースから参照されているが、リポジトリには含まれない(`local-docs/` は `.gitignore` 済み)。

## 技術的負債・リスク

| # | 内容 | 場所 | 重さ |
|---|------|------|------|
| 1 | エスケープ関数が 4 か所に重複 | core `sanitize`、runtime、converter、cli `report/escape.rs` | 低(テストで揃えを保証) |
| 2 | `run.rs` と `pipeline.rs` の「実行 → 分類 → 表示 → 記録 → 書き込み」の重複 | `cli/src/{run,pipeline}.rs` | 中(再試行を足すと 2 か所を揃えて変える必要) |
| 3 | 時刻・待ちの差し替え口がない | `cli/src/deps.rs`、`runtime/src/runner.rs` | 中(再試行の間隔のテストに影響) |
| 4 | `JobDefinition` などのリテラルがテストに散在 | `runtime/tests/common/mod.rs::job`、`core/src/types.rs` のテスト、cli の定義文字列 | 低〜中(フィールドの追加で機械的な更新) |
| 5 | ステートの形が固定(未知のキーは `corrupt`、版 1) | `runtime/src/state/reader.rs` | 中(ステートに足す設計なら版の扱いの判断が要る) |
| 6 | 設計文書が外にある | `local-docs/` | 低 |
| 7 | 排他制御なし、時間の上限なし(設計上の非保証) | README「保証の範囲」 | 設計上 |

## 変更時の注意(不変条件)

- 定義・ステートとも、未知のキーを拒否する。キー表の更新を忘れると、新しい設定が `unknownKey` になる。
- エラーの表示に入力の値を入れない。固定の文 + エスケープしたキー名・ジョブID。
- シグナルは、子プロセスの回収前に、1 回だけ転送する。
- ジョブの結果は、`Durable` で書いてから終了コードを返す(書き込みに失敗したら 65 と「ジョブは実行されました」)。
- 再試行を設定しない既存の定義の挙動は変えない(後方互換)。
