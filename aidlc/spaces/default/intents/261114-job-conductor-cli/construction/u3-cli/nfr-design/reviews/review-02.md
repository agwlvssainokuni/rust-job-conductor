## Review

**Verdict:** READY
**Reviewer:** aidlc-architecture-reviewer-agent
**Date:** 2026-11-15
**Iteration:** 2
**Verdict Dictated:** no

### Findings

| ID | Severity | Location | Finding | Required action | Status |
|---|---|---|---|---|---|
| R-01 | Major | aidlc/spaces/default/intents/261114-job-conductor-cli/construction/u3-cli/nfr-design/traceability.json > upstream_ids / coverage (NFR4.8、NFR6.x) | `upstream_ids` は 21 件だが、nfr-requirements/traceability.json が上流として挙げる NFR6(NFR6.1〜NFR6.4。observability-design の全体の根拠)、NFR2.1(C コンパイル不要)、NFR3.1(OS 固有コードを 1 モジュールに閉じ込める)、NFR5.1(行カバレッジ 80%)が入っていない。observability-design は NFR6.1〜6.4 を本文で引いているのに、追跡表には現れない。さらに NFR4.8 は status が OK で、target が「security-design の範囲外であることを明記」とあるが、security-design には NFR4.8 を範囲外とする記述がなく、ヘッダの上流の列挙に `NFR4.8` の文字があるだけ。target が設計成果物ではなく要件文書を指している。 | NFR6.1〜6.4、NFR2.1、NFR3.1、NFR5.1 を `upstream_ids` と `coverage` に加え、具体の設計の節(例: observability-design の「表示の部品」、logical-components のモジュールの分け方、logical-components の Q1 の構成とカバレッジ)に結びつける。NFR4.8 は、status を「範囲外(後のステージで実装)」の別の値にするか、security-design に「リリースの成果物の完全性(NFR4.8)は、CI のステージの要件で、この設計では扱わない」と一文を足し、target をその節にする。 | Unresolved |
| R-02 | Major | aidlc/spaces/default/intents/261114-job-conductor-cli/construction/u3-cli/nfr-design/reliability-design.md > 失敗の経路と振る舞い(表)と設計の要点 1 | 設計の要点 1 は「表の全項目をテストで確かめる」とするが、表に、functional-design が定める失敗の経路が抜けている。(a) 初期ステートの書き込みの失敗(BR2.5 の (1): 65、ジョブは起動しない)、(b) Skip のときの書き込み・初期化の失敗(BR2.4)、(c) ジョブのあいだの中断で再開位置を書けない失敗(BR4.2)、(d) 「完了の判定ができない」(BR3.2: 65、初期化しない)、(e) シグナルの準備の失敗(70。security-design にのみ記載)、(f) シグナルを受けたあと、起動前に初期ステートを書いたまま残るステートファイル(BR4.2a)。表が正のテスト対象なので、これらは、設計上、検証されない。ジョブ未起動の 65 と、ジョブ実行済みの 65 は、再実行の安全さが違うのに、区別されていない。 | 表に (a)〜(f) の行を足し、各行に、終了コード、ステート、ジョブが起動済みかを書く。(a) は「ジョブ未起動で、再実行は安全」、ジョブのあとの書き込みの失敗は「ジョブ実行済み」と区別して書く。 | Resolved (reviewer) |
| R-03 | Major | aidlc/spaces/default/intents/261114-job-conductor-cli/construction/u3-cli/nfr-design/logical-components.md > trait(差し替え口)と部品 SignalWatcher、security-design の 3 | Q2: A の差し替え口は `Runtime`/`Sink`/`Env` だけで、SignalWatcher は含まれず、`run_cli` の中で実際のシグナルのハンドラを登録する作りに読める。Q1: A は「テストをライブラリを同じプロセスで呼ぶ」ことが前提で、その場合、(i) Windows の `ctrlc::set_handler` は 1 プロセスで 1 回しか成功しない(2 回目以降はエラー)ため、2 つ目以降の run/pipeline のテストが「シグナルの準備の失敗(70)」になる、(ii) UNIX では、テストごとに `forever()` のスレッドが残り、実際の SIGINT がテスト全体に影響する。ジョブのあいだ・起動前の中断(NFR2.5)も、偽の `Runtime` から合図を立てる手段が、設計に無い。 | SignalWatcher を、`Env`/`Runtime` と同じ形の差し替え口(例: 合図を作って返す trait、またはプロセスごとに 1 回だけ登録して、`InterruptSignal` を渡す形)として設計に加える。テストの実装は、手で合図を立てられるようにし、Windows の登録は、プロセスで 1 回だけにする。 | Resolved (reviewer) |
| R-04 | Minor | aidlc/spaces/default/intents/261114-job-conductor-cli/construction/u3-cli/nfr-design/scalability-design.md > 設計の表「流れの計算量」と D2、performance-design の 設計の要点 3・5 | performance-design の 3 は「1,000 個でも線形」と書くが、scalability-design は一括実行を O(n²) と書いており、食い違っている。また、`pipeline` の再開(`alreadyExecuted` が多数)で、W3 は Skip のたびに「`apply_skip` で記録して書き」とあるが、u2-runtime は `alreadyExecuted` でステートを変えないと定めており、変更が無いのに `Durable` の書き込み(約 10ms)を n 回行うかが決まっていない。容量の測定は `run` だけ。 | 「線形」の記述を直す。変更のない Skip は書き込まないことを、設計として定める。1,000 個の `pipeline` の測定(再開を含む)を加える。 | Resolved (reviewer) |
| R-05 | Minor | aidlc/spaces/default/intents/261114-job-conductor-cli/construction/u3-cli/nfr-design/performance-design.md > 設計の要点 1、Assumptions(`OnceLock`) | 遅延を `std::sync::OnceLock`(プロセスで 1 つ)で行うと、Q2 の `Env` の差し替えで、同じプロセスの複数のテストが、最初に判定した言語・色の結果を共有し、日本語と英語のテスト、`NO_COLOR` のテスト、NFR1.4 の「判定の回数」のテストが互いに影響する。 | 遅延の保持を、Reporter(または Env)のインスタンスごとのセルにすると明記し、静的な大域の `OnceLock` は使わない。 | Resolved (reviewer) |
| R-06 | Minor | aidlc/spaces/default/intents/261114-job-conductor-cli/construction/u3-cli/nfr-design/security-design.md > 1(引数の解釈) | 文法の表が、`--config=path`・`--to=fmt` の形、`--` 以降の扱い、`-vv`・`-v` の重複(`lexopt` は結合した短いオプションを分けるため、`-vv` が通り、observability-design D2 の「1 段階」と食い違う)、オプションの値の欠落、先頭が `-` のジョブ ID、同じオプションの繰り返しを定めていない。`--help` を「先に処理」する順序(`run --bogus --help`)も未定。 | 文法の表に、これらの扱い(受け付ける/68)を行として足す。 | Resolved (reviewer) |
| R-07 | Minor | aidlc/spaces/default/intents/261114-job-conductor-cli/construction/u3-cli/nfr-design/security-design.md > 4 の `watch` の 65、D3(70 の根拠) | `watch` の標準出力の失敗に 65 を割り当てているが、functional-spec W4 では 65 は「ステートの誤り」の意味で、新しい意味が足されている(契約 E1 に反映されていない)。また D3 は、101 を「100 番台がジョブの終了コードと紛れる」として退けるが、70(と 64〜68)も、ジョブが返しうる値で、同じ紛れがあり、この理由は、70 の選択を支えない。 | `watch` の標準出力の失敗の終了コードの意味を、契約 E1 の反映の対象として functional-spec の差分の節に記録する(または、専用の扱いを決める)。D3 の根拠を、「sysexits 風の既存の 64〜68 の帯に揃える」など、実際の理由に直す。 | Resolved (reviewer) |
| R-08 | Minor | aidlc/spaces/default/intents/261114-job-conductor-cli/construction/u3-cli/nfr-design/security-design.md > 3(シグナル)、D2 | `InterruptSignal` の `raise(n)` と「立ったら下りず、最初の番号を保つ」は、u2-runtime の functional-spec と contract-summary(「Cli が用意する読み取り専用の合図」)のどちらにも定義がなく、どの Unit が型を持つかが決まっていない。設計は、存在する前提で書かれている。 | 型の所有(Cli が定義して u2-runtime が読む、など)と、`raise` の意味を、契約への反映の節に記録するか、この Unit の中で型を定義すると設計に書く。 | Resolved (reviewer) |
| R-09 | Minor | aidlc/spaces/default/intents/261114-job-conductor-cli/construction/u3-cli/nfr-design/scalability-design.md > 設計の表「容量の測定」 | 容量の測定は「`run` の起動からジョブの起動まで 1 秒以内」だけで、performance-design の設計の要点 3 が定める「`run` と `pipeline` の両方で測る」と食い違う。2 乗になりうる `pipeline`(再開で `alreadyExecuted` が多数の場合を含む)の測定の基準が、どこにもない。 | 容量の測定の行に、1,000 個の `pipeline`(最初からと再開)の測定と、仮の上限を加える。 | New |
| R-10 | Minor | aidlc/spaces/default/intents/261114-job-conductor-cli/construction/u3-cli/nfr-design/logical-components.md > 論理的な部品の入口(main)と trait の表 | 入口は `run_cli(args, env, io)` と書かれ、`Runtime` と `Signals` を注入する経路が見えない。また、本番の `Signals` の「2 回目の呼び出しは成功とする」は、本番の実装を同じプロセスで 2 回呼ぶテストで、2 つ目の合図が配線されないまま成功に見える。 | `run_cli` の引数に、`Runtime` と `Signals` を含めることを書く。本番の `Signals` を使うテストは、プロセスにつき 1 回に限ると注記する。 | New |
| R-11 | Minor | aidlc/spaces/default/intents/261114-job-conductor-cli/construction/u3-cli/nfr-design/traceability.json > upstream_ids | 上流の列挙に NFR1.5、NFR1.6 が入る一方、security-design の NFR4.8 の節が「CI の成果物」の要件を別の Unit に委ねており、その委ね先(cicd-pipeline)の受け入れが、この追跡表のどこにも結びつかない。 | `coverage` の NFR4.8 に、委ね先のステージと取り決めの 3 点を結びつける注記を加える(R-01 の追加と同時でよい)。 | New |

> R-01 Reviewer note: security-design に NFR4.8 の節(範囲外と、取り決め 3 点)と「他の NFR の設計」の節が加わり、NFR4.8 の target は実在する節を指すようになった。一方、traceability.json の `upstream_ids` は 21 件のままで、NFR6.1〜6.4、NFR2.1、NFR3.1、NFR5.1 が入っておらず、`coverage` にも行がない。設計の本文には書かれたが、追跡表には現れない。

> R-02 Reviewer note: 失敗の表に、初期ステートの書き込み失敗(ジョブ未起動)、スキップ・初期化の失敗、ジョブのあいだの中断の書き込み失敗、完了の判定不能、シグナルの準備の失敗(70)、検査中のシグナルの行が加わり、D1 が「未起動」と「実行済み」の 65 を文言で区別すると定めた。

> R-03 Reviewer note: 差し替え口に `Signals` が加わり、テストの偽が起動前・ジョブのあいだ・検査の途中で `raise` でき、登録失敗(70)も偽で確かめられる。本番の実装は、プロセスで 1 回だけ登録する。

> R-04 Reviewer note: performance-design は、`run` は線形、`pipeline` は 2 乗と書き、scalability-design と揃った。変更のない Skip は書き込まないことが、設計(BR2.4 との差分)として定まった。測定の範囲の残りの食い違いは、新しい指摘に記した。

> R-05 Reviewer note: 「静的な初期化・大域の変数を置かない」と「OnceCell 相当」で、インスタンスごとの遅延になった。Assumptions に `OnceLock` の語が残るが、型の安定化の確認だけで、静的な大域とは読めない。

> R-06 Reviewer note: 文法の細目(1a)に、`--opt=value`、`--`、`-vv` の拒否、値の欠落(missingValue)、`--help` の順序が加わった。

> R-07 Reviewer note: `watch` の標準出力失敗の 65 が、既存の成果物との差分の表(契約 E1 に足す)に記録され、D3 から根拠の弱い論拠が除かれた。

> R-08 Reviewer note: `InterruptSignal` の所有は u2-runtime の型で、この Unit は `raise` を呼ぶだけ、と Assumptions に明記された(u2-runtime の functional-spec の「Cli が用意する合図」とも矛盾しない)。

### Validation Tool Results

| Tool | Result | Interpretation |
|---|---|---|
| (stage に指定の検証ツールなし) | traceability.json と成果物を目視で突き合わせた | 追跡表の漏れ(R-01)は残る。他の前回の指摘は解消した。 |

### Summary

前回の 8 件のうち 7 件が解消し、失敗の経路の表、`Signals` の差し替え口、文法の細目は、実装に足りる具体さになった。未解消は追跡表(traceability.json)の漏れ 1 件(Major)だけで、Critical は無く、Major は 2 以下のため READY とする。この漏れは、承認の前に追跡表へ追記してほしい。
