## Review

**Verdict:** READY
**Reviewer:** aidlc-architecture-reviewer-agent
**Date:** 2026-11-14T23:28:00Z
**Iteration:** 1
**Verdict Dictated:** no

### Findings

| ID | Severity | Location | Finding | Required action | Status |
|---|---|---|---|---|---|
| R-01 | Minor | crates/u2-runtime/src/runner.rs:48-71 (`wait_loop`) and crates/u2-runtime/src/runner.rs:228-238 (unit tests) | 「回収したあとにはシグナルを送らない」(NFR4.8 の条件 3)は、コードの並び(`try_wait` が `None` の回だけ転送する)で守られているが、それを確かめるテストがない。単体テストは、失敗する子、合図が立っている終わらない子、合図のない子の 3 つだけである。「合図が立っているのに、最初の `try_wait` が終了を返す子」で `forward` が呼ばれないことを確かめるものがない。統合テスト `先に終わったジョブは転送なしで観測される` は、`run_job` が返ったあとに合図を立てるため、この性質には何も触れず、失敗しえない。 | `wait_loop` の単体テストを足す。合図を先に立て、1 回目の `try_wait` で `Some(status)` を返す fake の子を渡し、`forward` が 0 回であることを確かめる。`runner_signal.rs` の `先に終わったジョブは転送なしで観測される` は、確かめている内容に合う名前と主張に直すか、削除する。 | New |
| R-02 | Minor | crates/u2-runtime/src/state/writer.rs:148-177 (`remove_stale_temps`、特に 160 行目) | 古い一時ファイルの後始末は、名前が `.<ステートファイル名>.tmp.` で始まるかだけを見て、24 時間以上たった通常のファイルを消す。設計は「同じ名前の型」(`.<名>.tmp.<プロセス番号>.<時刻>.<連番>`)としているが、後ろの 3 つが数字であることを確かめていない。利用者が同じディレクトリに置いた `.conductor.toml.state.tmp.bak` のようなファイルも消える。`古い一時ファイルだけを消す` のテストは、別のステートの名前と普通の名前しか試さず、この形は試していない。 | 接頭辞のあとが `<数字>.<数字>.<数字>` の形に完全に一致するものだけを消すようにする。接頭辞は同じだが形が違う古いファイルが残ることを確かめるテストを足す。 | New |
| R-03 | Minor | crates/u2-runtime/src/runner.rs:142-146 (`Waited::WaitFailed`) と crates/u2-runtime/src/lib.rs:57-64 (保証の範囲) | `try_wait` が失敗すると、子プロセスを止めも回収もせずに `start_failure = true` を返す。ジョブはまだ動いている(または回収されていない)可能性があるのに、u1-core はそれをエラー(127)として扱い、再開すると同じジョブの 2 つ目が、1 つ目と同時に動きうる。code-summary の 6 は「起動の失敗と区別がつかない」としか書かず、この重複実行の可能性と、lib.rs の「保証の範囲」には、この経路の記載がない。 | lib.rs の「保証の範囲」に、この経路(`try_wait` の失敗で、ジョブが動き続けたまま起動失敗として返る)を書く。ほとんど起きない経路だという根拠(`ECHILD` など)も添える。 | New |
| R-04 | Minor | crates/u2-runtime/tests/runner_signal.rs:60-79 と crates/u2-runtime/tests/runner_basic.rs:122-150 | 時間と環境への依存で、まれに失敗しうる。(1) `転送しない番号の合図…`(`sleep 1`)と `終了の観測の直後に…`(`sleep 2`)は、`ready` を見たあとに合図を立てるまでが、1〜2 秒より長くかかると、ジョブが先に終わって `interrupted` が立たず失敗する(負荷の高い CI)。(2) `sigint_も転送される` は、テストの実行が SIGINT を無視する状態(`nohup` や、非対話シェルのバックグラウンド実行)で始まると、無視の設定が子に引き継がれ、`terminated_by_signal == Some(2)` にならない。 | `sleep` の秒数を、`stop` ファイルが現れるまで続く形(`シグナルは_1回だけ転送される` と同じ)に変えて、時間への依存をなくす。SIGINT のテストは、子の起動の前に SIGINT の処理を既定に戻す、または、無視の状態を検知したら、理由を出して飛ばす。 | New |
| R-05 | Minor | aidlc/spaces/default/intents/261114-job-conductor-cli/construction/u2-runtime/code-generation/traceability.json (NFR4.9)、code-summary.md「実行していないテスト・測定」、unit-test-instructions.md「合計: 約 80 件」 | 記録が実態と少しずれている。(1) NFR4.9 が `OK`(対象は `runner.rs`)になっているが、確かめるのは Windows の CI の `.cmd` のテストで、まだ一度も実行されていない(nfr-requirements も「CI で確かめるまでは未確認」としている)。(2) code-summary は `cfg(windows)` のコードを「コンパイルも実行もしていない」とするが、`cargo check --target x86_64-pc-windows-msvc -p conductor-runtime --all-targets --locked` は通る(コンパイルは確認できる。実行は未確認)。(3) unit-test-instructions は約 80 件とするが、実際は 152 件である。 | NFR4.9 の `status` を、CI で確かめるまでは未確認とわかる形にする(または、対象に `runner_basic.rs` の `cmd_*` を足して、備考を付ける)。code-summary の記述に、Windows 向けの型検査が通ることを足す。unit-test-instructions の件数を直す。 | New |
| R-06 | Minor | crates/u2-runtime/src/state/reader.rs:43-47 | ステートのパスが名前つきパイプ(FIFO)のとき、`File::open` が書き込み側が現れるまで返らず、大きさの確認(`metadata`)と上限つきの読み取りに到達できない。ステートファイルを置くのは利用者の領分で起きにくいが、NFR4.3 / NFR4.7 の「終わりのない入力でも止まる」の範囲の外にある。 | 開く前に、`symlink_metadata` などで通常のファイルであることを確かめる(通常でなければ `unreadable`)。または、起きうることを lib.rs の「守る性質」に書く。 | New |
| R-07 | Minor | .idea/rust-job-conductor.iml(コミット 969b740 の変更) と aidlc/spaces/default/intents/261114-job-conductor-cli/construction/u2-runtime/code-generation/source-manifest.json | `source-manifest.json` の `writes` は `Cargo.lock` と `crates/u2-runtime/` だけだが、同じコミットは IDE の設定 `.idea/rust-job-conductor.iml` も書き換えている(`u2-runtime` の `sourceFolder` を 2 行足した)。code-summary の「作成・変更したファイル」にも書かれていない。 | この変更が意図したものなら、code-summary と source-manifest に載せる。IDE が自動で書いたものなら、この Unit の変更から外す。 | New |

### Validation Tool Results

| Tool | Result | Interpretation |
|---|---|---|
| `cargo test -p conductor-runtime --locked` | PASS(単体 78、`flow_with_state` 20、`state_read` 12、`state_write` 10、`runner_basic` 8、`runner_signal` 6、`nfr_robust` 6、`nfr_source` 5、`nfr_content` 3、`toml_depth` 3、`nfr_performance` 1。失敗 0) | code-summary の 152 件と一致する。1 回の実行で通ったが、時間に頼るテストの安定性は、この 1 回では判定できない(上の 4 件目)。 |
| `cargo clippy -p conductor-runtime --all-targets --locked -- -D warnings` | PASS | 警告なし。 |
| `cargo fmt --all -- --check` | PASS | 差分なし。 |
| `cargo check --target x86_64-pc-windows-msvc -p conductor-runtime --all-targets --locked` | PASS | `platform/windows.rs` と `cfg(windows)` のテストは、型検査まで通る(実行は CI で確かめる)。 |
| ソースの確認(unsafe、シェル、パニック) | `nfr_source` が通過。`unsafe` は `platform/unix.rs` の `libc::kill` の 1 か所 | NFR4.8 の条件 1(子の pid だけ)、2(`pid_t::try_from` で 0 以下を拒否)、4(`forwarded` で 1 回)はコードで確認できた。条件 3(回収の前)は、`try_wait` が `None` を返した回の後だけ送る順序で守られている(テストの不足は、上の 1 件目)。 |
| u1-core と `.gitignore` の変更 | なし(`git log` でも、この Unit のコミットはどちらにも触れていない) | 範囲の取り決めを守っている。 |

### Summary

signal・プロセスの安全(単一の `unsafe`、回収前の 1 回の送信、新しいプロセスグループ)、BR1.1 の分け方、読み込みの検査順、原子的な書き込み、`apply_*` の意味(個別実行の `@end` を含む)は、コードと設計の差分表に照らして整合しており、Critical・Major は見つからなかった。残るのは、回収後に送らない性質のテストの不足、古い一時ファイルの後始末が接頭辞だけで消すこと、`try_wait` 失敗時の重複実行の可能性の記載漏れなどの Minor であり、実装の前に直しておくとよいが、READY を妨げない。
