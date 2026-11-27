# 要件の分析: 質問

v0.10.2 で仕様書と `team.md` をどこまで直すかを決めます。今の状態は、読み直したコードの知識(`aidlc/spaces/default/codekb/rust-job-conductor/code-quality-assessment.md` の「仕様書と実装の差」と「テストの数」)と、引き渡し(`inception/reverse-engineering/developer-scan.md`)に基づいて書いています。実装と逆のことを書いた文(例: 01:64「出力を保存しない」、07:206「出力を読まない」、05:531「ステートは途中で書かれない」)は、「仕様書を実装に合わせる」という依頼のとおり、書き足しではなく書き換えるので、質問にしていません。

## Q1. [プロダクト] 直す仕様書の範囲

`docs/spec/rust/` の 13 文書には、v0.10.0 の機能がまったく書かれていません。`docs/spec/language-agnostic/` も「v0.10.1 までの挙動」と名乗りながら、v0.10.0 の機能の記述が無く、誤りの数も古いままです(33 種・13 種)。ほかの 5 つの言語版(csharp、go、powershell、python、typescript)は、冒頭の版が v0.8.0 か v0.9.0 です。

A. `docs/spec/rust/` だけを直す(依頼の文のとおり)。language-agnostic と他の言語版は、次の intent に回す
B. `docs/spec/rust/` と `docs/spec/language-agnostic/` を直す。他の言語版は次に回す
C. B に加えて、他の 5 つの言語版の冒頭に「この版は vX までの挙動」という注記を足すだけ行う
X. Other (please specify)

[Answer]: B

## Q2. [プロダクト] 書き足す内容の細かさ

v0.10.0 の機能(`watch --follow`/`--interval`、`log_dir`、`unset_env`、`[pipeline.retry]` と `retry = false`、表の `env_file`、`graph --format dot`、`[[job.attempt]]` と所要時間の列)と v0.10.1 の変更(ログの名前の空白、読む前の印、`flow` の内部の入口)を、どの細かさで書きますか。

A. 今の章と同じ細かさで書く(キー・型・理由の文・検査の順序・誤りの種類・終了コード・表示の文・データの形・関数と型の表、12 章の作り直しの段)。仕様書だけで作り直せる状態を保つ
B. 機能ごとの要点だけを書き、細部は README とコードを参照させる
X. Other (please specify)

[Answer]: A

## Q3. [プロダクト] `team.md` のテストの数の直し方

`team.md` の Testing Posture の「テストの種類」の数(core 33、runtime 14、cli 25、converter 2 のファイル、`crates/cli/src/tests/` 11、`crates/converter/src/tests/` 4、`#[test]` 合計 1,170)が古くなっています。今は 36・16・26・2、12(または 10)、4、1,460 です。`team.md` はチームの決まりの文書で、ふだんは段階の承認の場で直します。

A. コード生成の計画に「`team.md` の Testing Posture のその 1 行の数だけを書き換える」ステップを入れ、計画の承認(人の確かめ)を経て直接直す。数の他の文は変えない
B. 学びの手順(承認の場で人に「残しますか」と聞く)で、`team.md` の `## Corrections` に「テストの数は 2026-11-28 時点で…」の 1 行を足す。元の行は書き換えない
C. 直さない
X. Other (please specify)

[Answer]: A

## Q4. [プロダクト] `crates/cli/src/tests/` のファイルの数え方

`crates/cli/src/tests/` には、直下の `.rs` が 10 個、`mod.rs` と `common/` の中を含めると 12 個あります。`team.md` の 11 は、書いた時点ですべてを数えた値と読めます(推論)。

A. すべての `.rs`(`mod.rs` と補助を含む)を数え、12 と書く。数え方を括弧で添える
B. 直下のテストのファイルだけ(`mod.rs` を除く)を数え、10 と書く。数え方を括弧で添える
X. Other (please specify)

[Answer]: A

## Q5. [プロダクト] 範囲の外の古いコードの文書

`crates/runtime/src/lib.rs` の 72〜82 行の「呼び出しの順序」が、v0.10.0 より前の形のままです(仕様書ではなく、コードのドキュメントコメント)。

A. ついでに直す(v0.10.1 で同じファイルの契約の表を直したのと同じ扱い)
B. 直さない(次の intent に回す)
X. Other (please specify)

[Answer]: A

## Q6. [プロダクト] language-agnostic の書き足しの細かさと、AC の番号(Q1 B の続き)

Q1 で language-agnostic も直すと決まりました。language-agnostic は、受け入れの条件に番号(AC)を振る体系を持っています。Q2 A(今の章と同じ細かさ)を language-agnostic にもそのまま当てると、v0.10.0〜v0.10.1 の機能ごとに新しい AC の番号を振ることになります。

A. language-agnostic も今と同じ細かさで書き、新しい AC の番号を今の体系の続きで振る(誤りの数 34 種・14 種も直す)
B. language-agnostic は、誤りの数などの食い違いを直し、機能の挙動を要点で書く。新しい AC の番号は振らない
X. Other (please specify)

[Answer]: A

## Consolidated Summary Confirmation

Does this all look correct before I generate the artifact?

- 直す範囲: `docs/spec/rust/`(13 文書)と `docs/spec/language-agnostic/`。ほかの 5 つの言語版(csharp、go、powershell、python、typescript)は次の intent に回す(Q1 B)。
- 細かさ: どちらも今の章と同じ細かさで書く(キー・型・理由の文・検査の順序・誤りの種類・終了コード・表示の文・データの形、rust の関数と型の表と 12 章の作り直しの段)。language-agnostic には、新しい AC の番号を今の体系の続きで振り、誤りの数を 34 種・14 種に直す(Q2 A、Q6 A)。
- 対象の機能: v0.10.0 の `watch --follow`/`--interval`、`log_dir`、`unset_env`、`[pipeline.retry]` と `retry = false`、表の `env_file`、`graph --format dot`、`[[job.attempt]]` と所要時間の列、v0.10.1 のログの名前の空白・読む前の印・`flow` の内部の入口。実装と逆のことを書いた文は書き換える(依頼の文)。
- `team.md`: Testing Posture の「テストの種類」の 1 行の数だけを、コード生成の計画の承認を経て直接書き換える(Q3 A)。`crates/cli/src/tests/` は `mod.rs` と補助を含めてすべて数え(12)、数え方を括弧で添える(Q4 A)。
- `crates/runtime/src/lib.rs` の「呼び出しの順序」のドキュメントコメントも直す(Q5 A)。
- 最後に版を 0.10.2 にする(依頼の文)。

- Looks correct
- Request changes

[Answer]: Looks correct
