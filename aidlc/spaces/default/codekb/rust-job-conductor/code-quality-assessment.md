# Code Quality Assessment

## 評価(good / fair / poor)

| 項目 | 評価 | 根拠 |
|------|------|------|
| 命名の明瞭さ | good | Rust 標準の命名、日本語の `//!` 文書、BR・NFR の ID を参照 |
| 関数の大きさ | fair | `walk.rs` の `Walker` は小さな関数に分かれる。`ctx.rs` の `report_execution`、`run.rs`・`pipeline.rs` の `run_*` はやや長い。runtime のステート部(`reader.rs` の `walk`、`walk_job`)は、読みやすい大きさ |
| エラー処理 | good | 失敗は戻り値、外部入力でパニックしない(NFR)。誤りを集めて返す。表示は固定文+エスケープ。ステートの失敗は kind ごとの固定の文とパス |
| テストの有無 | good | `#[test]` 約 567 件(前回の概数: core 115、runtime 154、converter 113、cli 185)。今回 runtime は src 内約 80 + tests 内約 80 を数えた。単体、統合、バイナリ実行、NFR(性能・安全・ソース規約) |
| 重複 | fair | `escape` 相当の関数が 4 クレートに重複(cli の文書が明記、表で揃えを検査) |
| 不要コード | good | TODO/FIXME/HACK は 0。`#[allow]` はテスト共通の `dead_code` と runtime の `unsafe_code` 1 か所 |

## CI/CD・品質ゲート

- lint(`cargo fmt --check`、`clippy -D warnings`)、build-check(Linux・Windows、MSRV 1.85)、test(3 OS)、msrv-test、coverage(行 80% 未満で失敗)、docs-check(README の必須語)、startup-bench(警告のみ)、audit(警告のみ)。
- release はタグで、4 ターゲットのバイナリを作る。(前回の記録。今回はワークフローを開いていない)

## ドキュメント

`README.md` は利用者向けで、定義の書式、command の書き方(文字列と配列)、終了コードの表、ステートの場所と動作、保証の範囲を網羅する。ステートファイルの**キーの書式は README に無い**(場所・消える条件・再開・65 と `reset` だけ)。`demo/README.md` は場面 1〜6 の表と、`notify` が配列の command で書かれていることを説明する。コード内の文書は充実(`runtime/src/lib.rs` の表、`reader.rs`・`writer.rs` の `//!`)。

## 今回の変更(state-job-key)に対する所見

1. **書き換え箇所は runtime に閉じる**: ステートの `jobs` のキーは、`reader.rs` の 2 か所(`TOP_KEYS`、`walk`)と `writer.rs` の 1 か所(`serialize`)、文面 `error.rs` の `ORDER_MISMATCH`、テストの TOML リテラルにある。いずれも文字列リテラルで、定数ではない(散在)。
2. **旧形式の扱い**: `format_version` は 1 のまま、旧形式(`[[jobs]]`)を読む互換は設けない方針。`reader.rs` は、版の確認のあとに `TOP_KEYS` で未知のキーを拒むため、旧ファイルは `corrupt`(65)になり、`conductor reset` で消す流れになる見込み(README の「読めない・壊れている」の記述と矛盾しない)。専用のエラー文は無い(コードは未実行で、挙動は読み取りによる推定)。
3. **テストの量**: ステートの TOML を直接書くテストは runtime 内の 4 ファイル(`state_read.rs`、`nfr_robust.rs`、`toml_depth.rs`、`state_write.rs`)と `reader.rs` の単体テスト。`nfr_robust.rs` は `[[jobs]` という壊れた形の入力(L74)を意図的に使う(キー名の変更のあとも、壊れた形として残してよい)。`toml_depth.rs` は `toml` クレートの深さの挙動の確認で、ステートのキーとは別の文脈でも `[[jobs]]` を使う(置換は任意)。
4. **README と demo**: README にキーの書式が無いため、書式を足すか否かは要件次第。L168 の `[[jobs]]` は定義ファイルの旧書式の注記で、ステートではない。demo はステートの中身に触れず、変更は不要の見込み。
5. **in-memory のフィールド名**: `PipelineState.job_states` を `jobs` などに改めるかは別の判断(core の型の変更になり、runtime の `record.rs`・テストと cli の `watch` に及ぶ)。ファイルのキーだけを変えれば、core・cli は影響を受けない。

## 技術的負債

- ステートのキー名と文面の散在(上記 1)と、`escape` の重複。ほかに目立つ負債は検出していない(ビルド・テストは実行していない)。
