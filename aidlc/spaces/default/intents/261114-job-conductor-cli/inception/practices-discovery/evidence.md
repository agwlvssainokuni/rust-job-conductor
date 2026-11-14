# Evidence

## 調べたこと(Lead: aidlc-pipeline-deploy-agent、単独。協力者なし)

- `aidlc/spaces/default/memory/org.md` の 5 セクション(Way of Working、Walking Skeleton、Testing Posture、Deployment、Code Style)を読み、インタビューの提案する既定値として扱った。
- `aidlc/spaces/default/memory/team.md` は空で、確定済みの基準はなかった。
- 上流成果物を確認した: intent-statement、constraint-register(C1〜C9)、scope-document。
- プロジェクト事実: Rust 製の単一バイナリ CLI、開発者は Rust に不慣れ(C5)、Linux・macOS・Windows で動作(C3)、期限・予算・規制の制約なし(C7〜C9)。
- greenfield のため、git 履歴や CI 設定からの推定はしていない。

## 推定したこと(インタビュー前の提案)

- Rust 固有のツール選択(`rustfmt`、`clippy`、`cargo test`、3 OS の CI)は Rust の標準的な慣行からの推定だった。Q4・Q6 でチームが承認した。
- Deployment: org.md の「マージ時にステージングへデプロイ」は CLI に当てはまらないため、タグによるリリース成果物作成に読み替えた。Q5 で承認された。

## インタビュー決定

- Q1 Way of Working: B。`main` を統合ブランチとし、squash せず履歴をそのまま残す(org.md の squash 既定を変更)。
- Q2 Walking Skeleton: B。最初の薄い縦切りは作らず、機能を 1 つずつ作る(提案を変更)。
- Q3 Testing Methodology: A。test-after(提案どおり)。
- Q4 Testing 基準: A。行カバレッジ 80% 下限。「主な開発環境の OS 以外は CI(GitHub Actions)で」との補足があり、ローカルは主な開発環境の OS、他の OS は CI で実行する。
- Q5 Release: A。タグ付けで CI が 3 OS 向けリリースバイナリを自動ビルドする。
- Q6 Code Style: A。`rustfmt` と `clippy`、CI 失敗でマージをブロック、命名は Rust 標準。
- Q7: 厳格な必須・禁止ルールはなし。discovered-rules.md の Mandated / Forbidden は `None.` のまま。
- Summary Confirmation: Looks correct。

## 未解決の不確実性

- 「主な開発環境の OS」がどの OS かは未特定。CI のマトリクスで 3 OS を実行するか、主な OS を除くかは後続ステージで決める。
- 配布先の詳細(GitHub Releases など)とリリース公開の承認の要否は、タグ → 自動ビルドまでを確認したのみで、公開手順は未確定。
