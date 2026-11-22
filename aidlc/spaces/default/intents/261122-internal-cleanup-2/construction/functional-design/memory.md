<!-- INVARIANT: examples are single-line HTML comments so a fresh template parses to total=0 (MEMORY_EMPTY). Do NOT un-comment or split across lines. t100 guards this. -->
> This file is kept up to date automatically while the stage runs.

## Interpretations
<!-- example: 2026-05-29T10:14:32Z — chose REST over GraphQL; the consuming team only needs CRUD, revisit if subscriptions land -->
- 2026-11-22 — 「上限の数と定数の const assert」は、調査の場所に加えて runtime/src/error.rs:103 と cli の英語の文 messages.rs:324・326 にも同じ数があるため、対象に含めた(inventory.md 1h)。
- 2026-11-22 — 仕様書 11 章のテストの件数は、直すのでなく書くのをやめると決めた。ずれが 5 か所あり、OS で件数が変わる(windows_demo は macOS で 0)ため(inventory.md 1f)。

## Deviations
<!-- example: 2026-05-29T10:14:32Z — skipped the optional caching layer the stage prose suggested; the dataset is small enough that it adds risk -->
- 2026-11-22 — 調査との違い: show_path は未使用でなく init の表示が使う。使い方の誤り 68 は 9 でなく 7 か所。「親が空なら .」は 4 か所。resume.rs にも O(n²) の同じ形がある。実物に従った(functional-spec.md §3)。
- 2026-11-22 — .env の解析の重複の除去は任意にした(上限の入力で debug 40.6 ms。性能のテストは足さない)。

## Tradeoffs
<!-- example: 2026-05-29T10:14:32Z — picked TDD over BDD this run; the team is unit-first and the domain is well-understood -->
- 2026-11-22 — PlanMode・EdgeKind・EdgeTarget は定義を消して pub use … as の別名にした。古い名前を消すと graph と plan のテスト 5 ファイルの変更が要るため、承認の対象を増やさない方を選んだ(functional-spec.md §4 の注)。
- 2026-11-22 — RunIdArg に Deref<Target = str> を持たせればテストの 5 行を変えずに済むが、列挙型の意味をぼかすので as_str とテストの読み方の変更(T7)を勧め、代案として併記した。
- 2026-11-22 — 性能のテストの上限は、直す前の debug の中央値の 4.6〜18%、直した後の見積もりの 5.7〜11.5 倍に置いた(CI の遅い機械と計測つきの coverage でも落ちず、二乗に戻れば手元でも落ちる)。

## Open questions
<!-- example: 2026-05-29T10:14:32Z — confirm the retention window with compliance before the next stage hardens the schema -->
- 2026-11-22 — functional-spec.md §4 の T1〜T8 のテストの変更(と T7 の代案の選択)は、ユーザーの承認が要る。
