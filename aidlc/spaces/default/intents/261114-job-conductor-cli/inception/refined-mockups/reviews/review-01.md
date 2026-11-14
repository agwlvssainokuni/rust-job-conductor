## Review

**Verdict:** READY
**Reviewer:** aidlc-product-lead-agent
**Date:** 2026-11-14T14:57:47Z
**Iteration:** 1
**Verdict Dictated:** no

### Findings

| ID | Severity | Location | Finding | Required action | Status |
|---|---|---|---|---|---|
| R-01 | Major | aidlc/spaces/default/intents/261114-job-conductor-cli/inception/refined-mockups/interaction-spec.md > 2. 結果の分類 / 3. スケジューラに返す終了コード | Q4 で「0 を書かなければ 0 でも成功にならない」と決めたため、`success_exit_codes = [3]` のジョブが 0 で終わると「エラー」に分類される。ところが 3 章は、エラーならジョブの終了コードをそのまま返すとしており、その値が 0 になる。スケジューラには成功として見え、エラー停止もステートへのエラー記録も起きない。この矛盾は仕様のどこにも解決されていない。 | 0 で終わったが成功に含まれないジョブの扱いを決めて書く。たとえば、エラーとして停止し、`conductor` は非0の固定値を返す、とする。3 章の表にその行を加える。 | New |
| R-02 | Major | aidlc/spaces/default/intents/261114-job-conductor-cli/inception/refined-mockups/interaction-spec.md > 2. 結果の分類(および mockups.md > 1. 定義ファイル) | Q4 の「書けば書いた値だけが成功」は、requirements の FR2.2(0 または成功コードは成功)と食い違う。同じ食い違いは FR2.5 の説明にもある。この食い違いは Assumptions に挙がっていない。Q8 は要件との差分として明記されているが、Q4 は同じ扱いを受けていない。要件のままだと、`[0, 3]` を例にした定義が、その意味で実装されるかどうか分からない。 | Q4 の内容が FR2.2 の更新を要することを、Assumptions & Open Questions に明記する。承認時に、要件側を直すかどうかを人が決められる形にする。 | New |
| R-03 | Minor | aidlc/spaces/default/intents/261114-job-conductor-cli/inception/refined-mockups/mockups.md > 4. `conductor watch` | 例では、`slack_notify_and_skip`(定義の最後のジョブ)が success で終わった後の状態を表示している。FR4.2 では、最後のジョブが終わった時点でステートが初期化され、`@end` への到達も完了とされる。このため、この例の表示はステートが空になるはずで、実装者とテスターが誤読する。 | 実行途中の状態(たとえば最後のジョブがまだ pending)に例を直すか、完了後は空になる旨を注記する。 | New |
| R-04 | Minor | aidlc/spaces/default/intents/261114-job-conductor-cli/inception/refined-mockups/mockups.md > 2.5 定義ファイルの誤り | 例の誤りは `conductor.toml:21` の `transform` が `extract` へ遷移する、というもの。1 章の定義例には `transform` に遷移がなく、21 行目もそれと対応しない。例がどのファイルに対するものか読み取れない。 | 定義例の誤りを実際に起こした版にして行番号を合わせるか、別の定義ファイルでの例だと注記する。 | New |
| R-05 | Minor | aidlc/spaces/default/intents/261114-job-conductor-cli/inception/refined-mockups/interaction-spec.md > 1. オプション | `watch` と `reset` に付ける `-v` が何を表示するのかが、どこにも書かれていない。一方、`reset` の既定は何も表示しない。 | `-v` の効果が無い、あるいは何を出すかを一行で書く。または、`-v` の対象を `run`、`pipeline` に絞る。 | New |

### Summary

CLI のため画面の代わりにコマンドの入出力と定義ファイルで具体化する方針は妥当で、Q1〜Q8 の回答は概ね反映されている。Q8(シグナル転送)が要件に未反映であることも、前提として明記されている。実装前に詰めるべきは、成功コードを限定したときの 0 の扱いと、要件 FR2.2 との食い違いの明記である。この 2 点が Major で、どちらも小さな修正で足りるため、READY とする。
