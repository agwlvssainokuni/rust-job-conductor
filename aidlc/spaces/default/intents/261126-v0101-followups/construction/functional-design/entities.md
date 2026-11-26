# 実体: v0.10.1 の整理・修正

出典: 要件(`requirements.md`、承認済み)、設計の質問(`functional-design-questions.md` の Q1 と、それを見直した Q2、まとめの確かめ済み)、コードの知識(`aidlc/spaces/default/codekb/rust-job-conductor/`)。refactor の作業のため、Units とドメインの設計は無く、今のコードの形をそのままドメインの設計として扱う。新しい永続のデータ(ステートの形、定義の書式)は足さない。

```yaml
entities:
  - name: LogStem
    description: ログのファイル名の頭(ジョブ ID から作る、OS によらず同じ名前)。runtime の safe_log_stem が作る。
    owner: conductor-runtime (log)
    attributes:
      - name: job_id
        type: string
        required: true
        constraints: 定義の検証を通ったジョブ ID
      - name: head
        type: string
        required: true
        constraints: 置き換え後の名前のうち、最初の "." より前の部分。予約名と比べるときは前後の ASCII の空白を除いた値を使う(FR2.1)
      - name: stem
        type: string
        required: true
        constraints: 100 バイト以下、UTF-8 の文字の途中で切らない、ディレクトリの区切りを含まない、末尾が "." でも空白でもない
    relationships: []
  - name: CopyStop
    description: 待ち切れのあとに立てる「止める」印。出力を写す読み取りのスレッドが共有する(runtime の Shared.stop)。
    owner: conductor-runtime (runner)
    attributes:
      - name: stop
        type: boolean
        required: true
        default: false
        constraints: 一度立てたら下ろさない
    relationships:
      - target: LogStem
        cardinality: none
        note: 関係は無い。同じ F2 の課題に属するだけ
  - name: PlanSimulationLookup
    description: 計画の模擬が、ジョブごとに判断の本体へ渡す「定義の位置」と「最初の済んでいないジョブの位置の目印」。F3 で直すときだけ現れる(Q2 A)。判断の本体と目印を進める関数は flow が持ち、plan は値を持ち回るだけ。
    owner: conductor-core (flow。plan は持ち回るだけ)
    attributes:
      - name: position
        type: integer
        required: true
        constraints: 定義の記述順の位置(0 始まり)。模擬は定義の順に進むので、繰り返しの位置から渡せる
      - name: first_unsettled
        type: integer
        required: false
        constraints: 記述順で最初の、成功・警告・スキップのどれでもないジョブの位置。状態は済む向きにしか変わらないので、模擬の中では減らない。flow の関数が進める
      - name: recorded_status
        type: enum
        allowed: [pending, success, warning, error, skipped]
        constraints: 位置で引き、位置のジョブ ID が違うときだけ探す(今の status_at の形)
    relationships: []
  - name: DemoPause
    description: デモのスクリプトの「Enter を待つ」所。DEMO_PAUSE=1 のときだけ待つ。
    owner: demo (run-demo.sh)
    attributes:
      - name: enabled
        type: boolean
        required: true
        constraints: 環境変数 DEMO_PAUSE の値が "1" のときだけ真
      - name: points
        type: list
        constraints: 各場面のあと。v0.10.1 から、UNIX 版の場面 4 の watch --follow の前を足す(FR5.1)
    relationships: []
  - name: FlakeRecord
    description: 原因不明の失敗が再び起きたときに残す記録(F4)。コードには現れず、手順として記録に書く。
    owner: intent record (build-and-test-summary.md)
    attributes:
      - name: run
        type: string
        required: true
        constraints: CI の実行の番号、または手元の実行の日時
      - name: test_name
        type: string
        required: true
      - name: output
        type: text
        required: true
      - name: os
        type: string
        required: true
      - name: commit
        type: string
        required: true
    relationships: []
```

## まとめ

- `LogStem` と `CopyStop` は F2 の 2 つの直しの対象で、どちらも runtime の中に閉じる。外から見える変化は、空白を除くと予約名になるジョブ ID のログの名前だけ(FR2.2)。
- `PlanSimulationLookup` は F3 で直すときの対象。判断の本体と目印を進める関数は `flow` が持ち、`plan` は位置と目印を持ち回るだけ。共有の公開の関数 `job_index`・`record_of` そのものと、実行の流れの動きは変えない(Q2 A)。
- `DemoPause` は F5 の対象で、UNIX 版のデモのスクリプトだけを変える。
- `FlakeRecord` は F4 の手順が求める項目で、記録の文書にだけ現れる。
