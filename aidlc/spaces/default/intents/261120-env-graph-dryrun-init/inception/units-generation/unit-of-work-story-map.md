# ユニットと要件の対応(Story Map): v0.9.0 の機能拡張

ユーザーストーリーは作っていない(省略したステージ)ので、要件(`requirements.md` の FR のサブ要件)を、行の単位にする。

| 要件 | Unit ID | ディレクトリ |
|------|---------|--------------|
| FR1.1 | U1 | u1-env-injection |
| FR1.2 | U1 | u1-env-injection |
| FR1.3 | U1 | u1-env-injection |
| FR1.4 | U1 | u1-env-injection |
| FR1.5 | U1 | u1-env-injection |
| FR1.6 | U1 | u1-env-injection |
| FR1.7 | U1 | u1-env-injection |
| FR1.8 | U1 | u1-env-injection |
| FR1.9 | U1 | u1-env-injection |
| FR1.10 | U1 | u1-env-injection |
| FR1.11 | U1 | u1-env-injection |
| FR1.12 | U1; U2; U3 | u1-env-injection; u2-graph; u3-dry-run |
| FR1.13 | U1 | u1-env-injection |
| FR2.1 | U2 | u2-graph |
| FR2.2 | U2 | u2-graph |
| FR2.3 | U2 | u2-graph |
| FR2.4 | U2 | u2-graph |
| FR2.5 | U2 | u2-graph |
| FR2.6 | U2 | u2-graph |
| FR2.7 | U2 | u2-graph |
| FR3.1 | U3 | u3-dry-run |
| FR3.2 | U3 | u3-dry-run |
| FR3.3 | U3 | u3-dry-run |
| FR3.4 | U3 | u3-dry-run |
| FR4.1 | U4 | u4-init |
| FR4.2 | U4 | u4-init |
| FR4.3 | U4 | u4-init |
| FR4.4 | U4 | u4-init |
| FR4.5 | U4 | u4-init |
| FR5.1 | U1; U2; U3; U4 | u1-env-injection; u2-graph; u3-dry-run; u4-init |
| FR5.2 | U1; U2; U3; U4 | u1-env-injection; u2-graph; u3-dry-run; u4-init |

## 複数のユニットにまたがる要件

- FR1.12(環境変数の値を、出力とステートに出さない): U1(統合と名前だけの要約)、U2(graph の出力に値を出さない)、U3(dry-run の出力に値を出さない)。
- FR5.1(日本語と英語のメッセージ)、FR5.2(README とデモ): 各ユニットが、自分の機能の分を作る(ユニットの質問 Q3)。

## ユニット内の実装の順序(参考。拘束しない。順序は Delivery Planning と、各ユニットの設計が決める)

- U1: 定義の型と新しいキーの検査 → `.env` の書式の検査と層の統合(`core`) → ファイルの読み取り(`runtime`) → 起動への反映と `validate` の警告(`cli`) → README、デモ、メッセージ。
- U2: 辺の取り出しの公開(`core::flow`) → 辺とレーンの配置(`core`) → text と Mermaid の整形(`cli`) → `--state` の重ね合わせ → README、デモ、メッセージ。
- U3: 計画の生成(`core`) → `--dry-run` の流れ(`cli`) → `--assume` の模擬 → README、デモ、メッセージ。
- U4: 排他的な作成(`runtime`) → ひな形の文字列と `init` の流れ(`cli`) → README、デモ、メッセージ。

## 網羅の確認

- 要件の 31 のサブ要件(FR1.1 から FR5.2)は、すべていずれかのユニットに割り当てた。
- 4 つのユニットのすべてに、行がある。
