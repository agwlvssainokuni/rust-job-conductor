# External Dependency Map(外部への依存)

上流: bolt-plan、constraint-register、raid-log、team-practices。**Bolt** とは、Construction での作業の1回のまとまり(1つ以上の Unit を作り、動くものに到達するまで)である。

## 外部への依存

作業を止めうる、チームの外の事情(外部の API、データ、承認、他チームの引き渡しなど)は、ない。[Q2: A] 既存のジョブスケジューラとは、`conductor` を呼び出してもらう接点があるが、`conductor` の開発の進行を止める要素ではない。(実際のスケジューラは、利用の段階で結びつく)

## 自分で用意するもの

| 項目 | 内容 | 使う Bolt | 担当 | 遅れたときの対応 |
|------|------|-----------|------|------------------|
| GitHub Actions(CI) | 3つの OS(Linux・macOS・Windows)でのテストの実行と、タグ付けでのリリースバイナリの作成 | B1 以降(3つの OS でのテストは、B1・B2 から回す計画) | 依頼者 | CI の設定が遅れたときは、主な開発環境の OS でのローカルのテストを先に進め、他の OS のテストは、CI の用意ができた時点でまとめて行う |
| 3つの OS の実機・環境 | CI で足りない場合に、OS の挙動を手元で確かめる | B3、B4 | 依頼者 | CI の結果から原因を調べ、必要になったときだけ、手元の環境を用意する |

## Assumptions & Open Questions

- 「主な開発環境の OS」がどれかは、未確定である(practices-discovery の未解決事項)。確定は、B1 の開始前に行う。 [assumption]
- 完成したバイナリの配布先と、公開の承認の要否は、未確定である(practices-discovery の未解決事項)。Deployment のステージで確認する。 [assumption]
