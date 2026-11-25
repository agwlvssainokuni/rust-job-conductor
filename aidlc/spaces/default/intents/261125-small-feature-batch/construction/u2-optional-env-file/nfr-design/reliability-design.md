# 信頼性の設計: u2-optional-env-file

出典: 信頼性の要件(`reliability-requirements.md` の NFR1.1、NFR1.2、NFR2.1〜NFR2.4、NFR3.1、NFR3.2)、技術の選択(`tech-stack-decisions.md`)、機能の仕様(`functional-spec.md` の 3〜5、7)、契約(`contract-summary.md` の C1、C3)。

## 1. 後方互換(NFR1.1、NFR1.2)

- 文字列の項目は required = true の EnvFileRef になり、読み取りの結果は今と同じ。
- 型を `Vec<String>` から `Vec<EnvFileRef>` に変えるため、定義を組み立てているテストの補助(`test_support.rs` や各テストの組み立て)は書き換える。確かめる内容(期待する結果)は変えない。
- 理由の文を見ているテスト(`env_file` の型の誤り)だけは、新しい文に合わせて直す。

## 2. テストの配置(NFR2.1〜NFR2.4)

test-after。core → runtime → cli の順に、層を作ってからその層のテストを書く。

| テストの場所 | 確かめること | 規則 |
|---|---|---|
| `crates/core/tests/env_definition.rs` | 表の読み取り(path、required、省略時 true)、文字列と表の混在、1 つの表、各誤り | BR1.1〜BR1.3 |
| `crates/core/src/envfile/mod.rs` の `mod tests`(または `env_merge.rs`) | `env_file_refs` の重複除きと required の論理和、範囲(`run` と全体) | BR2.1 |
| `crates/core/tests/env_merge.rs` | 引き当てに無い項目は何も足さない(無くてもよいファイルが無いとき) | BR2.3 |
| `crates/core/tests/nfr_security.rs` | 新しい誤りの表示に値が出ない | NFR6.1 |
| `crates/runtime/tests/env_loader.rs` | 無い × required、ディレクトリ・大きすぎる・UTF-8 でない × required=false、別の文字列が同じパスに解決されるときの論理和 | BR2.2〜BR2.5 |
| `crates/cli/tests/flow_env.rs`、`flow_dry_run.rs` | 偽の `Runtime` で、無くてもよいファイルが無いときの `run`・`--dry-run` | BR3.1 |
| `crates/cli/tests/binary_env.rs` | 実バイナリで `run`・`pipeline`・`validate`・`--dry-run` が 0 で何も足さない。あるが UTF-8 でないときは 64 | NFR2.2、BR3.1 |
| `crates/cli/tests/unix_demo.rs`、`windows_demo.rs` | 更新したデモが今の出力で通る | NFR3.2 |

- 本番にテスト専用の関数や `pub` を足さない。偽の `Runtime` は引数の型の変更に合わせるだけ(NFR2.4)。
- 行カバレッジ 80% は確認のコマンドで確かめる(NFR2.3)。3 つの OS と MSRV は CI(NFR3.1)。

## 3. 失敗のときの動き

- 必ず要るファイルの誤りと、無くてもよいファイルの「無い以外」の誤りは、今と同じく全部集めて 64。誤りのある状態でジョブを動かさない。
