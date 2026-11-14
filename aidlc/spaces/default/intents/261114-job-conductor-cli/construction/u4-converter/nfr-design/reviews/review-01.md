## Review

**Verdict:** NOT-READY
**Reviewer:** aidlc-architecture-reviewer-agent
**Date:** 2026-11-14T21:29:08Z
**Iteration:** 1
**Verdict Dictated:** no

### Findings

| ID | Severity | Location | Finding | Required action | Status |
|---|---|---|---|---|---|
| R-01 | Major | aidlc/spaces/default/intents/261114-job-conductor-cli/construction/u4-converter/nfr-design/security-design.md > 設計の要点 1(段階 2・4)と 3(a)(b) | 「変換の流れ」は、読み取り(段階 2)が木を返したあと、写し取り(段階 4)がアンカー・エイリアス・マージキーを展開し、要素数と深さを数える、という流れで書かれている。これが成り立つのは、ライブラリが、エイリアスを展開しない木(またはイベントの列)を返す場合(3 の (a))だけである。(b) の場合(ライブラリが展開済みの木を一度に返す)は、展開は読み取りの中で終わっており、段階 4 の (i)(ii) は何もしない。しかも (b) の見積もりは「読み取りの前」に行うため、段階の順序が変わる。すると、文法の誤りや複数の文書や最上位がマップでない入力でも、先に `tooLarge` が返りうる。これは BR1.12 の 5〜7 が 8 より先、という順序に反する。「段階と BR1.12 が 1 対 1」(D3)という主張は、(b) では成り立たない。設計は、どちらのライブラリの形を前提にするかを書いておらず、Assumptions にも、木がエイリアスを保つ前提がない。 | 流れを (a) の場合と (b) の場合に分けて書き、それぞれについて、各段階が BR1.12 のどの項目に当たるかを示す。(b) の見積もりは、文法の誤り・複数の文書・最上位がマップかの確認より後に置くか、その順序の例外を、BR1.12 の見直しとして明示する。「ライブラリの木がエイリアスとマージキーを保つこと」を、ライブラリ選定の条件(Code Generation の最初の確認)と Assumptions に加える。 | New |
| R-02 | Major | aidlc/spaces/default/intents/261114-job-conductor-cli/construction/u4-converter/nfr-design/security-design.md > 設計の要点 3 の最終行、D2、変換の流れの段階 2〜4 | 深さの上限 32 は「読み取り時(展開前)と展開後の2回」数えるとあるが、展開前の 32 の確認を行う段階が決まっていない。D2 の主案は、ライブラリの再帰の制限に任せるが、その制限の値は 32 ではない(深さ 100,000 を拒否できればよい)。深さ 33 から、制限の値までの入力は、ライブラリを通るため、32 の確認は、段階 4(写し取り)で行うしかない。段階 4 は、複数の文書の確認(段階 3)と最上位の確認の後であり、BR1.12 では、展開前の深さの超過(`syntax`、項目 5)は、`unrepresentable`(6)と `rootNotMap`(7)より先である。そのため、深さ 40 の配列が最上位の YAML は、`syntax` ではなく `rootNotMap` が返る。TOML は展開がなく、深さは 1 回しか数えないが、設計は一律に「2回」と書いている。写し取りと書き出しが、再帰で書かれるのか、深さが 32 に制限されたあとにだけ再帰するのかも、書かれていない。 | 展開前の深さ 32 の確認を、どの段階が、どの方法で行うかを決める(たとえば、読み取りの直後に、木またはイベントを、再帰を使わずに走査して数える)。そのとき BR1.12 の項目 5 の位置になるように、段階の順序を直す。TOML は 1 回、YAML は 2 回と区別して書く。写し取りと書き出しの再帰は、32 の確認の後だけで行うことを、明記する。 | New |
| R-03 | Major | aidlc/spaces/default/intents/261114-job-conductor-cli/construction/u4-converter/nfr-design/security-design.md > D2 と 設計の要点 1 の段階 2 の (b)(事前の深さの走査) | 事前の走査 (b) は「再帰を使わない深さの走査」とだけあり、YAML の何を数えるかが決まっていない。YAML はインデントで入れ子になり、フロー形式の括弧、ブロックスカラー、複数行の引用符つき文字列、コメント、アンカーが混ざる。文字や行を見るだけでは、正しい深さを数えられない(文字列の中の括弧や、コメントを数える誤り、または数え落とし)。実用になるのは、パーサーのイベントの列を、反復で数える方法だけである。その場合は、ライブラリがイベントの列を出すこと、そのパーサーが反復で動くことが、前提になる。これは T1 の条件にも書かれていない(NFR4.3 の (b) の条件にしか書かれていない)。D2 の主案が成り立たないとき、実際の防ぎ方が、設計上は決まっていないことになる。D2 の「別のスレッドで読む案」の退ける理由は妥当だが、(b) の実現可能性の根拠がない。 | (b) の方式を、YAML は「パーサーのイベントの列の入れ子の深さを、反復で数える」、TOML は「選んだライブラリで確かめた手段、または、同様に字句を数える方法」と、具体にする。そのために必要なライブラリの条件(イベントの列を出し、反復で動くこと)を、tech-stack の T1 の条件と、この設計の Assumptions に加える。条件を満たす候補がないときの扱いは、止めて人に確認する手順に、つなげて書く。 | New |
| R-04 | Minor | aidlc/spaces/default/intents/261114-job-conductor-cli/construction/u4-converter/nfr-design/traceability.json > upstream_ids と coverage | traceability は、NFR4.1〜NFR4.7 だけを扱っている。security-design の冒頭と logical-components は、NFR1.1、NFR2.1、NFR3.1、NFR5.1 を上流として挙げている(Value の行が NFR2.1・NFR3.1 を、Intake の説明が NFR5.1 を引く)が、coverage にこれらがない。NFR1.1(`run` と `pipeline` の起動経路に、この Unit の初期化を含めない。静的な初期化を持たない)を、どこで守るかの設計は、nfr-design のどの文書にもない。 | upstream_ids と coverage に、NFR1.1、NFR2.1、NFR3.1、NFR5.1 を加える。NFR1.1 は、「静的な初期化を持たず、YAML のライブラリを `convert` のときだけ初期化する」という設計を、logical-components に書いて、対応づける。他は、該当する設計(または「設計の追加なし」)を示す。 | New |
| R-05 | Minor | aidlc/spaces/default/intents/261114-job-conductor-cli/construction/u4-converter/nfr-design/security-design.md > D1 と 設計の要点 1 の段階 4・5(往復 BR1.11 の実現) | 途中の表現(Value)は「整数・浮動小数・マップ」とだけあり、BR1.11 の往復に必要な性質が決まっていない。(1) 整数の幅(i64 か)と、YAML の範囲を超える整数を、ライブラリが浮動小数や文字列として返したとき、型が黙って変わらないよう、段階 4 が元の字句で判定すること。(2) 書き出しで、整数と浮動小数の区別を保つこと(`1.0` が `1` にならない)、`-0.0`、`nan`/`inf` の符号。(3) マップが、入力の出現順を保つ型か(BR1.6 の「入力に現れる順で最初」と、BR1.12 の確定性は、順序が保たれる前提である。`toml` の既定のテーブルは、キーの順に整列する)。書き出しの設計は、文字列の引用符だけを扱っている。 | Value の整数の型、マップの順序の扱い、浮動小数の書き出しの規則(整数との区別、`-0.0`、`nan`/`inf`)を、設計に書く。範囲を超える整数が、型を変えて読まれないことの確認を、段階 4 の責務に加える。往復のテストの対象に、これらの値を挙げる。 | New |
| R-06 | Minor | aidlc/spaces/default/intents/261114-job-conductor-cli/construction/u4-converter/nfr-design/security-design.md > D4 | 共有をしない理由が、「依存の DAG のため」とされているが、u1-core と u4-converter が、共通の葉のクレートに依存しても、DAG は壊れない。実際の理由は、「Unit の独立と、規模が小さいこと」であり、その方が、退けた案の説明として正しい。また「内容は、テストで揃える」とあるが、2か所の実装を、どのテストで揃えるか(同じ入力の表で、両方を検査するなど)が、決まっていない。他の Unit の内容を読めないため、揃っていることの確認は、設計上の約束がないと、保てない。 | 退ける理由を、「Unit の独立と規模」に直す。2か所の関数が同じ規則であることを、確かめる方法(たとえば、規則を表にして、各 Unit のテストにそのまま写す)を、設計に書く。 | New |

### Validation Tool Results

| Tool | Result | Interpretation |
|---|---|---|
| (none listed for this stage) | not run | Verified by reading: nfr-design traceability omits NFR1.1, NFR2.1, NFR3.1, NFR5.1; the flow-to-BR1.12 mapping was traced by hand against W1 steps 1-9. |

### Summary

NFR4.1〜NFR4.7 は、いずれも設計に対応づけられているが、「段階と BR1.12 が 1 対 1」という主張は、(b) の見積もりと、展開前の深さ 32 の確認を置く段階が決まっていないために成り立たず、エイリアスの展開と深さの防ぎ方も、YAML では実装者が推測しないと作れない。3 つの Major(流れの順序、深さの確認の位置、事前の走査の実現性)を直すまでは、NOT-READY とする。
