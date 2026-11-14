## Review

**Verdict:** READY
**Reviewer:** aidlc-architecture-reviewer-agent
**Date:** 2026-11-14T18:41:10Z
**Iteration:** 1
**Verdict Dictated:** no

### Findings

| ID | Severity | Location | Finding | Required action | Status |
|---|---|---|---|---|---|
| R-01 | Major | aidlc/spaces/default/intents/261114-job-conductor-cli/inception/units-generation/unit-of-work-dependency.md > 「データ型のみの依存と、Unit の独立性」および yaml の depends_on | U1 と U2 は、型の形で互いを参照している(Flow は U2 の JobState・JobExecution を引数に取り、U2 は U1 の PipelineDefinition・JobDefinition を参照する)。yaml では両者とも `depends_on: []` であり、「循環はない」「互いに依存しない」と書かれているが、型の層では U1 と U2 の相互依存になっており、Rust で別クレートに分ければ循環になる。「各 Unit は自分のコードと依存先だけでビルド・テストが通る」(Q2)という前提も、共有型の置き場が決まるまで成り立たない。この判断は次の Contract Design に先送りされている。 | 次のいずれかを今のうちに選ぶか、選択肢と制約を明記して Contract Design の入力にする。(a)共有型だけを持つ辺を、独立した Unit(または U1 か U2 の一方が所有する型)として DAG に明示する、(b)片方向にそろえる(例: U2 が U1 の型に依存し、Flow は U1 側で自前の入力型を定義して U3 が写し替える)、(c)共有型を別 Unit に切り出す。少なくとも「共有型は依存の辺ではない」という言い切りを、「未解決の潜在的な依存」に改める。 | New |
| R-02 | Major | aidlc/spaces/default/intents/261114-job-conductor-cli/inception/units-generation/unit-of-work-dependency.md > 「Unit どうしの接点」表、「データ型のみの依存」の行 | 表では U2 が U1 から受け取るのは PipelineDefinition の「ジョブIDと記述順だけ」とあるが、上流の components.md では JobRunner が JobDefinition(コマンドと、成功・警告の終了コードの扱い)に依存し、JobExecution と JobState も JobDefinition を参照している。U2 から U1 への型の参照は、表の記述より広い。 | 接点の表を components.md の「データ型のみの依存」の表に合わせ、U2→U1 は JobDefinition(command、各終了コード、transitions の範囲)まで含むことを明記する。上の finding の共有型の切り分けに、この範囲を反映する。 | New |
| R-03 | Minor | aidlc/spaces/default/intents/261114-job-conductor-cli/inception/units-generation/unit-of-work.md > U2 の「対応するストーリー」 | U2 のストーリー一覧に US3.2 がないが、story-map では US3.2 が U1;U2;U3 に割り当てられ、traceability.json でも U1 が主でその他は U2 を含む。U2 の件数(story-map では 10)とも合わない(一覧は 9 本)。 | U2 の一覧に US3.2 を加えるか、US3.2 から U2 を外す理由を story-map に書く。 | New |
| R-04 | Minor | aidlc/spaces/default/intents/261114-job-conductor-cli/inception/units-generation/unit-of-work.md > 冒頭、単一バイナリの記述 | 「requirements の NFR2」が文の末尾に断片として置かれ、文として終わっていない。 | 「(NFR2)」のように文に組み込む。 | New |
| R-05 | Minor | aidlc/spaces/default/intents/261114-job-conductor-cli/inception/units-generation/unit-of-work.md > 一覧の「種類(kind)」U3 | U3 は「service」だが、実体はデプロイされるサーバではなく、単一バイナリの CLI である。パッケージングを独立 Unit にしない前提(Assumptions)と合わせ、後続ステージで `service` を「デプロイされる実行ファイル」として読むことが前提になっている。 | 後続ステージで混乱が出ないよう、kind の定義(実行ファイル=service)を yaml の隣にも一行で書く。 | New |

### Validation Tool Results

| Tool | Result | Interpretation |
|---|---|---|
| 手作業の確認(依存の yaml) | 名前は一意、kind は有効、depends_on の名前は宣言済み、自己依存なし、辺は U3→U1/U2/U4 のみ | 呼び出しの依存としては非循環。型の依存は上の Major を参照 |
| 手作業の確認(部品の割り当て) | 7つの部品がそれぞれちょうど1つの Unit に割り当てられ、エンティティの所有も components.md と一致 | 問題なし |
| 手作業の確認(ストーリー) | US1.1〜US7.1 の17本すべてが story-map と traceability.json に存在し、primary は各行で1つ、story-map の対象に含まれている | U2 の一覧の US3.2 欠落(Minor)を除き一貫している |

### Summary

呼び出しの依存の DAG は非循環で、7部品・17ストーリーの割り当ては完全で、実装順序の推奨は含まれていない。最大の懸念は、U1 と U2 の「データ型のみの依存」が、実際には相互の型参照であり、「独立してビルドできる」という Q2 の前提を、共有型の置き場が決まるまで満たさない点である。これは Contract Design に先送りされていることが明記されているため、承認後にその判断を必ず行う前提で READY とする。
