# Tech Stack Decisions(u2-runtime)

上流: functional-spec、rules、requirements(NFR1〜NFR6)、contract-summary(C3、E3)、team-practices(`rustfmt`・`clippy`、test-after、`cargo test`、行カバレッジ 80%)、質問への回答(Q1〜Q3)、u1-core・u4-converter の tech-stack-decisions(同じワークスペースの方針)。実装言語は Rust(constraint-register の C4)。この Unit(u2-runtime、library)に関わる選択だけを記す。各行は、inception の NFR の ID に枝番を付けた ID を持つ。

## 選択

| ID | 項目 | 選択 | 理由 | 出典 |
|----|------|------|------|------|
| NFR2.1 | 言語と配布の形 | Rust。外部のランタイムに依存せず、単一バイナリに組み込める形にする。C のコンパイラを必要とするライブラリには、依存しない(`libc` は、C の関数の宣言だけで、C のコンパイルは要らない) | 単一バイナリ、3つの OS 対応のため | NFR2、NFR3 |
| NFR3.1 | OS への依存 | OS ごとの違い(シグナルの送信、プロセスグループ)は、この Unit の中の1つのモジュール(`platform`)に閉じ込め、`cfg(unix)` と `cfg(windows)` で分ける。それ以外のコードに、OS 固有の分岐を置かない。ファイルのパスは、標準ライブラリのパス型で扱う | OS の違いを、この Unit に閉じ込める(unit-of-work の U2 の境界、ADR-001) | NFR3 |
| — | ワークスペースと edition | u1-core と同じワークスペース(`crates/u2-runtime/`、パッケージ名 `conductor-runtime`)。edition 2024、最小のバージョンは 1.85。この Unit は、u1-core に依存する(共有する型と、`resume_point_on_interrupt` の考え方) | 同じバイナリに組み込まれ、同時にビルドされるため | unit-of-work-dependency |
| NFR4.1、NFR4.2 | ジョブの起動 | 標準のライブラリの `std::process::Command` で、シェルを介さずに起動する。UNIX 系では、`CommandExt::process_group(0)` で、新しいプロセスグループにする(標準のライブラリにある)。コマンド名の探し方と、Windows の `.bat`・`.cmd` の引数の安全性も、標準のライブラリに従う | 追加の依存なし。標準の挙動が、コマンドの注入の修正を含む | NFR4、rules の BR1.2 |
| NFR4.8 | シグナルの送信(UNIX 系) | `libc` のライブラリを、UNIX 系の `platform` モジュールにだけ使い、`kill` を呼ぶ [Q1: A]。バージョンは、その時点の安定版を選び、`Cargo.lock` で固定する | 標準のライブラリは、強制終了しかできない | NFR4、rules の BR1.7 |
| NFR4.2 | command の分け方 | `shlex` などのライブラリを使わず、この Unit の中の小さな関数で実装する(BR1.1 の規則) | 3つの OS で同じ規則にするため。既存のライブラリは、規則(バックスラッシュの扱い、Windows のパス)が異なる | rules の BR1.1、Functional Design の Q3: A |
| NFR4.3、NFR4.4 | ステートファイルの読み書き | u1-core・u4-converter と同じ `toml` のライブラリ(バージョンを揃える。機能は、読み取りと書き出しに必要なもの)で、TOML を読み書きする [Functional Design の Q2: A(形式)]。大きさの上限は 1 MiB。原子的な書き込みは、標準のライブラリ(一時ファイル、`File::sync_all`、`rename`)で行う | 依存を増やさない。人が読める | NFR4、rules の BR2.2〜BR2.4 |
| — | エラーの型 | `thiserror`(u1-core と同じ)。種類ごとに固定の表示の文を付ける | 記述が短く、揃うため | rules |
| NFR1.1 | 起動経路への負担 | ステートの読み込みと照合は、`run` と `pipeline` の起動経路に入る。起動から、ジョブの起動(またはスキップの判定)までの予算 100ms のうち、この Unit の分(ステートの読み込み、照合、ジョブの起動前の初期ステートの書き込み)は、ジョブ10個の定義で、**p95 20ms 以内**を、仮の目標とする。**ジョブの起動前の初期ステートの書き込みは、ディスクへの同期(`sync_all`)を省く**(空に近い初期ステートは、失われても再作成でき、macOS などで、同期が数十ミリ秒かかりうるため)。結果の書き込み(ジョブのあと)は、同期を行う | 起動の予算を守りつつ、結果の記録の耐久性を保つため | NFR1、rules の BR2.4、BR2.9 |
| NFR5.1 | テスト | `cargo test` による単体テストと、補助の実行ファイル(引数・環境変数・終了コードを出力する、テスト専用のもの)を使う統合テスト。行カバレッジ 80% 以上(`cargo llvm-cov`。Linux で測る)。`tempfile` を開発用の依存とする | チームの実践(test-after、80%)。プロセスの起動を、実際に試すため | NFR5、team-practices |
| — | 整形と静的検査 | `rustfmt` と `clippy`。CI で失敗したらマージできない | チームの実践 | team-practices |

## 判断の記録(代替案との比較)

### T1 シグナルの送信に `libc` を使う [Q1: A]

- **Context:** ジョブにシグナル(SIGTERM など)を送る必要がある。標準のライブラリは、強制終了(SIGKILL)しか送れない。
- **Decision:** `libc` を、UNIX 系の1つのモジュールにだけ使い、`kill` を呼ぶ(`unsafe` は、その1か所)。
- **Consequences:** 良い点は、依存が小さく、C のコンパイルが要らない。悪い点は、`unsafe` が1か所入る(NFR4.8 で限定する)。
- **Alternatives Rejected:** (B) `nix` のライブラリ: 安全な包みで `unsafe` を避けられるが、依存が増え、この Unit の用途(`kill` 1つ)に対して大きい。(C) `kill` コマンドを、別のプロセスとして起動する案: 起動の遅さと、PATH への依存、`unsafe` を避ける利点が、小さいため退けた。

### T2 command の分け方を自前で実装する

- **Context:** 3つの OS で同じ規則にする(Functional Design の Q3: A)。引用符の中の `\"` だけを解釈し、それ以外のバックスラッシュを、そのままにする(Windows のパスを、そのまま書けるように)。
- **Decision:** この Unit の小さな関数で実装する。規則の表を、テストにする。
- **Consequences:** 良い点は、規則が完全に制御でき、依存が増えない。悪い点は、境界のケースを、自分でテストする必要がある。
- **Alternatives Rejected:** (1) `shlex` 系のライブラリ: POSIX の規則で、バックスラッシュを解釈するため、Windows のパスで壊れる。(2) OS ごとの規則(Q3 の B): 3つの OS で同じ結果という要件(NFR3)に反するため退けた。

### T3 ステートの形式に TOML を使う [Functional Design の Q2: A]

- **Context:** 契約 E3 は、書式の版つきとだけ定めている。
- **Decision:** TOML(Functional Design の Q2: A)。`toml` のライブラリを、u1-core・u4-converter と共有する。
- **Consequences:** 良い点は、人が読め、依存が増えない。悪い点は、キーの順序の整列など、ライブラリの既定に従う(読み戻しの等しさだけが要件)。
- **Alternatives Rejected:** JSON: `serde_json` などの依存が増える。独自のテキスト形式: 読み書きの実装と検証の手間が大きい。

## 測定できる基準(NFR1.1、NFR2.1、NFR3.1、NFR5.1)

| ID | 基準 | 確認の方法 |
|----|------|------------|
| NFR1.1 | ジョブ10個の定義で、`check_command`(全ジョブ)+ ステートの読み込み + 照合 + 初期ステートの書き込み(同期なし)が、p95 20ms 以内(仮。プロセスの起動と、ジョブの実行は含まない。Code Generation で実測して見直す)。基準を超えても、CI を失敗にせず、警告として記録する(u1-core と同じ扱い。合否の判定は、実測のあと、Code Generation で決める) | ベンチマークのテストで、100 回測り、p95 を標準エラーと CI の要約に出す。3つの OS |
| NFR2.1 | 依存するライブラリは、すべて Rust だけで書かれ、C のコンパイルを要しない(`libc` は宣言だけ) | `cargo tree` の出力に、`-sys` で終わるクレートがなく、ビルドに C のコンパイラを必要としないこと(CI) |
| NFR3.1 | OS 固有のコードは、`platform` モジュールの中だけにある。Linux・macOS・Windows の3つで、同じテストが通る | ソースの検索(`cfg(unix)` と `cfg(windows)` が `platform` の外にない)。CI の3つの OS でのテストの成功 |
| NFR5.1 | 行カバレッジ 80% 以上。測定は、`cargo llvm-cov`(u1-core と同じ道具)で、この Unit のクレート(`crates/u2-runtime/src/`)に対して、Linux で行う。`cfg(windows)` のコードは、Linux のカバレッジに含まれないため、その分の下限は、Windows の CI のテストの成功で確かめる(カバレッジの数値では確かめない) | CI の測定(CI Pipeline のステージで導入)。ローカルで未導入なら、Build and Test が全体のテストで測る |

## 依存するライブラリの方針

- この Unit が使う外部のライブラリは、`toml`、`thiserror`、`libc`(UNIX 系のみ)を基本とする。`toml` の機能は、読み取り(`parse`)と、書き出し・値の型(`display`、`serde`。`serde` と `serde_core` が間接の依存として増える)で、u4-converter と同じ機能の集合に揃える。いずれも、C のコンパイルを要しない(NFR2.1)。
- ライブラリの具体のバージョンは、Code Generation で、その時点の安定版を選び、`Cargo.lock` で固定する。
- ライブラリを増やすときは、起動オーバーヘッド(NFR1)と、単一バイナリ(NFR2)への影響を確認する。

## この Unit に該当しない NFR

- NFR6(状態の表示の判別性、色の無効化)は、表示を受け持つ u3-cli の Reporter に該当し、この Unit にはない。

## Assumptions & Open Questions

- `std::process::CommandExt::process_group` が、最小のバージョン(1.85)で使えること(標準のライブラリにある。1.64 以降)は、Code Generation で確かめる。 [assumption]
- NFR1.1 の 20ms と、初期ステートの書き込みで同期を省く判断は、実測で見直す。同期を省く判断は、NFR Design で、書き込みの関数の引数(耐久性の指定)として、設計する。 [assumption]
- ステートの読み込み・書き込みに使う `toml` の機能(読み取り、書き出し、`serde` など)は、u4-converter の選択と揃える。ワークスペースの機能の統合は、Code Generation で確かめる(`preserve_order` を有効にしないことを、u4-converter と揃える)。 [assumption]
