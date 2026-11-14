## Review

**Verdict:** NOT-READY
**Reviewer:** aidlc-architecture-reviewer-agent
**Date:** 2026-11-14T20:31:52Z
**Iteration:** 1
**Verdict Dictated:** no

### Findings

| ID | Severity | Location | Finding | Required action | Status |
|---|---|---|---|---|---|
| R-01 | Major | aidlc/spaces/default/intents/261114-job-conductor-cli/construction/u1-core/nfr-design/security-design.md > 設計の要点 1 の手順 3〜4、および 3(行番号) | 手順 3 で作った汎用の木(toml の Value)は、バイト位置の情報を持たない。手順 4 は「同じ木から」型付きの構造に変換するとあるため、`unknownKey` / `typeMismatch` に NFR4.1・NFR4.2・BR1.6 が求める行を付けられない。「エラーが指す位置から改行を数える」という行番号の算出は、構文エラーにしか成り立たない。手順 5 の構造の検査(BR1.1〜BR1.4 など、遷移やジョブごとの誤り)にも、行を得る手段が書かれていない。 | 型付けの段階で位置を得る方法を決めて書く。例: 版の確認の後、同じ文字列から型付きの構造を読み直す。または位置つきの値を使う。構造の検査の誤りに行を付ける方法(各ジョブ・遷移の位置の保持)も書く。 | New |
| R-02 | Major | aidlc/spaces/default/intents/261114-job-conductor-cli/construction/u1-core/nfr-design/security-design.md > 設計の要点 1 の手順 4、設計の要点 3、D1 | 型付きの構造への変換は、serde では最初の1件の失敗で止まる。それなのに、手順 4 の `unknownKey` / `typeMismatch` と、手順 5 の「すべての誤りをまとめて返す」(functional-spec W1 の手順 8)の関係が書かれていない。さらに、serde の失敗を `unknownKey` / `typeMismatch` / 必須の欠落に振り分けるには、ライブラリの標準のエラー文の解析が必要になりかねない。設計の要点 3 は、標準のエラー文をそのまま使わないと定めている。この点は、設計の中で矛盾しており、実現の見通しがない。 | 種類の振り分け方法を決めて書く。例: Value の木を手で走査して許可するキーと突き合わせる、またはエラーの種類を表す構造化された情報を使う。変換の段階で複数の誤りを返すのか、1件で止めるのかも明記する。止める場合は、W1 の手順 8 との差を明記する。 | New |
| R-03 | Major | aidlc/spaces/default/intents/261114-job-conductor-cli/construction/u1-core/nfr-design/security-design.md > 設計の要点 4(入れ子の深さ)、Assumptions & Open Questions | 「読み取る前の簡易な走査、または読み取りの設定で拒否する」は、2案が未決のまま並んでいる。深さの数え方(テーブル、配列、インラインテーブルのどれを数えるか)も定まっていない。文字列やコメントを含む TOML の生の文字列に対する簡易な走査は、誤検知しやすい。`toml` クレートが深さの上限を設定する手段を持つ根拠も示されていない。さらに、型付きの構造への変換も再帰で走る。このため、NFR4.7 の「巨大な入れ子でパニックしない」について、32 段の拒否が単体テストで合否を判定できる形になっていない。Q1 の B(上限を置かない)を退けた理由も、設計の判断には記録されていない。 | 実装する方法を1つに決める(例: 読み取りの前に、文字列を走査する関数を置く、またはライブラリの上限を明記する)。深さの数え方と、合否を確かめるテストの入力(32 段は通り、33 段は `syntax` になる)を書く。D の判断の一つとして、代替案と理由を残す。 | New |
| R-04 | Major | aidlc/spaces/default/intents/261114-job-conductor-cli/construction/u1-core/nfr-design/security-design.md > 追加する種類、および設計の要点 1 の手順 1 | `syntax` と `tooLarge` の追加は、entities.md の `kind` の拡張としてしか Code Generation への引き継ぎが書かれていない。nfr-requirements の security-requirements.md の NFR4.4 は、上限超過を `unreadable` と明記しており、これと矛盾したままである。rules.md の BR1.9 と BR1.12、functional-spec.md の W1(手順 2 に上限超過がなく、構文エラーの手順もなく、版の確認が手順 3)も、新しい順序と種類を反映していない。DefinitionError の `kind` は、u2-runtime と u3-cli が、終了コード 64 への割り当てや表示の出し分けで使う共有の型である。引き継ぎの範囲が狭く、実装者が旧版を正とする恐れがある。また、非 UTF-8 の入力をどの種類にするかも決まっていない(手順 1 の「文字列」への変換)。 | 反映が必要な箇所を一覧にして書く(entities.md の `kind`、rules.md の BR1.9 と BR1.12、functional-spec.md の W1 の手順、security-requirements.md の NFR4.4 の判定の基準)。非 UTF-8 の入力の種類(`unreadable` か `syntax`)を決めて書く。 | New |
| R-05 | Minor | aidlc/spaces/default/intents/261114-job-conductor-cli/construction/u1-core/nfr-design/traceability.json > upstream_ids / coverage | upstream_ids は NFR4.1〜NFR4.7 だけで、設計が参照する NFR1.1・NFR2.1・NFR3.1・NFR5.1 が無い。引き継ぎ(6)の、NFR1.1・NFR2.1・NFR3.1 に測定できる基準を足す件は、設計のどこにも扱われていない。 | NFR1.1・NFR2.1・NFR3.1・NFR5.1 の対応を追加する。または、対象外とする理由を書く。測定基準(例: 3つの OS の CI での成功、依存が Rust のみであることの確認)をどこで足すかを決めて書く。 | New |
| R-06 | Minor | aidlc/spaces/default/intents/261114-job-conductor-cli/construction/u1-core/nfr-design/security-design.md > 設計の判断(D1〜D4) | 二段階の読み取り(Value 経由)は、この設計の最も影響の大きい判断だが、判断として記録されておらず、代替案が無い。例: 版だけを部分的に読む案、1回の型付きの読み取りで版を先に確認する案。D4 の代替案は、実質的な比較になっていない。 | 二段階の読み取りを判断(D5)として足し、2案以上の代替案とトレードオフを書く。 | New |
| R-07 | Minor | aidlc/spaces/default/intents/261114-job-conductor-cli/construction/u1-core/nfr-design/security-design.md > 設計の要点 3(Describe) | 固定の文だけから表示を作ると、NFR4.2 が求める「キー名」や、型の誤りで期待する型を、どこから取るかが示されていない。キー名は入力由来であり、長さの上限や制御文字の扱いも書かれていない。 | 表示に含める入力由来の項目(キー名、パス)と、その長さの上限・エスケープの方針を書く。 | New |

### Validation Tool Results

| Tool | Result | Interpretation |
|---|---|---|
| (ステージ定義に検証ツールの指定なし。Library の Unit のため、performance などの設計ファイルは対象外) | 未実行 | traceability.json は手で確認した。NFR4.1〜NFR4.7 はすべて対応づけられている。ただし、NFR4.5 の対応先は Intake の入出力の限定(安全だが静的な主張)であり、検証の単体テストの方法は示されていない。 |

### Summary

型の読み取りの安全性の大枠(版の確認を先に、上限 + 1 バイトの読み取り、固定の文からの表示、`syntax` / `tooLarge` の分離)は妥当である。しかし、二段階の読み取りでは型付けの段階の行番号が得られないこと、型の誤りの種類の振り分けと複数の誤りの返却の見通しが無いこと、深さの上限の実装が未決で合否を判定できないことの、3点の Major がある。さらに、新しい `kind` の反映範囲が狭い。これらを直せば READY に近づく。
