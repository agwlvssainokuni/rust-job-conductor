<!-- INVARIANT: examples are single-line HTML comments so a fresh template parses to total=0 (MEMORY_EMPTY). Do NOT un-comment or split across lines. t100 guards this. -->
> This file is kept up to date automatically while the stage runs.

## Interpretations
<!-- example: 2026-05-29T10:14:32Z — chose REST over GraphQL; the consuming team only needs CRUD, revisit if subscriptions land -->

## Deviations
<!-- example: 2026-05-29T10:14:32Z — skipped the optional caching layer the stage prose suggested; the dataset is small enough that it adds risk -->

## Tradeoffs
<!-- example: 2026-05-29T10:14:32Z — picked TDD over BDD this run; the team is unit-first and the domain is well-understood -->

## Open questions
<!-- example: 2026-05-29T10:14:32Z — confirm the retention window with compliance before the next stage hardens the schema -->
- 2026-11-14T20:28:00Z — レビュー(1回目、READY)で挙がった指摘の引き継ぎ(NFR Design と Code Generation で拾う): (1) TOML の文法の誤りに対応する DefinitionError の種類が無い(NFR4.7)。Code Generation で `syntax` のような種類を足すか、`typeMismatch` に含めるかを決める。(2) 技術の選択(serde+toml、thiserror、1 MiB、edition 2024)の、退けた案とトレードオフの記録が、成果物に無い。NFR Design の判断記録(decisions)に書く。(3) `toml` の標準のエラー文は、ソースの行や値を含みうる。表示する理由は、範囲と種類から組み立てる(NFR4.6)。(4) 1 MiB の上限は、読み取り自体を「上限 + 1 バイト」までに制限して強制する。「上限超過」を `unreadable` に含めるのは BR1.9 の拡張であり、種類を明示する。(5) NFR4.7 の入れ子の上限は、ライブラリの機能に合わせて、Code Generation で数値にする。(6) NFR1.1、NFR2.1、NFR3.1 に、測定できる基準(例: 3つの OS の CI での成功、依存が Rust のみであることの確認)を足す。(7) 依存の方針: `Cargo.lock` をコミットしてバージョンを固定する、`cargo-deny` / `cargo-audit` でライセンスと脆弱性を確認する、版の検査を先に行う二段階の読み取り(まず `toml::Value`、次に型付きの構造)、行番号の算出の方法を、NFR Design に書く。
