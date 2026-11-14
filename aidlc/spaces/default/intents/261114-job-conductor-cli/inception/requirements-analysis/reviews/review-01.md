## Review

**Verdict:** NOT-READY
**Reviewer:** aidlc-product-lead-agent
**Date:** 2026-11-14T14:26:22Z
**Iteration:** 1
**Verdict Dictated:** no

### Findings

| ID | Severity | Location | Finding | Required action | Status |
|---|---|---|---|---|---|
| R-01 | Major | aidlc/spaces/default/intents/261114-job-conductor-cli/inception/requirements-analysis/requirements.md > FR1.3 と FR3.2 / FR2.5 | FR1.3 は「結果(成功・警告・エラー)ごと」に遷移先を指定できるとするが、FR3.2 はエラーなら必ず停止、FR2.5 はエラー時にジョブの終了コードを返すとしている。エラー時の遷移先を定義に書いた場合に、ジャンプするのか停止するのかが決まらない。個別実行と一連の実行で挙動が分かれる可能性もある。 | エラー結果に遷移先を指定できるかを決める。指定できないなら FR1.3 からエラーを外し、FR1.4 の検証で拒否する。指定できるなら FR3.2 と FR2.5 に例外を書く。どちらの場合も Given/When/Then を追加する。 | New |
| R-02 | Major | aidlc/spaces/default/intents/261114-job-conductor-cli/inception/requirements-analysis/requirements.md > Open Questions(遷移ルール未記載時の既定動作)、FR1、FR3 | 成功時の遷移先や、遷移ルールを省略した結果の扱いが Open Question のまま残っている。ジョブの実行順(定義ファイルの記述順か)、パイプラインがどのジョブで構成されるか、`pipeline <pipeline-name>` が1ファイルに複数パイプラインを許すか、定義ファイルの場所(既定パス・指定方法)も、どの要件にも書かれていない。これらは FR1〜FR3 の核心で、後続ステージに回すと FR3 の受け入れ基準(「3つが順に実行」)を実装も検証もできない。 | 少なくとも次を要件として決める。(a) 遷移未指定時の既定(定義順の次のジョブへ進む、など)。(b) ジョブの順序の定義方法。(c) パイプラインとジョブの関係と、1ファイルに置けるパイプライン数。(d) 定義ファイルの探索方法。それぞれに受け入れ基準を添える。 | New |
| R-03 | Major | aidlc/spaces/default/intents/261114-job-conductor-cli/inception/requirements-analysis/requirements.md > FR4.2 と Open Questions(「最後まで終わった」の判定) | FR4.2 の自動初期化と、その受け入れ基準(「最後のジョブが終わる」)は、「最後」の定義に依存するが、これが未決のまま。個別実行では conductor がパイプライン全体の完了を直接知らない。最後のジョブがスキップ対象になる場合や、遷移の終点が複数ある場合は判定がぶれる。判定を誤ると、ステートが残って次回の実行が誤ってスキップされるか、早く消えて再開(FR4.3)が壊れる。 | 完了の判定を要件として定める。例として、定義上の最後のジョブの完了、または遷移の終点への到達とし、最後のジョブがスキップされる場合の扱いも定める。FR4.2 の受け入れ基準を、スキップ経由の完了を含めて書き直す。 | New |
| R-04 | Major | aidlc/spaces/default/intents/261114-job-conductor-cli/inception/requirements-analysis/requirements.md > FR4(ステート)、FR4.3 | ステートの保存後に定義ファイルが変わった場合(ジョブIDの削除・改名・順序変更)の扱いがない。ステートに存在しないIDや、定義に存在しないIDが残ると、再開(FR4.3)やスキップ判定が誤った結果になりうる。ステートがパイプラインごと、定義ごとのどちらに紐づくかも、どこにも書かれていない。 | ステートと定義の不一致を検知する要件を追加する。不一致なら実行せずエラーにするか、ステートを無効にして先頭から実行するかを決める。ステートの対象単位(パイプラインごとか、定義ファイルごとか)も定め、受け入れ基準を添える。 | New |
| R-05 | Minor | aidlc/spaces/default/intents/261114-job-conductor-cli/inception/requirements-analysis/requirements.md > FR1.2、FR2.2 | 同じ終了コードが `warning_exit_codes` と成功コードの両方に書かれた場合や、0 を警告に指定した場合の扱いが決まっていない。成功コードの定義ファイル上のキー名もない。 | 重複や 0 の再指定を FR1.4 の検証でエラーにするか、優先順位を決める。受け入れ基準を追加する。 | New |
| R-06 | Minor | aidlc/spaces/default/intents/261114-job-conductor-cli/inception/requirements-analysis/requirements.md > FR2.1、FR2.2、FR2 受け入れ基準(存在しないコマンド) | コマンドを起動できない場合と、シグナルで終了して終了コードがない場合に、どの終了コードを返すか、エラーとして分類しステートに記録するかが決まっていない。FR2.5 の「ジョブの終了コードをそのまま返す」は、これらの場合は適用できない。作業ディレクトリと環境変数を引き継ぐかも書かれていない。 | 起動失敗とシグナル終了の分類と返す終了コードを決める。作業ディレクトリと環境変数の扱いも一言で書く。 | New |
| R-07 | Minor | aidlc/spaces/default/intents/261114-job-conductor-cli/inception/requirements-analysis/requirements.md > NFR1 | 「通常規模の定義ファイル」と測定条件が未確定で、統計の取り方(最大値か p95 か、繰り返し回数)もない。受け入れ基準が実際には検証できない。Windows のプロセス起動も含むため、3つの OS で同じ値かも不明。 | 定義ファイルの規模(ジョブ数など)、測定の統計、3つの OS で同じ値かどうかを、要件か、後続ステージで確実に決める項目として明記する。 | New |
| R-08 | Minor | aidlc/spaces/default/intents/261114-job-conductor-cli/inception/requirements-analysis/requirements.md > 受け入れ基準全般(FR2.1、FR2.6〜FR2.8、FR4.1、FR4.4、NFR2、NFR4、NFR5) | これらのサブ要件には対応する Given/When/Then がなく、合否が判定できない。特に、FR2.1(シェルを介さない)は、シェルのメタ文字を含む引数が解釈されないことで検証できる。FR2.6 は、既定で表示がなく、エラー時のみ表示されることで検証できる。FR4.4 は中断時にファイルが壊れないことで検証できる。 | 各サブ要件に、少なくとも1つの Given/When/Then(または測定可能な基準)を追加する。 | New |
| R-09 | Minor | aidlc/spaces/default/intents/261114-job-conductor-cli/inception/requirements-analysis/requirements.md > FR1.4 の出典、FR7.1 | FR1.4 の `[Q5]` は、この質問ファイルの Q5(同時実行)と紛らわしいが、実際は wireframes 側の Q5 を指している。FR7.1 は「同じ内容の YAML」とするが、コメントや記述順など、変換で失われるものが許容か書かれていない。 | FR1.4 の出典を wireframes 側であると明確にする(`[Q5]` を外すなど)。FR7.1 に、保持する内容の範囲(意味が同じであること、コメントは保持しない、など)を書く。 | New |

### Summary

要件は全体として出典が付き、Q1〜Q9 の確認済み回答(排他制御なし、タイムアウトなし、自動再試行なし、シェルを介さない起動、100ms)とも矛盾していない。traceability.json の ID も requirements.md と一致している。ただし、エラー時の遷移と停止の食い違い、遷移未指定時の既定動作・ジョブ順・定義ファイルの場所、パイプライン完了の判定、ステートと定義の不一致は、実装者が質問に戻らずに実装できる水準になく、承認前に決める必要がある。
