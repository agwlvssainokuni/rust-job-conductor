# 機能の設計の質問: u1-pipeline-retry(パイプライン共通のリトライの既定)

出典: ユニットの定義(`unit-of-work.md` の U1)、要件(`requirements.md` の FR1.1〜FR1.6)、設計の決定(`decisions.md` の ADR-003、ADR-008)、契約(`contract-summary.md` の C3)、今のコード(`crates/core/src/definition/walk/retry.rs`、`walk/mod.rs`)。

すでに決まっていること(質問しない): `[pipeline.retry]` の項目と検査は `[job.retry]` と同じ(FR1.1)。ジョブに `retry` の表があれば丸ごと使い、既定と混ぜない(FR1.3)。打ち消しは `retry = false`、`retry = true` は誤り(ADR-008)。各ジョブの実効の retry は検証の終わりに 1 度だけ決め、リトライの判断と流れは変えない(ADR-003)。既定の `exit_codes` と、既定を当てはめるジョブの成功・警告の終了コードの重なりは 64(FR1.5)。誤りの表示に入力の値(ジョブの ID を含む)を入れない(NFR6)。

## Q1. [開発者] 既定と成功・警告の重なりの誤りを、どこに何回出すか

既定の `exit_codes`(例 `[1]`)が、あるジョブの `success_exit_codes` や `warning_exit_codes` と重なると 64 にします(FR1.5)。誤りの表示には値もジョブの ID も入れられないため、利用者が原因を探す手がかりは行の番号だけです。

A. 重なるジョブごとに 1 つずつ、そのジョブの行(`[[job]]` の位置)で出す。どのジョブが原因かが行で分かる
B. `[pipeline.retry]` の `exit_codes` の行で 1 回だけ出す。どのジョブと重なったかは分からない
C. 重なるジョブごとに 1 つずつ、`[pipeline.retry]` の `exit_codes` の行で出す(同じ行の誤りが数だけ並ぶ)
X. Other (please specify)

[Answer]: A

## Q2. [利用者] `[pipeline.retry]` が無いのに `retry = false` を書いたジョブ

打ち消す既定が無いため、`retry = false` は何も起こしません。書き間違いの疑いとして止めるか、無害なので通すかを決めます。

A. 許す(何もしない)。既定を後から足したり外したりしても、ジョブ側を書き直さずに済む
B. 定義の誤り(64)にする。意味の無い記述を残さない
X. Other (please specify)

[Answer]: A

## Q3. [利用者] `graph -v` と `--dry-run` で、既定から当てはめたリトライを見分けて出すか

今の `graph -v` と `--dry-run` は、ジョブのリトライの設定を 1 行で出しています。既定を当てはめた後は、どちらも「実効のリトライ」を出すことになります(ADR-003)。

A. 見分けない。実効のリトライを今の形のまま出す。定義の型は「既定から来たか」を持たない
B. 既定から来たものに印(例「(既定)」)を付ける。定義の型に、その区別を残す
X. Other (please specify)

[Answer]: A

## Q4. [利用者] 文書とひな形の更新の範囲

機能の最後に README とデモを更新します(FR8.1)。デモには、リトライを見せる `demo/retry.toml` と Windows 版の `demo/windows/retry.toml` があります。`conductor init` のひな形は、今 `[job.retry]` の例を含みます。

A. README とデモ(`demo/retry.toml` と Windows 版)を更新する。`init` のひな形は変えない
B. A に加えて、`init` のひな形にも `[pipeline.retry]` の例を足す
X. Other (please specify)

[Answer]: A
