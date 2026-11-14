## Review

**Verdict:** READY
**Reviewer:** aidlc-architecture-reviewer-agent
**Date:** 2026-11-14T21:51:05Z
**Iteration:** 1
**Verdict Dictated:** no

### Findings

| ID | Severity | Location | Finding | Required action | Status |
|---|---|---|---|---|---|
| R-01 | Major | aidlc/spaces/default/intents/261114-job-conductor-cli/construction/u4-converter/infrastructure-design/cicd-pipeline.md > 段階 6 カバレッジ、および traceability.json の NFR5.1 | 段階 6 は「`crates/u4-converter/src/` を含むワークスペースの行カバレッジ」を測って 80% 未満で失敗にするとある。これはワークスペース全体の合算値であり、この Unit 単体の 80% を保証しない。他の Unit のカバレッジが高ければ、u4-converter が 80% を下回っても CI は通る。tech-stack-decisions の NFR5.1 は「この Unit のクレートに対して測る」としており、cicd-pipeline.md と食い違う。traceability.json は NFR5.1 を OK としているが、この Unit の下限は実際には強制されない。 | 測定の範囲を決めて書く。この Unit に 80% を強制するなら、`cargo llvm-cov -p u4-converter --fail-under-lines 80` のようにクレート単位で測る。ワークスペース合算で足りるとするなら、tech-stack-decisions の記述と traceability.json の説明を合算に直し、この Unit 単体では強制しないことを書く。 | New |
| R-02 | Minor | aidlc/spaces/default/intents/261114-job-conductor-cli/construction/u4-converter/infrastructure-design/cicd-pipeline.md > 段階 5 と「3つの OS 固有の注意」の1項目目 | 「Windows のスタックの小さい主スレッドでも」とあるが、`cargo test` は各テストを新しいスレッドで動かす。そのスレッドのスタックは、既定ではどの OS でも約 2 MiB である。この前提は事実と合わない。また、深さ 100,000 のテストを走らせるスレッドのスタックの大きさが決まっていない。決まっていないと、再帰が混入したときに、OS ごとに結果が分かれうる。 | 深い入力のテストは、固定の小さなスタック(例: 明示的に `stack_size` を指定したスレッド)で動かすと書く。「主スレッド」の記述は削る。 | New |
| R-03 | Minor | aidlc/spaces/default/intents/261114-job-conductor-cli/construction/u4-converter/infrastructure-design/cicd-pipeline.md > 段階 5(スレッドと recv_timeout による 30 秒の上限) | スレッドと `recv_timeout` の方式は、無限ループによるハングなら検知でき、ハングしたスレッドはプロセス終了とともに終わる。しかし、展開の爆発が回帰したときの失敗は、ハングではなくメモリの枯渇になりうる。その場合、時間の上限は効かず、ランナーの OOM キルでジョブ全体が原因の見えない形で落ちる。ハングしたスレッドも、他のテストの実行中は CPU を使い続ける。 | この2つの失敗の形を、仮定の欄か段階 5 に書く。そのうえで、暴走のテストを `--test-threads=1` の別の呼び出しに分けるか、同じ効果の手段を採るかを選ぶ。 | New |
| R-04 | Minor | aidlc/spaces/default/intents/261114-job-conductor-cli/construction/u4-converter/infrastructure-design/traceability.json > NFR4.5(N/A)、および cicd-pipeline.md > 段階 4 | NFR4.5 は N/A で、「ソースの検索は段階 4 のテストの一部」と説明している。しかし段階 4 の内容には、ソース検索(`std::process`、`std::net` の不使用)も、副作用のないことのテストも書かれていない。N/A としながら CI で確かめるという書き方は矛盾しており、検索がどの段階で実行されるのかも不明である。 | NFR4.5 の確認(ソース検索と副作用のテスト)を、段階 4 の内容に明記する。traceability の status は N/A ではなく OK にし、対象の段階を指す。 | New |
| R-05 | Minor | aidlc/spaces/default/intents/261114-job-conductor-cli/construction/u4-converter/infrastructure-design/cicd-pipeline.md > 段階 3(最小の Rust のバージョン 1.85 でのビルド) | 1.85 でのビルドの確認について、コマンドが書かれていない。固定する Rust のバージョンの方針(edition 2024 の最新の安定版)と、1.85 の確認の関係もあいまいである。`--locked` の `Cargo.lock` の下で、1.85 のビルドを、開発用の依存を除いて確かめるのか(`cargo +1.85 build -p u4-converter --locked`)、`cargo test` まで含めるのかが決まっていない。 | 1.85 の確認を専用のジョブにする。対象をクレートに絞ったコマンドと、使う OS を書く。 | New |
| R-06 | Minor | aidlc/spaces/default/intents/261114-job-conductor-cli/construction/u4-converter/infrastructure-design/cicd-pipeline.md > 段階 5(`timeout-minutes: 15`)、および「ロールバック」 | 15 分の根拠が書かれていない。3つの OS で、ワークスペース全体の整形・静的解析・ビルド・テストを行い、Linux ではさらに `llvm-cov` も走らせる。Unit が増えるほど、この値の妥当性は変わる。ロールバックは、`git revert -m 1` を前提にしているが、「squash しない」取り込みの方式(マージコミットか、リベースか、早送りか)は決まっていない。早送りやリベースでは、マージコミットが存在せず、`-m 1` は使えない。 | 15 分は、実測のあとに見直す仮定として、仮定の欄に載せる。ロールバックは、取り込みの方式ごとの戻し方(マージコミットなら `-m 1`、そうでなければ個々のコミットの revert)を書く。 | New |
| R-07 | Minor | aidlc/spaces/default/intents/261114-job-conductor-cli/construction/u4-converter/infrastructure-design/traceability.json > upstream_ids | 上流の tech-stack-decisions は、NFR6 が u3-cli だけに該当し、この Unit には該当しないと明記している。しかし traceability.json にその行(N/A とその理由)がなく、読む人が漏れと区別できない。 | NFR6 を N/A として、理由つきで traceability に足す。 | New |

### Validation Tool Results

| Tool | Result | Interpretation |
|---|---|---|
| (none run) | この Unit は library で、このステージの成果物は設計の文書と traceability.json だけである。実行できる検証ツールはない。 | 参照は、nfr-requirements、nfr-design、requirements の ID と手作業で突き合わせた。NFR1.1、NFR2.1、NFR3.1、NFR5.1、NFR4.1〜NFR4.7 の ID は、上流で解決できる。 |

### Summary

CI の設計は library の Unit として妥当で、team.md(GitHub Actions、3つの OS、rustfmt と clippy、タグでのリリース、必須チェックでのパス絞り込みなし)と、nfr-design の段階にも整合する。主な問題は、カバレッジの測定範囲がワークスペース全体の合算で、この Unit 単体の 80% が強制されないことである(Major、1件)。残りは Minor で、READY とする。
