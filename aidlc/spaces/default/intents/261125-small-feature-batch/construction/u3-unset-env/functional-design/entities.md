# エンティティ: u3-unset-env(`unset_env`)

出典: ユニットの定義(`unit-of-work.md` の U3)、要件とユニットの対応(`unit-of-work-story-map.md`。FR3.1〜FR3.5、FR8.1 がこのユニット)、要件(`requirements.md` の FR3.1〜FR3.5)、コンポーネントの一覧(`components.md` の CoreTypes、CoreDefinition、CoreEnvFile、RuntimeRunner、CliDeps)、契約(`contract-summary.md` の C1、C3)、設計の決定(ADR-007)、機能の設計の質問(`functional-design-questions.md` の Q1〜Q3)。

```yaml
entities:
  - name: PipelineDefinition
    kind: 組み立て後の定義(既存。CoreTypes)
    attributes:
      - name: unset_env
        type: 名前の並び
        required: true
        default: 空
        constraints: 名前の規則は env と同じ。CONDUCTOR_* は書けない。同じ名前は書いた最初の位置に 1 つにまとめる

  - name: JobDefinition
    kind: 組み立て後の定義(既存。CoreTypes)
    attributes:
      - name: unset_env
        type: 名前の並び
        required: true
        default: 空
        constraints: PipelineDefinition.unset_env と同じ

  - name: EffectiveEnv
    kind: 統合の結果(既存。CoreEnvFile。契約 C1 の output_u3)
    description: 1 つのジョブの 1 回の起動に渡す環境の変更。
    attributes:
      - name: entries
        type: 足す変数(名前と値)の並び
        required: true
        constraints: 今のまま
      - name: removals
        type: 消す名前の並び
        required: true
        default: 空
        constraints: >
          パイプラインの unset_env、次にジョブの unset_env を書いた順に並べ、同じ名前(完全一致)は
          最初の 1 つにまとめる。entries の名前とは重ならない(検証で止めるため)

  - name: DefinitionErrorKind
    kind: 誤りの種類(既存。core)
    attributes:
      - name: UnsetEnvConflict
        type: 新しい種類
        required: true
        constraints: unset_env の名前が、同じジョブに当てはまる env の名前と重なる

  - name: EnvFileErrorKind
    kind: .env の誤りの種類(既存。core)
    attributes:
      - name: UnsetConflict
        type: 新しい種類
        required: true
        constraints: .env の名前が、そのファイルを読むジョブに当てはまる unset_env の名前と重なる
```

## まとめ

| エンティティ | 変化 |
|---|---|
| PipelineDefinition / JobDefinition | `unset_env` を足す |
| EffectiveEnv | `removals` を足す(足す変数の形は変えない。C1) |
| DefinitionErrorKind | `UnsetEnvConflict` を足す |
| EnvFileErrorKind | `UnsetConflict` を足す |
