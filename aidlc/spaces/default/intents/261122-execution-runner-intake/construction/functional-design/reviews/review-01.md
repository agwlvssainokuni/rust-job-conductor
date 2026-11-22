## Review

**Verdict:** READY
**Reviewer:** aidlc-architecture-reviewer-agent
**Iteration:** 1
**Verdict Dictated:** no

### Findings

| ID | Severity | Location | Finding | Required action | Status |
|---|---|---|---|---|---|
| R-01 | Minor | aidlc/spaces/default/intents/261122-execution-runner-intake/construction/functional-design/functional-spec.md:243(N3)、:298(D4) | N3 は「root なら確かめない。既存の `権限がないと_unreadable` と同じ扱い」とある。だが既存のテストは `if let Err(e) = read_definition(&p)` で、root では読めて `Ok` になるので何も確かめずに通る。N3 の対象は 1 MiB 超なので、形 B では root でも開けて `Err(TooLarge)` が返る。既存と同じ書き方を写すと、root で実行したときに `Unreadable` の確認が落ちる。 | N3 の書き方を明記する。mode 000 にしたあとで `File::open` が成功するなら(root)、テストを早く終える(確認しない)。既存の `if let Err` の形を写さない。 | New |
| R-02 | Minor | functional-spec.md:§2.2.2、:§6 コミット 5 | `run_job` に残す「空のプログラム名 → `StartFailed`」の防御が、起動前の合図の確認より前か後かを決めていない。今のコードは、分割の失敗(`runner.rs:119-125`)が合図の確認(`:127-132`)より前にある。後ろに置くと、空のプログラム名で合図が先に立っていたとき、今の `StartFailed` が `NotStarted` に変わる(動きを変えないという段 2 の方針に反する)。 | 防御を合図の確認より前に置くと決めて書く。N2 または T17 で、その順(合図が立っていても `StartFailed`)を確かめるかどうかも決める。 | New |
| R-03 | Minor | functional-spec.md:§2.3.1(「確かめてから開くまでに差し替わっても…」)、§2.3.2 の FIFO の行 | 形 B でも、手順 1 のあとにパスが FIFO に差し替わると、手順 2 の `File::open` で待ち続ける。「今の core は止まらない」「形 B は止まらない」の記述は、差し替えの競合を除いた話になっている。実害は小さい(通常のファイルを FIFO に差し替える競合のみ)。 | 注記を足す。「止まらない」は競合を除く、と書く(仕様書の注記にも同じ限定を付ける)。 | New |
| R-04 | Minor | functional-spec.md:§2.1.1 | `ExecutionResult` を `conductor_core` のルートから公開するかが書かれていない。runtime、cli、各テストが `ExecutionResult` を import する(T1〜T12)。`JobExecution` は今、core のルートから公開している。 | `conductor_core` のルートの `pub use` に `ExecutionResult` を足すと明記する(コミット 2 の範囲)。 | New |
| R-05 | info | functional-spec.md:§2.1.3、§2.1.4、§4.2 の T1 | 確認したこと: 今の 13 の作り場所(inventory 1b)は、6 行の対応表に漏れなく入る。`classify`(順 1・3・4・5・6・7〜10)、`result_detail`、`is_retry_target`、`step.rs` の `after`・中断の表示、cli の `last_exit_code` は、表のとおり同じ結果になる。順 2 は作れない場合なので消してよい。`JobExecution` に `serde` の derive は無く、ステートの書式は変わらない。T1 の偽物が `StartFailure` に合図を足さなくなる点は、`Exec::StartFailure` を使う 5 か所のテストが `during_job` を使わないので、結果は変わらない。 | 対応不要。 | New |
| R-06 | info | functional-spec.md:§2.3.2 | 確認したこと: 形 B は、無い(`fileMissing`)、ディレクトリと FIFO(手順 1 で `unreadable`・通常でない。Windows のディレクトリも開かない)、権限なし小(`unreadable`・権限)、権限なし大(`unreadable`・権限。今は `tooLarge`)、上限超過(開いたハンドルの大きさで `tooLarge`)、UTF-8 でない(`unreadable`)、親に入れない(`unreadable`・権限)のどれも、表の種類になる。1 MiB 超で UTF-8 でない場合は `tooLarge`(読み取りの上限超過で)になる。converter の `read_input` と比べて、FIFO 以外は種類が同じ。 | 対応不要。 | New |
| R-07 | info | functional-spec.md:§4.2、§5、§6、§2.3.4 | 確認したこと: 変更箇所の件数(段 1 は 3+3+7+3+1+12+5+4+9+5+2+4 = 58、段 2 は 2+10+1+1+3 = 17)は合う。`run_job(` を呼ぶテストの場所(runtime の `tests` 10 か所と `runner.rs` の単体 4 か所)は T12、T14〜T17 で拾えている。件数の見込みは 1,137(今回 `cargo test --locked --workspace` で確認。1 無視)→ 1,138 → 1,139 → 1,142 で整合する。各コミットでビルドが通る並びで、コミット 2 を 1 つにまとめる理由も妥当。版の更新の範囲は `ea09b46`(16 の仕様書、`Cargo.toml` × 4、`Cargo.lock`、`binary_basic.rs`)と同じで、「v0.9.5 までは」の注記を上げない扱いも正しい。`nfr_source.rs` の `Command::new(&line.program)` の検査は、引数の名前を `line` にすれば通る。新しい依存、`unsafe`、`cfg` は足さず、converter は core に依存しない。 | 対応不要。 | New |

### Summary

設計は実装できる水準にあり、READY とする。段 1 の対応表と同値の論証は、コードの読み取り(`classify.rs`、`retry.rs`、`step.rs`、`runner.rs`)と合っていて、ステートの書式も変わらない。段 3 の形 B は、各場合で意図した誤りの種類になる。★ の変更(D3)は、型で作れなくなる組み合わせか防御の移動か版の更新で、理由は妥当。指摘は、N3 の root での扱い、空のプログラム名の防御の位置、FIFO の差し替えの競合の注記、`ExecutionResult` の公開の 4 点の軽微なものだけ。
