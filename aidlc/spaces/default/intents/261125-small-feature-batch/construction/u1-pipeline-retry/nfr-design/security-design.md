# セキュリティの設計: u1-pipeline-retry

出典: セキュリティの要件(`security-requirements.md` の NFR6.1、NFR6.2、NFR5.1、NFR4.1)、技術の選択(`tech-stack-decisions.md`)、機能の仕様(`functional-spec.md` の 6)、契約(`contract-summary.md` の C3)。

認証・認可・暗号は当てはまらない(単体の CLI、ネットワークなし)。設計の要は、入力の境界(定義ファイル)での検証と、誤りの表示に値を漏らさないことである。

## 1. 入力の検証(境界は定義の読み取り)

- `[pipeline.retry]` は、`[job.retry]` と同じ読み取りの手続き(書けるキーの許可リスト、型、範囲)を通す。手続きを 2 つ持たず、表 1 つを読む関数をジョブと既定で共有する。こうすれば、片方だけ検査が緩むことが無い(NFR6.1 の前提)。
- ジョブの `retry` は「無し・`false`・表」の 3 つのどれかだけを受け付ける。`true`、数値、文字列、配列は誤り(64)。
- 誤りが 1 つでもあれば定義を組み立てず、どのジョブも動かない(今のまま)。

## 2. 誤りの表示(NFR6.1、NFR6.3)

- 新しい理由の文は、すべて固定の文字列の定数にする。`format!` で値を差し込まない(差し込むのは今の規則どおりキー名だけで、このユニットの新しい文にはキー名も入れない)。

| 場面 | 種類(既存) | 理由の文(固定) | 位置 |
|---|---|---|---|
| `[pipeline]` の `retry` が表でない | RetryNotTable | `[pipeline.retry]` はテーブルで書きます | `retry` の値 |
| ジョブの `retry` が表でも `false` でもない | RetryNotTable | `retry` はテーブルか `false` で書きます | `retry` の値 |
| 既定とジョブの成功・警告の重なり | RetryExitCodesOverlap | 既定のリトライ(`[pipeline.retry]`)の `exit_codes` に、このジョブの成功または警告の終了コードと同じ値があります | ジョブの表(`[[job]]`)の始まり |

- **確かめ方**: `crates/core/tests/nfr_security.rs` の、各種類を起こす入力の一覧(`cases`)に、上の 3 つの場面を足す。値の位置(既定の `exit_codes` の値、ジョブの ID、`retry` に書いた値)に目印の文字列や目印の数を入れる。表示の文に目印が出ないことを、今の「すべての種類で表示の文に入力の値が出ない」テストで確かめる。

## 3. ステートと出力(NFR6.2)

- 既定は組み立て後の定義に残さないため、ステートを書く側(runtime)とリトライの通知(cli)は何も変わらない。今のステートの中身のテスト(`nfr_content.rs`)がそのまま通ることで確かめる。

## 4. 依存とコードの規則(NFR4.1、NFR5.1)

- 変更は `crates/core/src/definition/walk/`(`mod.rs`、`retry.rs`)と `types.rs` の文書に閉じる。新しい依存、`unsafe`、`cfg` を書かない。`nfr_source.rs` と CI の `build-check` が確かめる。
