## Review

**Verdict:** READY
**Reviewer:** aidlc-architecture-reviewer-agent
**Date:** 2026-11-14T20:26:12Z
**Iteration:** 1
**Verdict Dictated:** no

### Findings

| ID | Severity | Location | Finding | Required action | Status |
|---|---|---|---|---|---|
| R-01 | Major | aidlc/spaces/default/intents/261114-job-conductor-cli/construction/u1-core/nfr-requirements/security-requirements.md > NFR4.7 | NFR4.7 は「壊れた TOML」を `DefinitionError` として返すと定めている。しかし functional-design の entities.md にある DefinitionError の kind 一覧には、TOML の文法エラー用の種類がない。NFR4.1 は型の誤りを `typeMismatch` と定めており、文法エラーをどの種類で返すかは、どこにも書かれていない。実装者は種類を推測するしかなく、判定の基準(`DefinitionError` が返る)でも種類まで確かめられない。 | 文法エラーの扱いを決めて書く。既存の kind(例: `typeMismatch`)に含める場合はそう明記し、NFR4.7 の判定の基準にも種類を書く。新しい kind が必要な場合は、functional-design へ戻す項目として明記する。 | New |
| R-02 | Major | aidlc/spaces/default/intents/261114-job-conductor-cli/construction/u1-core/nfr-requirements/tech-stack-decisions.md > 選択 の表(`serde`+`toml`、`thiserror`、1 MiB、edition 2024) | 重要な選択の行に、検討した代替案とトレードオフが書かれていない。代替案(手書きの読み取り、標準ライブラリだけのエラー型、edition 2021、上限なし)は、質問ファイルの選択肢にしか残っておらず、成果物からは辿れない。inception の指針(代替案を2つ以上検討する)を、この成果物では満たしていない。 | 各行(または表の後ろの節)に、却下した代替案と、却下の理由(依存の増減、実装の手間、保守性など)を書く。 | New |
| R-03 | Minor | aidlc/spaces/default/intents/261114-job-conductor-cli/construction/u1-core/nfr-requirements/security-requirements.md > NFR4.6 | `toml` クレートの標準のエラー表示は、問題のある行の抜粋を含むことがあり、serde の型エラーの文言にも、書かれた値が入ることがある。定義のコマンド文字列に認証情報が書かれていると、「定義の内容の全文や値を含めない」と衝突しうる。判定の基準(ファイル名・行・種類・理由に限る)だけでは、理由の文字列に値が混じることを検出できない。 | 理由の文字列はライブラリの表示をそのまま使わず、位置(スパン)と種類から組み立てる、と方針を書く。検証用のテストに、値を含む定義でエラーの文言に値が出ないことの確認を加える。 | New |
| R-04 | Minor | aidlc/spaces/default/intents/261114-job-conductor-cli/construction/u1-core/nfr-requirements/security-requirements.md > NFR4.4 | 「読み取りの前にファイルの大きさを確認する」方式は、メタデータの大きさが 0 のファイル(名前付きパイプルなど)や、確認と読み取りの間の変更を防げない。また、上限超過を `unreadable` にまとめることは、rules の BR1.9(ファイルが無い、または読めない)の意味を広げるため、functional-design 側の記述との整合の確認がない。 | 大きさの確認に加え、読み取り自体も上限+1 バイトで打ち切る、と書く。上限超過を `unreadable` の理由の一つとして BR1.9 に追記する項目を、後続のステージへの引き継ぎとして明記する。 | New |
| R-05 | Minor | aidlc/spaces/default/intents/261114-job-conductor-cli/construction/u1-core/nfr-requirements/security-requirements.md > NFR4.7 と Assumptions | 「巨大な入れ子」の上限を Code Generation に先送りしているため、NFR4.7 の判定の基準が測定できる形になっていない。深い再帰によるスタックの枯渇は、パニックではなく異常終了になり、`DefinitionError` に変換できない。1 MiB の入力でも、十分に深い入れ子は作れる。 | テストに使う入力の具体(例: 入れ子の深さ N の配列、長い文字列)を定める。使うライブラリの再帰の上限の有無を、Code Generation の最初の確認項目として tech-stack-decisions の Assumptions に加える。 | New |
| R-06 | Minor | aidlc/spaces/default/intents/261114-job-conductor-cli/construction/u1-core/nfr-requirements/tech-stack-decisions.md > NFR1.1、NFR2.1、NFR3.1 | これらの行には、判定の基準の列がなく、合格・不合格を判断できない(NFR3.1 は、3つの OS の CI でこの Unit のテストが通ること、といった基準が書かれていない)。 | 各行に、測定できる基準を付ける。例: NFR2.1 は依存クレートがすべて純粋な Rust であること、NFR3.1 は3つの OS の CI で `cargo test` が合格すること。 | New |
| R-07 | Minor | aidlc/spaces/default/intents/261114-job-conductor-cli/construction/u1-core/nfr-requirements/tech-stack-decisions.md > 依存するライブラリの方針、Assumptions | 版の固定の方針があいまいである(`Cargo.lock` で固定するのか、`Cargo.toml` で厳密に指定するのか)。依存のライセンスの確認と脆弱性の検査(`cargo-deny`、`cargo audit` など)にも触れていない。バージョンの数字を Code Generation に任せることは問題ないが、方針は決められる。 | 固定の方法(`Cargo.lock` をコミットする、など)を書く。ライセンスと脆弱性の検査は、採用するか、未決の項目として担当を付けて Assumptions に書く。 | New |
| R-08 | Minor | aidlc/spaces/default/intents/261114-job-conductor-cli/construction/u1-core/nfr-requirements/tech-stack-decisions.md > 定義ファイルの読み取り、Assumptions | BR1.7 は、版の確認を他の検査より先に行うと定めている。`deny_unknown_fields` を付けた型への読み取りを一度に行うと、版より前に未知のキーや型の誤りが報告されうる。そのため、先に汎用の値(`toml::Value`)として読んで版を確認し、そのあとで型へ変換する二段構えが必要になる。この点が書かれておらず、行の位置の取得方法も確認項目に含まれていない(行は、スパンから自前で計算する必要がある)。 | 二段階の読み取りで版を先に確認することを、方針に書く。行の位置は、エラーのスパンと元の文字列から計算することを、Code Generation の最初の確認項目に加える。 | New |

### Validation Tool Results

| Tool | Result | Interpretation |
|---|---|---|
| (stage defines no validation tools; manual check of traceability.json) | NFR1〜NFR5 は OK で、対象の行(NFR1.1、NFR2.1、NFR3.1、NFR4.1〜NFR4.7、NFR5.1)が成果物に存在する。NFR6 は理由付きの N/A | 被覆は完全である。NFR4.1〜NFR4.7 は security-requirements.md に、NFR1.1、NFR2.1、NFR3.1、NFR5.1 は tech-stack-decisions.md に実在する |

### Summary

セキュリティ要件は、library が利用者の書く TOML を読むという実態に合っており、functional-design の規則(BR1.5〜BR1.9、BR1.13)や、回答済みの質問(serde+toml、edition 2024、thiserror、1 MiB)とも、大きくは矛盾しない。主な懸念は、TOML の文法エラーに対応する DefinitionError の種類が決まっていないこと、および技術選択に代替案が書かれていないことの2点であり、いずれも Major が2件に収まるため、READY とする。Minor は、承認の場で確認する価値があるものとして挙げた。
