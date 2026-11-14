## Review

**Verdict:** READY
**Reviewer:** aidlc-architecture-reviewer-agent
**Date:** 2026-11-14T20:34:35Z
**Iteration:** 2
**Verdict Dictated:** no

### Findings

| ID | Severity | Location | Finding | Required action | Status |
|---|---|---|---|---|---|
| R-01 | Major | aidlc/spaces/default/intents/261114-job-conductor-cli/construction/u1-core/nfr-design/security-design.md > 設計の要点 1 の手順 3〜4、および 3(行番号) | 手順 3 で作った汎用の木(toml の Value)は、バイト位置の情報を持たない。手順 4 は「同じ木から」型付きの構造に変換するとあるため、`unknownKey` / `typeMismatch` に NFR4.1・NFR4.2・BR1.6 が求める行を付けられない。「エラーが指す位置から改行を数える」という行番号の算出は、構文エラーにしか成り立たない。手順 5 の構造の検査(BR1.1〜BR1.4 など、遷移やジョブごとの誤り)にも、行を得る手段が書かれていない。 | 型付けの段階で位置を得る方法を決めて書く。例: 版の確認の後、同じ文字列から型付きの構造を読み直す。または位置つきの値を使う。構造の検査の誤りに行を付ける方法(各ジョブ・遷移の位置の保持)も書く。 | Resolved (reviewer) |
| R-02 | Major | aidlc/spaces/default/intents/261114-job-conductor-cli/construction/u1-core/nfr-design/security-design.md > 設計の要点 1 の手順 4、設計の要点 3、D1 | 型付きの構造への変換は、serde では最初の1件の失敗で止まる。それなのに、手順 4 の `unknownKey` / `typeMismatch` と、手順 5 の「すべての誤りをまとめて返す」(functional-spec W1 の手順 8)の関係が書かれていない。さらに、serde の失敗を `unknownKey` / `typeMismatch` / 必須の欠落に振り分けるには、ライブラリの標準のエラー文の解析が必要になりかねない。設計の要点 3 は、標準のエラー文をそのまま使わないと定めている。この点は、設計の中で矛盾しており、実現の見通しがない。 | 種類の振り分け方法を決めて書く。例: Value の木を手で走査して許可するキーと突き合わせる、またはエラーの種類を表す構造化された情報を使う。変換の段階で複数の誤りを返すのか、1件で止めるのかも明記する。止める場合は、W1 の手順 8 との差を明記する。 | Resolved (reviewer) |
| R-03 | Major | aidlc/spaces/default/intents/261114-job-conductor-cli/construction/u1-core/nfr-design/security-design.md > 設計の要点 4(入れ子の深さ)、Assumptions & Open Questions | 「読み取る前の簡易な走査、または読み取りの設定で拒否する」は、2案が未決のまま並んでいる。深さの数え方(テーブル、配列、インラインテーブルのどれを数えるか)も定まっていない。文字列やコメントを含む TOML の生の文字列に対する簡易な走査は、誤検知しやすい。`toml` クレートが深さの上限を設定する手段を持つ根拠も示されていない。さらに、型付きの構造への変換も再帰で走る。このため、NFR4.7 の「巨大な入れ子でパニックしない」について、32 段の拒否が単体テストで合否を判定できる形になっていない。Q1 の B(上限を置かない)を退けた理由も、設計の判断には記録されていない。 | 実装する方法を1つに決める(例: 読み取りの前に、文字列を走査する関数を置く、またはライブラリの上限を明記する)。深さの数え方と、合否を確かめるテストの入力(32 段は通り、33 段は `syntax` になる)を書く。D の判断の一つとして、代替案と理由を残す。 | Resolved (reviewer) |
| R-04 | Major | aidlc/spaces/default/intents/261114-job-conductor-cli/construction/u1-core/nfr-design/security-design.md > 追加する種類、および設計の要点 1 の手順 1 | `syntax` と `tooLarge` の追加は、entities.md の `kind` の拡張としてしか Code Generation への引き継ぎが書かれていない。nfr-requirements の security-requirements.md の NFR4.4 は、上限超過を `unreadable` と明記しており、これと矛盾したままである。rules.md の BR1.9 と BR1.12、functional-spec.md の W1(手順 2 に上限超過がなく、構文エラーの手順もなく、版の確認が手順 3)も、新しい順序と種類を反映していない。DefinitionError の `kind` は、u2-runtime と u3-cli が、終了コード 64 への割り当てや表示の出し分けで使う共有の型である。引き継ぎの範囲が狭く、実装者が旧版を正とする恐れがある。また、非 UTF-8 の入力をどの種類にするかも決まっていない(手順 1 の「文字列」への変換)。 | 反映が必要な箇所を一覧にして書く(entities.md の `kind`、rules.md の BR1.9 と BR1.12、functional-spec.md の W1 の手順、security-requirements.md の NFR4.4 の判定の基準)。非 UTF-8 の入力の種類(`unreadable` か `syntax`)を決めて書く。 | Resolved (reviewer) |
| R-05 | Minor | aidlc/spaces/default/intents/261114-job-conductor-cli/construction/u1-core/nfr-design/traceability.json > upstream_ids / coverage | upstream_ids は NFR4.1〜NFR4.7 だけで、設計が参照する NFR1.1・NFR2.1・NFR3.1・NFR5.1 が無い。引き継ぎ(6)の、NFR1.1・NFR2.1・NFR3.1 に測定できる基準を足す件は、設計のどこにも扱われていない。 | NFR1.1・NFR2.1・NFR3.1・NFR5.1 の対応を追加する。または、対象外とする理由を書く。測定基準(例: 3つの OS の CI での成功、依存が Rust のみであることの確認)をどこで足すかを決めて書く。 | Unresolved |
| R-06 | Minor | aidlc/spaces/default/intents/261114-job-conductor-cli/construction/u1-core/nfr-design/security-design.md > 設計の判断(D1〜D4) | 二段階の読み取り(Value 経由)は、この設計の最も影響の大きい判断だが、判断として記録されておらず、代替案が無い。例: 版だけを部分的に読む案、1回の型付きの読み取りで版を先に確認する案。D4 の代替案は、実質的な比較になっていない。 | 二段階の読み取りを判断(D5)として足し、2案以上の代替案とトレードオフを書く。 | Resolved (reviewer) |
| R-07 | Minor | aidlc/spaces/default/intents/261114-job-conductor-cli/construction/u1-core/nfr-design/security-design.md > 設計の要点 3(Describe) | 固定の文だけから表示を作ると、NFR4.2 が求める「キー名」や、型の誤りで期待する型を、どこから取るかが示されていない。キー名は入力由来であり、長さの上限や制御文字の扱いも書かれていない。 | 表示に含める入力由来の項目(キー名、パス)と、その長さの上限・エスケープの方針を書く。 | Resolved (reviewer) |
| R-08 | Minor | aidlc/spaces/default/intents/261114-job-conductor-cli/construction/u1-core/nfr-design/security-design.md > 設計の要点 2(行番号)と流れ 5 | 誤りの行は「その要素の範囲の先頭」から求めるとあるが、対応する要素がない誤りの行の決め方が書かれていない。例: 必須のキーが無い(`typeMismatch`)、ジョブが0(`noJobs`)、パイプライン名の不一致。また、あるジョブの型が不正なとき、そのジョブへの遷移の検査(`unknownTarget` など)をどう扱うか(後続の誤りを出すか、抑えるか)も決まっていない。開発者が推測することになる。 | 要素のない誤りは、親の要素の範囲(なければファイルの先頭)の行を使う、と一文で決める。型の誤りで検査できなくなった部分の、後続の検査の扱い(省略する、など)も一文で決める。 | New |
| R-09 | Minor | aidlc/spaces/default/intents/261114-job-conductor-cli/construction/u1-core/nfr-design/security-design.md > D1(Q1: A と読み取りの方式) | D1 は、Q1 の選択「serde と toml」を、`toml` の仲間を使うことで「満たされる」としているが、実際の方式は serde を使わない(D0)。選択の文言と決定が食い違う。さらに、D4 の Q1(NFR Design)とは別の質問への参照が、同じ「Q1」の表記になっている。 | D1 の見出しの参照を、どのステージの Q1 かが分かる表記にし、serde を読み取りに使わない点を、Q1 の選択からの意図した逸脱として明記する。 | New |

> R-01 Reviewer note: D0 は位置(バイト範囲)を保つ文書の木を読み、行番号を範囲の先頭から求める。構文の誤りも読み取りの結果が持つ範囲から行を求めるため、型付きの段階で行が失われる問題は解消した。

> R-02 Reviewer note: 検査を自分で走査して行うため、誤りの種類は検査で決まり、ライブラリのエラー文を分類する必要がない。1回の走査で全部の誤りを集める流れが「読み取りの流れ 5・6」に書かれている。

> R-03 Reviewer note: D4 で、深さは読み取り後の走査で数える(32段)と決めた。判定基準(深さ 33 は `syntax`、深さ 1000 で異常終了しない)があり、別スレッドの代替も書かれている。Q1: B の却下も D4 の Alternatives に記録された。

> R-04 Reviewer note: 「既存の成果物との差分」の表が、entities の kind、BR1.9、BR1.12、W1 の順序、NFR4.4、BR1.11 の範囲を網羅した。終了コードの対応は、kind ではなく分類で行うと明記された(rules.md の BR5.1 も「種類ごと」で矛盾しない)。

> R-05 Reviewer note: security-design に NFR1.1、NFR2.1、NFR3.1 の測定できる基準の表が入ったが、traceability.json の upstream_ids と coverage は NFR4.1〜NFR4.7 だけのままである。NFR1.1、NFR2.1、NFR3.1、NFR5.1 の行がない。NFR5.1(網羅テスト)は logical-components が参照するのに、測定できる基準もない。

> R-06 Reviewer note: 二段階の読み取りは、D0 として、代替案3つ(serde の自動変換、位置なしの木、位置つきの型)と退けた理由つきで記録された。

> R-07 Reviewer note: 入力に由来するキー名は、制御文字をエスケープし 64 文字で切ると決まった。期待する型は、走査の検査が持つ固定の文字列になった。

### Validation Tool Results

| Tool | Result | Interpretation |
|---|---|---|
| なし(ステージ定義が検証ツールを挙げず、この Unit は library のため性能・拡張・信頼性・可観測性の設計は対象外) | 実行なし | 参照は、rules.md(BR1.9、BR1.11〜BR1.13、BR5.1)と contract-summary を手で照合した。解決しない参照はなかった。 |

### Summary

1回目の Major 4件は、位置を保つ木を自分で走査する設計(D0)と、差分の表で解消した。実装を妨げる Critical・Major は残っていない。残る Minor は、トレーサビリティの欠落(R-05)、要素のない誤りの行の決め方、D1 の参照表記である。
