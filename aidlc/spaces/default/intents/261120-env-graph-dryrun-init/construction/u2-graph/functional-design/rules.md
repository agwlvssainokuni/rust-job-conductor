# ビジネスルール(Rules): U2 u2-graph(`conductor graph`)

出典: 要件(`requirements-analysis/requirements.md` の FR2.1〜FR2.7、FR1.12、FR5.1、FR5.2、NFR1、NFR5、NFR6)、契約(`contract-design/contract-summary.md` の C2、C5、C6)、機能の設計の質問(`functional-design-questions.md`。以下 Q1〜Q5)、`entities.md`、`functional-spec.md`(期待する出力の例 G1〜G9)。

ルールは `BR{グループ}.{連番}`。グループ: BR1 辺、BR2 レーン(text の図)、BR3 `-v` の詳細、BR4 `--state`、BR5 Mermaid、BR6 コマンドの扱い、BR7 メッセージと文書。

```yaml
rules:
  # --- BR1: 辺 ---
  - id: BR1.1
    statement: ジョブから出る辺は、結果が「成功」のときの遷移先と、「警告」のときの遷移先。遷移を書かなければ、記述順で次のジョブ(最後のジョブなら終点 `@end`)
    category: calculation
    applies_to: [FlowEdge]
    trigger: 図の構造の生成
    logic: IF そのジョブに、成功(警告)の遷移がある THEN その行き先(ジョブ ID、または `@end`)。ELSE 次のジョブ。最後のジョブの次は `@end`
    violation: なし(定義の検査が、後ろ向きの遷移を既に拒否している)
    source: FR2.2、FR2.4

  - id: BR1.2
    statement: 成功と警告の行き先が同じなら、1 つの辺にまとめる(kinds に両方を持つ)
    category: calculation
    applies_to: [FlowEdge]
    trigger: 図の構造の生成
    logic: IF 成功の行き先 = 警告の行き先 THEN 辺 1 つ(kinds = [success, warning])。ELSE 辺 2 つ。辺の並びは、記述順の次のジョブへの辺を先に。どちらも次のジョブへ向かわないとき(成功と警告が、別の遠い行き先へ飛ぶとき)は、行き先が近い(記述順で前の)辺を先に、`@end` を最後にする
    violation: なし
    source: FR2.2

  - id: BR1.3
    statement: エラーには辺がない(ジョブはそこで止まるため)。分類のない中断も同じ
    category: constraint
    applies_to: [FlowEdge]
    trigger: 図の構造の生成
    logic: 辺は、成功と警告のものだけを作る
    violation: なし
    source: FR2.4

  - id: BR1.4
    statement: 辺の skipped_jobs は、元と行き先のあいだにある、すべてのジョブ(定義の記述順)。行き先が次のジョブ、または、最後のジョブから `@end` なら空。ジョブから `@end` へ飛ぶときは、元より後ろのすべてのジョブ
    category: calculation
    applies_to: [FlowEdge]
    trigger: 図の構造の生成
    logic: skipped_jobs = 記述順で、元の位置 + 1 から、行き先の位置 - 1 まで(行き先が `@end` なら、末尾まで)
    violation: なし
    source: FR2.2、FR2.3

  - id: BR1.5
    statement: 辺は、実行のときに次のジョブを決める判断と、同じ関数(遷移の解決)から取り出す。図と実行で、判断を二重に書かない
    category: policy
    applies_to: [CoreFlow, CoreGraph]
    trigger: 実装
    logic: "`next_step`(実行後の次の動き)が使う、遷移の解決を、公開する関数として切り出し、`classify_and_decide` と、辺の取り出しの両方が、それを呼ぶ。既存の `classify_and_decide` の結果は、変わらない(既存のテストが、そのまま通る)"
    violation: なし(テストで確かめる)
    source: FR2.4、契約 C2

  # --- BR2: レーン(text の図) ---
  - id: BR2.1
    statement: 図の行は、上から下へ、定義の記述順で、最後に終点 `@end`。各ジョブは、ちょうど 1 回、現れる
    category: policy
    applies_to: [LaneLayout]
    trigger: レーンの配置
    logic: ジョブの節を記述順に並べ、終点を最後に置く。到達できないジョブ(どの辺も指さない)も、省略しない
    violation: なし
    source: FR2.2

  - id: BR2.2
    statement: "記号は、ジョブが `*`、終点が `o`、線が `|`、分岐が `\\`、合流が `/`。すべて ASCII(Windows の端末でも崩れない)"
    category: policy
    applies_to: [CliReport]
    trigger: text の整形
    logic: 上の文字だけを使う。罫線文字や色は使わない
    violation: なし
    source: FR2.2

  - id: BR2.3
    statement: "レーンの図(text)は、飛び越しの形が「対応する形」(BR2.4)のとき、`git log --graph` と同じ見た目で描く。見た目の正解は、`layout-oracle.md` の 537 個の例(実際の git の出力から取った)で、すべての例に、1 文字も違わず一致すること。説明用の手順: ジョブは主線(左の列)に記述順で並ぶ。飛び越しの辺は、右のレーンの線で、ジョブの行のあと(分岐の行 `\\`)に、主線の右隣(列 1)に挿入され、すでにある右のレーンの線は、1 つ右へずれる(ずれは、ノードの行と分岐の行に `\\` で描く)。行き先のジョブの手前で、その線(列 1。入れ子では、最も内側の線)が、主線へ合流し(合流の行 `/`)、残りの線は、1 つ左へ詰まる。この手順と例が食い違うときは、例が正しい"
    category: policy
    applies_to: [LaneLayout]
    trigger: レーンの配置
    logic: "実装は、手順(説明)から作り、`layout-oracle.md` のすべての例に一致するまで、直す。ラベルの位置(行ごとの、グラフ部の幅)と、ノードの行に出る `\\` の位置は、例が正解である(実装が、例から確かめる)。`_` は、対応する形では、現れない"
    violation: なし
    source: FR2.2、質問 Q5(A)

  - id: BR2.4
    statement: "「対応する形」の条件: (1) 各ジョブの飛び越しの辺は、多くて 1 つ(もう 1 つの辺は、記述順の次のジョブへ向かう)。(2) 飛び越しの行き先が、どの 2 つの辺でも違う(最後のジョブの、記述順の次の `@end` への辺は、数えない)。(3) 飛び越しの区間が、交差しない(2 つの辺の元と行き先が、元 1 < 元 2 < 行き先 1 < 行き先 2 になる組がない)。**どれか 1 つでも満たさない形(交差、行き先の共有、成功と警告がどちらも次のジョブを飛び越す形)は、text では、レーンの図の代わりに、「辺の一覧」(BR2.8)を出す**。Mermaid は、形によらず、常に出せる"
    category: constraint
    applies_to: [CoreGraph, CliReport]
    trigger: レーンの配置
    logic: "対応する形かを、辺から判定する純粋な関数を、`CoreGraph` に置く(`is_lane_drawable`)。判定が偽なら、レーンの配置を呼ばず、辺の一覧に切り替える。(3 つの条件のうち、(1) を満たさない形は、記述順の次のジョブが、どの辺からも指されない(到達できない)ことがある。この形も、辺の一覧で出す)"
    violation: なし
    source: FR2.2、質問 Q5(A)(ユーザーの選択: 交差は辺の一覧)

  - id: BR2.5
    statement: 各行の末尾の空白は取り除く。ラベル(ジョブ ID、`@end`、状態の記号)の位置(グラフ部の幅)は、`layout-oracle.md` の例が正解で、おおよそ、その行と、続く分岐の行の、列の数 × 2 の位置にそろう
    category: policy
    applies_to: [CliReport]
    trigger: text の整形
    logic: グラフ部の幅は、例から確かめる。幅の決め方(例: ノードの行に `\\` が出る行は、分岐のあとの列の数で決まる)は、実装が、例に一致するように決める。ラベルを置いたあと、行末の空白は削る
    violation: なし
    source: FR2.2

  - id: BR2.6
    statement: "`git` のソースコード(GPL)を、写さない。このプロジェクトは Apache-2.0 のため、出力の例と BR2.3 の規則から、自前で書く"
    category: constraint
    applies_to: [CoreGraph]
    trigger: 実装
    logic: 配置の処理は、この文書の規則と例だけを根拠に、新しく書く
    violation: なし
    source: project.md(新しい依存を足さない)、ライセンスの整合

  - id: BR2.7
    statement: ジョブが 1,000 個ある定義で、`graph` は 1 秒以内に終わる
    category: constraint
    applies_to: [CoreGraph, CliReport]
    trigger: テスト
    logic: 出力の大きさは、最悪で、行の数 × 列の数(約 1,000 × 500)。1 秒以内に終わること。テストの入力は、最悪の 2 つの形: (a) 入れ子の最大(前半の各ジョブが、後半のジョブへ、入れ子に飛び越す。列が約 500)、(b) 交差する形(辺の一覧に切り替わる。辺は約 1,000)。どちらも、`graph` を、text と Mermaid で実行して、1 秒以内
    violation: テストの失敗
    source: NFR1

  - id: BR2.8
    statement: "辺の一覧(レーンの図にできない形の text の出力): 先頭に、注記の行「注記: 飛び越しが交差するなどの形のため、レーンの図の代わりに、辺の一覧を出します。」(英語は「note: the skip edges cross or share a target, so the lane diagram is replaced by a list of edges.」)。続けて、「ジョブ:」の見出しと、ジョブの行(記述順。`--state` のとき、行頭に状態の記号、再開位置の行末に「  <- 再開位置」)、「辺:」の見出しと、辺の行(`<元> -> <行き先>`。主線の辺(次のジョブへ、成功と警告の両方)以外は、末尾に ` (成功)` ` (警告)` ` (成功・警告)`(英語は success、warning)を添える。終点は `@end`)。`-v` では、ジョブの行の下に、詳細の行(BR3.1 と同じ文)を、2 つの空白で字下げして置く"
    category: policy
    applies_to: [CliReport]
    trigger: text の整形(対応する形でないとき)
    logic: ジョブの行と辺の行は、2 つの空白で字下げする。`-v` の詳細の行は、ジョブの行の下に、4 つの空白で字下げする(辺の一覧では、レーンの `|` を描かない)。再開位置の注記(「  <- 再開位置」)は、ジョブの行の末尾に置く。辺の並びは、元のジョブの記述順、同じ元では BR1.2 の順。Mermaid の辺の順も、BR1.2 の順に従う。各行の末尾の空白は取り除く。見出しは、ロケールに従う(日本語は「ジョブ:」「辺:」、英語は「Jobs:」「Edges:」)
    violation: なし
    source: FR2.2、質問 Q5(A)

  # --- BR3: -v の詳細 ---
  - id: BR3.1
    statement: "`-v` を付けると、ジョブごとに、コマンド、リトライの設定、飛び越しの注記を、詳細の行として出す。詳細の行は、ノードの行(と、分岐の行)のあとに置き、各列の線(`|`)を続けて描く"
    category: policy
    applies_to: [CliReport]
    trigger: "`-v`"
    logic: "詳細の行は、ノードの行のあと、分岐の行があれば、その分岐の行のあと(合流の行の前)に置く。詳細の行 = 分岐のあとに残る各列に `|` を描き(列の数 × 2 の幅にそろえる)、詳細の文を続ける。出す順は、コマンド、リトライ、飛び越し。該当しない項目(リトライの設定が無い、など)は出さない。終点には詳細を出さない。`--state` の再開位置の注記は、ノードの行の末尾に置く(詳細の行には付けない)。リトライの要約: 「リトライ: 終了コード 75、76 を最大 3 回、間隔 5 秒」。倍率と上限があるときは、「(倍率 2、上限 60 秒)」を添える(英語は「retry: exit codes 75, 76, up to 3 times, every 5 s (x2, max 60 s)」)。`-v` は、`--format mermaid` には、影響しない(無視する)"
    violation: なし
    source: FR2.3、質問 Q1

  - id: BR3.2
    statement: "飛び越しの注記は、辺が 2 つあるとき、または飛び越すジョブがあるときに、辺の種類と行き先を書く(例: 成功 → d(b、c を飛び越す)、警告 → b)。`-v` なしでは、書かない"
    category: policy
    applies_to: [CliReport]
    trigger: "`-v`"
    logic: 辺ごとに「<種類> → <行き先>」を、成功、警告の順(辺の並び BR1.2 とは関係なく)に、「、」でつなぐ。skipped_jobs があれば、行き先のあとに「(<ジョブ>、… を飛び越す)」を足す。成功と警告が同じ行き先なら、「成功・警告 → <行き先>」と 1 つにする。日本語と英語の両方を用意する
    violation: なし
    source: FR2.3、質問 Q1

  - id: BR3.3
    statement: コマンドは、定義に書いたとおりに表示する。制御文字はエスケープする。配列のコマンドは、各要素を、空白か空のときだけ二重引用符で囲み、空白でつなぐ
    category: policy
    applies_to: [CliReport]
    trigger: "`-v`"
    logic: 文字列のコマンドは、そのまま(エスケープのみ)。配列は、要素ごとに整形して「 」でつなぐ。環境変数(名前も値も)は、出さない
    violation: なし
    source: FR2.3、FR1.12

  # --- BR4: --state ---
  - id: BR4.1
    statement: "`--state` では、ジョブの状態を、記号で重ねる: 成功と警告は `[OK]`、スキップは `[--]`、エラーは `[NG]`、未実行は `[  ]`。記号は、ノードの記号のあと、ジョブ ID の前に置く(例: `* [OK] a`)。終点には付けない"
    category: policy
    applies_to: [CoreGraph, CliReport]
    trigger: "`--state`"
    logic: ステートの各ジョブの状態から、NodeState を決める(success、warning → ok。skipped → skipped。error → error。pending → pending)
    violation: なし
    source: FR2.7、質問 Q2

  - id: BR4.2
    statement: ステートが完了していなくて、再開位置があるときは、そのジョブの行の末尾に、「  <- 再開位置」(英語は「  <- resume here」)を添える
    category: policy
    applies_to: [CoreGraph, CliReport]
    trigger: "`--state`"
    logic: IF ステートの resume_job_id がある THEN そのジョブの is_resume = true。完了したステートには、再開位置を示さない
    violation: なし
    source: FR2.7、質問 Q2

  - id: BR4.3
    statement: "`--state` では、ステートを、完了したものも含めて読む(`read_state_raw`。`read_state` は、完了したステートを「無い」として返すため、使わない)。完了したステートは、ジョブの状態を重ねるが、再開位置は示さない。ステートが無い(ファイルが無い、または長さ 0)ときは、エラーにせず、すべてのジョブを `[  ]` で表示する"
    category: policy
    applies_to: [CliFlows]
    trigger: "`--state`"
    logic: "`read_state_raw` の結果が空なら、すべて pending の重ね合わせを使う。空でなければ、そのステートの内容から重ね合わせを作る(完了していれば、`resume_job_id` は使わない)"
    violation: なし
    source: FR2.7、質問 Q3

  - id: BR4.4
    statement: "`--state` で、ステートが定義と食い違うとき(ジョブの並びが違うなど)は、既存の照合と同じ誤り(ステートの誤り)を表示して、終了コード 65 で止まり、図を出さない。照合は、完了したステートにも行う"
    category: constraint
    applies_to: [CliFlows]
    trigger: "`--state`"
    logic: "`pipeline`、`run` と同じ照合(`check_consistency`)を使い、同じ表示にする"
    violation: 終了コード 65
    source: FR2.7、質問 Q3

  - id: BR4.5
    statement: "`--run-id` は、`--state` と一緒のときだけ使える。`--state` なしで付けたときは、使い方の誤り(終了コード 68)。`--state` のときの run ID の決め方は、`pipeline`、`run` と同じ(`--run-id`、なければ定義の `run_id_env` の環境変数)"
    category: validation
    applies_to: [CliArgs, CliFlows]
    trigger: 引数の解釈
    logic: IF `--run-id` があり、かつ `--state` がない THEN 使い方の誤り。run ID の形の誤りは、既存の誤り(68)
    violation: 終了コード 68
    source: FR2.7、契約 C5、質問 Q3

  - id: BR4.6
    statement: ステートは、読むだけ。作成も、更新も、削除も、しない。シグナルの準備もしない
    category: constraint
    applies_to: [CliFlows]
    trigger: "`--state`"
    logic: "`read_state_raw` だけを呼ぶ(`write_state`、`reset_state` は呼ばない)"
    violation: なし(テストで確かめる)
    source: FR2.1

  # --- BR5: Mermaid ---
  - id: BR5.1
    statement: "`--format mermaid` は、`flowchart TD`(上から下)の図を出す"
    category: policy
    applies_to: [CliReport]
    trigger: "`--format mermaid`"
    logic: 先頭の行は `flowchart TD`。次に、節の定義、辺の定義、(再開位置があるときだけ)`classDef` と `class` の順。各行は、2 つの空白で字下げする(先頭の行を除く)
    violation: なし
    source: FR2.5

  - id: BR5.2
    statement: 節の ID は、連番(`n0`、`n1`、…。終点は `nend`)にし、ジョブ ID は、二重引用符つきのラベルで書く(`n0["job-id"]`。終点は `nend(["@end"])`)
    category: policy
    applies_to: [CliReport]
    trigger: "`--format mermaid`"
    logic: どのジョブ ID(ハイフン、ドット、日本語、予約語)でも、図が壊れない
    violation: なし
    source: FR2.5、質問 Q4

  - id: BR5.3
    statement: 辺は `-->` で書く。ラベル(`|"成功"|`、`|"警告"|`、`|"成功・警告"|`。英語は success、warning)は、記述順の次への辺で、成功と警告が同じ行き先のとき(主線の辺)以外に付ける
    category: policy
    applies_to: [CliReport]
    trigger: "`--format mermaid`"
    logic: 主線の辺(行き先が次のジョブ、または最後のジョブから終点で、kinds が両方)は、ラベルなし。それ以外は、kinds をラベルにする
    violation: なし
    source: FR2.5

  - id: BR5.4
    statement: "ラベルの中の `#` は `#35;`、`\"` は `#quot;`、`<` は `#lt;`、`>` は `#gt;`、`&` は `#amp;`、バッククォートは `#96;`、制御文字は `#<十進の番号>;` に置き換える(`#` を最初に置き換える)"
    category: validation
    applies_to: [CliReport]
    trigger: "`--format mermaid`"
    logic: ジョブ ID に含まれる文字を、エスケープする。ジョブ ID の値は、そのほかには変えない。これらの文字を含むジョブ ID を使うテストを置く
    violation: なし
    source: FR2.5、質問 Q4

  - id: BR5.5
    statement: "`--state` では、節のラベルの先頭に、状態の記号を付ける(例: `n0[\"[OK] a\"]`)。再開位置があるときは、辺の定義のあとに、`classDef resume stroke-width:3px` の行と、`class <節の ID> resume` の行を、それぞれ 1 つ足す"
    category: policy
    applies_to: [CliReport]
    trigger: "`--format mermaid` と `--state`"
    logic: BR4.1、BR4.2 と同じ判断を使う
    violation: なし
    source: FR2.7、FR2.5

  # --- BR6: コマンドの扱い ---
  - id: BR6.1
    statement: "`graph` は、`validate` と同じ検査(定義の検証、`command` の検査、`.env` の読み取り)を行い、誤りになるときは、`validate` と同じ誤りと終了コード(定義と `.env` の誤りは 64)で止まり、図を出さない。`validate` が通る定義だけが、`graph` でも図になる。`.env` の重複キーの警告は、`validate` と同じに、標準エラーに出す(終了コード 0)。パイプライン名の引数(`graph <name>`)は、`validate` と同じに、定義の `name` と照合する(一致しなければ 64)"
    category: constraint
    applies_to: [CliFlows]
    trigger: "`graph`"
    logic: "`validate` の中身(`check_definition` と `.env` の読み取りと警告の表示)を、共通の関数に切り出して、`validate` と `graph` の両方が呼ぶ(U1 で作った処理を使う)"
    violation: 終了コード 64
    source: FR2.6、契約 C5

  - id: BR6.2
    statement: "`graph` は、ジョブを実行せず、ステートを読み書きしない(`--state` の読み取りを除く)。シグナルの準備もしない。`.env` は、BR6.1 の検査のためにだけ読み、図にも出力にも、値を含めない"
    category: constraint
    applies_to: [CliFlows]
    trigger: "`graph`"
    logic: 定義の読み込みと検査(`.env` の読み取りを含む)、(`--state` のとき)ステートの読み取り、標準出力への書き込みだけを行う
    violation: なし(テストで確かめる)
    source: FR2.1、FR1.12

  - id: BR6.4
    statement: "引数の文法は、既存の規則に従う: `--format` と `-v` と `--state` と `--run-id` は、それぞれ 1 回だけ(2 回目は使い方の誤り 68)。`--format` の値は `text` か `mermaid`(それ以外は 68)。`--format`、`--state`、`-v`(`validate` などの既存の `-v` を除く)は、`graph` 以外のコマンドでは、許されないオプション(68)"
    category: validation
    applies_to: [CliArgs]
    trigger: 引数の解釈
    logic: "既存の引数の検査(`OptionNotAllowed`、重複、`InvalidValue`)を使い、`graph` の許可するオプションの一覧を足す"
    violation: 終了コード 68
    source: FR2.1、契約 C5

  - id: BR6.3
    statement: 図は、標準出力に書く。成功したときの終了コードは 0。標準出力に書けないときは、終了コード 65(`watch` の出力の失敗と同じ)
    category: policy
    applies_to: [CliFlows]
    trigger: "`graph`"
    logic: 図の全体を作ってから、1 回で書く。誤りのときは、標準出力に何も書かない
    violation: 終了コード 65
    source: FR2.1

  # --- BR7: メッセージと文書 ---
  - id: BR7.1
    statement: 新しいメッセージ(使い方、注記、再開位置、誤り)は、日本語と英語の両方で用意する。環境変数の値は、どの出力にも出さない
    category: policy
    applies_to: [CliReport]
    trigger: メッセージの表示
    logic: 既存のメッセージの網羅のテストに、新しい種類を足す
    violation: なし
    source: FR5.1、FR1.12

  - id: BR7.2
    statement: README に、`graph` の使い方と出力例を、版(v0.9.0 以降)つきで書く。デモに、`graph` の場面を足す
    category: policy
    applies_to: [CliReport]
    trigger: ユニットの完了
    logic: README の必須語句の検査(`docs-check`)に合わせて書き、デモのテストを足す
    violation: なし
    source: FR5.2
```

## ルールの要約(`rules` の YAML から作った表)

| 規則 | 内容 | 出典 |
|------|------|------|
| BR1.1 | 辺の元(成功と警告の遷移先、なければ次) | FR2.2、FR2.4 |
| BR1.2 | 同じ行き先は 1 つの辺 | FR2.2 |
| BR1.3 | エラーに辺はない | FR2.4 |
| BR1.4 | 飛び越すジョブの一覧 | FR2.2、FR2.3 |
| BR1.5 | 辺は実行と同じ関数から | FR2.4 |
| BR2.1 | 行の順(記述順、最後に終点) | FR2.2 |
| BR2.2 | 記号(ASCII) | FR2.2 |
| BR2.3 | 対応する形は git と同じ見た目(正解データ 537 例) | FR2.2、Q5 |
| BR2.4 | 「対応する形」の条件(交差などは辺の一覧) | FR2.2、Q5 |
| BR2.5 | ラベルの位置、行末の空白 | FR2.2 |
| BR2.6 | git のソースを写さない | ライセンス |
| BR2.7 | 1,000 ジョブで 1 秒以内(最悪の形 2 つ) | NFR1 |
| BR2.8 | 辺の一覧(text。対応しない形) | FR2.2、Q5 |
| BR3.1 | `-v` の詳細の行 | FR2.3、Q1 |
| BR3.2 | 飛び越しの注記 | FR2.3、Q1 |
| BR3.3 | コマンドの表示、値を出さない | FR2.3、FR1.12 |
| BR4.1 | 状態の記号 | FR2.7、Q2 |
| BR4.2 | 再開位置 | FR2.7、Q2 |
| BR4.3 | 完了したステートも読む(`read_state_raw`)。ステートが無いとき | FR2.7、Q3 |
| BR4.4 | 食い違いは 65 | FR2.7、Q3 |
| BR4.5 | `--run-id` は `--state` と一緒だけ(68) | FR2.7、C5、Q3 |
| BR4.6 | ステートは読むだけ | FR2.1 |
| BR5.1 | `flowchart TD` | FR2.5 |
| BR5.2 | 節の ID は連番、ラベルに ID | FR2.5、Q4 |
| BR5.3 | 辺のラベル | FR2.5 |
| BR5.4 | ラベルのエスケープ | FR2.5、Q4 |
| BR5.5 | 状態と再開位置(Mermaid) | FR2.7、FR2.5 |
| BR6.1 | `validate` と同じ検査(`.env` を含む)。誤りは 64 | FR2.6、C5 |
| BR6.2 | 実行せず、書かない | FR2.1、FR1.12 |
| BR6.3 | 標準出力、0、出力失敗は 65 | FR2.1 |
| BR6.4 | 引数の文法(重複、値、許されないオプションは 68) | FR2.1、C5 |
| BR7.1 | メッセージは日本語と英語 | FR5.1、FR1.12 |
| BR7.2 | README とデモ | FR5.2 |
