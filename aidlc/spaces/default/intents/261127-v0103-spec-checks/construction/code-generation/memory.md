<!-- INVARIANT: examples are single-line HTML comments so a fresh template parses to total=0 (MEMORY_EMPTY). Do NOT un-comment or split across lines. t100 guards this. -->
> This file is kept up to date automatically while the stage runs.

## Interpretations
<!-- example: 2026-05-29T10:14:32Z — chose REST over GraphQL; the consuming team only needs CRUD, revisit if subscriptions land -->
- 2026-11-28T00:29:11Z — Step 2 の「関数名の部分の `_` を空白に」は、語の区切りの `_` を空白にし、`ac_7_1_1` のような番号の識別子はそのまま残すと読んだ(ほかの 588 個の書き方に合わせた)。`配列形式の_command_は往復する_*` の末尾の `_*` も `*` の前を空白にした。
- 2026-11-28T00:29:11Z — 全体の探しの #1・#2 は、設計の「手がかりより後ろで最も近い N 種」に、README の「34 種のエラーの種類」(数が前)と、1 行に 2 つの数が並ぶ行を正しく扱う規則を足した(すぐ前の「N 種の」、間に別の数の手がかりがあれば結び付けない、48 文字の上限)。設計の 3.1 の「調整してよい」の範囲と判断し、code-summary.md の付録 A に表で残した。
- 2026-11-28T00:29:11Z — #12 の全体の探しから「場面 1〜N」を外した。AC-433「場面 1〜2」のような場面の一部を指す範囲と区別できないため。違う数を逃がす調整ではない(当たる行の数はどれも正しい)。

## Deviations
<!-- example: 2026-05-29T10:14:32Z — skipped the optional caching layer the stage prose suggested; the dataset is small enough that it adds risk -->
- 2026-11-28T00:29:11Z — 依頼の「シェルでファイルを書かない」に従い、`cargo fmt` でファイルを書き換えず、`cargo fmt --all -- --check` の差分をファイルの道具で反映した(1 回は全体を Write で書き直した)。
- 2026-11-28T00:29:11Z — Step 5 の確かめのため、テストに一時的に `eprintln!` を入れて当たる行を表示し、コミットの前に取り除いた。
- 2026-11-28T00:29:11Z — 計画の補助の関数「5 つ」に加えて、失敗の集め役 `Checker` と、#1・#2 の結び付け `bound_numbers` を同じファイルに置いた(テストの中だけで、本番のコードは変えていない)。
- 2026-11-28T00:29:11Z — 計画のチェックボックス・code-summary.md・この日記は、Step 8 以降の依頼で続けて書くので、コミットしていない。

## Tradeoffs
<!-- example: 2026-05-29T10:14:32Z — picked TDD over BDD this run; the team is unit-first and the domain is well-understood -->
- 2026-11-28T00:29:11Z — 版の注記は行ごとに見るので、別の話の注記がある行(R02:116、R12:419、L12:355)の今の数は全体の探しで見ない。注記を文の単位で見る形は複雑になるので取らず、数が一致することを目で確かめて記録した。
- 2026-11-28T00:29:11Z — #10 の全体の探しは、設計の「`interval` か『秒』」では `retry.interval`・DEMO_SLEEP の「既定 1 秒」・`--assume` の「0〜255」を拾うので、手がかりの語より後ろで最も近い範囲と、`--interval`・`--follow` を含む行の既定に絞った。

## Open questions
<!-- example: 2026-05-29T10:14:32Z — confirm the retention window with compliance before the next stage hardens the schema -->
- 2026-11-28T00:29:11Z — L11:608(AC-214)の `convert_ac::ちょうど1_mibの正しい入力は変換できる` の `_` は、計画の 14 個に無いので残した。空白にするかは人に確かめる。
- 2026-11-28T00:29:11Z — R12:107 の段 10 の「13 種の文と理由」は、注記の無い履歴の数(今は 14 種)。設計の 3.2 の目安の一覧に無く、直さずに人に知らせる。
