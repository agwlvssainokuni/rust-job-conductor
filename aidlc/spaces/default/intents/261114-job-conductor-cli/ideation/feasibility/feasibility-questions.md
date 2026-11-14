# Feasibility & Constraints — 質問

## Q1. [開発] 動作させる OS・環境

`conductor` はどの環境で動かす必要がありますか(配布形態と技術的な実現性に影響します)。

- A. Linux のみ(既存ジョブスケジューラが動くサーバー環境)
- B. Linux と macOS(ローカルでのデバッグ・テスト用を含む)
- C. Linux・macOS・Windows
- D. Not yet defined(まだ定まっていない)
- X. Other (please specify)

[Answer]: C

## Q2. [運用] 既存のジョブスケジューラとの接続条件

既存スケジューラは `conductor` をジョブのコマンドの前に挟んで呼び出す想定です。スケジューラ側の制約を確認します。

- A. ジョブのコマンドを任意に書き換えられる(`conductor` をラッパーとして挟める)
- B. コマンドは書き換えられるが、終了コードは 0 / 非0 の2値しか受け付けない
- C. スケジューラの詳細はまだ把握していない
- D. Not applicable(該当なし)
- X. Other (please specify)

[Answer]: B

## Q3. [開発] 開発チームのスキルと技術スタックの制約

実装言語や技術スタックについて、決まっていること・制約はありますか。

- A. Rust で実装することが決まっており、開発者に Rust の経験がある
- B. Rust で実装することが決まっているが、開発者は Rust に不慣れである
- C. 言語は未決定
- D. Not yet defined(まだ定まっていない)
- X. Other (please specify)

[Answer]: B

## Q4. [プロダクト] 期限と予算の制約

初版の完成時期や予算の制約はありますか。

- A. 期限・予算の制約はない
- B. 期限はあるが予算の制約はない(期限は Other に記入)
- C. Not yet defined(まだ定まっていない)
- X. Other (please specify)

[Answer]: A

## Q5. [運用] 規制・セキュリティ・データの取り扱い

実行対象のジョブや状態ファイルに、機密情報や守るべき規制はありますか。

- A. 特になし(規制・機密データなし)
- B. ジョブの引数や環境変数に認証情報が含まれうるため、ログや状態ファイルへの出力に配慮が必要
- C. 社内の情報セキュリティ規程への準拠が必要(内容は Other に記入)
- D. Not yet defined(まだ定まっていない)
- X. Other (please specify)

[Answer]: A

## Q6. [開発] 起動速度に関する許容値

個別実行モードでは毎ステップ `conductor` が起動されます。許容できる起動のオーバーヘッドの目安はありますか(実現性の評価基準になります)。

- A. 数値の目安はなく、ジョブ本体に比べて無視できる程度であればよい
- B. 数値の目安がある(値は Other に記入)
- C. Not yet defined(まだ定まっていない)
- X. Other (please specify)

[Answer]: A

## Q7. [プロダクト] 組織上の障害・競合する優先事項

進行を妨げうる事情(変更凍結期間、他の優先事項など)はありますか。

- A. 特になし
- B. あり(内容は Other に記入)
- C. Not identified(特定できていない)
- X. Other (please specify)

[Answer]: A

## Consolidated Summary Confirmation

Does this all look correct before I generate the artifact?

- Looks correct
- Request changes

[Answer]: Looks correct
