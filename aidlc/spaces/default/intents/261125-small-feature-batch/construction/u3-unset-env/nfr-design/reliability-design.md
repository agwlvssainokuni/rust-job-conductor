# 信頼性の設計: u3-unset-env

出典: 信頼性の要件(`reliability-requirements.md` の NFR1.1、NFR1.2、NFR2.1〜NFR2.3、NFR3.1、NFR3.2)、技術の選択(`tech-stack-decisions.md`)、機能の仕様(`functional-spec.md` の 2、3、5)、契約(`contract-summary.md` の C1)。

## 1. `.env` との重なりの検査の置き場所と形

機能の設計のレビューの指摘(読み取りの結果に行番号もファイル名も残らない)に応えて決める。

- core の `EnvEntry` に、その項目の行番号 `line` を足す(解析の時点で分かっている)。`EnvFileContent` に、表示用のファイル名 `file` を足す(`parse_env_file` に渡している名前)。どちらも値ではないので秘密を増やさない。
- 重なりの検査は core の新しい関数 `check_unset_conflicts(def, job_scope, files) -> Vec<EnvFileError>` に置き、cli の `.env` を読む入口(`ctx.rs` の `load_env_files` の後)で呼ぶ。範囲は今の読み取りの範囲と同じ(`run` はそのジョブ、ほかは全ジョブ)。
- 重複を除く: 同じ `.env` を複数のジョブが読んでも、誤りは「ファイル名と行」で 1 つにまとめる(要件のレビューの指摘)。並びはファイルの読み取り順、行の順。
- 確かめ方: core の単体テスト(重なり、範囲、重複を除く、行とファイル)。cli の流れのテストで `run` と `pipeline` の範囲の違い。

## 2. 後方互換(NFR1.1、NFR1.2)

- 統合の出力の足す変数(`entries`)は今のまま。`removals` を足すだけ。`EnvEntry` と `EnvFileContent` に項目が増えるため、組み立てているテストの補助だけを直す。
- `unset_env` が空なら、`env_remove` を呼ばず、今の起動と同じ。

## 3. テストの配置(NFR2.1〜NFR2.3、NFR3.1、NFR3.2)

| テストの場所 | 確かめること |
|---|---|
| `crates/core/tests/env_definition.rs` | `unset_env` の型・名前の規則・予約名、`env` との重なり(大文字小文字を区別しない)、重複は誤りにしない |
| `crates/core/tests/env_merge.rs` | `removals` の並びとまとめ、`entries` が変わらない、`check_unset_conflicts` |
| `crates/core/tests/env_file_parse.rs` | `EnvEntry.line` |
| `crates/core/tests/nfr_security.rs` | 新しい誤りの表示に値が出ない |
| `crates/runtime/tests/runner_basic.rs` | 親に置いた変数が消える、無い名前は何もしない |
| `crates/cli/tests/flow_env.rs`、`flow_dry_run.rs` | 偽の `Runtime` に消す名前が渡る、`--dry-run` の表示 |
| `crates/cli/tests/binary_env.rs` | 実バイナリで消える、64、`--dry-run` |
| `unix_demo.rs`、`windows_demo.rs` | デモの場面 |

- 本番にテスト専用の項目を足さない。行カバレッジ 80% は確認のコマンド、3 つの OS と MSRV は CI。
