# Code Summary(u4-converter)

15 ステップをすべて完了した。`cargo fmt --all -- --check`、`cargo clippy --workspace --all-targets --locked -- -D warnings`、`cargo test --workspace --locked`(この Unit は 103 件、失敗 0。u1-core の既存のテストも通過)が通った。

## 作成・変更したファイル

- `crates/u4-converter/`(パッケージ `conductor-converter`): `Cargo.toml`、`src/{lib,types,error,intake,raw,scalar,yaml_read,toml_read,resolve,yaml_write,toml_write}.rs`。
- テスト: `tests/{read_resolve,convert_roundtrip,convert_ac,nfr_security}.rs`、`tests/common/mod.rs`、`tests/fixtures/{pipeline.toml,pipeline.yaml,broken.yaml,broken.toml}`、`src/` 内の単体テスト。
- ワークスペース: `Cargo.lock`(`saphyr-parser`、`arraydeque`、`serde_core` を追加)。

## 主な実装の判断

- **YAML のライブラリ(Step 1 の確認結果):** `saphyr-parser` 0.3.4(MIT または Apache-2.0、rust-version 1.85.0)。イベントの列(`Parser`、`Event`)だけを使い、値の木(`saphyr`)は使わない。条件は、すべて実際のコードで確かめた: (a) 展開しないイベントを、再帰なしに読める(アンカー、エイリアス、位置つき)、(b) スカラーの解決は、`scalar.rs` で YAML 1.2 のコアスキーマとして行う、(c) タグが得られる、(d) 行の位置が得られる、(e) `cargo tree` に `-sys` がない、(f) 1.85.0 を宣言している、(g) 保守が続いている。フローの入れ子 1,000 段以上は、ライブラリが `ScanError`(recursion limit)を返し、異常終了しない。
- **TOML(T2 の確認結果):** `toml` 1.2.1。深さ 1,000 と 100,000 の入力は `Err` で返り、異常終了しない。深さ 33 は読めるため、読み取り後に深さを数え、33 を `syntax` にする。日時は判別でき、`i64` の端が扱え、文字列・`"<<"`・`nan`・`-inf`・`-0.0`・`1.0` を書いて読み戻せる。
- 流れは security-design のとおり(Intake → Read(YAML はイベント + 明示のスタック)→ RootCheck → Resolve(展開、数え、表せない値の記録)→ Write)。展開のために訪れたノードは、すべて数える(捨てられる項目、複製を含む)。
- テスト: 暴走の入力(深さ 100,000、アンカーの爆発、1,000 キーのマップを 200 回マージ)は、各 30 秒の上限(別スレッドと `recv_timeout`)の中で、すべて 1 秒以内に終わった。ランダムな入力 3,000 件と、フィクスチャのすべての先頭部分の入力で、パニックしなかった。

## 設計との差(計画からの逸脱と、設計の成果物への反映)

1. `toml` の機能に、`display` と `serde` を足した(書き出しと `toml::Table`・`Value` に必要)。`serde_core` が増える。
2. `toml` の `preserve_order` は有効にしていない(ワークスペースの機能の統合で、u1-core のマップの型が変わるのを避けるため)。そのため、**TOML のキーの順は整列された順**になる。TOML → YAML の出力のキーが、アルファベット順になり、TOML の入力で「入力に現れる順で最初の表せない値」は、整列の順になる(TOML で表せない値は日時だけで、影響は報告される key_path に限る)。security-design の §2「書き出しの順序は読み取りの順」と食い違うが、BR1.3 の範囲。 **→ 設計の隙間として、u3-cli またはワークスペースの設計で決める**(`preserve_order` をワークスペース全体で有効にするか)。
3. 公開の項目を足した: `#[doc(hidden)] pub mod testing`(往復の比較のため、`read_tree` と `Value`)、`intake::{read_input, file_label, MAX_INPUT_BYTES}`。
4. 権限なしのファイルのテストは、UNIX 専用のため、理由の対応の単体テストに置き換えた(NFR3.1)。実際の権限の経路は、実行していない。
5. YAML 1.1 の語(`yes`、`no`、`on`、`off`、`y`、`n`)は、YAML への出力で引用符つきにする(例: `"on": warning`)。YAML 1.1 の読み手でも文字列になる。往復の等しさには影響しない。
6. 進め方の逸脱: 開発担当のエージェントが、1 回、シェルの一時的なスクリプトで `resolve.rs` の一部を書き換えた(ファイルの書き込みはファイルのツールで行う、という規則に反する)。結果は通常のソースで、内容はレビューの対象。

## 実装の決まり(設計の隙間を埋めたもの)

- 要素の数え方: 訪れたノード(キーのスカラー、`<<` のキーを含む)を数える。エイリアスは、参照先の訪問として数え、エイリアス自身は数えない。100,000 は受理、100,001 は `tooLarge`(YAML と TOML の両方でテスト)。
- マージキー: 展開できないものは、その場で `unrepresentable`(`b.<<` のような key_path)。マージされるマップでは、マップ自身のキーが先、次にマージされたキー(優先順位の順)。`<<` は、引用符なし、タグなしの `<<` だけがマージキー。エイリアスを通してだけ現れる重複するキーは、Resolve で `syntax`。
- タグ: `!!str` は常に文字列。`!!int`・`!!float`・`!!bool`・`!!null` は、解決した型が合うときだけ受理。`!!map`・`!!seq` は、集合の型が合うときだけ。それ以外(`!!binary`、`!!set`、独自のタグ)は `unrepresentable`。
- 範囲: `i64` を超える整数、無限大に桁あふれする浮動小数(`1e999`)は `unrepresentable`。
- key_path の書式: `jobs[2].transitions[0]`(キーは `.` でつなぎ、配列は `[n]`)。キーは、エスケープして 64 文字に切る。文字列でないキーは、`<マップの経路>.(非文字列キー)`。表示は `file:line: <kind の文>: <reason>(場所: key_path)`。
- 空の YAML、コメントだけの YAML は `rootNotMap`(reason は「文書が空です」)。複数の文書の YAML は、ストリームの終わりまで読むため、後の文書の文法の誤りが、複数の文書の `unrepresentable` より先に返る(BR1.12 の 5 → 6)。最初の文書の木だけを保つ。
- TOML の文法の誤りの行は、バイトの範囲から求める。範囲のない失敗(非常に深いテーブルなど)は、行を持たない。`convert_text` は 1 MiB の上限を適用しない(`convert` の Intake が適用する)。

## 設計の隙間(後のステージ・Unit で解決する)

- **TOML のキーの順(上の 2):** `preserve_order` を、ワークスペース全体で有効にするか、整列のままにするか。
- **D4(エラー表示の補助の、u1-core との揃え):** u1-core のテストを変えていないため、共通の入力と期待値の表は、まだ置いていない。レビューで見比べる。
- **`cargo +1.85 build`:** この環境には、安定版(1.97)しかなく、最小の 1.85 でのビルドは、実行していない。let チェーンは使っていない。CI Pipeline のステージで、1.85 のビルドを確かめる。

## テスト(103 件、1 件は無視される補助)

| 範囲 | 件数 |
|------|------|
| `src/` の単体テスト | 47 |
| read_resolve | 26 |
| convert_roundtrip | 10 |
| convert_ac | 8 |
| nfr_security | 12(+ 無視される補助 1) |

## 実行していないテスト・測定

- **行カバレッジ:** `cargo llvm-cov` が未導入のため、測っていない。80% の判定は Build and Test が行う。
- **Linux と Windows:** macOS でだけ実行した。他の OS は、CI(CI Pipeline のステージで作成)で確かめる。
- CI のワークフローは、まだない(CI Pipeline のステージで作成)。
