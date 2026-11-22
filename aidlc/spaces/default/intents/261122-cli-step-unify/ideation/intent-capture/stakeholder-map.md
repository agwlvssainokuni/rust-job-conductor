# 関係者の地図(Stakeholder Map): run と pipeline の重複をなくすリファクタリング

## Stakeholders

| 関係者 | 関心 | 決める人か、影響する人か | Source |
|--------|------|--------------------------|--------|
| この作業を頼んだ人(開発者) | 1 つのジョブを処理する手順が 1 か所になり、機能を足すときに 2 か所を揃えなくてよくなること。動きが変わらないこと | 決める人(範囲と確かめ方) | [desc] [Q2] [Q3] |
| `conductor run` と `conductor pipeline` を使う人 | 出力、終了コード、ステートが、今までと同じであること | 影響する人 | [desc] [Q1] |
| 仕様書(`docs/spec/rust`)を読んで実装する人 | 実行の流れの章で、run と pipeline の手順が共通の手順として 1 回だけ書かれていること | 影響する人 | [Q1] |

## Communication Requirements

| 必要なこと | Source |
|------------|--------|
| 動きを変えないことを、既存のテストを変えずにすべて通すことで示す | [Q3] |

## Assumptions & Open Questions

None.
