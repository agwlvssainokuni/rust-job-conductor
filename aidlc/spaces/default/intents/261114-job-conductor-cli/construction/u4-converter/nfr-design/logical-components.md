# Logical Components(u4-converter)

上流: security-design(この Unit の security-design.md)、functional-spec、unit-of-work、components、contract-summary。この Unit は library であり、サーバーや常駐の仕組み、データベース、ネットワークの通信を持たない。ここでは、NFR のパターンが、この Unit のどこに当てはまるかを、論理的な部品の見取り図として示す。流れの順は、security-design の「変換の流れ」と同じ。

## 論理的な部品

| 部品 | 責務 | 入出力 | 関わる NFR |
|------|------|--------|------------|
| 取り込み(Intake) | ファイルを開き、通常のファイルか確かめ、「上限 + 1 バイト」までを読む。UTF-8 でなければ読めないものとして扱う | パス → 文字列(または `fileMissing` / `unreadable` / `tooLarge`) | NFR4.2 |
| 読み取り(Read) | 文字列を、入力の形式(YAML または TOML)として、文書の終わりまで読む。深さの超過を、再帰の制限、または事前の走査で拒否する | 文字列 → ライブラリの木(または `syntax`) | NFR4.4、NFR4.7 |
| 文書の確認(RootCheck) | YAML の文書の数、最上位がマップかを確かめる | ライブラリの木 → ライブラリの木(または `unrepresentable` / `rootNotMap`) | NFR4.1 |
| 写し取り(Normalize) | ライブラリの木を、途中の表現へ写す。アンカー・エイリアス・マージキーの展開、展開後の要素数と深さの数え、表せない値の記録を行う | ライブラリの木 → 途中の表現(または `tooLarge` / `syntax` / `unrepresentable`) | NFR4.3、NFR4.4、NFR4.5 |
| 途中の表現(Value) | 文字列・整数・浮動小数・真偽・配列・マップだけを持つ小さな型。入出力に依存しない | — | NFR2.1、NFR3.1 |
| 書き出し(Write) | 途中の表現を、相手の形式の文字列にする。文字列は、読み戻したときに文字列のままになるように、必要なら引用符で囲む | 途中の表現 → 文字列(ConvertedText) | NFR4.1(BR1.2) |
| 表示用の組み立て(Describe) | 誤りの内容(ファイル名・行・種類・キーの経路)を、固定の文から組み立てる。キー名は、制御文字をエスケープし、64 文字までに切る | 誤り → ConversionError | NFR4.6 |

- 上の「取り込み」だけが、ファイルシステムに触れる。それ以外は、入出力に触れない純粋な処理であり、文字列だけで単体テストできる。(NFR4.5、NFR5.1)
- 部品の呼び出しの順は、Intake → Read → RootCheck → Normalize → Write。どの段階でも、失敗は、Describe を通して ConversionError として返す。

## 失敗の範囲(failure domain)と影響の及ぶ範囲(blast radius)

| 失敗 | 影響の範囲 | 備考 |
|------|------------|------|
| 変換の失敗 | 1回の `convert` の呼び出し(出力は作られない) | 常に ConversionError として返し、パニックしない(NFR4.7) |
| 共有する型の変更 | なし(この Unit は、他の Unit と共有する型を持たない。ConversionError と ConvertedText は、u3-cli が使う) | 契約 C4 の変更の規則に従う |

- この Unit は、状態を持たず、常駐もしないため、他の実行や他の Unit に影響を及ぼさない。

## 共有する資源

- なし。この Unit は、共有の資源(接続、キャッシュ、キュー、ファイルのロック)を持たない。読み取る入力のファイルは、読み取り専用で、書き込まない。

## 他の Unit との関係

- u3-cli(Cli)が、`convert` のときだけ、この Unit を呼ぶ。結果の書き出しと、ConversionError の表示は、u3-cli が行う。 ADR-002
- u1-core とは、依存しない(依存の DAG)。

## Assumptions & Open Questions

- 論理的な部品の名前(Intake など)は、設計上の呼び名であり、Rust のモジュールやファイルの分け方は、Code Generation で決める。 [assumption]
