# Evidence

## 調べたこと(Lead: aidlc-pipeline-deploy-agent、単独ドラフト)

- `aidlc/spaces/default/memory/org.md` の 5 セクション(Way of Working、Walking Skeleton、Testing Posture、Deployment、Code Style)を読み、**提案する既定値**として扱った。確立済みのチーム事実としては扱っていない。
- `aidlc/spaces/default/memory/team.md` は空で、確定済みの基準はない。
- 上流成果物を確認した: intent-statement、constraint-register(C1〜C9)、scope-document。
- プロジェクト事実: Rust 製の単一バイナリ CLI、開発者は Rust に不慣れ(C5)、Linux・macOS・Windows で動作(C3)、期限・予算・規制の制約なし(C7〜C9)。
- greenfield のため、git 履歴や CI 設定からの推定はしていない。

## 推定したこと(すべて提案する既定値)

- 統合ブランチ `main` とトランクベース開発: org.md の既定値。
- Methodology は test-after: org.md の未確定時の既定値。Rust 未経験との関連は未判断。
- Rust 固有のツール選択(`rustfmt`、`clippy`、`cargo test`、3 OS の CI): Rust の標準的な慣行からの推定であり、チームが選んだものではない。
- Deployment: org.md の「マージ時にステージングへデプロイ」は CLI には当てはまらないため、タグによるリリース成果物作成に読み替えた。

## インタビュー決定

- まだ実施していない(Step 4 で確認する)。

## 未解決の不確実性

- Methodology(test-after / tdd / bdd / atdd / custom)と Ordering の最終決定。
- カバレッジ下限 80% を採用するか。
- Walking Skeleton の採否と範囲。
- 配布方法(GitHub Releases など)、リリース承認の要否。
- CI 基盤(GitHub Actions など)が未定。
- 厳格な禁止・必須ルール(Mandated / Forbidden)は、人間から述べられていないため `None.`。
