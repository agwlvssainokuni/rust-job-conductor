# 結合テストの手順(Integration Test Instructions): graph の配置のリファクタリング

ユニットは 1 つで、外部のサービスはない。結合の確かめは、実バイナリと README の検査で行う(Minimal)。

## コマンド

```bash
cargo test --locked -p conductor-cli --test binary_graph --test docs_graph --test graph_report
```

## 内容

- `binary_graph.rs`: 定義ファイルから `conductor graph` を実際に動かす。N1(「両方の飛び越し」と通常の飛び越しの交差)が、辺の一覧ではなく、レーンの図で出ること。分岐、両方の飛び越しの図が、新しい列 0 の規則で出ること。
- `graph_report.rs`: text の全文、`-v` の詳細の行の列 0(N1、N2、N5)、`--state`、辺の一覧の整形(注記の文言は、今の理由「レーンの図で描けない形」)。
- `docs_graph.rs`: README の図の例が、実出力と 1 文字も違わず一致すること。

## 手での確認(済み)

ビルドしたバイナリで、B1、N1、N2 の定義と、N1 の `-v` を出し、設計(`functional-spec.md` の §4、§6b)の図と一致することを確かめた。
