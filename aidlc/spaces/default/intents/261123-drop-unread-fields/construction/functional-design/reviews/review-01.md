## Review

**Verdict:** READY
**Reviewer:** aidlc-architecture-reviewer-agent
**Iteration:** 1
**Verdict Dictated:** no

### Findings

| ID | Severity | Location | Finding | Required action | Status |
|---|---|---|---|---|---|
| R-01 | Minor | aidlc/spaces/default/intents/261123-drop-unread-fields/construction/functional-design/functional-spec.md:166-167 (§4.6 の T2 の 4 行の「残る行で確かめられない点」) | `success.explicit` と `j998.explicit` を消すと、「書いた遷移の辺」と「書かずに記述順の次へ進む辺」の区別を確かめる行がテストから無くなる。残る `to` と `kinds` は、`on_success = "j999"` と書いた定義でも同じ値になるため、「記述順の次へ進む辺であることを確かめる」は言い過ぎで、列の「なし」も正確でない。本番はこの区別を読まないので動きは変わらず、消してよい判断だが、承認する人にはこの点が見えない。 | 「残る行で確かめられない点」の欄に「書いた遷移か記述順の次かの区別(本番で読まれない項目なので、確かめる対象が無くなる)」と書く。承認の文面に、この区別が確かめられなくなることを含める。 | New |
| R-02 | Minor | aidlc/spaces/default/intents/261123-drop-unread-fields/construction/functional-design/functional-spec.md:168 と :282 (§4.6 の T7 の行と「12 行」の数え方) | T7 の `env_merge.rs:95-100` は、コメント 1 行、`let origin` の 1 行、`assert_eq!` の 4 行で 6 行ある(実物で確認した)。行には「コメントと補助と 4 行」とあり、§10 は「T7 の 5 行」と数えるので、12 行の内訳が読み手によって 11 行・12 行・13 行になる。補助 `origins`(`:53-58`)の削除と、`:180-181` の書き換えは別の扱いで、§4.3 の T7 の行にしか書かれていない。 | 「5 行」を「確かめの 5 行(`let origin` と 4 つの `assert_eq!`。コメント 1 行も一緒に消す)」のように数え方を書く。12 行の内訳を 1 か所の表にそろえる。 | New |
| R-03 | info | aidlc/spaces/default/intents/261123-drop-unread-fields/construction/functional-design/functional-spec.md:30、:70-80 (D1 の決定) | D1 の決定で、名前の 1 項目だけの構造体 `EnvNameEntry { name }` が本番に残る。cli を変えずに済み、型の削除より差分は小さいが、1 項目の包みが残る(推奨の案の判断の手がかりに書いてある内容のとおり)。本番に新しい要素は足していない(項目が減るだけ)ので、方針の違反ではない。 | 対応は不要。ユーザーの決定のとおり。承認の記録に、包みが残ることを書き残しておく。 | New |

### 確かめたこと(根拠)

- **網羅(観点 1)**: 設計の §4.0 の検索を HEAD 79ab4d0 で走らせた。`explicit` は `graph/mod.rs:60`、`graph/edges.rs` と、T1〜T5 の 9 か所(`graph_edges.rs:42,49`、`graph_lookup_perf.rs:146,150,156,160`、`graph_drawable.rs:264`、`graph_types.rs:32`、`graph_common/mod.rs:135`)だけだった。`origin` と `EnvOrigin` は core の src の 5 か所と、T6〜T9 の箇所だけで、runtime、cli の src、converter、Windows でだけコンパイルされるテストに参照は無い。`EnvFileContent.path` を読むのは `env_file_parse.rs:120` の 1 件だけで、`EnvEntry.line` を読むのは `env_parse_perf.rs:66` の 1 件だけ(`cli/src/report/mod.rs:579` の `e.path` は別の型)。構造体を直接作る所は `plan_types.rs`、`env_merge_perf.rs`、`plan_perf.rs`、`env_types.rs`、cli の `plan_report.rs` で、T10〜T20 とすべて一致する。`merge.rs` の `Item` と `collect_*`・`push_builtins`・`merge_env_names` の参照は `merge.rs` の中と `simulate.rs:306` だけで、コミット 2・3 の範囲に入っている。`graph_edges.rs` の `edge(` の呼び出しは 14 件(単行 13 件と `:153-159` の複数行 1 件)で T1 の数と合う。src の単体テストと `[`..`]` 形の rustdoc リンクに消す名前への参照は無い。
- **§7 の検索の基準(観点 5)**: 消す前の状態で、`名前と出どころ`・`出どころを持` は crates に 8 件(simulate 2、merge 1、plan/mod 3 のうちドキュメントコメント、テスト名 2 件)で、設計の直す一覧と合う。`line: line_no` は `parse.rs:88`(警告)と `:104`(項目)で、5 の後は警告の 1 件だけになる。`pub line` は 3 件で、5 の後は `EnvFileError` と `EnvWarning` の 2 件。`pub path|origin|explicit` は消す 3 件が当たる。仕様書の `origin`・`explicit`・`出どころ` を直す前に検索すると、S1〜S21 の行(行番号を含む)と過不足なく一致した。残すべき語句は `00-README` の層の用語 2 件(rust 75、la 91)だけで、`EnvWarning { file, line, name, kind` の行(rust 03:364)と la 03:530 の「警告(重複キー: ファイル、後ろの行、名前)」は S17〜S20 の直し方に含まれない。`EnvNameEntry` の rust の 4 行(02:42、05:724、860、861)も数が合う。定義の `format_version` と `description` に触れる行は 1 つも無い。凍結した仕様書を直す行も無い。
- **§4.6 の削除(観点 2)**: T7 は同じテストの `:86-93` が `V`、`W`、`P` を層ごとに違う値で確かめ、`CONDUCTOR_JOB_ID` は `:154-155` が値を確かめる。T8 の `:106` は直前の `e.name == b` が自動の変数の名前を確かめ、`:123` と `:147` は `content` が値を `"{ファイル}-{位置}"` で作るので、値の接頭辞(`p.env-`、`j.env-`)で層が分かる。T10 は `:115-118` が警告の `file == "t.env"` を確かめる。T14 は直前の 2 行が名前の並びと後半の値(`b{i}`)を確かめ、警告の 5,000 件も確かめる。T2 だけ、上の minor の点がある。
- **利用者から見える動き(観点 3)**: cli の `plan_text.rs:161-165` は `e.name` だけを読む。`EffectiveEnv`・`EffectiveEntry` を cli と runtime の src が `Debug` で出す箇所は無い(`entries` は `plan.entries` の別の型だけ)ので、`Debug` の項目が減っても出力に出ない。`graph` の text と Mermaid は `from_job`・`to`・`kinds`・`skipped_jobs` だけを読む(`explicit` は並べ替えにも使われない)。`sort_by_key` は安定で、辺の並びとまとめ方は変わらない。重複キーの警告の行は `parse.rs:88` の `line_no` で、変えない。`LoadedEnvFiles` は位置(`by_ref`)で引くので、`path` を消しても引き方は変わらない。実行時の並びと上書きの順は `keep_last_positions` と `additions()` で、変える行は無い。
- **本番への足し算(観点 4)**: 設計が本番のコードに足すものは無い(項目と引数と戻り値の要素を減らすだけ)。テスト専用の引数・`pub`・関数も足さない。`merge_env_names` は戻り値が `Vec<&str>` になるが、`pub(crate)` のままで、名前を読む用途に使うだけ。
- **件数とコミット(観点 6)**: 消すテスト 0、足すテスト 0 で 1,153 のまま。コミットは 5 件で、項目ごとの途中の形(2 の後は `Item` に出どころが残り `merge_env_names` だけが名前を返す、3 の後に `EnvOrigin` と再公開が消える、4 と 5 で `path` と `line` を別々に消す)が、import の消し忘れと `-D warnings` に当たらない順になっている。複数のコミットにまたがるファイルの表(§8)の各段階の形も、実物の使い方と矛盾しない。
- **制約(観点 7)**: 新しい依存、`unsafe`、src の `cfg`、版の更新、凍結した仕様書の変更は、設計のどこにも含まれない。converter と runtime の変更は 0 件。
- **traceability.json と rules.md**: SM1〜SM7 が BR に対応し、BR4.1 と BR4.2 の S 番号の分け(rust と la)は inventory §4 の表と一致する。`entities.md` の説明も D1 の決定のとおり。
- **検証ツール**: この段にはツールの指定が無い。HEAD 79ab4d0 の実物に対する検索だけを行った(コードは変えていない)。テスト 1,153 通過の基準の件数は、依頼の記載に従い、再実行していない。

### Validation Tool Results

| Tool | Result | Interpretation |
| --- | --- | --- |
| grep(§4.0 の検索を HEAD で再実行) | 消す名前の参照はすべて T1〜T20 と本番の変更の一覧に入っている | 網羅に漏れは見つからない |
| grep(仕様書の `origin`・`explicit`・`出どころ`・`EnvNameEntry`) | S1〜S21 と一致し、残す記述(00-README の 2 件、重複キーの警告の行)は含まれない | 直し漏れと消しすぎの両方を §7 の検索で見つけられる |
| cargo test / clippy | 実行していない(設計の段で、コードは未変更) | 設計の検証は、設計書が記す写しの木での試し(30 件のエラー、1,153 通過)に頼る。再現はコード生成の段で確かめる |

### Summary

消す名前のすべての参照が T1〜T20 と本番の変更に入っており、漏れは見つからなかった。§4.6 の削除は、1 件(T2)を除き、残る行が同じ意味を確かめる。T2 は、書いた遷移か記述順の次かの区別が確かめられなくなる点を明記すれば足りる。利用者から見える動き、仕様書の直す記述と残す記述の分け方、件数、5 件のコミットの並び、制約はいずれも問題無し。指摘は minor 2 件と info 1 件で、READY。
