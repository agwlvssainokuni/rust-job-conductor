# Tech Stack Decisions(u4-converter)

上流: functional-spec、rules、requirements(NFR1〜NFR6)、contract-summary(C4)、team-practices(`rustfmt`・`clippy`、test-after、`cargo test`、行カバレッジ 80%)、質問への回答(Q1〜Q3)、u1-core の tech-stack-decisions(同じワークスペースの方針)。実装言語は Rust(constraint-register の C4)。この Unit(u4-converter、library)に関わる選択だけを記す。各行は、inception の NFR の ID に枝番を付けた ID を持つ。

## 選択

| ID | 項目 | 選択 | 理由 | 出典 |
|----|------|------|------|------|
| NFR2.1 | 言語と配布の形 | Rust。外部のランタイムに依存せず、単一バイナリに組み込める形にする。C など、他の言語のライブラリには依存しない(Rust だけで完結するライブラリを選ぶ) | 単一バイナリ、3つの OS 対応のため | NFR2、NFR3 |
| NFR3.1 | OS への依存 | この Unit に、OS 固有のコードを置かない。ファイルの読み取りは、標準ライブラリのパス型で行う | Linux・macOS・Windows で、同じ結果にするため | NFR3 |
| — | ワークスペースと edition | u1-core と同じワークスペース(`crates/` の下に `crates/u4-converter/`)。edition 2024、最小のバージョンは 1.85(u1-core と同じ) | 同じバイナリに組み込まれ、同時にビルドされるため | team-practices |
| NFR4.1〜NFR4.4 | YAML の読み書き | ライブラリは、いま固定しない。次の条件を、Code Generation の最初に確かめて選ぶ [Q1: A]: YAML 1.2 のコアスキーマ(`yes`・`on` は文字列)、アンカー・エイリアス・マージキーの扱い、重複するキーと再帰するアンカーの検知、文法の誤りの位置(行)、タグつきの値を検知できること、Rust だけ、保守が続いていること。条件を満たすものがなければ、設計の規則(BR1.2、BR1.7)を見直す | 設計の規則(読み戻すと同じ木、YAML 1.2、マージキー)を満たす必要があるため。名前を先に決めると、条件を満たさないリスクがあるため | rules の BR1.2、BR1.7、BR1.11 |
| NFR4.1〜NFR4.4 | TOML の読み書き | u1-core と同じ `toml` のライブラリを使い、読み取りも書き出しも賄う。ワークスペースでバージョンを揃える [Q3: A]。日時の値を判別でき、整数の範囲を扱えること、書き出しで文字列を正しく引用符で囲めることを、Code Generation で確かめる | 依存を増やさず、ビルドを単純に保つため | NFR4、u1-core の tech-stack-decisions |
| — | エラーの型 | `thiserror` を使う(u1-core と同じ)。種類(`fileMissing` など)ごとに固定の表示の文を付ける | 記述が短く、u1-core と揃うため | rules の BR1.10 |
| NFR4.2 | 入力の大きさの上限 | 1 MiB。読み取りを「上限 + 1 バイト」までに制限する [Q2: A] | 誤って巨大なファイルを渡す事故の防止 | NFR4 |
| NFR4.3 | YAML の展開後の要素数の上限 | 100,000。展開しながら数え、超えた時点で打ち切る [Q2: A] | 展開で膨らむ入力から、メモリを守るため | NFR4 |
| NFR1.1 | 起動経路への負担 | この Unit は、`convert` コマンドのときだけ呼ばれる。`run` と `pipeline` の起動経路には、この Unit の初期化を含めない(YAML のライブラリの読み込みや初期化を、起動時に行わない) | 起動オーバーヘッド(100ms 以内)の予算を、定義の読み込みを行う Unit に残すため | NFR1 |
| NFR5.1 | テスト | `cargo test` による単体テスト。読み戻すと同じ木(BR1.11)の確認(往復のテスト)を中心にする。行カバレッジ 80% 以上。`tempfile` を開発用の依存とする(u1-core と同じ) | チームの実践(test-after、80%)と、BR1.11 の合否の基準 | NFR5、team-practices |
| — | 整形と静的検査 | `rustfmt` と `clippy`。CI で失敗したらマージできない | チームの実践 | team-practices |

## 依存するライブラリの方針

- この Unit が使う外部のライブラリは、YAML のライブラリ(Code Generation で選ぶ)、`toml`、`thiserror` を基本とする。いずれも、Rust だけで書かれていること(NFR2.1)。
- ライブラリの具体のバージョンは、Code Generation で、その時点の安定版を選び、`Cargo.lock` で固定する。
- ライブラリを増やすときは、起動オーバーヘッド(NFR1)と、単一バイナリ(NFR2)への影響を確認する。

## この Unit に該当しない NFR

- NFR6(状態の表示の判別性、色の無効化)は、表示を受け持つ u3-cli の Reporter に該当し、この Unit にはない。

## Assumptions & Open Questions

- YAML のライブラリが、BR1.2、BR1.7 の条件(YAML 1.2、マージキー、重複するキーの検知、行の位置、タグの検知)を満たすかは、Code Generation の最初に確かめる。満たすものがなければ、設計の規則を見直す(Functional Design に戻るか、規則を緩めるかを、人に確認する)。 [assumption]
- NFR1.1 は、u3-cli が、`convert` のときだけ、この Unit を呼ぶことを前提とする。u3-cli の設計で確かめる。 [assumption]
