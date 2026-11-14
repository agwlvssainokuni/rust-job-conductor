## Review

**Verdict:** READY
**Reviewer:** aidlc-product-lead-agent
**Date:** 2026-11-14T15:00:27Z
**Iteration:** 1
**Verdict Dictated:** no

### Findings

| ID | Severity | Location | Finding | Required action | Status |
|---|---|---|---|---|---|
| R-01 | Major | aidlc/spaces/default/intents/261114-job-conductor-cli/inception/refined-mockups/interaction-spec.md > 2. 結果の分類 / 3. スケジューラに返す終了コード | Q4 で「0 を書かなければ 0 でも成功にならない」と決めたため、`success_exit_codes = [3]` のジョブが 0 で終わると「エラー」に分類される。ところが 3 章は、エラーならジョブの終了コードをそのまま返すとしており、その値が 0 になる。スケジューラには成功として見え、エラー停止もステートへのエラー記録も起きない。この矛盾は仕様のどこにも解決されていない。 | 0 で終わったが成功に含まれないジョブの扱いを決めて書く。たとえば、エラーとして停止し、`conductor` は非0の固定値を返す、とする。3 章の表にその行を加える。 | Resolved (reviewer) |
| R-02 | Major | aidlc/spaces/default/intents/261114-job-conductor-cli/inception/refined-mockups/interaction-spec.md > 2. 結果の分類(および mockups.md > 1. 定義ファイル) | Q4 の「書けば書いた値だけが成功」は、requirements の FR2.2(0 または成功コードは成功)と食い違う。同じ食い違いは FR2.5 の説明にもある。この食い違いは Assumptions に挙がっていない。Q8 は要件との差分として明記されているが、Q4 は同じ扱いを受けていない。要件のままだと、`[0, 3]` を例にした定義が、その意味で実装されるかどうか分からない。 | Q4 の内容が FR2.2 の更新を要することを、Assumptions & Open Questions に明記する。承認時に、要件側を直すかどうかを人が決められる形にする。 | Resolved (reviewer) |
| R-03 | Minor | aidlc/spaces/default/intents/261114-job-conductor-cli/inception/refined-mockups/mockups.md > 4. `conductor watch` | 例では、`slack_notify_and_skip`(定義の最後のジョブ)が success で終わった後の状態を表示している。FR4.2 では、最後のジョブが終わった時点でステートが初期化され、`@end` への到達も完了とされる。このため、この例の表示はステートが空になるはずで、実装者とテスターが誤読する。 | 実行途中の状態(たとえば最後のジョブがまだ pending)に例を直すか、完了後は空になる旨を注記する。 | Resolved (reviewer) |
| R-04 | Minor | aidlc/spaces/default/intents/261114-job-conductor-cli/inception/refined-mockups/mockups.md > 2.5 定義ファイルの誤り | 例の誤りは `conductor.toml:21` の `transform` が `extract` へ遷移する、というもの。1 章の定義例には `transform` に遷移がなく、21 行目もそれと対応しない。例がどのファイルに対するものか読み取れない。 | 定義例の誤りを実際に起こした版にして行番号を合わせるか、別の定義ファイルでの例だと注記する。 | Resolved (reviewer) |
| R-05 | Minor | aidlc/spaces/default/intents/261114-job-conductor-cli/inception/refined-mockups/interaction-spec.md > 1. オプション | `watch` と `reset` に付ける `-v` が何を表示するのかが、どこにも書かれていない。一方、`reset` の既定は何も表示しない。 | `-v` の効果が無い、あるいは何を出すかを一行で書く。または、`-v` の対象を `run`、`pipeline` に絞る。 | Resolved (reviewer) |
| R-06 | Minor | aidlc/spaces/default/intents/261114-job-conductor-cli/inception/requirements-analysis/requirements.md > FR2.5、および inception/refined-mockups/mockups.md > 2.4 エラー | FR2.5 と mockups 2.4 は「エラーのときはジョブの終了コードをそのまま返す」としか書いていない。改訂後の FR2.2 では、成功コード外の 0 は 1、起動不可は 127、シグナル終了は 128 + シグナル番号を返す。FR2.5 の文だけを読むと、これらの例外と食い違って見える。interaction-spec の終了コード表だけが全体を正しく書いている。 | FR2.5 と mockups 2.4 に「ジョブ自身の終了コードがあるエラーのとき」と限定するか、FR2.2 の例外を参照する一文を足す。 | New |
| R-07 | Minor | aidlc/spaces/default/intents/261114-job-conductor-cli/inception/refined-mockups/mockups.md > 2 節・3 節 | シグナル転送(FR2.9)と、成功コード外の 0 → 終了コード 1 の体験例が、mockups にない。interaction-spec と要件・ストーリーにはある。`pipeline` 実行中のシグナルの扱いも、どこにも書かれていない。 | mockups に、成功コード外の 0 で終わる例と、シグナルを受けたときの動きを 1 行ずつ足す。`pipeline` でも同じ動きになるかを明記する。 | New |

> R-01 Reviewer note: 成功コードに含まれない 0 は、終了コード 1(固定)と決まった。interaction-spec の「結果の分類」と「終了コード表」に行があり、要件 FR2.2、AC2.1.6、US2.1 とも一致している。

> R-02 Reviewer note: 要件 FR2.2 を「書いた値だけが成功」に書き換え、受入基準も追加された。mockups と interaction-spec の Assumptions にも、承認時の確認を受けて反映した旨が書かれている。矛盾は残っていない。

> R-03 Reviewer note: watch の例は途中のステートだと明記され、最後のジョブ後は一覧が空になる旨の注記もある。

> R-04 Reviewer note: 2.5 節の例は「1 節の例とは別の誤った定義」と明記され、行番号と原因が対応している。

> R-05 Reviewer note: `-v` は `run` と `pipeline` だけが受け付けると、mockups の 5 節と interaction-spec のオプション表に書かれている。
