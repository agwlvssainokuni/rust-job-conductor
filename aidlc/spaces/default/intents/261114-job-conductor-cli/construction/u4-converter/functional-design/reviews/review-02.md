## Review

**Verdict:** READY
**Reviewer:** aidlc-architecture-reviewer-agent
**Date:** 2026-11-14
**Iteration:** 2
**Verdict Dictated:** no

### Findings

| ID | Severity | Location | Finding | Required action | Status |
|---|---|---|---|---|---|
| R-01 | Major | aidlc/spaces/default/intents/261114-job-conductor-cli/construction/u4-converter/functional-design/rules.md > BR1.2, BR1.9 | 「キーと値の意味が同じ」(AC7.1.1、AC7.1.2)の合否の基準がない。BR1.9 は「文法として正しい」だけを求めており、出力を読み直すと入力と同じ木になる、という基準は書かれていない。スカラーの曖昧さの扱いも未定である。TOML の文字列 `"yes"`、`"1"`、`"null"` を YAML に書くとき引用するか。YAML 1.1 の `yes/no/on/off` と 8 進数風の `0755` をどう読むか。YAML の重複キー(エラーか後勝ちか)。浮動小数の NaN と inf、-0 の扱い。特に、この製品の定義が使うキー `on` は、YAML 1.1 では真偽値のキーになり、BR1.6 では文字列でないキーとして拒否される可能性がある。また TOML から YAML への出力で `on:` を引用するかどうかで、他の YAML 読み手の結果が変わる。 | BR1.2 に検証基準を足す。「出力を相手の形式で読み直した木が、入力の木と同じ(マップのキーの順序を除く)。型(文字列、整数、浮動小数、真偽値)も同じ」とする。曖昧になる文字列は引用して出力する。YAML は 1.2 のコアスキーマで読むのか 1.1 なのかを決める。重複キー、NaN と inf、`on` や `yes` のようなキーの扱いを、それぞれ規則にする。 | Resolved (reviewer) |
| R-02 | Major | aidlc/spaces/default/intents/261114-job-conductor-cli/construction/u4-converter/functional-design/functional-spec.md > W1 の手順 3〜6、rules.md > BR1.4、BR1.6、BR1.8 | 失敗の種類を決める順序が曖昧で、同じ入力に複数の結果が考えられる。(a) 複数文書の YAML は、手順 3 の読み取りが文法の誤り(syntax)にするのが普通だが、BR1.6 は unrepresentable とし、検出は手順 6 に置かれている。手順 3 で止まれば手順 6 には届かない。(b) 先頭の文書が配列で 2 つ目があるファイルは、rootNotMap(手順 4)か unrepresentable(手順 6)か決まらない。空の文書と複数文書の組み合わせも同じである。(c) 手順 6 の「1つ見つかった時点で止めてよい」では、どの keyPath が返るかが走査順で変わり、テストで固定できない。(d) 手順 2 で、UTF-8 でないこととサイズ超過の優先も未定である。 | 失敗の判定順序を、W1 に表で固定する。複数文書の検出をどの段階(読み取りの前か途中か)で行い、どの kind にするかを決める。走査順(深さ優先で、キーの順)を定め、最初に見つかった 1 件だけを返すと明記する。unreadable と tooLarge の優先も書く。 | Resolved (reviewer) |
| R-03 | Major | aidlc/spaces/default/intents/261114-job-conductor-cli/construction/u4-converter/functional-design/rules.md > BR1.7、Assumptions & Open Questions のマージキー | 2 点に問題がある。(1) YAML のマージキー `<<` を展開しない場合に「通常のキーとして扱う」と書かれている。すると TOML に `<<` というキーが残り、意味が変わる。これは Q3: A(意味を変えずに変換できないものは変換しない)と BR1.2 に反し、しかも Code Generation に先送りされている。(2) BR1.7 はアンカーを無制限に展開する。入力の上限 1 MiB は入力のバイト数にしか効かず、入れ子のエイリアスで出力が指数的に大きくなる(billion laughs)。再帰するアンカーの扱いも書かれていない。construction の入力検証の方針とも合わない。 | マージキーは、展開してその意味どおりに変換するか、unrepresentable として拒否するかのどちらかを規則で決める。Code Generation 任せにしない。展開後のサイズ(または展開の回数)の上限と、超えたときの失敗の種類を決める。再帰するアンカーも拒否する。 | Resolved (reviewer) |
| R-04 | Minor | aidlc/spaces/default/intents/261114-job-conductor-cli/construction/u4-converter/functional-design/functional-spec.md > 他の Unit との分担(U3 の項) | 「conductor 自身のエラーの終了コード 67。rules の BR5.1」と具体の値と他 Unit の規則 ID を書いているが、共有の契約は具体の値を未解決事項としている(contract-summary の E1 の終了コード)。この Unit の文書からは確かめられない。 | 具体の値と BR5.1 への言及を削り、「非 0 の終了コードは u3-cli が決める」とする。値が確定済みなら、共有の契約を根拠として示す。 | Resolved (reviewer) |
| R-05 | Minor | aidlc/spaces/default/intents/261114-job-conductor-cli/construction/u4-converter/functional-design/functional-spec.md > Assumptions & Open Questions(Q1: A) | US7.1 の目的は「既存の YAML の資産を conductor.toml として使える」ことだが、Q1: A では変換の結果が conductor の定義として読めるとは限らない。たとえば YAML の `key:`(null)は unrepresentable で止まるが、キー名の違いなどは通る。この食い違いは仮定の一行に留まり、利用者への知らせ方は決まっていない。 | 目的との差を、既知の制約として明記する。変換後に `conductor` で読み込んで確かめるよう案内する、という u3-cli への申し送りを分担の項に加える。 | Resolved (reviewer) |
| R-06 | Minor | aidlc/spaces/default/intents/261114-job-conductor-cli/construction/u4-converter/functional-design/rules.md > BR1.10、entities.md > ConversionError | AC7.1.3 は「原因が表示される」を求めるが、BR1.10 は reason を固定の文だけにし、ライブラリのエラー文を含めない。syntax の原因は「文法が誤っている」と line だけになり、原因として弱い。また、文字列でないキーのとき、keyPath が何を指すか(親のパスか)が未定である。 | syntax の reason を、原因の分類(例: 閉じていない引用、不正なインデント)で固定の文に分けるか、AC7.1.3 の「原因」は場所つきの固定の文で足りる、と明記する。文字列でないキーの keyPath の指す先も決める。 | Resolved (reviewer) |
| R-07 | Minor | aidlc/spaces/default/intents/261114-job-conductor-cli/construction/u4-converter/functional-design/rules.md > BR1.8 | 空の TOML は空のマップとして変換するが、空の YAML 文書は rootNotMap で拒否する。方向によって扱いが非対称で、理由が書かれていない(YAML の `{}` は通る)。 | 非対称の理由を書くか、空の YAML 文書も空のマップとして扱うよう揃える。 | Resolved (reviewer) |
| R-08 | Major | aidlc/spaces/default/intents/261114-job-conductor-cli/construction/u4-converter/functional-design/rules.md > BR1.2(文字列の引用符)、BR1.7(マージキー)、BR1.11 | 引用符で囲む規則は「文字列」の値だけを述べ、マップのキーを述べていない。TOML から YAML への向きで、キー `<<` をそのまま出力すると、BR1.7 のマージキーとして読み戻され、BR1.11 の読み戻しの等しさを満たせない。同じく、キー `yes`・`1`・`null`・`true` を引用符なしで出力すると、非文字列キーとして読み戻される。BR1.6 の「`<<` という文字列のキー」の扱いも、どこにも書かれていない | BR1.2 の引用符の規則を、値とキーの両方に広げる。キー `<<` を、必ず引用符つきで出力すると明記する。「TOML の `<<` キーが、YAML に写って読み戻しても同じ」という場合を、BR1.11 の確認の例に加える | New |
| R-09 | Major | aidlc/spaces/default/intents/261114-job-conductor-cli/construction/u4-converter/functional-design/rules.md > BR1.12 段階 5・8 と BR1.7。functional-spec.md > W1 手順 3・6 | 段階 5(文書の終わりまで読んだ段階)の syntax に「深さの超過」と「再帰するアンカー」が入っているが、BR1.7 では、深さは展開後の入れ子で数え、展開は段階 8(W1 では手順 6)で行う。展開を経たエイリアスによる深さの超過は、段階 5 では分からない。実装者は、深さを解析時に数えるか(エイリアスを通した深さは見逃す)、展開後に数えるか(複数文書や rootNotMap より後になる)を選べ、同じ入力で kind が変わる。たとえば、複数文書で、再帰するアンカーがある入力が、syntax か unrepresentable かが定まらない | 深さの上限を、解析時の入れ子の深さ(段階 5)と、展開後の深さ(段階 8、kind は syntax)に分けて書く。再帰するアンカーは、解析時に分かる場合のみ段階 5 とし、そうでなければ段階 8 とする。BR1.12、BR1.7、W1 手順 3 と 6 を、同じ分担にそろえる | New |
| R-10 | Minor | aidlc/spaces/default/intents/261114-job-conductor-cli/construction/u4-converter/functional-design/rules.md > BR1.2 と BR1.7 | BR1.2 は YAML 1.2 のコアスキーマとするが、マージキー `<<` は 1.1 の機能であり、1.2 のコアスキーマでは通常の文字列キーである。仕様は、1.2 に拡張を重ねたものであることを述べておらず、Assumptions の技術選択の確認も、マージキー対応を含めていない | BR1.7 に「1.2 コアスキーマに、マージキーの展開を加えた読み方」と明記し、Assumptions のライブラリの確認項目に、マージキーの対応を加える | New |
| R-11 | Minor | aidlc/spaces/default/intents/261114-job-conductor-cli/construction/u4-converter/functional-design/rules.md > BR1.7 | 「入れ子の深さ 32 は u1-core の定義の深さの上限と同じ」は、共有の契約(contract-summary)に根拠がない。また、展開後の要素数の上限は、展開を終えてから数える書き方であり、1 MiB の入力から、巨大な展開(エイリアスの繰り返し)を作るおそれがある。TOML から YAML への向きの深さの扱いも書かれていない | 深さの 32 は、共有の契約か Assumptions で、仮の値として示す。上限は「展開しながら数え、超えたら途中で打ち切る」と書く。TOML の深さについて、同じ上限を適用するかを決める | New |
| R-12 | Minor | aidlc/spaces/default/intents/261114-job-conductor-cli/construction/u4-converter/functional-design/rules.md > BR1.6 | YAML のタグ(`!!binary`、独自のタグ、`!!set` など)と、YAML 1.2 コアで文字列になる日付風の値の扱いが、BR1.6 の一覧にない。BR1.11 の「型が変わらない」を満たせるか、実装者に判断が残る | タグつきの値は、unrepresentable、または、タグを無視して読む、のどちらかを BR1.6 に書く | New |

> R-01 Reviewer note: BR1.11 が、読み戻した木の等しさ(キーの順序は問わない、型が変われば不一致、NaN は等しい、0.0 と -0.0 は区別)を定めた。BR1.2 が YAML 1.2 コア、`on` / `yes` は文字列、重複キーは syntax を定めた。ただし、キーの引用符と `<<` の扱いが残る(新規の指摘を参照)

> R-02 Reviewer note: BR1.12 が 1〜9 の順を固定し、functional-spec の W1 の順序(ファイル、サイズ、UTF-8、文法、複数文書、最上位、展開、走査)と一致する。ただし、段階 5 の syntax に展開後にしか分からない条件が入っている(新規の指摘を参照)

> R-03 Reviewer note: BR1.7 が、エイリアス・マージキーの展開、上限(100,000 要素は tooLarge)、再帰するアンカー(syntax)、展開できないマージキー(unrepresentable)を定めた

> R-04 Reviewer note: 終了コードは「契約 E1 の conductor 自身のエラー。具体の値は u3-cli の設計で定める」となり、事実としての断定がなくなった

> R-05 Reviewer note: functional-spec の Assumptions に、Q1: A の結果と、US7.1 の目的に対する利用者への案内(変換後に読み込みで確かめる)が追加された

> R-06 Reviewer note: BR1.6 に keyPath の規則(深さ優先で最初のもの、非文字列キーは `(非文字列キー)`、複数文書は reason で示す)が入った。syntax の reason は固定の文と line で足りると読める

> R-07 Reviewer note: BR1.8 が、空の TOML は空のテーブル(= 空のマップ)、空の YAML は null の文書であるため rootNotMap、と非対称の理由を示した

### Validation Tool Results

| Tool | Result | Interpretation |
|---|---|---|
| (なし。この段階の検証ツールは定義されていない) | 実行なし | 規則、仕様、エンティティ、契約を、手で突き合わせた |

### Summary

前回の 7 件はすべて直っており、BR1.11 と BR1.12 で、合否の基準と失敗の順序が定まった。残る Major は 2 件で、キー(特に `<<`)の引用符と、段階 5 に入れた展開後にしか分からない条件(深さ、再帰するアンカー)である。どちらも局所的な修正で済み、Major が 2 件以下のため READY とするが、Code Generation の前に直すことを勧める。
