## Review

**Verdict:** READY
**Reviewer:** aidlc-architecture-reviewer-agent
**Date:** 2026-11-14T19:54:28Z
**Iteration:** 1
**Verdict Dictated:** no

### Findings

| ID | Severity | Location | Finding | Required action | Status |
|---|---|---|---|---|---|
| R-01 | Major | aidlc/spaces/default/intents/261114-job-conductor-cli/inception/units-generation/unit-of-work-dependency.md > 「データ型のみの依存と、Unit の独立性」および yaml の depends_on | U1 と U2 は、型の形で互いを参照している(Flow は U2 の JobState・JobExecution を引数に取り、U2 は U1 の PipelineDefinition・JobDefinition を参照する)。yaml では両者とも `depends_on: []` であり、「循環はない」「互いに依存しない」と書かれているが、型の層では U1 と U2 の相互依存になっており、Rust で別クレートに分ければ循環になる。「各 Unit は自分のコードと依存先だけでビルド・テストが通る」(Q2)という前提も、共有型の置き場が決まるまで成り立たない。この判断は次の Contract Design に先送りされている。 | 次のいずれかを今のうちに選ぶか、選択肢と制約を明記して Contract Design の入力にする。(a)共有型だけを持つ辺を、独立した Unit(または U1 か U2 の一方が所有する型)として DAG に明示する、(b)片方向にそろえる(例: U2 が U1 の型に依存し、Flow は U1 側で自前の入力型を定義して U3 が写し替える)、(c)共有型を別 Unit に切り出す。少なくとも「共有型は依存の辺ではない」という言い切りを、「未解決の潜在的な依存」に改める。 | Resolved (reviewer) |
| R-02 | Major | aidlc/spaces/default/intents/261114-job-conductor-cli/inception/units-generation/unit-of-work-dependency.md > 「Unit どうしの接点」表、「データ型のみの依存」の行 | 表では U2 が U1 から受け取るのは PipelineDefinition の「ジョブIDと記述順だけ」とあるが、上流の components.md では JobRunner が JobDefinition(コマンドと、成功・警告の終了コードの扱い)に依存し、JobExecution と JobState も JobDefinition を参照している。U2 から U1 への型の参照は、表の記述より広い。 | 接点の表を components.md の「データ型のみの依存」の表に合わせ、U2→U1 は JobDefinition(command、各終了コード、transitions の範囲)まで含むことを明記する。上の finding の共有型の切り分けに、この範囲を反映する。 | Resolved (reviewer) |
| R-03 | Minor | aidlc/spaces/default/intents/261114-job-conductor-cli/inception/units-generation/unit-of-work.md > U2 の「対応するストーリー」 | U2 のストーリー一覧に US3.2 がないが、story-map では US3.2 が U1;U2;U3 に割り当てられ、traceability.json でも U1 が主でその他は U2 を含む。U2 の件数(story-map では 10)とも合わない(一覧は 9 本)。 | U2 の一覧に US3.2 を加えるか、US3.2 から U2 を外す理由を story-map に書く。 | Resolved (reviewer) |
| R-04 | Minor | aidlc/spaces/default/intents/261114-job-conductor-cli/inception/units-generation/unit-of-work.md > 冒頭、単一バイナリの記述 | 「requirements の NFR2」が文の末尾に断片として置かれ、文として終わっていない。 | 「(NFR2)」のように文に組み込む。 | Resolved (reviewer) |
| R-05 | Minor | aidlc/spaces/default/intents/261114-job-conductor-cli/inception/units-generation/unit-of-work.md > 一覧の「種類(kind)」U3 | U3 は「service」だが、実体はデプロイされるサーバではなく、単一バイナリの CLI である。パッケージングを独立 Unit にしない前提(Assumptions)と合わせ、後続ステージで `service` を「デプロイされる実行ファイル」として読むことが前提になっている。 | 後続ステージで混乱が出ないよう、kind の定義(実行ファイル=service)を yaml の隣にも一行で書く。 | Resolved (reviewer) |
| R-06 | Minor | aidlc/spaces/default/intents/261114-job-conductor-cli/inception/units-generation/unit-of-work.md > U1 / U2 の「所有するエンティティ」と「共有する型の置き場」 | 型の定義場所と所有の分け方が読み手に紛らわしい。U2 は JobExecution、PipelineState、JobState を「所有」するとしているが、これらの型の定義は U1 にある。PipelineState の識別子 `definitionKey` は U2 の Functional Design で決める予定で(U2 の実装の注意)、型の変更が U1 に及ぶ。U1 だけでビルドとテストが通る(Q2)ことは保たれるが、変更が及ぶ範囲は明記されていない。 | U2 の Functional Design で型の形が変わるとき、U1 の型定義も更新することを、一文で書き添える。または、Contract Design で決める項目に挙げる。 | New |
| R-07 | Minor | aidlc/spaces/default/intents/261114-job-conductor-cli/inception/units-generation/unit-of-work-dependency.md > 並行にできる Unit | 冒頭は、推奨の実装順序は Delivery Planning で決めると述べている。しかし最終行は「依存関係の順に1つずつ進める」と進め方を述べており、順序の推奨にあたる。U2 は U1 の型の定義が済めば進められる、という記述も、順序の指示に近い。 | 「実際の進め方」の行は削除するか、Delivery Planning で決めると書き換える。 | New |
| R-08 | Minor | aidlc/spaces/default/intents/261114-job-conductor-cli/inception/units-generation/unit-of-work.md > U1「共有する型の置き場」 | 文末の句点が抜けている。「U1 に依存する。 unit-of-work-dependency を参照」で、参照文が文の体を成していない。 | 「(unit-of-work-dependency を参照)」のように整える。 | New |

> R-01 Reviewer note: 共有する型は U1 に置くと確定し、「辺ではない」という書き方は消えた。u2-runtime は u1-core に依存し、DAG に循環はない。yaml の `depends_on` も本文と一致している。

> R-02 Reviewer note: 接点の表に、JobRunner が JobDefinition からコマンドと終了コードの扱いを受け取ること、StateStore の照合に PipelineDefinition のジョブIDと記述順を使うことが書かれた。components.md のデータ型のみの依存の表とも合っている。

> R-03 Reviewer note: U2 のストーリー一覧は US3.2 を含み、10本になった。story-map の U2 の行数(10)と一致する。

> R-04 Reviewer note: 冒頭の文は「NFR2 による。」で終わっている。

> R-05 Reviewer note: `service` と `library` の意味が yaml の直後に書かれ、U3 が `service` である理由も示されている。

### Validation Tool Results

| Tool | Result | Interpretation |
|---|---|---|
| 手動の確認(DAG) | 合格 | u1-core と u4-converter は依存なし。u2-runtime は u1-core に依存する。u3-cli は u1-core、u2-runtime、u4-converter に依存する。循環はなく、yaml は本文と一致する。 |
| 手動の確認(story-map と traceability.json) | 合格 | 17本のストーリーがすべて割り当てられている。件数は U1 が10、U2 が10、U3 が9、U4 が1で、各 Unit の説明と一致する。traceability.json の 17 の ID と target は、story-map の主な Unit の記述と一致する。 |
| 手動の確認(components.md との整合) | 合格 | 型の置き場を U1 にしても、components.md が定めるエンティティの所有と振る舞いは変わらない。U2、U3 から U1 への辺は、components.md のデータ型のみの依存の表で説明がつく。 |

### Summary

前回の R-01 から R-05 はすべて解消された。型の置き場を U1 に決めたことで、U1 と U2 のあいだの循環は消え、DAG、yaml、story-map、traceability.json の整合も取れている。残るのは、型の所有と定義場所の説明、順序の推奨の一文、文末の整えのみで、いずれも Minor であり、実装を妨げない。
