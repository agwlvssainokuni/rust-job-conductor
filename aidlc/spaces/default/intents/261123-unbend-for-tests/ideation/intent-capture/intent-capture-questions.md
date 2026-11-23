# 意図の把握の質問(Intent Capture Questions): テストの都合で曲げられた本番のコードと、残りの整理

## Sources

- [desc] Initial description: "5 回目の調査の候補を 1 つの intent で(動きを変えない。版は上げない): (1) テストの都合で本番のコードが曲げられている所を直す(テストのコードの都合で本番のコードを曲げない、という前の intent の判断と同じ観点): cli の Reporter::language_resolved と color_resolved、runtime の ResetAllReport.removed、core の graph::is_lane_drawable、core の LoadedEnvFiles::add、runtime の remove_stale_temps の引数 now、core と converter の誤りの種類の code()、誤りの種類の一覧 ALL(5 つ。CommandKind::ALL を除く)、cli のパニックのフックの差し替え口(on_panic、install_panic_hook_with)、cli のシグナルの AlreadyRegistered と登録済みの印、core の graph::node_state_of と flow_edges の公開。(2) ふつうのリファクタリング: 古くなった説明(.env の重複キーの警告を graph も出す、run_validate の .env の検査、runtime の lib.rs のモジュール表、自動の再試行はしないという言い方 4 か所、converter のモジュール表、env_file_refs、Reporter::show)、ステートの definition_key と job_order(型から消し、ファイルの形は保つ)、runtime の CommandError.reason をメソッドに、ExitCodeClass、小さい整理(非公開にできる定数と関数、常に真の条件、テストだけの use の置き場所、Path の包み直し)、ステートが無いときの判断の重複(cli と計画)。範囲の外: converter の testing モジュール、cli のライブラリの公開範囲(別の intent)、上限つき読み取りの core と runtime の共有、skip_reason_kind、エスケープの確保、JobExecution の包み、辺の間のジョブの複製"
- [scope] Workflow-selected scope: `refactor`.
- [memory:M1] `aidlc/spaces/default/memory/project.md#Mandated`: "ALWAYS 変更は小さな単位に保ち、小さな単位でレビューする(チームの慣行)。 (affirmed 2026-11-20)"
- [memory:M2] `aidlc/spaces/default/memory/project.md#Mandated`: "ALWAYS 行カバレッジ 80% 以上を保つ(CI の `coverage` ジョブ。`--fail-under-lines 80`)。 (affirmed 2026-11-20)"
- [memory:M3] `aidlc/spaces/default/memory/project.md#Mandated`: "ALWAYS 最小の Rust(MSRV 1.100)でビルドとテストが通ることを保ち、引き上げるときは README の記載と一致させる(`build-check`、`msrv-test`、`msrv_consistency.rs`)。 (affirmed 2026-11-20)"
- [memory:M4] `aidlc/spaces/default/memory/project.md#Mandated`: "ALWAYS `cargo fmt` と `cargo clippy -D warnings` を通してからマージする(CI の `lint`)。 (affirmed 2026-11-20)"
- [memory:M5] `aidlc/spaces/default/memory/project.md#Forbidden`: "NEVER 今回の作業で新しい外部ライブラリ(依存)を追加しない。`.env` の読み込みとグラフの描画は自前で書く(インタビュー Q4)。 (affirmed 2026-11-20)"

## Q1. [プロダクト] 進め方(段ごとに止まるか)

背景: 中身は 2 つ((1)テストの都合で曲げられた所、(2)ふつうの整理)で、前回より小さい見込みです。

- A. 設計とコードの計画は 1 回で承認し、実装は段の順に進める。各段の終わりにテストが通ることを確かめ、結果は最後にまとめて見せる(前回と同じ)
- B. 段ごとに止まり、結果を見せて、続けるかを確かめてから次の段へ進む
- X. Other (please specify)

[Answer]: A

## Q2. [プロダクト] 既存のテストの扱い

背景: (1)では、本番から消す関数や項目を使っているテストを、別の書き方(公開の入口から確かめる、偽物の側で記録する、テストの補助に同じ処理を書く、単体テストへ移す)に直します。また、テストのための関数そのものを試しているだけのテスト(誤りの種類の識別子 `code()` が重ならないこと、など)は、関数と一緒に消えます。

- A. 認める。確かめる中身(利用者から見える動き)と期待する出力の文字列は変えない。書き方を変えるテスト、移すテスト、消すテストは、検索で網羅した一覧にし、消すものは理由(テストのための関数だけを試していた、など)を添えて設計の段で承認を取る
- B. テストを消すことはせず、書き方の置き換えだけにする(テストのための関数だけを試しているテストがある項目は、範囲から外す)
- X. Other (please specify)

[Answer]: A

## Q3. [プロダクト] cli のパニックのフックの差し替え口

背景: パニックしたときに「内部の誤り」を出して終了コード 70 で終わるフックは、終了の仕方を差し替えられる形(`install_panic_hook_with(exit)`)になっています。差し替えはテストのためです。本体(誤りの文を書いて 70 を返す)は、非公開の関数にして単体テストで確かめられます。ただし「フックを設定すると、パニックで 70 の終了が呼ばれる」ことを確かめる 1 件は、差し替えが無いと確かめられません(リリースのバイナリにはパニックを起こす経路を持たせない方針なので、実バイナリでも確かめられません)。

- A. 差し替え口をなくす。本体は非公開の関数にして単体テストへ移す。「フックを設定すると 70 で終わる」の 1 件は消し、確かめられなくなることを記録する
- B. 差し替え口は残す(この項目は範囲から外す。テストの都合の例外として記録する)
- X. Other (please specify)

[Answer]: A

## Q4. [プロダクト] cli のシグナルの `AlreadyRegistered` と登録済みの印

背景: シグナルの合図の登録は、本番では 1 回しか行いません。「すでに登録済み」の種類と、それを見分ける印は、同じプロセスで何度も登録するテストのためにあります(最初の設計書にもそう書かれています)。

- A. なくす。テストは、1 つのテストで 1 回だけ登録する形に直す(必要なら、テストのファイルを分けて別のプロセスにする)
- B. 残す(この項目は範囲から外す)
- X. Other (please specify)

[Answer]: A

## Q5. [プロダクト] 誤りの種類の一覧 `ALL` の置き場所

背景: 誤りの種類の一覧 `ALL`(5 つ)は、本番では使わず、テストだけが「全部の種類の表示を確かめる」ために使います。テストの側に置き直し、`match` で全部の種類を並べる補助を付けると、種類が増えたときにコンパイルで気づけるので、今より強くなります。代わりに、core の種類の一覧は core と cli の両方のテストに写しができます。

- A. テストの側へ移す(網羅の `match` つき)。本番から `ALL` を消す
- B. 今のまま本番に残す(この項目は範囲から外す)
- X. Other (please specify)

[Answer]: A
