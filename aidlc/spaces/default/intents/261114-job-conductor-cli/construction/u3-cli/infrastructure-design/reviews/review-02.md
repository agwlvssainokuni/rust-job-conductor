## Review

**Verdict:** READY
**Reviewer:** aidlc-architecture-reviewer-agent
**Date:** 2026-11-15T00:00:00Z
**Iteration:** 2
**Verdict Dictated:** no

### Findings

| ID | Severity | Location | Finding | Required action | Status |
|---|---|---|---|---|---|
| R-01 | Major | aidlc/spaces/default/intents/261114-job-conductor-cli/construction/u3-cli/infrastructure-design/cicd-pipeline.md > CI のジョブの構成 (`startup-bench`), リリースのワークフロー; monitoring-design.md > Metrics & KPIs 1行目; traceability.json > NFR1.1 | NFR1.1(AC2.5.3 の 100ms)は、どこでも強制されない。`startup-bench` は「配分の確定」までは警告だけで、必須のチェックでもない。「確定」を行う担当・時点・切り替えの手段が書かれていない。また、100ms の合計そのものは仮ではなく、仮なのは内訳の配分だけなのに、合計まで警告扱いにしている。リリースの `release.yml` の段階(1〜6)に `startup-bench` は無いのに、monitoring-design は「リリースのたびに守っていることの確認」と書いており、事実と食い違う。20 回の p95 は 19〜20 番目の値で、共有ランナーでは不安定になる。それでも traceability は NFR1.1 を OK としている。 | 切り替えの条件と担当(例: Code Generation の測定記録をコミットした時点で、基準をワークフロー内の値として有効にする)を書く。合計 100ms は今すぐ失敗にし、内訳(20/20/10)だけを警告にするか、仮の基準としてその理由を明記する。リリースの前に `startup-bench` を含めるか、monitoring-design の「リリースのたびに」の記述を直す。標本数とばらつきへの対処(例: 中央値の複数回、余裕のある閾値)を決める。traceability の NFR1.1 は、実際に強制される範囲に合わせる。 | Resolved (reviewer) |
| R-02 | Major | aidlc/spaces/default/intents/261114-job-conductor-cli/construction/u3-cli/infrastructure-design/cicd-pipeline.md > リリースのワークフロー 段階1(検査), 起動の条件 | リリース前の検査は `lint` と `test` だけで、必須のチェックである `build-check`(最小バージョンのビルド、`cc` 等の混入と `unsafe` の検索)、`coverage`、`docs-check` を含まない。NFR2.1 の確認と README の保証文書が、リリースされるコミットでは確かめられない。タグのコミットが `main` から到達できること、または `ci.yml` が成功済みであることの確認も無い。`v*` のタグを、`main` に入っていない、または CI を通っていないコミットに付ければ、`contents: write` のジョブがそのまま公開する。 | リリースは、タグのコミットで必須のチェックがすべて成功していること(`ci.yml` を再利用するか、成功を確認する)と、そのコミットが `main` の履歴に含まれることを前提とする。段階1の対象を、必須のチェックと同じ集合にそろえる。 | Resolved (reviewer) |
| R-03 | Major | aidlc/spaces/default/intents/261114-job-conductor-cli/construction/u3-cli/infrastructure-design/traceability.json > NFR2.5; cicd-pipeline.md > CI のジョブの構成 (`test`) | u2-runtime の cicd-pipeline は、Windows の Ctrl+C の経路を「OS が配るため、u3-cli のテストで確かめる」と、u3-cli に引き渡している。この文書の NFR2.5 の確認は「UNIX 系」だけで、Windows の中断(`ctrlc` 経路: 転送、記録、終了コード、再開位置)を確かめるテストが、どこにも無い。ワークスペース全体で、Windows の中断は確認されないまま、NFR2.5 が OK になっている。 | `test` の Windows 側に、中断のテストを加える(例: 子プロセスをコンソールのイベントで中断する)。実行が現実的でなければ、確認できない範囲と代替の確認(手動の確認の手順など)を、u2-runtime への返答として明記し、traceability にも未確認と書く。 | Resolved (reviewer) |
| R-04 | Minor | aidlc/spaces/default/intents/261114-job-conductor-cli/construction/u3-cli/infrastructure-design/traceability.json > upstream_ids | 上流の NFR は 28 個(NFR1.1〜NFR6.4)あるが、一覧は 26 個で、NFR6.2(既定は静か、`-v`)と NFR6.3(ロケールの切り替え)が無い。どちらも、CI で確かめる内容(`test` の表示のテスト)がある。 | 2つを一覧に加え、`test` の確認に結びつける。外した理由があれば書く。 | Unresolved |
| R-05 | Minor | aidlc/spaces/default/intents/261114-job-conductor-cli/construction/u3-cli/infrastructure-design/cicd-pipeline.md > リリースのワークフロー 段階2・3・6 | (1) Linux の `musl-tools` は、純粋な Rust の依存だけなら不要で、導入は「C のコンパイラ不要」の説明と紛らわしい(リンクには `cc` を使うため、リンカの前提として書くのが正確)。(2) 静的リンクであることを確かめる段階が無く、NFR2.1 の「musl の静的リンク」はターゲット名で主張されているだけ。(3) スモークテストはまとめる前の実行ファイルで行い、アーカイブの中身(`LICENSE`、実行ビット)は確かめない。(4) `x86_64-apple-darwin` は、Rosetta が無ければ確認を省いて「記録して続ける」が、記録の場所が無く、未確認のバイナリが公開される。 | `ldd` か `file` による静的リンクの確認を足す。スモークテストは、展開したアーカイブに対して行う。Rosetta での確認が省かれたときは、リリースの本文に未確認と明記するか、公開を止める。`musl-tools` の理由を書く。 | Resolved (reviewer) |
| R-06 | Minor | aidlc/spaces/default/intents/261114-job-conductor-cli/construction/u3-cli/infrastructure-design/cicd-pipeline.md > リリースのワークフロー(段階5・6) | 4つのビルドが並列のマトリクスなのに、全アーカイブを集めて `SHA256SUMS` を作り、1回で公開するジョブの構成(成果物の受け渡し、`needs`)が書かれていない。1つの対象が失敗したとき、部分的なリリースが公開されないことも保証されていない。タグ `v*` は `v0.1.0-rc1` のような版も拾うが、プレリリースの扱いも無い。 | ジョブの構成(ビルド → 集約・チェックサム・公開)を書く。全対象が成功したときだけ公開し、可能なら下書きのリリースに添付してから公開する。プレリリースの版の扱いを決める。 | Resolved (reviewer) |
| R-07 | Minor | aidlc/spaces/default/intents/261114-job-conductor-cli/construction/u3-cli/infrastructure-design/cicd-pipeline.md > 秘密情報, ツールチェインとキャッシュ; infrastructure-specification.md > Infrastructure Services | 署名の無い配布物で、サードパーティのアクションを使う前提だが、アクションをコミットの SHA で固定する方針、リリースのビルドでキャッシュを使わないこと、`macos-latest` / `ubuntu-latest` のように動くランナーの扱い、`v*` タグを付けられる人の制限(タグの保護)が、どれも書かれていない。「秘密情報は `GITHUB_TOKEN` だけ」は事実だが、供給網の説明としては足りない。 | アクションの固定の方針、リリースのビルドはキャッシュを使わないこと、ランナーの版を固定するか浮動を許すか、タグの保護規則を、方針として書く。`contents: write` は公開のジョブだけに限り、ビルドのジョブは読み取りだけにする。 | Resolved (reviewer) |
| R-08 | Minor | aidlc/spaces/default/intents/261114-job-conductor-cli/construction/u3-cli/infrastructure-design/cicd-pipeline.md > CI のジョブの構成 (`build-check`, `startup-bench`) | 他の Unit との食い違い。(1) u1-core、u2-runtime、u4-converter は `-sys` で終わるクレートを禁じ、`--target all` と許可リストを使う。この文書は `cc`・`cmake`・`pkg-config`・`bindgen` の不在だけで、`windows-sys` を許す扱いが別になっている。(2) u1-core の起動時間(段階 6、20ms、100 回)と u2-runtime の起動前(段階 7)は、同名のジョブ `startup-bench` に属すが、この文書の `startup-bench` は `conductor` 全体の測定だけで、両者を含めると書いていない。(3) `build-check`、`lint`、`docs-check` に `timeout-minutes` が無い。 | 依存の検査の規則を4つの Unit で1つにそろえる(許可リストを含む)。`startup-bench` に、u1-core・u2-runtime の測定が含まれることを明記する。残りのジョブにも上限の時間を決める。 | Resolved (reviewer) |
| R-09 | Minor | aidlc/spaces/default/intents/261114-job-conductor-cli/construction/u3-cli/infrastructure-design/cicd-pipeline.md > CI のジョブの構成 (`docs-check`); monitoring-design.md > Alerts | `docs-check` は見出しの検索で、見出しの言語・表記が決まっておらず壊れやすい。見出しがあるだけで中身が空でも通る。README の作成(Code Generation)より先に必須のチェックとして入ると、すべてのプルリクエストが失敗する。monitoring-design の「リポジトリの管理者のメール」は、GitHub の実際の通知(タグをプッシュした人、ワークフローを最後に変更した人への通知)と合わない。 | 見出しの文字列を固定し、段落が空でないことも確かめる。`docs-check` を必須にする時期を、README の作成の後と定める。通知の宛先は、実際の動作に合わせて書き直すか、リリースの失敗の確認を、担当者の手順にする。 | Unresolved |
| R-10 | Minor | aidlc/spaces/default/intents/261114-job-conductor-cli/construction/u3-cli/infrastructure-design/cicd-pipeline.md > リリースのワークフロー 段階 3 と段階 4 | 段階 3 のスモークテストは「まとめたファイルを展開して」実行するが、アーカイブを作るのは段階 4 で、順序が逆になっている。 | 「まとめ」を動作の確認より前に置くか、段階 3 と 4 を入れ替える。チェックサムは、確認したのと同じアーカイブから求めると明記する。 | New |
| R-11 | Minor | aidlc/spaces/default/intents/261114-job-conductor-cli/construction/u3-cli/infrastructure-design/cicd-pipeline.md > 起動の条件(`ci.yml`) と リリース段階 1 | リリースは `ci.yml` を `workflow_call` で呼ぶが、`ci.yml` の起動の条件は push とプルリクエストだけで、`workflow_call` の受け口が書かれていない。 | `ci.yml` の起動の条件に `workflow_call` を加え、`docs-check` と `startup-bench` の有効・無効を呼び出し側から渡す方法(入力か設定値)を書く。 | New |
| R-12 | Minor | aidlc/spaces/default/intents/261114-job-conductor-cli/construction/u3-cli/infrastructure-design/cicd-pipeline.md > CI のジョブの構成(`startup-bench`) | 「同じジョブの再実行(最大 2 回)でも再現したとき」とあるが、GitHub Actions のジョブの再実行は手動で、自動で最大 2 回とする手段が書かれていない。リリースの検査の中でも、同じ曖昧さがある。 | ジョブの中で測定を最大 3 回繰り返し、すべて基準を超えたときだけ失敗にする、などの手段を書く。 | New |

> R-01 Reviewer note: `STARTUP_BENCH_ENFORCE` に、担当(開発者)・切り替えの時点(Code Generation の実測と配分の確定)・再実行で再現したときだけ失敗にする規則が書かれた。合計 100ms は確定した基準と明記され、リリースの検査にも含まれる。monitoring-design とも一致する。切り替えまでは警告のため、リリースが基準を強制しない期間は残るが、条件は明示されている。

> R-02 Reviewer note: 段階 0 でタグのコミットが `main` に含まれることを確かめ、段階 1 は必須のチェックのすべてを `workflow_call` で実行する。ただし `ci.yml` 側の `workflow_call` の記載が無い点は、新しい指摘に記す。

> R-03 Reviewer note: Windows の `test` に Ctrl+C のテストが加わり、自動化できない場合の手動チェックリストと記録が定められ、traceability の NFR2.5 にも反映された。

> R-04 Reviewer note: traceability.json の上流 ID は 26 個のままで、NFR6.2 と NFR6.3 が無い。

> R-05 Reviewer note: 静的リンクの確認、展開したアーカイブでのスモークテスト、Intel macOS 未確認のリリース本文への記載が加わった。順序の不整合は、新しい指摘に記す。

> R-06 Reviewer note: ビルド、まとめ・チェックサム、公開(下書きに全部を添付してから公開)のジョブの流れと、先行版の扱いが書かれた。

> R-07 Reviewer note: アクションの SHA 固定、キャッシュなし、ランナー版の固定、タグの保護、公開のジョブだけに `contents: write` が書かれた。

> R-08 Reviewer note: 依存の検査の規則(許可リストを含む)がそろい、u1-core・u2-runtime の測定が `startup-bench` に含まれ、全ジョブに `timeout-minutes` が付いた。

> R-09 Reviewer note: `docs-check` を必須にする時期と通知の記述は直ったが、見出しの文字列の固定と、見出しの下が空でないことの確認は、まだ書かれていない(「見出しとキーワードの検索」のまま)。

### Validation Tool Results

| Tool | Result | Interpretation |
|---|---|---|
| traceability.json の上流 ID の突き合わせ(NFR の表と比較) | 26 個。NFR6.2、NFR6.3 が無い | R-04 のとおり Minor が残る |
| 他の Unit の cicd-pipeline との照合(u1-core、u2-runtime、u4-converter) | 依存の規則(`-sys` の禁止、許可リスト)、`startup-bench` の引き渡し、Windows の Ctrl+C の引き渡しは、u3-cli 側で受け止められている | 食い違いは解消した |

### Summary

前回の Major 3件は、仕組みと担当が書かれて解消した。Critical は無く、Major も無い。残るのは、トレーサビリティの 2 件の漏れ、README の確認の厳密さ、リリースの段階の順序、`workflow_call` の受け口、再実行の手段という、実装の中で直せる Minor だけで、READY とする。
