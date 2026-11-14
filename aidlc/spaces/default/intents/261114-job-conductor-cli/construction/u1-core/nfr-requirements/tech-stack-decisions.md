# Tech Stack Decisions(u1-core)

上流: functional-spec、rules、requirements(NFR1〜NFR6)、contract-summary、team-practices(`rustfmt`・`clippy`、test-after、`cargo test`、行カバレッジ 80%)。実装言語は、要件で Rust と決まっている(constraint-register の C4)。この Unit(u1-core、library)に関わる選択だけを記し、バイナリ全体の選択は、u3-cli(service)で扱う。各行は、inception の NFR の ID に枝番を付けた ID を持つ。

## 選択

| ID | 項目 | 選択 | 理由 | 出典 |
|----|------|------|------|------|
| NFR2.1 | 言語と配布の形 | Rust。外部のランタイム(JVM、インタプリタなど)に依存せず、OS ネイティブの単一バイナリに組み込める形にする。C など、他の言語のライブラリには依存しない(Rust だけで完結するライブラリを選ぶ) | 単一バイナリ、3つの OS 対応のため。ビルドを単純に保つ | NFR2、NFR3 |
| NFR3.1 | OS への依存 | この Unit の部分には、OS 固有のコードを置かない。ファイルのパスは、標準ライブラリのパス型で扱う | Linux・macOS・Windows で、同じ結果にするため。OS の違いは、u2-runtime に閉じ込める(ADR-001) | NFR3 |
| — | Rust の edition とバージョン | 最新の安定版を使い、edition は 2024 とする。対応する最小のバージョンは、edition 2024 を使える最低のものに置く [Q2: A] | 新しい機能と、将来の保守のしやすさ。開発環境はこの版に合わせる | team-practices |
| NFR4.1、NFR4.2 | 定義ファイルの読み取り | シリアライズ用の `serde` と、TOML 用の `toml` を使う [Q1: A]。知らないキーは、型の定義で拒否する設定にする | 型の誤り・不明なキーの検出を、型に載せられ、実績がある | NFR4、rules の BR1.5、BR1.6 |
| — | エラーの型 | `thiserror` を使い、エラーの種類(`DefinitionError` など)に、表示の文を付ける [Q3: A] | 記述が短く、表示の文と型が近くにある | rules の BR1.x |
| NFR4.4 | 読み取りの大きさの上限 | 1 MiB。読み取りの前に、ファイルの大きさを確認する [Q4: A] | 誤って巨大なファイルを渡す事故の防止 | NFR4 |
| NFR1.1 | 起動経路への負担 | この Unit の起動時の処理は、定義ファイルの読み取りと検証に限り、1回だけ行う。起動時に、不要な初期化や、ネットワークへのアクセスを行わない | 起動オーバーヘッド(100ms 以内)の予算を、他の Unit に残すため。測定は u3-cli の起動経路で行う | NFR1 |
| NFR5.1 | テスト | `cargo test` による単体テスト。Flow の判断ロジックは、ファイルとプロセスなしで、境界条件まで網羅する。行カバレッジ 80% 以上 | チームの実践(test-after、80%)と、ADR-005 | NFR5、team-practices |
| — | 整形と静的検査 | `rustfmt` と `clippy`。CI で失敗したらマージできない | チームの実践 | team-practices |

## 依存するライブラリの方針

- この Unit が使う外部のライブラリは、`serde`、`toml`、`thiserror` の3つを基本とする(Q1、Q3 の選択)。いずれも、Rust だけで書かれている(NFR2.1)。
- ライブラリの具体のバージョンは、Code Generation で、その時点の安定版を選び、固定する。(このステージでは、バージョンの数字を決めない)
- ライブラリを増やすときは、起動オーバーヘッド(NFR1)と、単一バイナリ(NFR2)への影響を確認する。

## この Unit に該当しない NFR

- NFR6(状態の表示の判別性、色の無効化)は、表示を受け持つ u3-cli の Reporter に該当し、この Unit にはない。

## Assumptions & Open Questions

- `serde` と `toml` が、知らないキーを拒否する設定、ファイルの行の位置の報告を、必要な範囲で提供できるかは、Code Generation の最初に確かめる。足りない場合は、その部分だけを補い、選択(Q1: A)は変えない。 [assumption]
- ライブラリの具体のバージョンと、最小の Rust のバージョンの数字は、Code Generation で確定する。 [assumption]
