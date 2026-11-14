# Logical Components(u1-core)

上流: security-design(この Unit の security-design.md)、functional-spec、unit-of-work、components、contract-summary。この Unit は library であり、サーバーや常駐の仕組み、データベース、ネットワークの通信を持たない。ここでは、NFR のパターンが、この Unit のどこに当てはまるかを、論理的な部品の見取り図として示す。読み取りの流れは、security-design の「定義ファイルの読み取りの流れ」と同じ順である。

## 論理的な部品

| 部品 | 責務 | 入出力 | 関わる NFR |
|------|------|--------|------------|
| 取り込み(Intake) | 定義ファイルの大きさを確認し、上限 + 1 バイトまでを読み取る。UTF-8 でない内容、通常のファイルでないものを、読めないものとして扱う | ファイル → 文字列(または `tooLarge` / `unreadable` / `fileMissing`) | NFR4.4 |
| 読み取り(Parse) | 文字列を、位置の情報(バイトの範囲)を保った、型を決めない文書の木として読み取る。文法の誤りを `syntax` として返す | 文字列 → 木(または `syntax`) | NFR4.7 |
| 深さの検査(Depth) | 木を走査して、入れ子の深さを数える。32段を超えたら、`syntax` で拒否する | 木 → 木(または `syntax`) | NFR4.7 |
| 版の確認(VersionCheck) | 木の版のキーを確認する。通らなければ、ここで終える | 木 → 版(または `versionMissing` / `versionUnsupported`) | NFR4.3 |
| 構造の検査(Walk) | 木を自分で走査し、許されたキー、型、構造の制約、遷移、終了コード、名前の一致を検査する。誤りをすべて集める | 木 → PipelineDefinition(または複数の DefinitionError) | NFR4.1、NFR4.2(BR1.1〜BR1.8、BR1.13) |
| 位置の変換(Locate) | 要素の範囲から、行番号を求める(ファイルの先頭からの改行の数) | 範囲 → 行 | NFR4.1、NFR4.2 |
| 表示用の組み立て(Describe) | 誤りの内容(ファイル名・行・種類・理由)を、固定の文から組み立てる。入力に由来するキーの名前は、制御文字をエスケープし、64 文字までに切る | 誤り → DefinitionError | NFR4.6 |
| 判断(Flow) | 分類、終了コード、遷移、完了、起動前の判断。ファイルやプロセスに触れない | 引数 → Decision / JobOutcome / NextStep | NFR5.1(網羅テスト)、ADR-005 |
| 共有する型(Types) | データの形だけの定義(entities.md の型、追加する `syntax`・`tooLarge` を含む)。入出力に依存しない | — | NFR2.1、NFR3.1 |

- 上の「取り込み」だけが、ファイルシステムに触れる。それ以外は、入出力に触れない純粋な処理であり、単体テストで、ファイルなしに確かめられる。(NFR4.5、NFR5.1)
- 部品の呼び出しの順は、Intake → Parse → Depth → VersionCheck → Walk(Locate を使う)。どの段階でも、誤りは、Describe を通して DefinitionError として返す。

## 失敗の範囲(failure domain)と影響の及ぶ範囲(blast radius)

| 失敗 | 影響の範囲 | 備考 |
|------|------------|------|
| 定義の読み取りの失敗 | 1回の実行(ジョブは1つも起動されない) | 常に DefinitionError として返し、パニックしない(NFR4.7) |
| 判断(Flow)の不具合 | 1回の実行の判断(誤ったスキップなど) | 入出力がなく、網羅的な単体テストで守る(NFR5.1) |
| 共有する型の変更 | u2-runtime、u3-cli のビルド | Contract Design の変更の規則に従う(同時に修正する) |

- この Unit は、状態を持たず、常駐もしないため、他の実行や他のジョブに影響を及ぼさない。

## 共有する資源

- なし。この Unit は、共有の資源(接続、キャッシュ、キュー、ファイルのロック)を持たない。読み取る定義ファイルは、読み取り専用で、書き込まない。

## 他の Unit との関係

- u2-runtime は、この Unit の共有する型(Types)を使う。 unit-of-work-dependency
- u3-cli は、読み取り(Intake からの一連)と判断(Flow)を呼び出す。Cli が、定義の読み込みの結果と、実行の結果を、Flow に渡す。 ADR-002

## Assumptions & Open Questions

- 論理的な部品の名前(Intake など)は、設計上の呼び名であり、Rust のモジュールやファイルの分け方は、Code Generation で決める。 [assumption]
