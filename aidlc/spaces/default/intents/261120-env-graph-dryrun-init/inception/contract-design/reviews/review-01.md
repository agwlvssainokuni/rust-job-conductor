## Review

**Verdict:** READY
**Reviewer:** aidlc-architecture-reviewer-agent
**Date:** 2026-11-21T12:01:45Z
**Iteration:** 1
**Verdict Dictated:** no

### Findings

| ID | Severity | Location | Finding | Required action | Status |
|---|---|---|---|---|---|
| R-01 | Major | aidlc/spaces/default/intents/261120-env-graph-dryrun-init/inception/contract-design/contract-summary.md > C2 | C2 は「次のジョブを決める判断」(入力: job_index、classification)だけを公開する。しかし components.md の CorePlan は CoreFlow から「起動前の判断」と「再開位置」も使い、CoreGraph の FlowEdge は kind、skipped_jobs、explicit を持つ。C2 はこれらを含まない。u3 が u2 に依存する理由(unit-of-work-dependency.md の「辺の判断の入口」)を、C2 の入出力だけでは実装できない。「純粋な関数で失敗しない」という記述も、起動前の判断と再開位置を含めた範囲では確かめられない。 | C2 の入出力に、起動前の判断と再開位置の決定を加える。または、それらは既存の公開済みの関数で、契約の対象外であることを明記する。辺の取り出し(FlowEdge の項目)も、C2 に含めるか、C6 の内部であることを記す。 | New |
| R-02 | Major | aidlc/spaces/default/intents/261120-env-graph-dryrun-init/inception/contract-design/contract-summary.md > C7(と C6) | C7 は「出力の構造(どの項目が、どの順に出るか)を、契約として固定する」と述べるが、項目と順序そのものを書いていない。fenced spec もなく、`--assume` の模擬の出力、再開位置が無いときの出力、エラー時(FR3.3)の出力も示さない。open questions に C7 の項目もない(C6 の具体例だけがある)。固定すると宣言した契約の中身が空で、u3 の開発者が推測せずに作れない。C6 も同様に、fenced spec がなく散文だけである。 | C7 の出力の項目と順序を、spec のブロックで定義する(再開位置、ジョブごとの扱い、分割後のコマンド、環境変数の名前、リトライの設定、`--assume` の仮定)。具体の例は、機能の設計に回すなら、open questions に C7 の行を足して、u3 をブロック対象にする。C6 も、spec のブロックか、期待出力を決める行を open questions に足す。 | New |
| R-03 | Minor | aidlc/spaces/default/intents/261120-env-graph-dryrun-init/inception/contract-design/contract-summary.md > C1 errors、C3/C4 rejects | 要件 FR1.8(ファイルが無い、読めない)と FR1.9(1 MiB、10,000 項目、NUL を含む値)の誤りが、C1 の errors と C4 の rejects に無い。C4 の `name_rule` は `pipeline.env` にだけ書かれ、`job.env` には無い(FR1.2 は両方)。`env_file` の配列の要素が文字列でない場合も rejects に無い。 | C1 の errors に、ファイルの欠落と読み取り不可、上限超過、NUL を加える。C4 の `job.env` にも `name_rule` を書く。`env_file` の要素の型の拒否を rejects に足す。 | New |
| R-04 | Minor | aidlc/spaces/default/intents/261120-env-graph-dryrun-init/inception/contract-design/contract-summary.md > 契約の表(共有する型) | unit-of-work-dependency.md の連携の点には「共有する型(CoreTypes)」があり、U1 が env と env_file の項目を U2、U3、U4 に提供する。しかし契約の表に対応する行がなく、DAG には u2 から u1 への辺もない(u2 は u1 に依存しない)。u2 は新しい型の項目を持つ CoreTypes を共有するため、依存の記述と食い違う。同じく「メッセージのカタログ」の契約もない。 | 共有する型とメッセージのカタログを、契約の表の行として足す(持ち主と変更の規則つき)。または、これらはユニット間の契約ではなく、同じクレート内の編集の調整であると、明記する。 | New |
| R-05 | Minor | aidlc/spaces/default/intents/261120-env-graph-dryrun-init/inception/contract-design/contract-summary.md > C5 と Open questions | `.env` の誤りの終了コード(FR1.8)が C5 の表にない(open questions にはある)。`--assume` は `--dry-run` なしでは使えるのか、`graph --run-id` は `--state` なしで許されるのか、`--dry-run` でも `.env` の誤りを報告するのか、といった組み合わせの規則も未記載である。 | C5 に、`.env` の誤りの終了コードの行(決定待ちと明記)と、オプションの併用の規則を足す。決められない分は、open questions に加える。 | New |

### Validation Tool Results

| Tool | Result | Interpretation |
|---|---|---|
| (none listed) | 読んで確認した | 依存の DAG の 3 つの辺(u1→u3、u2→u3、u1→u4)は、C1、C2、C3 に対応する。循環はない。FR1.x の優先順位、予約名、旧版の拒否(FR1.13)、FR4.2 の終了コード 69、FR2.6 の終了コード 64 は、要件と一致する。 |

### Summary

ユニット間の 3 つの辺には、すべて契約があり、要件(FR1 の優先順位と予約名、FR4 の終了コード 69、FR2.6 の 64)とも矛盾しない。ただし、C2 は components.md の CorePlan が使う範囲(起動前の判断と再開位置)を覆わず、C7 は固定すると宣言した出力の構造を定義していない。この 2 点(Major)は、承認の前か機能の設計の入口で、補うことを勧める。
