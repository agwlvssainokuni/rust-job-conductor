## Review

**Verdict:** NOT-READY
**Reviewer:** aidlc-architecture-reviewer-agent
**Date:** 2026-11-14T21:16:33Z
**Iteration:** 1
**Verdict Dictated:** no

### Findings

| ID | Severity | Location | Finding | Required action | Status |
|---|---|---|---|---|---|
| R-01 | Major | aidlc/spaces/default/intents/261114-job-conductor-cli/construction/u4-converter/functional-design/rules.md > BR1.2, BR1.9 | 「キーと値の意味が同じ」(AC7.1.1、AC7.1.2)の合否の基準がない。BR1.9 は「文法として正しい」だけを求めており、出力を読み直すと入力と同じ木になる、という基準は書かれていない。スカラーの曖昧さの扱いも未定である。TOML の文字列 `"yes"`、`"1"`、`"null"` を YAML に書くとき引用するか。YAML 1.1 の `yes/no/on/off` と 8 進数風の `0755` をどう読むか。YAML の重複キー(エラーか後勝ちか)。浮動小数の NaN と inf、-0 の扱い。特に、この製品の定義が使うキー `on` は、YAML 1.1 では真偽値のキーになり、BR1.6 では文字列でないキーとして拒否される可能性がある。また TOML から YAML への出力で `on:` を引用するかどうかで、他の YAML 読み手の結果が変わる。 | BR1.2 に検証基準を足す。「出力を相手の形式で読み直した木が、入力の木と同じ(マップのキーの順序を除く)。型(文字列、整数、浮動小数、真偽値)も同じ」とする。曖昧になる文字列は引用して出力する。YAML は 1.2 のコアスキーマで読むのか 1.1 なのかを決める。重複キー、NaN と inf、`on` や `yes` のようなキーの扱いを、それぞれ規則にする。 | New |
| R-02 | Major | aidlc/spaces/default/intents/261114-job-conductor-cli/construction/u4-converter/functional-design/functional-spec.md > W1 の手順 3〜6、rules.md > BR1.4、BR1.6、BR1.8 | 失敗の種類を決める順序が曖昧で、同じ入力に複数の結果が考えられる。(a) 複数文書の YAML は、手順 3 の読み取りが文法の誤り(syntax)にするのが普通だが、BR1.6 は unrepresentable とし、検出は手順 6 に置かれている。手順 3 で止まれば手順 6 には届かない。(b) 先頭の文書が配列で 2 つ目があるファイルは、rootNotMap(手順 4)か unrepresentable(手順 6)か決まらない。空の文書と複数文書の組み合わせも同じである。(c) 手順 6 の「1つ見つかった時点で止めてよい」では、どの keyPath が返るかが走査順で変わり、テストで固定できない。(d) 手順 2 で、UTF-8 でないこととサイズ超過の優先も未定である。 | 失敗の判定順序を、W1 に表で固定する。複数文書の検出をどの段階(読み取りの前か途中か)で行い、どの kind にするかを決める。走査順(深さ優先で、キーの順)を定め、最初に見つかった 1 件だけを返すと明記する。unreadable と tooLarge の優先も書く。 | New |
| R-03 | Major | aidlc/spaces/default/intents/261114-job-conductor-cli/construction/u4-converter/functional-design/rules.md > BR1.7、Assumptions & Open Questions のマージキー | 2 点に問題がある。(1) YAML のマージキー `<<` を展開しない場合に「通常のキーとして扱う」と書かれている。すると TOML に `<<` というキーが残り、意味が変わる。これは Q3: A(意味を変えずに変換できないものは変換しない)と BR1.2 に反し、しかも Code Generation に先送りされている。(2) BR1.7 はアンカーを無制限に展開する。入力の上限 1 MiB は入力のバイト数にしか効かず、入れ子のエイリアスで出力が指数的に大きくなる(billion laughs)。再帰するアンカーの扱いも書かれていない。construction の入力検証の方針とも合わない。 | マージキーは、展開してその意味どおりに変換するか、unrepresentable として拒否するかのどちらかを規則で決める。Code Generation 任せにしない。展開後のサイズ(または展開の回数)の上限と、超えたときの失敗の種類を決める。再帰するアンカーも拒否する。 | New |
| R-04 | Minor | aidlc/spaces/default/intents/261114-job-conductor-cli/construction/u4-converter/functional-design/functional-spec.md > 他の Unit との分担(U3 の項) | 「conductor 自身のエラーの終了コード 67。rules の BR5.1」と具体の値と他 Unit の規則 ID を書いているが、共有の契約は具体の値を未解決事項としている(contract-summary の E1 の終了コード)。この Unit の文書からは確かめられない。 | 具体の値と BR5.1 への言及を削り、「非 0 の終了コードは u3-cli が決める」とする。値が確定済みなら、共有の契約を根拠として示す。 | New |
| R-05 | Minor | aidlc/spaces/default/intents/261114-job-conductor-cli/construction/u4-converter/functional-design/functional-spec.md > Assumptions & Open Questions(Q1: A) | US7.1 の目的は「既存の YAML の資産を conductor.toml として使える」ことだが、Q1: A では変換の結果が conductor の定義として読めるとは限らない。たとえば YAML の `key:`(null)は unrepresentable で止まるが、キー名の違いなどは通る。この食い違いは仮定の一行に留まり、利用者への知らせ方は決まっていない。 | 目的との差を、既知の制約として明記する。変換後に `conductor` で読み込んで確かめるよう案内する、という u3-cli への申し送りを分担の項に加える。 | New |
| R-06 | Minor | aidlc/spaces/default/intents/261114-job-conductor-cli/construction/u4-converter/functional-design/rules.md > BR1.10、entities.md > ConversionError | AC7.1.3 は「原因が表示される」を求めるが、BR1.10 は reason を固定の文だけにし、ライブラリのエラー文を含めない。syntax の原因は「文法が誤っている」と line だけになり、原因として弱い。また、文字列でないキーのとき、keyPath が何を指すか(親のパスか)が未定である。 | syntax の reason を、原因の分類(例: 閉じていない引用、不正なインデント)で固定の文に分けるか、AC7.1.3 の「原因」は場所つきの固定の文で足りる、と明記する。文字列でないキーの keyPath の指す先も決める。 | New |
| R-07 | Minor | aidlc/spaces/default/intents/261114-job-conductor-cli/construction/u4-converter/functional-design/rules.md > BR1.8 | 空の TOML は空のマップとして変換するが、空の YAML 文書は rootNotMap で拒否する。方向によって扱いが非対称で、理由が書かれていない(YAML の `{}` は通る)。 | 非対称の理由を書くか、空の YAML 文書も空のマップとして扱うよう揃える。 | New |

### Validation Tool Results

| Tool | Result | Interpretation |
|---|---|---|
| 手作業の相互参照の確認 | traceability.json の AC7.1.1 から AC7.1.3 の対象 BR1.1〜BR1.7 は、すべて rules.md にある。逆方向の N/A の 4 行(BR1.6、BR1.8、BR1.9、BR1.10)にも根拠がある。BR1.7 は AC7.1.2 の対象に入っている | 参照は解決する。構造上の問題はない。BR1.6 は FR7.1 を出典にしており N/A の記述は弱いが、阻害要因ではない |

### Summary

参照と責務の分担は整合しているが、変換の核となる意味の同一性に合否の基準がなく、失敗の判定順序が曖昧で、マージキーとエイリアスの扱いが Q3: A の方針を満たさないため、Major が 3 件あり NOT-READY である。
