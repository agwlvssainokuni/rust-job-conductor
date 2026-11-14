## Review

**Verdict:** READY
**Reviewer:** aidlc-architecture-reviewer-agent
**Date:** 2026-11-14T23:02:20Z
**Iteration:** 1
**Verdict Dictated:** no

### Findings

| ID | Severity | Location | Finding | Required action | Status |
|---|---|---|---|---|---|
| R-01 | Major | aidlc/spaces/default/intents/261114-job-conductor-cli/construction/u2-runtime/nfr-design/security-design.md > 3 ステートの書き込み(StateWriter) 1 | 一時ファイル名は `.<名前>.tmp.<プロセス番号>.<連番>` で、新規作成(既存ならエラー)とされている。連番はプロセスごとに 0 から始まるため、conductor が強制終了されて残った一時ファイルと、プロセス番号と連番が同じ名前の実行(コンテナなど、プロセス番号が小さく似通う環境で起こりうる)では、作成が必ず失敗し、`writeFailed` になる。残った一時ファイルは、読まず自動でも消さない(BR2.4)ため、衝突は自然には解消せず、以後の書き込みが、ずっと失敗しうる。残った一時ファイルの蓄積への対処も書かれていない | 名前が既に存在するときの動作を決める(例: 連番を進めて、上限つきで作り直す。または、時刻や乱数を名前に加える)。置き換え先をディレクトリにする失敗の試験とは別に、同名の一時ファイルが既にある場合の試験を、NFR5.1 に加える。残った一時ファイルを消さない方針は、衝突の再試行で、書き込みが失敗しないことを前提に、記録として残す | New |
| R-02 | Major | aidlc/spaces/default/intents/261114-job-conductor-cli/construction/u2-runtime/nfr-design/security-design.md > 他の NFR の設計 NFR5.1、D2 | テスト専用の補助の実行ファイルは、シグナルの受信を出力する(SIGTERM を受けたら事実を出力して終わる)ため、受信の処理(UNIX 系ではシグナルの処理の登録)が要る。設計は、`unsafe` を `platform` の1か所だけに限り、ソースの検索で他に `unsafe` がないことを確かめるとするが、`bin` は別のクレートとして扱われ、`#![deny(unsafe_code)]` の対象にならない。補助が `libc` を直接使えば、`unsafe` がもう1か所増え、使わなければ、シグナルの処理を行うライブラリが要るが、依存の方針(`toml`、`thiserror`、`libc` のみ)にない。また D2 は、配布のビルドに補助が含まれうることを悪い点としながら、除外の手段(`required-features`、`test = false` など)を決めていない。さらに、補助の `src/bin/` の行が、`crates/u2-runtime/src/` に対する 80% のカバレッジの測定に入るかが、不明である | 補助がシグナルを受ける方法を決める(例: 補助は処理を登録せず、シグナルで終了させて、親が `terminatedBySignal` で確かめる。または、補助の `unsafe` を、2か所目として NFR4.8 に明記する)。補助を、配布のビルドと、既定の `cargo build` から外す手段と、カバレッジの測定から外す扱いを、D2 に書く | New |
| R-03 | Minor | aidlc/spaces/default/intents/261114-job-conductor-cli/construction/u2-runtime/nfr-design/security-design.md > 1 4 WaitLoop の疑似コード | `child.try_wait()?` が失敗したとき(`SIGCHLD` を無視する設定を引き継いだ場合の `ECHILD` など)の写し先が決まっていない。`JobExecution` には、起動失敗以外に、待ちの失敗を表す形がなく、疑似コードの `?` のままでは、NFR4.7(失敗は戻り値で返す)を満たす形が示されていない | `try_wait` の失敗を、どの `JobExecution` の形(または誤り)で返すか、その場合にジョブを回収したか、を書く | New |
| R-04 | Minor | aidlc/spaces/default/intents/261114-job-conductor-cli/construction/u2-runtime/nfr-design/security-design.md > 1 6 シグナルの送信(platform) | シグナル番号の検査が 1〜64 の範囲だけである。SIGSTOP(Linux は 19、macOS は 17)を転送すると、ジョブが止まったまま、終了を待ち続けるおそれがある(強制終了の手段がない。BR1.7)。macOS では 31 を超える番号が無効で、`kill` の失敗が無視されるため、転送されないまま `forwarded = true` になる | 転送を許す番号を、u3-cli が実際に合図にする集合(INT、TERM、HUP など)に絞るか、OS ごとの有効な範囲を、`platform` の中で検査する。無効な番号を黙って捨てないことを、試験に加える | New |
| R-05 | Minor | aidlc/spaces/default/intents/261114-job-conductor-cli/construction/u2-runtime/nfr-design/security-design.md > 設計の判断 D3 | ジョブをシェルスクリプトにした場合、新しいプロセスグループにしたことで、端末の Ctrl+C が子孫に届かなくなり、先頭のプロセスだけへの転送では、子孫が走り続ける。D3 は、これを悪い点として書くが、代替案(グループ全体に送る)を退ける理由が弱い。グループの番号は、起動時に自分で作った先頭の子の pid と同じで、その先頭の子を回収する前なら、pid と同じ条件で再利用の事故を避けられる。「負の pid の誤りが広い範囲に影響」は、`process_group(0)` で作ったグループには当てはまりにくい | D3 の代替案を、グループ全体への送信が、回収前の条件で安全にできるかを含めて、比較し直す。ジョブの責務とする判断を残すなら、その根拠(BR1.7)を、理由として明記する | New |
| R-06 | Minor | aidlc/spaces/default/intents/261114-job-conductor-cli/construction/u2-runtime/nfr-design/security-design.md > 設計の判断 D4、StateReader の 2 | 同期を省いた書き込みのあとの電源断で起こる破損を、長さ 0 のファイルだけとしているが、ファイルシステムによっては、置き換え後に、長さがあっても内容が欠けた(ゼロで埋まった、途中までの)ファイルが残りうる。それは `corrupt` になり、AC4.1.3 の経路で、`reset` が要る。D4 の「再作成できる」は、そのとおりには言えない。また、結果以外の記録(`apply_skip`、`apply_interrupt` の書き込み)が、`Durable` と `Fast` のどちらかが、書かれていない | 空に近い初期ステートの破損が、`corrupt` になりうること(`reset` で回復できること)を、D4 の悪い点に加える。skip と interrupt の書き込みの耐久性の指定を、決める | New |
| R-07 | Minor | aidlc/spaces/default/intents/261114-job-conductor-cli/construction/u2-runtime/nfr-design/traceability.json > coverage NFR4.9、NFR1.1 | NFR4.9 の対象は「security-design の 1 の 3」とされているが、その節には、`.bat`・`.cmd` の引数の扱いと Windows の CI での確認が書かれていない(起動に失敗したら startFailure とだけある)。基準を満たさなかった場合に、先に拒否する案へ切り替える設計もない。NFR1.1 の対象にも、p95 20ms の数値がなく、1ms の間隔と同期の省略だけが書かれている | NFR4.9 の設計(標準のライブラリの挙動に頼ること、CI の確認、外れた場合の切り替え)を、security-design に書くか、対象の記述を、実際の記述に合わせる。NFR1.1 の対象に、20ms の目標と測定の方法を書く | New |

### Validation Tool Results

| Tool | Result | Interpretation |
|---|---|---|
| (stage definition lists no validation tools for this stage) | 実行なし | 参照の確認は、手で行った。NFR4.1〜NFR4.9、NFR1.1 は、traceability.json の coverage にすべて現れ、対象の節が存在する(NFR4.9 の対象の記述の不一致は、Minor として記録した) |

### Summary

待ちのくり返しは、回収の前だけに送信する条件(NFR4.8)を、疑似コードの構造で満たしており、Windows の扱いも BR1.7 と整合する。一方、一時ファイル名の衝突による書き込みの恒久的な失敗と、補助の実行ファイルが NFR4.8 の `unsafe` の限定・配布・カバレッジに与える影響は、Code Generation の前に、決めておくべき2点(Major)である。Major は2件のため、READY とする。
