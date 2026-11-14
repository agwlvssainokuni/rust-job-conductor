# Entities(u3-cli)

上流: unit-of-work(U3 u3-cli の責務)、components(Cli、Reporter)、contract-summary(E1、C2、C3、C4)、requirements、この Unit の質問への回答(Q1〜Q3)、u1-core・u2-runtime・u4-converter の functional-design。この Unit は、`conductor` の実行ファイルそのもので、コマンドとオプションの受け付けと、1回の実行の流れの組み立て(Cli)、表示(Reporter)を受け持つ。**共有する型(PipelineDefinition、JobExecution、PipelineState など)の形は、u1-core の entities.md が定める**。ここには、この Unit が足す型を置く。技術に依存しない記述であり、言語やデータ形式は定めない。

```yaml
entities:
  - name: CliCommand
    owner: Cli
    description: 利用者が指定したコマンド(引数を解釈した結果)
    attributes:
      - { name: kind, type: enum, required: true, allowed: [run, pipeline, watch, reset, convert] }
      - { name: jobId, type: text, required: false, constraints: kind が run のときのみ。必須 }
      - { name: pipelineName, type: text, required: false, constraints: kind が pipeline のときのみ。必須 }
      - { name: convertInput, type: text, required: false, constraints: kind が convert のときのみ。変換する入力ファイルのパス。必須 }
      - { name: convertTarget, type: enum, required: false, allowed: [yaml, toml], constraints: kind が convert のときのみ。`--to` の値。必須 }
      - { name: configPath, type: text, required: false, constraints: "`--config <パス>`。kind が run、pipeline、watch、reset のときのみ。省略時は、実行ディレクトリの conductor.toml" }
      - { name: verbose, type: boolean, required: true, default: false, constraints: "`-v` / `--verbose`。kind が run、pipeline のときのみ指定できる" }

  - name: UsageError
    owner: Cli
    description: コマンドの使い方の誤り(引数の解釈の失敗)。ジョブは実行されない
    attributes:
      - { name: kind, type: enum, required: true, allowed: [unknownCommand, missingArgument, unexpectedArgument, unknownOption, optionNotAllowed, missingValue, invalidValue, unknownJob] }
      - { name: reason, type: text, required: true, constraints: 固定の文。使い方の案内(コマンドの一覧)を含めてよい }

  - name: ExitCodeMap
    owner: Cli
    description: conductor が返す終了コードの決め方(契約 E1)
    attributes:
      - { name: success, type: integer, required: true, default: 0, constraints: 成功、警告、スキップ }
      - { name: jobError, type: integer, required: false, constraints: ジョブの終了コードをそのまま(規則 BR5.2 で、OS の範囲に合わせる) }
      - { name: successCodeNotListedZero, type: integer, required: true, default: 1 }
      - { name: startFailure, type: integer, required: true, default: 127 }
      - { name: signal, type: integer, required: false, constraints: 128 + シグナル番号 }
      - { name: definitionError, type: integer, required: true, default: 64 }
      - { name: stateError, type: integer, required: true, default: 65 }
      - { name: rejected, type: integer, required: true, default: 66 }
      - { name: conversionError, type: integer, required: true, default: 67 }
      - { name: usageError, type: integer, required: true, default: 68, constraints: Q3: A。この Unit が新しく足す }

  - name: DisplayEvent
    owner: Reporter
    description: 表示する内容(種類とデータ)。Cli が作り、Reporter が、文言の組み立てと出力を行う
    attributes:
      - { name: kind, type: enum, required: true, allowed: [definitionErrors, commandErrors, usageError, stateError, consistencyMismatch, rejected, startFailure, successCodeNotListed, jobResult, jobSkipped, transition, interrupted, watchTable, watchEmpty, conversionError] }
      - { name: level, type: enum, required: true, allowed: [error, verbose, output], constraints: error は常に表示(標準エラー)、verbose は -v のときだけ(標準エラー)、output は結果の出力(watch の表、convert の変換結果。標準出力) }
      - { name: data, type: record, required: true, constraints: 種類ごとの値(ジョブID、終了コード、遷移先、スキップしたジョブID、エラーの一覧、ステートの表、など)。入力の値は、そのまま文言に埋め込まず、エスケープして表示する }

  - name: Language
    owner: Reporter
    description: 表示の言語
    attributes:
      - { name: value, type: enum, required: true, allowed: [en, ja], default: en, constraints: 環境のロケールで決める(規則 BR6.1)。判定できなければ en }

  - name: ColorMode
    owner: Reporter
    description: 色を使うかどうか
    attributes:
      - { name: enabled, type: boolean, required: true, constraints: 規則 BR6.5 で決める。色を使っても、状態は文字でも表す }
```

## 要約

- 値の「生成」は、CliCommand と UsageError と DisplayEvent(Cli が作る)、ExitCodeMap(定数)、Language と ColorMode(Reporter が決める)。永続する状態は、この Unit にはない(ステートは u2-runtime)。
- 終了コードの値の確定: 64(定義の誤り)、65(ステートの誤り)、66(拒否)、67(変換の誤り)は、u1-core の functional-design の BR5.1 で決め済み。**68(使い方の誤り)を、この Unit が足す**(Q3: A)。

## Assumptions & Open Questions

- `UsageError` の `unknownJob`(`run` に渡されたジョブIDが、定義にない)は、使い方の誤りとして、終了コード 68 にする。定義の誤りではない。 [assumption]
- 論理的な型(text、integer など)は、技術に依存しない記述であり、実装の型への対応は Code Generation で決める。 [assumption]
