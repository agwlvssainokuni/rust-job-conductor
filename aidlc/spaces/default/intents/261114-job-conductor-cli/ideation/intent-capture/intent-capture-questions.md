# Intent Capture & Framing — 質問

## Sources

- [scope] Workflow-selected scope: `mvp`.

> 取り込み済みの要件定義書の内容は、ここでの確認済み回答 `[Q<n>]` を通じてのみ成果物に反映します。

## Q1. [プロダクト] 解決したい課題

既存のジョブ実行環境は、ジョブを一列に順次実行するだけで、終了判定も「正常 / 異常」中心です。この課題の捉え方を確認します(成果物の Problem Statement の根拠になります)。

- A. 既存環境が直列実行かつ終了判定が限定的なため、「警告終了時は別処理へ分岐」「特定ジョブをスキップ」といった制御ができない
- B. A に加えて、ジョブ間で実行状態を引き継げない(ステート管理がない)ことも主要な課題
- C. Not yet defined(まだ定まっていない)
- D. Not applicable(該当なし)
- X. Other (please specify)

[Answer]: B

## Q2. [プロダクト] 想定する利用者

`conductor` を実際に使うのは誰ですか(Target Customer の根拠になります)。

- A. 既存の直列ジョブスケジューラ上でバッチジョブを運用する、組織内の運用担当者・開発者
- B. A に加えて、ローカルでパイプラインをデバッグ・テストする開発者
- C. 組織外の顧客・ユーザー
- D. Not yet defined(まだ定まっていない)
- X. Other (please specify)

[Answer]: B

## Q3. [プロダクト] 成功の基準

何をもって初版が成功したと判断しますか(Success Metrics の根拠になります)。

- A. 「個別実行」と「一連の実行」の2モードで、警告時の分岐・スキップが定義どおりに動作する(機能面の達成)
- B. A に加えて、1回の起動あたりのオーバーヘッドの目標値を数値で定める(値は Other で指定)
- C. Not yet defined(まだ定まっていない)
- X. Other (please specify)

[Answer]: A

## Q4. [プロダクト] 今回の取り組みのきっかけ

なぜ今これを作るのですか(Initiative Trigger の根拠になります)。

- A. 既存実行環境の制約を、環境自体は変えずに上位レイヤーで補いたい
- B. 現行運用での障害対応や手作業(警告時の手動での切り分けなど)を減らしたい
- C. Not identified(特定できていない)
- D. Not applicable(該当なし)
- X. Other (please specify)

[Answer]: A

## Q5. [プロダクト] 関係者と意思決定者

スコープや優先度を決める人、影響を与える人は誰ですか(Stakeholder Map の根拠になります)。

- A. 依頼者本人が単独で決定し、他の関係者はいない(個人・少人数の開発)
- B. 依頼者が決定し、既存ジョブ環境の運用チームが利用者・意見提供者として関わる
- C. Not identified(特定できていない)
- X. Other (please specify)

[Answer]: A

## Q6. [プロダクト] 報告・連絡の要件

進捗や結果を誰かに定期的に報告する必要はありますか。

- A. 特に報告先・頻度の要件はない
- B. ステージの承認ごとに依頼者が確認すれば足りる
- C. Not yet defined(まだ定まっていない)
- X. Other (please specify)

[Answer]: A

## Q7. [プロダクト] 初版で対象外とするもの(複数選択可 / select all that apply)

この初版でやらないことを選んでください(ここでの「対象外」はプロダクトの機能についてであり、ワークフローの進め方ではありません)。

- A. 通知機能(Slack 通知など)そのものの組み込み(ジョブ側のスクリプトで行う)
- B. 複数ホストにまたがる分散実行・ステート共有
- C. Web UI / GUI
- D. None(特になし)
- X. Other (please specify)

[Answer]: A, B, C

## Consolidated Summary Confirmation

Does this all look correct before I generate the artifact?

- Looks correct
- Request changes

[Answer]: Looks correct
