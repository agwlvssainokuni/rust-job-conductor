# 信頼性の設計: u1-pipeline-retry

出典: 信頼性の要件(`reliability-requirements.md` の NFR1.1、NFR1.2、NFR2.1〜NFR2.4、NFR3.1、NFR3.2)、技術の選択(`tech-stack-decisions.md`)、機能の仕様(`functional-spec.md` の 4、5、7)、契約(`contract-summary.md` の C3)。

サーキットブレーカ、フェイルオーバー、バックアップは当てはまらない。リトライの仕組みそのもの(待ち時間、上限、倍率)は今のまま使い、このユニットでは変えない。

## 1. 後方互換(NFR1.1、NFR1.2)

- 既定が無いとき、実効のリトライは「ジョブの表があればそれ、無ければ無し」で、今の組み立てと同じになる。組み立ての分かれ道を、既定の有無で分けずに 1 つの決め方(`functional-spec.md` の 3 の表)で書き、既定が無い列が今の動きと一致するようにする。
- 今のテストのうち、ジョブの `retry` が表でないときの理由の文を見ているものだけを、新しい文に合わせて直す。ほかは直さない。

## 2. テストの配置(NFR2.1〜NFR2.4)

test-after。core の検証の層を作ってから、その層のテストを書く。

| テストの場所 | 確かめること | 規則 |
|---|---|---|
| `crates/core/tests/definition_retry.rs` | 既定の読み取りと検査(正しい・各誤り)、`retry` の 3 つの形、実効のリトライの表(4 行 × 既定の有無)、既定の表を丸ごと使う(倍率を混ぜない)、重なり(ジョブごと・ジョブの行・false と自分の表は対象外・前の誤りがあれば調べない)、既定の上限と間隔の大小の誤りが 1 回 | BR1.1〜BR3.5 |
| `crates/core/tests/definition_valid.rs` | 既定を書かない今の定義が、今と同じ定義に組み立てられる | BR4.1 |
| `crates/core/tests/nfr_security.rs` | 新しい誤りの表示に値と ID が出ない(`security-design.md` の 2) | NFR6.1 |
| `crates/core/tests/definition_perf.rs` | 既定を持つ多数のジョブの定義の読み込み時間(`performance-design.md`) | NFR1.3 |
| `crates/cli/tests/flow_retry.rs` | 偽の `Runtime` で、既定によるリトライと `retry = false` の打ち消し | BR2.2 |
| `crates/cli/tests/binary_*`(実バイナリ) | `validate` が重なりで 64、既定の定義で `run` がリトライして成功する | NFR2.2 |
| `crates/cli/tests/unix_demo.rs`、`windows_demo.rs` | 更新したデモが通る | NFR3.2 |

- テストは公開の入口(定義の読み込み、cli の流れと実バイナリ)から書き、本番にテスト専用の関数や `pub` を足さない(NFR2.4)。
- 行カバレッジ 80% は CI の `coverage` で確かめる(NFR2.3)。3 つの OS と MSRV は CI の `test`・`build-check`・`msrv-test`(NFR3.1)。

## 3. 失敗のときの動き

- 定義の誤りは、すべて集めて行の順に返し、64 で止める(今のまま)。誤りのある定義を部分的に組み立てない。
