## Review

**Verdict:** READY
**Reviewer:** aidlc-architecture-reviewer-agent
**Date:** 2026-11-14T23:39:16Z
**Iteration:** 1
**Verdict Dictated:** no

### Findings

| ID | Severity | Location | Finding | Required action | Status |
|---|---|---|---|---|---|
| R-01 | Major | aidlc/spaces/default/intents/261114-job-conductor-cli/construction/u3-cli/functional-design/functional-spec.md > W3 手順3 と rules.md > BR3.2 | 一括実行のループの進め方が、2つの文書で食い違い、終わり方も決まっていない。W3 は「記述順に決めてくり返し、Skip なら apply_skip で記録して次へ」と書く。BR3.2 は「NextStep のとおり(遷移先があれば、間のジョブをスキップとして記録して、そのジョブへ)」と書く。ところが u2 の apply_outcome が、NextStep の skippedJobIds を、すでに skipped として記録している。そのため、(a) ループが nextJobId へ飛ぶのか、全ジョブを記述順に decide_before_run へ通すのか、(b) 間のジョブを記録するのは apply_outcome だけか、apply_skip でも再び記録するのか、が読み取れない。さらに W3 は、Skip のあとに completed になる場合を扱っていない。個別実行で @end に達した状態(end_reached が true)から pipeline を再開すると、残りは全部 afterEnd の Skip になる。最後のジョブの apply_skip で completed が true になるが、W3 には reset_state も終了コード 0 もなく、「次へ」のあとで、ジョブが尽きたときの終了が定義されていない。一方、BR2.4 は、個別実行の Skip で completed なら reset_state する、と書いている | W3 のループを、1つの方式に決めて書く。たとえば「全ジョブを記述順に decide_before_run へ通す。nextJobId は使わない。間のジョブの記録は apply_outcome が行い、後続の Skip は apply_skip で冪等に確認するだけ」とする。そのうえで、Skip の記録のあとに completed なら reset_state して終了コード 0、という手順を W3 と BR3.2 に足す。ジョブが尽きて completed でないときの扱い(起こらない、という不変条件でもよい)も書く | New |
| R-02 | Major | aidlc/spaces/default/intents/261114-job-conductor-cli/construction/u3-cli/functional-design/rules.md > BR5.2 と BR5.3 | BR5.3 は「終了コードに OS による違いを持ち込まない。OS 固有のコードは、シグナルの受け取りの準備と、標準出力の端末判定だけ」と書く。BR5.2 は UNIX で下位 8 ビットへ丸め(256 は 0 になり、エラーなら 1)、Windows では 32 ビットのままとする。同じジョブの終了コード 256 が、UNIX では 1、Windows では 256 になる。BR5.3 と矛盾し、AC2.5.5・AC3.1.4(3つの OS で同じ結果)の「結果」の範囲も曖昧になる。traceability.json は AC2.3.2 を BR5.1・BR5.2 で OK としているが、この OS による差を、前提として明示していない。契約への反映の一覧にも、この丸めを、利用者から見える E1 の約束として足す項目がない | BR5.3 に、BR5.2 を OS による違いの例外として明記する(OS の仕様による制約)。OS 固有のコードの一覧にも、終了コードの丸めを加える。契約への反映の一覧の E1 に、「UNIX では 0〜255 に丸め、エラーを 0 にしない。Windows は 32 ビット」を足す。AC2.5.5・AC3.1.4 の「同じ結果」が、255 以下の終了コードに限ることを、Assumptions に書く | New |
| R-03 | Minor | aidlc/spaces/default/intents/261114-job-conductor-cli/construction/u3-cli/functional-design/traceability.json > coverage の AC2.5.3 | AC2.5.3(起動オーバーヘッドの p95 100ms を、3つの OS で測る)を、BR6.1(ロケールの判定を遅らせる)だけに対応づけて OK としている。u2 の functional-spec は、起動時間(NFR1)の測定を u3 の責務としているが、u3 に、測定の方法や、起動の経路の予算(たとえば、run の経路に fsync を含む Durable 書き込みが入る位置)を定める規則がない | AC2.5.3 を、BR6.1 に加えて、測定を担う規則(または NFR Requirements への明示の持ち越し)に対応づける。N/A にするなら、どの段階で確定するかを書く | New |
| R-04 | Minor | aidlc/spaces/default/intents/261114-job-conductor-cli/construction/u3-cli/functional-design/rules.md > BR1.1 | `--help`・`-h`・`--version` と、引数なしの起動の扱いが決まっていない。BR1.1 の「知らないオプションは UsageError」を字義どおりに読むと、`--help` も終了コード 68 になる。Reporter の DisplayEvent にも、使い方の案内を標準出力に出す種類がない | `--help`・`--version` の動き(標準出力、終了コード 0 を推奨)と、引数なしの起動を BR1.1 に足す。DisplayEvent に対応する種類を足す | New |
| R-05 | Minor | aidlc/spaces/default/intents/261114-job-conductor-cli/construction/u3-cli/functional-design/rules.md > BR2.1 (3) と BR3.1 | 存在しないジョブIDの run は UsageError で終了コード 68、一致しないパイプライン名の pipeline は定義の誤りで終了コード 64、となっている。どちらも、引数の打ち間違いなのに、終了コードが違う。Q3 の「使い方の誤りは 68」とも、そろっていない | pipeline 名の不一致を 68 にそろえるか、64 のままにする理由を書く。u1-core の判断(不一致は DefinitionError)を使うなら、その理由を Assumptions に残す | New |
| R-06 | Minor | aidlc/spaces/default/intents/261114-job-conductor-cli/construction/u3-cli/functional-design/functional-spec.md > W3 手順2 と rules.md > BR2.5 (1) | 一括実行は、初期ステートを、最初の decide_before_run より前に書く(W3 手順2)。個別実行は、Run のあとに書く(BR2.5(1)、u2 の BR2.9)。空のステートでは、最初のジョブは必ず Run になるため、結果は同じだが、規則が2通りに見える | W3 手順2 を、「最初のジョブの Run のあと」に直す。または、空のステートでは必ず Run になることを理由に書く | New |
| R-07 | Minor | aidlc/spaces/default/intents/261114-job-conductor-cli/construction/u3-cli/functional-design/rules.md > BR4.1 と BR4.2 | シグナルの受け取りの準備は定義の検査の前だが、合図を確かめるのは run_job の中(起動の前)だけである。検査中に届いたシグナルは、定義の誤り(64)、拒否(66)、スキップ(0)のときは、無視されて、終了コードに現れない。BR4.2 の「検査のあいだと…確かめる」という見出しと、手順が合わない。あわせて、起動前に中断される場合は、初期ステートを書いたあとに中断されるため、ステートファイルが残る。この動きは、書かれていない | 検査中に届いたシグナルの扱い(無視する、またはその場で 128 + 番号で終わる)を決めて書く。起動前の中断で、初期ステートが残ることを、規則に書く | New |
| R-08 | Minor | aidlc/spaces/default/intents/261114-job-conductor-cli/construction/u3-cli/functional-design/rules.md > BR7.1 と BR7.3 | watch の表の書き込みの失敗(閉じたパイプ)の終了コードと、convert の標準出力の書き込み失敗(BR7.3 は 67)の、エラーの表示の種類が、DisplayEvent にない(conversionError は ConversionError 用)。SIGPIPE の扱い(Rust は既定で無視し、書き込みの失敗として返る)にも触れていない。watch の書き込み失敗の終了コードも未定である | 出力の書き込み失敗の DisplayEvent を決め、watch の終了コード(65 にはしない)を決める。SIGPIPE を無視して、書き込みの失敗として扱うことを書く | New |
| R-09 | Minor | aidlc/spaces/default/intents/261114-job-conductor-cli/construction/u3-cli/functional-design/rules.md > BR6.5 | 色の条件が「NO_COLOR が未設定のとき」になっている。NO_COLOR の慣例は、設定されていて空でないときに無効にする。また、Windows の端末で色の制御文字を有効にする操作も、OS 固有のコードだが、BR5.3 の OS 固有のコードの一覧にない | 条件を「NO_COLOR が設定され、かつ空でない」に直す。Windows で、制御文字を有効にできないときは、色を使わないことを書く | New |
| R-10 | Minor | aidlc/spaces/default/intents/261114-job-conductor-cli/construction/u3-cli/functional-design/functional-spec.md > 契約への反映が必要な点 の E2 | ジョブの表の名前を `jobs` から `job` に変える(Q1: A)対応を、u3 の Code Generation の中で、u1-core のコードを直す手順として行う、としている。E2 と C1 の所有は U1 で、契約の変更は所有する Unit が提案する、という契約の決まりと違う。u1-core の functional-design の成果物(entities、rules の `jobs` の記述)は、直さないままになる。反映までの正が、各 Unit の code-summary という記録に散らばる | E2 の変更を、U1 が所有する契約の変更として扱う(contract-summary への反映の依頼を、記録に残す)。u1-core の entities と rules の `jobs` の記述を、どの段階で直すかを、手順に書く | New |
| R-11 | Minor | aidlc/spaces/default/intents/261114-job-conductor-cli/construction/u3-cli/functional-design/rules.md > BR7.1 | watch は、定義を読まず、check_consistency も行わないため、別の定義のステートでも、そのまま表示する。この選択は意図したものとして書かれているが、表示された内容が、現在の定義と合っているかを、利用者が判断できない(ステートのパイプライン名を表示しない) | 表の見出しの上に、ステートのパイプライン名とステートファイルのパスを1行で出すことを検討する。出さないなら、その理由を Assumptions に書く | New |

### Validation Tool Results

| Tool | Result | Interpretation |
|---|---|---|
| traceability 照合(手作業) | 31 件の upstream_ids がすべて coverage にあり、すべての BR(BR1.1〜BR7.3)が、forward か reverse のどちらかに現れる。N/A は AC2.5.1、AC2.5.2 のみで、u2 に委ねる理由は妥当 | 構造は完全。AC2.5.3 の対応づけだけが弱い(Minor) |
| u1・u2・u4 の呼び出し順の照合 | decide_before_run(resume_job_id、end_reached)→ Run のあと initial_state(Fast)→ run_job → classify_and_decide(runMode)→ apply_outcome → write_state(Durable)→ completed なら reset_state、Reject はステートを変えない、という個別実行の順序は、u1 の W2・W3、u2 の W6・BR2.9 と合う | 個別実行は整合。一括実行のループだけが曖昧(Major) |

### Summary

個別実行の順序、終了コードの決め方、シグナルの扱い、契約の差分の一覧は、u1・u2・u4 の設計とおおむね整合し、AC の割り当ても妥当である。重大な問題は見つからなかった。ただし、一括実行のループの進め方と終わり方(W3 と BR3.2)、および終了コードの OS による違い(BR5.2 と BR5.3)は、実装の前に直しておく必要がある。この2点以外は、Minor の補足である。
