# RAID Log

上流: intent-statement と feasibility-questions の回答。以下は回答から導いた整理であり、リスクの影響度はアーキテクトの見立て(仮説)である。

## Risks

| ID | Risk | Impact(仮説) | Source |
|----|------|---------------|--------|
| R1 | Windows を含む複数OSで、プロセス制御・ファイル排他の挙動が異なり、品質にばらつきが出る。 | 中 | [Q1] |
| R2 | 開発者が Rust に不慣れなため、実装の進行や品質にばらつきが出る。 | 中 | [Q3] |
| R3 | 終了コードが 0 / 非0 のみのため、スキップと正常終了をスケジューラ側から区別できず、運用時の確認が難しくなる。 | 中 | [Q2] |

## Assumptions

None.

## Issues

None.

## Dependencies

| ID | Dependency | Source |
|----|------------|--------|
| D1 | 既存の直列ジョブスケジューラが `conductor` をジョブのコマンドの前に挟んで呼び出せること。 | [Q2] |

## Assumptions & Open Questions

None.
