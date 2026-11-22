# 意図の把握の質問(Intent Capture Questions): 動きを変えない小さな整理とテストの補助

## Sources

- [desc] Initial description: "動きを変えない小さな整理とテストの補助(調査の 1 と 2): (1) 文言の定数の重複(cli の messages.rs、core の環境変数の名前の規則・CONDUCTOR_・format_version・間隔の上下限)、runtime の一時ファイルの作り方の二重と境界、cli の push_line・job の場所・reset の案内の重複、到達しない分岐と効かない処理(decide の規則 5、args の lexopt の枝、retry.rs の既定値、simulate の unwrap_or と walk の try_from)、文字列での比較(コマンド名の逆引き、誤りの対象の直書き、reset --all の理由の文の比較)、説明のずれ(types.rs の生成元、sanitize のテスト名、args.rs の文法の表、仕様書 08 章の拒否の理由の表)、使われない program_name、walk の整理(success_declared、build_jobs の clone、integer_of の重複、depth の分岐)、(2) テストの補助の整理(README の読み取りと必須語句、run/kinds・median・SECRET・ステートの包み・status_of・send の重複、core の tests/common、JobExecution の手組みを exited などへ、cli の偽物の writes・台本の消費の順・分割済みのコマンドの記録・Default の derive、runtime の state/consistency.rs を core へ)。既存のテストの期待は変えない"
- [scope] Workflow-selected scope: `refactor`.
- [memory:M1] `aidlc/spaces/default/memory/project.md#Mandated`: "ALWAYS 変更は小さな単位に保ち、小さな単位でレビューする(チームの慣行)。 (affirmed 2026-11-20)"
- [memory:M2] `aidlc/spaces/default/memory/project.md#Mandated`: "ALWAYS 行カバレッジ 80% 以上を保つ(CI の `coverage` ジョブ。`--fail-under-lines 80`)。 (affirmed 2026-11-20)"
- [memory:M3] `aidlc/spaces/default/memory/project.md#Mandated`: "ALWAYS 最小の Rust(MSRV 1.100)でビルドとテストが通ることを保ち、引き上げるときは README の記載と一致させる(`build-check`、`msrv-test`、`msrv_consistency.rs`)。 (affirmed 2026-11-20)"
- [memory:M4] `aidlc/spaces/default/memory/project.md#Mandated`: "ALWAYS `cargo fmt` と `cargo clippy -D warnings` を通してからマージする(CI の `lint`)。 (affirmed 2026-11-20)"
- [memory:M5] `aidlc/spaces/default/memory/project.md#Forbidden`: "NEVER 今回の作業で新しい外部ライブラリ(依存)を追加しない。`.env` の読み込みとグラフの描画は自前で書く(インタビュー Q4)。 (affirmed 2026-11-20)"

## Q1. [プロダクト] 進め方(段ごとに止まるか)

背景: 中身は大きく 2 つ(本体の小さな整理、テストの補助の整理)で、前回の第 2 弾(約 36 コミット)と同じくらいの大きさになる見込みです。

- A. 設計とコードの計画は 1 回で承認し、実装は段の順に進める。各段の終わりにテストが通ることを確かめ、結果は最後にまとめて見せる(前回と同じ)
- B. 段ごとに止まり、結果を見せて、続けるかを確かめてから次の段へ進む
- X. Other (please specify)

[Answer]: A

## Q2. [プロダクト] テストのファイルを変えてよい範囲

背景: (2) のテストの補助の整理では、多くのテストのファイルの補助の関数を、共通の場所の関数に置き換えます。runtime の `state/consistency.rs` のテストは core に移します。cli のテストの偽物の中身(書くだけの `writes`、台本を消費する順、渡されたコマンドの記録)も直します。一方、意図の記述は「既存のテストの期待は変えない」です。

- A. 補助の置き換え・移動・偽物の中身の変更は認める。ただし、テストの名前、件数(移動したものも数える)、確かめる中身、期待する出力の文字列は変えない。設計の段で、変えるファイルと理由の一覧を、該当の行を検索で網羅して示し、承認を取る
- B. テストのファイルの変更は、本体の変更に伴って必要なもの(import など)だけにし、(2) の補助の整理は今回は外す
- X. Other (please specify)

[Answer]: A

## Q3. [プロダクト] 新しいテストを足すか

背景: 整理によって新しく確かめられるようになるものがあります。たとえば、コマンド名の逆引きを表にすると「名前 → 種類 → 名前」の往復を確かめられます。cli の偽物が渡されたコマンドを記録すると、cli の層で「分割済みのコマンドを渡し、分け直さない」ことを確かめられます。

- A. このような、整理で確かめられるようになったものに限って、少数の新しいテストを足す(設計の段で一覧を示す)
- B. 新しいテストは足さない(既存のテストが通ることだけで確かめる)
- X. Other (please specify)

[Answer]: A

## Q4. [プロダクト] 使われない `program_name` の扱い

背景: core の `Command::program_name` は、本体のコードから呼ばれておらず、説明も今の分割の仕方と合いません。使っているのは core の単体テスト 1 件だけです。消すと、そのテストも消えて件数が 1 減ります。

- A. メソッドとその単体テストを消し、仕様書の 1 行も直す(件数の減りは理由と一緒に記録する)
- B. メソッドは残し、説明だけを今の実装に合わせる
- X. Other (please specify)

[Answer]: A
