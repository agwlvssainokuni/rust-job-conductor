# エンティティ: u5-log-dir(ジョブの出力をファイルにも残す)

出典: ユニットの定義(`unit-of-work.md` の U5)、要件とユニットの対応(`unit-of-work-story-map.md`。FR5.1〜FR5.5、FR6.6 の時刻の部分、FR8.1 がこのユニット)、要件(`requirements.md` の FR5.1〜FR5.5、FR6.2、FR6.6)、コンポーネントの一覧(`components.md` の CoreTypes、CoreDefinition、CoreTime、RuntimeClock、RuntimeLog、RuntimeRunner、CliDeps、CliFlows)、契約(`contract-summary.md` の C2、C6)、機能の設計の質問(`functional-design-questions.md` の Q1〜Q5)。

```yaml
entities:
  - name: PipelineDefinition
    kind: 組み立て後の定義(既存。CoreTypes)
    description: パイプラインの定義。log_dir を足す。
    attributes:
      - name: log_dir
        type: 文字列
        required: false
        constraints: 空でない。書かなければ無し(今と同じ動き)。`[pipeline]` にだけ書ける

  - name: Timestamp
    kind: 値(CoreTime。新規。契約 C2)
    description: UTC の時刻。秒の単位。
    attributes:
      - name: unix_seconds
        type: 整数(64 ビット、符号なし)
        required: true
        min: 0
        max: 253402300799
        constraints: 1970-01-01T00:00:00Z から 9999-12-31T23:59:59Z まで
    relationships:
      - to: LogFileName
        cardinality: "1 対 多"
        direction: ファイル名の時刻に使う(同じ値をステートの started_at にも使う。U6)

  - name: LogTarget
    kind: 値(CliFlows が作り、CliDeps を通して RuntimeLog・RuntimeRunner に渡す)
    description: 1 回の試行のログの置き場所の指定。
    attributes:
      - name: dir
        type: パス
        required: true
        constraints: 解決済み(相対なら定義ファイルのディレクトリが基準)
      - name: job_id
        type: 文字列
        required: true
        constraints: 定義のジョブ ID(名前にするときに BR3.2 で置き換える)
      - name: started_at
        type: Timestamp
        required: true
        constraints: 流れがその試行の前に取った現在の時刻
      - name: attempt
        type: 整数
        required: true
        min: 1
        constraints: 1 から数える試行の番号(今の CONDUCTOR の試行の番号と同じ)

  - name: LogFileName
    kind: 値(RuntimeLog が決める)
    description: 1 回の試行のログのファイルの名前。
    attributes:
      - name: name
        type: 文字列
        required: true
        unique: true
        constraints: >
          `<安全な ID>.<YYYYMMDDTHHMMSSZ>.<試行の番号>.log`。同じ名前があれば
          `<安全な ID>.<時刻>.<試行の番号>-<n>.log`(n は 2 から)

  - name: LogOutcome
    kind: 値(RuntimeRunner の結果に添える)
    description: ログの書き込みの結果。
    attributes:
      - name: write_failed
        type: 真偽
        required: true
        default: false
        constraints: 実行中に一度でもファイルへの書き込みに失敗したら true(以後はファイルに書かない)
      - name: output_cut_short
        type: 真偽
        required: true
        default: false
        constraints: ジョブの終了の後、出力の読み取りを待ち切れなかったら true(BR4.5)
```

## まとめ

| エンティティ | 種類 | このユニットでの変化 |
|---|---|---|
| PipelineDefinition | 定義 | `log_dir` を足す |
| Timestamp | 値(core) | 新規。時刻の整形と読み取り、差 |
| LogTarget | 値 | 新規。試行ごとのログの指定 |
| LogFileName | 値 | 新規。名前の形と衝突の避け方 |
| LogOutcome | 値 | 新規。実行中の書き込みの失敗と、出力の待ち切れ |

ステートの形(試行の時刻)は U6 で変える。このユニットはステートを変えない。
