# Functional Spec(u4-converter)

上流: entities(この Unit の entities.md)、rules(この Unit の rules.md)、unit-of-work、unit-of-work-story-map、stories、requirements、components、contract-summary。この Unit が実装するストーリー: US7.1(AC7.1.1、AC7.1.2、AC7.1.3)。unit-of-work-story-map の US7.1 は、u3-cli(コマンドの受け付けと出力)と分担する。

この文書は、ワークフローの「正」である。データの形は entities.md、判断の規則は rules.md が正であり、ER 図と規則の要約は、そこから導いた参考の表示である。技術に依存しない記述で、コードは含まない。

## ワークフロー

### W1 定義ファイルの変換(`convert`。Converter。US7.1)

入力: ConversionRequest(inputPath、targetFormat)。出力: ConvertedText、または ConversionError。

失敗の判定は、次の順に行い、最初に当てはまるものを返して終わる(BR1.12)。

1. 入力の形式を決める。targetFormat が toml なら YAML、yaml なら TOML。拡張子は見ない。(BR1.1)
2. 入力ファイルを開く。ファイルがなければ `fileMissing`、権限がない・通常のファイルでなければ `unreadable`、大きさが上限(1 MiB)を超えれば `tooLarge`、UTF-8 でなければ `unreadable` を返す。(BR1.5)
3. 入力を、決めた形式として、文書の終わりまで読む。文法の誤り(重複するキー、再帰するアンカー、展開前の構造の深さ 32 の超過を含む)は、`syntax`(位置が分かれば line つき)を返す。(BR1.4、BR1.7、AC7.1.3)
4. YAML に、複数の文書があれば、`unrepresentable` を返す。(BR1.6)
5. 最上位(最初の文書)がマップであることを確かめる。そうでなければ `rootNotMap` を返す。空の TOML は、空のマップとして続ける。空の YAML は `rootNotMap`。(BR1.8)
6. YAML のアンカー、エイリアス、マージキーを展開する。展開後の要素数が上限を超えれば `tooLarge`、展開後の深さが 32 を超えれば `syntax`、展開できないマージキーは `unrepresentable` を返す。(BR1.7)
7. キーと値の木を、入力に現れる順の深さ優先で走査し、相手の形式で意味を変えずに表せない値(null、文字列でないキー、TOML の日時、範囲を超える整数)があれば、最初に見つかったものを `unrepresentable`(keyPath つき)で返す。(BR1.6)
8. 木を、相手の形式の文字列に書き出す。コメントと、マップのキーの記述順は保持しない。配列の要素の順序は保つ。文字列は、読み戻したときに文字列のままになるように出力する。(BR1.2、BR1.3、AC7.1.1、AC7.1.2)
9. 文字列(ConvertedText)を返す。書き出し先への出力は、呼び出し側(Cli)が行う。(BR1.9)

合否の基準: 返した文字列を、変換後の形式として読み戻した木が、入力の木と等しい(BR1.11)。

失敗のときは、どの段階でも、出力を作らない。エラーの表示の内容は、固定の文と場所だけで組み立てる。(BR1.10)

## 状態遷移

この Unit は、状態を持たない。1回の呼び出しは、入力から出力(または失敗)までを、1度で終える。

## 参考: ER 図(entities.md から導出)

```mermaid
erDiagram
  ConversionRequest ||--|| TargetFormat : targetFormat
  ConversionRequest ||--o| ConvertedText : "成功したとき"
  ConversionRequest ||--o| ConversionError : "失敗したとき"
```

Text fallback: ConversionRequest(入力のパスと TargetFormat)から、成功すれば ConvertedText、失敗すれば ConversionError のどちらか1つが得られる。

## 参考: 規則の要約(rules.md から導出)

| 規則 | 内容 |
|------|------|
| BR1.1 | 入力の形式は、--to の反対 |
| BR1.2〜BR1.3 | 意味を保つ変換。コメントと順序は保持しない |
| BR1.4〜BR1.5 | 文法の誤り、読めない入力は、変換せず原因を返す |
| BR1.6〜BR1.8 | 表せない値、アンカーとエイリアスの展開、最上位がマップでない入力 |
| BR1.9〜BR1.10 | 結果は文字列で返す(書き出しは Cli)。エラー表示の組み立て |
| BR1.11〜BR1.12 | 読み戻すと同じ木(合否の基準)。失敗の判定の順序 |

## 他の Unit との分担(この Unit が実装しない部分)

- U3 u3-cli: `conductor convert <file> --to <format>` の受け付け、結果の標準出力・ファイルへの書き出しと、書き出しの失敗の報告、ConversionError の表示(Reporter)、非 0 の終了コード(契約 E1 の「conductor 自身のエラー」。具体の値は u3-cli の設計で定める)。
- U1 u1-core: 変換した結果の定義としての検証(`conductor run` などでの読み込み時)。

## Assumptions & Open Questions

- 変換の結果は、`conductor` が読める定義になるとは限らない(Q1: A)。US7.1 の目的(`conductor.toml` として使える)のため、利用者は、変換のあとに `conductor run` などで読み込み、定義としての正しさを確かめる。u3-cli は、変換の結果を検証せず、必要なら、成功時の案内(「定義としての検証は、読み込み時に行われる」)を表示できる。 [assumption]
- 標準出力かファイルかの具体の指定(`--output` のような追加のオプション)は、u3-cli で決める。 [assumption]
