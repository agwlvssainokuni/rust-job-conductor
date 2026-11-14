# Scope Document

上流: intent-statement(課題・成功基準・初版の対象外)、feasibility-assessment(実現性評価)、constraint-register(制約 C1〜C9)。

## In Scope(初版)

| 機能 | 優先度(MoSCoW) | Source |
|------|------------------|--------|
| 個別実行モード(ジョブIDを指定して1ステップずつ呼び出し)と、警告・特定条件でのスキップ・ジャンプ | Must Have | [Q1] |
| 一連の実行モード(パイプライン全体を `conductor` 自体が駆動) | Must Have | [Q1] |
| 定義ファイル(`conductor.toml`)による、ジョブ・終了コードの対応・遷移ルールの一元定義 | Must Have | [Q1] |
| 実行状態をローカルのステートファイルで保持する機能 | Must Have | [Q1] |
| 状態の初期化(`reset`) | Must Have | [Q2] |
| 状態の確認(`watch`) | Should Have(初版に含めるが後回しでもよい) | [Q2] |
| YAML ⇔ TOML 変換(`convert`) | Should Have / Could Have(初版に含めるが他の機能が優先) | [Q3] |

## Out of Scope(初版)

- 通知機能(Slack 通知など)そのものの組み込み。通知はジョブ側のスクリプトで行う。 intent-statement の Initial Scope Signal
- 複数ホストにまたがる分散実行・ステート共有。 intent-statement の Initial Scope Signal
- Web UI / GUI。 intent-statement の Initial Scope Signal

## Boundaries and Constraints Carried Forward

- 既存のジョブ実行環境そのものは変更しない。 constraint-register の C1
- Linux・macOS・Windows のすべてで動作することが前提。 constraint-register の C3
- スケジューラが受け付ける終了コードは 0 / 非0 の2値のみ。 constraint-register の C2

## Sequencing Preference

- 依存関係の順に進める(定義・ステート → 個別実行 → 一連の実行 → 補助機能)。 [Q4]
- 機能に結びついた期限はない。 [Q5]

## Assumptions & Open Questions

None.
