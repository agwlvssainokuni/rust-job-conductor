# ビジネスルール: u3-unset-env(`unset_env`)

出典: 要件(`requirements.md` の FR3.1〜FR3.5、FR8.1、NFR1、NFR6)、ユニットの定義(`unit-of-work.md` の U3)、要件とユニットの対応(`unit-of-work-story-map.md`)、コンポーネントの一覧(`components.md`)、契約(`contract-summary.md` の C1、C3)、設計の決定(ADR-007)、機能の設計の質問(`functional-design-questions.md` の Q1〜Q3)。規則の番号はこのユニットの中のもの。

```yaml
rules:
  - id: BR1.1
    statement: "`[pipeline]` と `[job]` に `unset_env` を文字列の配列で書ける"
    category: validation
    applies_to: unset_env
    trigger: 定義の検証
    logic: >
      IF 配列でない THEN TypeMismatch(「`unset_env` は、文字列の配列で書きます」)。
      IF 要素が文字列でない THEN TypeMismatch(「`unset_env` の要素は、文字列で書きます」)
    violation: 定義の誤り(64)
    source: FR3.1

  - id: BR1.2
    statement: 名前の規則は env と同じ。空・規則違反・CONDUCTOR_* は誤り
    category: validation
    applies_to: unset_env の各名前
    trigger: 定義の検証
    logic: 空 → EmptyField。規則違反 → InvalidEnvName。CONDUCTOR_*(大文字小文字を問わない)→ ReservedEnvName。理由の文は env の名前と同じ固定の文
    violation: 定義の誤り(64)
    source: FR3.1, FR3.4

  - id: BR1.3
    statement: 同じ名前を 2 回書いても誤りにせず、1 つにまとめる
    category: policy
    applies_to: unset_env
    trigger: 定義の組み立てと統合
    logic: 同じ配列の中、パイプラインとジョブの間のどちらでも、完全一致の名前は最初の 1 つだけを残す
    violation: 該当なし
    source: FR3.2

  - id: BR2.1
    statement: unset_env の名前が、同じジョブに当てはまる env の名前と重なれば誤り
    category: constraint
    applies_to: unset_env と env
    trigger: 定義の検証(ジョブどうしの関係の検査)
    logic: >
      パイプラインの unset_env は、パイプラインの env とすべてのジョブの env と比べる。
      ジョブの unset_env は、パイプラインの env とそのジョブの env と比べる。完全一致で比べる。
      重なれば UnsetEnvConflict を unset_env の名前の位置で 1 つ(理由「`unset_env` の名前が、`env` の名前と重なっています」)
    violation: 定義の誤り(64)。値を表示しない
    source: FR3.3

  - id: BR2.2
    statement: unset_env の名前が、そのジョブに当てはまる .env の中身の名前と重なれば .env の誤り
    category: constraint
    applies_to: unset_env と .env
    trigger: .env を読んだ後(実行の前。コマンドの範囲のジョブについて)
    logic: >
      範囲のジョブごとに、パイプラインとジョブの env_file で読めたファイルの名前と、そのジョブの消す名前を完全一致で比べる。
      重なれば、その .env のファイルと行で UnsetConflict(理由「`unset_env` の名前と同じ名前があります」)。
      今の .env の誤りと一緒に集めて 64
    violation: .env の誤り(64)。値を表示しない
    source: FR3.3

  - id: BR3.1
    statement: 統合の結果は「足す変数」と「消す名前」
    category: calculation
    applies_to: EffectiveEnv
    trigger: ジョブの起動の前
    logic: 足す変数は今のまま。消す名前はパイプライン、次にジョブの unset_env を書いた順に並べ、まとめる(BR1.3)
    violation: 該当なし
    source: FR3.2

  - id: BR3.2
    statement: 親から引き継いだ環境から消す名前を除いてから、足す変数を加えて起動する
    category: policy
    applies_to: RuntimeRunner
    trigger: ジョブの起動
    logic: >
      消す名前を先に除き、足す変数を後に加える。名前の比べ方は OS に任せる
      (UNIX 系は完全一致、Windows は大文字小文字を区別しない。Q1)。親の環境に無い名前は何もしない
    violation: 該当なし
    source: FR3.2

  - id: BR3.3
    statement: 自動の変数(CONDUCTOR_*)は消されない
    category: constraint
    applies_to: RuntimeRunner
    trigger: ジョブの起動
    logic: 予約名は unset_env に書けない(BR1.2)ため、自動の変数は常に渡る
    violation: 該当なし
    source: FR3.4

  - id: BR4.1
    statement: "`--dry-run` は、足す名前と並べて消す名前も出す(値は無い)"
    category: policy
    applies_to: cli の計画の表示
    trigger: "`--dry-run`"
    logic: 消す名前が 1 つ以上あるジョブだけ、今の環境変数の名前の行の後に「消す」名前の行を足す(英日)
    violation: 該当なし
    source: FR3.2

  - id: BR4.2
    statement: "`unset_env` を書かない定義は今と同じに動く"
    category: constraint
    applies_to: 全体
    trigger: すべての場面
    logic: 消す名前が空なら、起動・表示・誤りは今のまま
    violation: 該当なし
    source: FR3.5

  - id: BR4.3
    statement: README とデモを更新する
    category: policy
    applies_to: 文書
    trigger: このユニットの最後
    logic: README に書き方、名前の規則、重なりの誤り、Windows の比べ方、`--dry-run` の表示を書く。デモの環境変数の場面に unset_env を足す
    violation: 該当なし
    source: FR8.1
```

## まとめ

| ID | 規則 | 出典 |
|---|---|---|
| BR1.1〜BR1.3 | 書き方、名前の規則、まとめる | FR3.1、FR3.2、FR3.4 |
| BR2.1、BR2.2 | env・.env との重なりは 64 | FR3.3 |
| BR3.1〜BR3.3 | 統合の結果、消してから足す、自動の変数 | FR3.2、FR3.4 |
| BR4.1〜BR4.3 | `--dry-run`、互換、文書 | FR3.2、FR3.5、FR8.1 |
