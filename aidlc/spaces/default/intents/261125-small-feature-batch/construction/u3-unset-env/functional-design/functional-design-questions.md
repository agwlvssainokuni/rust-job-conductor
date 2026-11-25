# 機能の設計の質問: u3-unset-env(`unset_env`)

出典: ユニットの定義(`unit-of-work.md` の U3)、要件(`requirements.md` の FR3.1〜FR3.5)、契約(`contract-summary.md` の C1、C3)、設計の決定(ADR-007)、今のコード(`crates/core/src/envfile/merge.rs`、`crates/core/src/definition/walk/env.rs`、`crates/runtime/src/runner.rs`)。

すでに決まっていること(質問しない): `[pipeline]` と `[job]` に名前の配列で書く。名前の規則は `env` と同じで、空・規則違反・`CONDUCTOR_*` は 64(FR3.1)。パイプラインとジョブの `unset_env` を合わせて使い、親から引き継いだ環境から消す(FR3.2)。同じジョブに当てはまる `env`・`env_file` の名前と重なれば 64、`.env` の中身との重なりは読んだときに 64(FR3.3)。統合の結果は「足す変数」と「消す名前」(C1、ADR-007)。書かなければ今と同じ(FR3.5)。名前の重なりの判定は、今の `env` の扱い(大文字小文字だけが違う名前は別の名前)に合わせる。

## Q1. [利用者] Windows での名前の比べ方

要件は「名前は完全一致で比べる」です(FR3.2)。ところが Windows の環境変数は、大文字小文字を区別しません(`Path` と `PATH` は同じ変数)。

A. OS のやり方に任せる。UNIX 系は完全一致、Windows は大文字小文字を区別せずに消す(Rust の標準の消し方と同じ)。README に書く
B. どの OS でも完全一致にする。Windows で `unset_env = ["PATH"]` と書いても、親の環境の `Path` は消えない
X. Other (please specify)

[Answer]: A

## Q2. [利用者] `--dry-run` で消す名前を見せるか

今の `--dry-run` は、各ジョブに足す環境変数の名前を出しています(値は出さない)。

A. 足す名前と並べて、消す名前も出す(値は無いので名前だけ)
B. 出さない(今の表示のまま)
X. Other (please specify)

[Answer]: A

## Q3. [利用者] `unset_env` の中で同じ名前を 2 回書いたとき

例: `unset_env = ["FOO", "FOO"]`、または `[pipeline]` と `[job]` の両方に `FOO`。

A. 許す。1 つにまとめて消す(同じ結果になるため)
B. 同じ配列の中の重なりは定義の誤り(64)。パイプラインとジョブの間の重なりは許す
X. Other (please specify)

[Answer]: A
