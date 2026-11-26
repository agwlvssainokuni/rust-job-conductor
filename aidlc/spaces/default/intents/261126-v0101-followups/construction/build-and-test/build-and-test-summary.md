# ビルドとテストのまとめ: v0.10.1

出典: `construction/code-generation/code-generation-plan.md`(Testing Contract を含む)、`unit-test-instructions.md`、`code-summary.md`、要件(`requirements.md` の NFR1〜NFR8、FR3.3、FR4.1)、機能の設計(`rules.md` の BR3.6、BR4.1)、project.md の Mandated、`build-instructions.md`、`test-results.md`。

## 全体の状態と前提

- ビルド: 成功(`cargo build --workspace --locked`)。
- テスト: 手元の macOS で、全体 1,437 passed、0 failed(1 ignored は前からのもの。やり直し 1 の後)。`unit-test-instructions.md` の 7 つのコマンドもすべて通った。CI 45912018052 の 12 のジョブがすべて成功。
- 前提: Rust の toolchain、`cargo-llvm-cov`。依存は増えていない(`Cargo.lock` の package 46)。
- Test Strategy は Minimal、scope は refactor。追加のテストの手順の文書(結合・性能・セキュリティ)は作らない(Minimal の決まり)。

## テストの種類

| 種類 | 場所 | この作業で足したもの |
|---|---|---|
| 単体テスト | 各モジュールの `mod tests` | `runner.rs` の 2 つ(読む前の印、読んだ後の印) |
| 統合テスト | `crates/*/tests/` | `log_file.rs` の名前の表に 6 行 |
| 実バイナリのデモのテスト | `crates/cli/tests/unix_demo.rs` | `DEMO_PAUSE=1` の場面 4 の待ちの位置 |
| 文書のテスト | `crates/cli/src/tests/docs.rs` など | なし(既存が通る) |
| 性能のテスト | `*_perf.rs`、`nfr_performance.rs` | なし(上限は変えない) |

## カバレッジ

- プロジェクト全体の行カバレッジ: 98.22%(下限 80%。やり直し 1 の後。最初の実行は 98.17%)。Units は無いので、ユニットごとの目標は無い。

## Target Verification Matrix

| Target ID | Source | Expected | Actual | Evidence | Owning Stage | Verdict |
|---|---|---|---|---|---|---|
| T-COV | project.md Mandated、Testing Contract(NFR2) | 行カバレッジ 80% 以上 | 98.22%(最初の実行は 98.17%) | `test-results.md` の T-COV(`cargo llvm-cov … --fail-under-lines 80` が exit 0) | build-and-test | Met |
| T-SUITE | Testing Contract の scope_floor(NFR1) | 既存のテストがすべて通る | 1,437 passed、0 failed(最初の実行は 1,432) | `test-results.md` の全体のテスト | build-and-test | Met |
| T-UNIT | `unit-test-instructions.md` | 7 つのコマンドがすべて通る | すべて exit 0 | `test-results.md` の単体のコマンド | build-and-test | Met |
| T-LINT | project.md Mandated(NFR7) | `cargo fmt --check` と `cargo clippy -D warnings` が通る | どちらも exit 0 | `test-results.md` | build-and-test | Met |
| T-DEP | project.md Forbidden(NFR4) | 依存を足さない | `Cargo.lock` の package 46(v0.10.0 と同じ) | `test-results.md` | build-and-test | Met |
| T-SRC | project.md Forbidden(NFR5、NFR6) | `unsafe`・`cfg` の場所、ステートとエラーの中身の決まりを守る | `nfr_source.rs`、`nfr_content.rs`、`nfr_security.rs` が通る | 全体のテストに含まれる | build-and-test | Met |
| T-PERF | requirements.md NFR8、FR3.1 | 性能のテストの上限を変えずに通る | `plan_simulate_perf.rs` ほか通る。上限の変更なし | `test-results.md`、`git diff ea3e1d1..HEAD -- crates/core/tests/plan_simulate_perf.rs` が空 | build-and-test | Met |
| T-F3 | requirements.md FR3.2・FR3.3、rules.md BR3.4〜BR3.6 | 測定し、基準で判定する。直したときは BR3.5(llvm-cov の下で 1,000 ジョブ 75ms 以下、2 倍ごとの伸び 2.5 倍以下)、直さないときは表と理由を記録する | 最初は「原因でない」と判定して直さなかったが、CI で上限を超え、人の決定(やり直し 1)で直した。直した後、llvm-cov の下で 1,000 ジョブ P1 約 3.5ms、P2 約 2.9ms、伸びは最大 2.01 倍 | 本書の「F3 の測定の記録」、`code-summary.md` の Step 5 と「やり直し 1」 | build-and-test | Met |
| T-F4 | requirements.md FR4.1、rules.md BR4.1 | 「原因不明の失敗の記録の手順」の節に 5 つの残すものと手がかりがある | 下の節に書いた | 本書の「原因不明の失敗の記録の手順」 | build-and-test | Met |
| T-CI | project.md Mandated、requirements.md NFR3 | ubuntu・macos・windows の `test` と、`msrv-test`、`build-check`、`lint`、`coverage`、`docs-check` が通る | 12 のジョブがすべて成功(最初の実行 45908344996 では `coverage` が失敗。やり直し 1 の後に通った) | CI 45912018052、`test-results.md` の「やり直し 1 の後の実行」 | build-and-test | Met |

- CI Pipeline の段階はこの計画に無い。T-CI はチームの決まり(マージ前に CI を通す)と要件 NFR3 から来るもので、人の push の後に動く CI で確かめる。

## F3 の測定の記録(BR3.6)

`code-summary.md` の Step 5 の表を写す。手元の macOS(Apple Silicon)、使い捨てのテストで、各ジョブの数で 5 回作った中央値。P1 はステートなしで全ジョブが実行する計画、P2 は `j0` の成功で `@end` に着いた後のステートからの計画。

| ジョブの数 | debug P1 | debug P2 | llvm-cov P1(1 / 2 / 3 回目) | llvm-cov P2(1 / 2 / 3 回目) |
|---|---|---|---|---|
| 250 | 3.34ms | 2.73ms | 4.52 / 4.27 / 3.72ms | 2.95 / 2.86 / 2.86ms |
| 500 | 6.46ms | 6.18ms | 6.89 / 6.90 / 6.72ms | 6.54 / 6.56 / 6.47ms |
| 1,000 | 21.97ms | 21.26ms | 25.56 / 23.84 / 23.72ms | 23.38 / 24.00 / 23.31ms |
| 2,000 | 77.75ms | 75.38ms | 84.99 / 85.86 / 85.05ms | 84.66 / 87.92 / 88.60ms |

2 倍ごとの伸び(llvm-cov の下、1 回目):

| 段 | P1 | P2 |
|---|---|---|
| 250 → 500 | 1.52 倍 | 2.22 倍 |
| 500 → 1,000 | 3.71 倍 | 3.57 倍 |
| 1,000 → 2,000 | 3.33 倍 | 3.62 倍 |

- 判定: **直さない**。BR3.3 の「どの段でも 3 倍以上」を、250 → 500 の段が満たさない(2・3 回目も P1 1.62・1.81 倍、P2 2.29・2.26 倍)。
- ただし、**500 → 1,000 と 1,000 → 2,000 の段は 3 倍を超えており、500 ジョブより先は二乗で伸びている**。走査(`job_index`、`record_of`、規則 6 の前のジョブの走査、`apply_skip_in` の `any(Error)` と `first_pending`)が 1,000 ジョブで計画の時間の大半を占める(`code-summary.md` の走査ごとの表)。
- 手元の llvm-cov の下で、1,000 ジョブの計画は約 24ms で、上限 150ms の約 6 分の 1。CI で上限を超えた原因は、この測定からは分からない。再び起きたら、次の節の手順で記録する。
- **その後(やり直し 1)**: CI 45908344996 の `coverage` で上限を超えた(P1 161.33ms、P2 154.96ms。CI の機械は手元の約 6〜9 倍遅い)。人が「Retry with fix」を選び、この判定を覆して、設計の Q2 A の形(`flow` の内部の入口 `decide_at`・`classify_and_decide_at` と目印 `SimulationMarks`)で直した。直した後の測定(llvm-cov の下、手元):

| ジョブの数 1,000 | 直す前 | 直した後 |
|---|---|---|
| P1 | 23.65ms | 3.53〜3.56ms |
| P2 | 23.06ms | 2.75〜2.98ms |
| 2 倍ごとの伸び(最大) | 3.64 倍 | 2.01 倍 |

- BR3.5 に合格し、CI 45912018052 の `coverage` で `plan_simulate_perf.rs` が通った。BR3.6 の「直さないときの記録」は、最初の判定の記録として上に残す。
- 教訓: 時間の上限を持つテストの判断を、手元の速い機械の測定だけで行うと、CI の遅い機械での結果を見誤る。

## 原因不明の失敗の記録の手順(FR4.1、BR4.1)

テスト(CI でも手元でも)が、コードの変更と関係なく 1 度だけ落ちたときは、次のとおり記録する。コードもテストも、その場では変えない(上限を下げない、テストを外さない)。

1. **残すもの**(5 つ。すべて書く):
   - 実行の番号(CI なら GitHub Actions の run の番号と job の名前。手元なら日時)
   - 落ちたテストの名前(`crate` と、テストのファイルと関数の名前)
   - 失敗の出力(assert の文、期待と実際、パニックの位置。CI ならログの該当の行)
   - OS(ubuntu・macos・windows、CI の runner の種類。手元なら OS と CPU)
   - コミット(`git rev-parse HEAD` の値)
2. **書く場所**: その時に進んでいる intent の `construction/build-and-test/test-results.md` の「原因不明の失敗」の節(無ければ作る)。intent が無いときは、次の intent の要件の材料として、メモに残して人に伝える。
3. **もう一度走らせる**: 同じコミットで同じテストを 1 回だけ走らせ直し、通ったか落ちたかも書く。
4. **調べ始めるときの最初の手がかり**: 実時計を挟んで比べるテスト(例: `crates/cli/tests/binary_basic.rs` の 414〜417 行のような、時刻を取って前後を比べるもの)と、時間の上限を持つテスト(`*_perf.rs`、特にカバレッジの計測の下の `plan_simulate_perf.rs`)。同じテストが 2 回以上記録されたら、次の intent の課題にする。

## 準備の度合い

- build-ready: はい
- test-ready: はい(手元と CI 45912018052)
- deployment-ready: はい(版は 0.10.1。全体のテスト 1,437 passed、行カバレッジ 98.22%、CI の 12 のジョブがすべて成功)

## 残っていること・限り

- 最初の実行の T-CI の失敗(CI 45908344996 の `coverage`)は、やり直し 1(`test-results.md` の Loop-Back Log、1/3)で直した。
- F3 の直しのコミット(5405854、6104853)は、版の更新(152c4a1)の後に入った。版は 0.10.1 のまま(計画のやり直し 1 に書いたずれ)。
- Windows のログの名前(F2)とデモの Windows 版は、CI の windows の `test` で確かめた(成功)。
- 全体のテストは 1,437 passed(最初の実行の 1,432 に、R-06 の 5 つを足した)。
