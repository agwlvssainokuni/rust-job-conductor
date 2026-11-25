## Review

**Verdict:** READY
**Reviewer:** aidlc-architecture-reviewer-agent
**Date:** 2026-11-25T10:32:02Z
**Iteration:** 1
**Verdict Dictated:** no

### Findings

| ID | Severity | Location | Finding | Required action | Status |
|---|---|---|---|---|---|
| R-01 | Minor | aidlc/spaces/default/intents/261125-small-feature-batch/construction/u1-pipeline-retry/nfr-design/security-design.md > 4 依存とコードの規則、および traceability.json の NFR5.1 | 「`nfr_source.rs` が確かめる」とあるが、core の `tests/` に `nfr_source.rs` は無い(実在するのは `crates/runtime/tests/nfr_source.rs` と `crates/cli/tests/nfr_source.rs`)。core の `unsafe` と `cfg` の禁止を実際に守るのは `#![forbid(unsafe_code)]` と CI の `build-check` で、`cfg` を core に書かないことを見るテストの所在が曖昧 | どのクレートの `nfr_source.rs` が core の変更を見るのかを書き直す。core 側に見るテストが無いなら、`#![forbid(unsafe_code)]` と `build-check` だけを根拠にする、と明記する | New |
| R-02 | Minor | aidlc/spaces/default/intents/261125-small-feature-batch/construction/u1-pipeline-retry/nfr-design/security-design.md > 3 ステートと出力 | 「`nfr_content.rs` がそのまま通る」と書くが、このテストは `crates/runtime/tests/nfr_content.rs` にあり、先に決めた B6(開始と終了の時刻を許す)で直す対象でもある。既定を当てはめたジョブのステートが `[job.retry]` を書いたジョブと同じ形になることを確かめるテストは、どの層にも割り当てられていない(NFR6.2 の合否の基準 2 つ目) | NFR6.2 の「同じ形」を確かめるテストの置き場所(runtime か cli の実バイナリのテスト)を 1 つ決めて、`reliability-design.md` の表に足す | New |
| R-03 | Minor | aidlc/spaces/default/intents/261125-small-feature-batch/construction/u1-pipeline-retry/nfr-design/reliability-design.md > 2 テストの配置(`crates/cli/tests/binary_*`) | 実バイナリのテストの置き場が `binary_*` と総称のみで、リトライ用の `binary_retry*.rs` は今は存在しない(`binary_basic`・`binary_env` などだけ)。新しいファイルを作るのか、既存のファイルに足すのか決まっておらず、NFR2.2 の 2 つの確認(`validate` で 64、`run` でリトライ成功)の持ち主が不明 | 新規ファイル名か、足し先の既存ファイルを指定する | New |
| R-04 | Minor | aidlc/spaces/default/intents/261125-small-feature-batch/construction/u1-pipeline-retry/nfr-design/reliability-design.md > 1 後方互換 | NFR1.1 は「変わるのは理由の文だけ」と言うが、`retry = false` は今(v0.9.9)は RetryNotTable の誤りで、このユニットで通るようになる。機能の仕様(BR2.1、BR2.3)と合う意図した変更だが、互換の設計がこれを例外として挙げていない | 後方互換の節に、`retry = false` が誤りから有効になる点を意図した変更として 1 行足す(既定を書かない定義でも `retry = false` を書いたものだけが対象) | New |

### Validation Tool Results

| Tool | Result | Interpretation |
|---|---|---|
| ソースの突き合わせ(`crates/core/src/definition/walk/retry.rs`、`mod.rs`、`tests/nfr_security.rs`、`tests/definition_perf.rs`、`src/error.rs`) | 参照はおおむね実在。既存の `cases` は種類ごとに目印の文字列を入れる形で、設計の追加方法と合う。`cross_check` は今、ジョブごとに `check_retry` を呼ぶので、既定との重なりの検査を同じ場所に足せる。定義の大きさの上限は 1 MiB で、1 万ジョブの性能テストは収まる | 設計は実装できる。上の Minor は所在の書き違いと割り当て漏れ |

### Summary

NFR の各項目(NFR1.1〜NFR6.2)は設計に対応し、誤りの文の固定化、後方互換、検証の層ごとのテストの置き方は、既存のコードとテストの構造に照らして実装可能。ブロックする問題は無く、テストの所在の書き違い 4 点(いずれも Minor)を直せば足りる。
