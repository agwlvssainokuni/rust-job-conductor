# エンティティ(Entities): テストのファイルの統合

永続のデータ(ステートファイル)は扱わない。利用者に見える出力も変えない。本番のコードの型・関数・可視性は 1 つも変えない。変わるのは、テストのファイルの置き場所、テストの補助とひな形の原本の置き場所、仕様書のテストの記述だけ。詳細は `inventory.md`。D1〜D8 は、ユーザーが決めた(すべて推奨のとおり。2026-11-25)。下の D の箇所は、その決定のとおりの形を書いている。T 表(T1〜T13)は、設計の承認で認めてもらう(承認待ち)。

```yaml
entities:
  # ---- 新しく作るファイル(テストだけ。本番の成果物に入らない) ----
  - name: crates/cli/src/tests/docs.rs(cli の単体テスト)
    description: README の検査 18 件と補助(readme、DOCS_CHECK_WORDS、assert_readme_has、assert_docs_check_words、block_after_heading)。README の並びの 5 つの節(リトライ、環境変数、graph、--dry-run、init)。src/tests/docs_graph.rs を git mv して作る。見出しの定数は RETRY_HEADING・ENV_HEADING(D1。決定: 推奨のとおり。ユーザー、2026-11-25)。依存は cli の既存の dev-dependencies(tempfile)と通常の依存(conductor_core)だけ。
  - name: crates/cli/tests/common/fakes.rs(cli のテストの共有の補助)
    description: 偽の Env(FakeEnv)・Sink(FakeSink)と state_error。統合テストは tests/common/mod.rs の mod fakes で、単体テストは src/tests/common/mod.rs の #[path] で読む。trait(Env、Sink)と言語(Language)は use super:: で、読む側の親のモジュールから取る。tests/ の直下ではないので、cargo は統合テストの実行ファイルにしない。src/ の外なので、cli の nfr_source.rs の検査の対象にならない。
  - name: crates/cli/tests/fixtures/init_ja.toml・init_en.toml(テストのデータ。D8。決定: 推奨のとおり。ユーザー、2026-11-25)
    description: init のひな形の原本(名前は既定の my-pipeline)。中身は今のテストの生の文字列とバイト単位で同じ。tests/flow_init.rs と src/tests/init_template.rs が include_str! で読む(定数 ORIGINAL_JA・ORIGINAL_EN の名前と型は同じ)。
  - name: .gitattributes(リポジトリの根。D5。決定: 推奨のとおり。ユーザー、2026-11-25)
    description: crates/cli/tests/fixtures/*.toml text eol=lf の 1 行。ひな形のデータのファイルを、どの OS の取り出しでも LF に保つ(include_str! は改行を変えないため)。

  # ---- 中身が増えるファイル ----
  - name: crates/cli/src/tests/args_grammar.rs
    description: init の節に InitTarget::from_path の 1 件(元 init_types.rs)、--run-id と --all の節に RunIdArg::parse の 1 件(元 run_id_arg.rs)。resolve_run_id の節の fr2_5_ の 2 件を、新しい --remove-on-complete の節へ並べ直す(D4。決定: 推奨のとおり。ユーザー、2026-11-25)。
  - name: crates/cli/src/tests/graph_report.rs
    description: 最後の節に、65 文字以上のジョブ ID を切らずに出す 2 件と補助 long_id_definition・long_id(元 graph_long_id.rs。補助の名前は D2)。
  - name: crates/converter/tests/convert_ac.rs
    description: 出力は毎回同じ(元 tests/convert_roundtrip.rs)と、定数 PIPELINE_TOML・PIPELINE_YAML。
  - name: crates/converter/src/tests/convert_roundtrip.rs
    description: 最後の節に、ファイルを読む入口 convert の結果の木を比べる受け入れ基準の 2 件と補助 fixture・request(元 src/tests/convert_ac.rs)。
  - name: crates/converter/tests/nfr_security.rs
    description: 表示の節の先頭に、変換の失敗の種類の一覧 CONVERSION_ERROR_KINDS、網羅の match、コンパイル時の確かめ(元 tests/common/kinds.rs。別名は K)。製品のコードの検査の部分は変えない。
  - name: crates/runtime/tests/runner_basic.rs
    description: Windows だけの 3 件の前の節に、補助 child_env と、run_job の環境変数の引数の 5 件(元 runner_env.rs)。
  - name: crates/runtime/tests/state_read.rs
    description: 最後の子のモジュール mod raw に、定数 COMPLETED と read_state_raw と read_state の差の 4 件(元 state_read_raw.rs。D3。決定: 推奨のとおり。ユーザー、2026-11-25)。

  # ---- 写しが無くなるファイル ----
  - name: crates/cli/tests/common/mod.rs(統合テストの補助)
    description: FakeEnv・FakeSink・state_error の写しを消し、mod fakes と再公開にする。テスト用の写しの enum Language と、ほかの偽の実装(Runtime、Signals、MemRuntime、Harness など)は残る。
  - name: crates/cli/src/tests/common/mod.rs(単体テストの補助)
    description: 写しを消し、kinds と、#[path] で読む fakes の再公開と、fakes が使う Env・Sink・Language の import だけになる(29 行)。

  # ---- 無くなるファイル(19) ----
  - name: cli の README のテスト(10)
    description: tests/docs_{retry,env,graph,dry_run,init}.rs、tests/docs_support/mod.rs、src/tests/docs_{graph,dry_run,init}.rs、src/tests/docs_support.rs。統合テストの実行ファイルが 5 つ減る。
  - name: cli の小さな単体テスト(3)
    description: src/tests/init_types.rs、src/tests/run_id_arg.rs、src/tests/graph_long_id.rs。
  - name: converter(4)
    description: tests/convert_roundtrip.rs、src/tests/convert_ac.rs、tests/common/mod.rs、tests/common/kinds.rs。統合テストの実行ファイルが 1 つ減る。
  - name: runtime(2)
    description: tests/runner_env.rs、tests/state_read_raw.rs。統合テストの実行ファイルが 2 つ減る。

  # ---- 仕様書の記述(実装に揃える) ----
  - name: docs/spec/rust の 02・07・11 章
    description: テストのファイルの表と、偽の実装の置き場所の文を、移した後のファイルに揃える(S1〜S16)。
  - name: docs/spec/language-agnostic の 11 章の根拠の欄
    description: 動かしたテストを指す 10 か所を、新しいモジュールの名前に揃える(L1〜L10。D6。決定: 直す。推奨のとおり。ユーザー、2026-11-25)。772 行目は直さない(D7。決定: 直さない。推奨のとおり。ユーザー、2026-11-25)。
```

## 利用者に見える変わること

なし。`conductor` の出力、誤りの表示、終了コード、ステートファイル、`--dry-run` の出力、`conductor --version`(`conductor 0.9.8`)は変わらない。配布する実行ファイルの中身も変わらない(本番のコードを変えないため)。
