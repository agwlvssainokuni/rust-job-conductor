# 構成のずれの報告: v0.10.0

出典: `feedback-optimization-questions.md`(まとめの確かめ済み)、`construction/ci-pipeline/ci-config.md`、`operation/deployment-pipeline/cd-config.md`、team.md、project.md。

| 対象 | 決まり | 実際 | ずれ |
|---|---|---|---|
| `ci.yml` | v0.10.0 でも変えない | 変えていない | 無し |
| `release.yml` | タグで下書きを作り、人が公開 | そのとおり動いた(45727642798) | 無し |
| 依存(`Cargo.lock`) | 足さない | package の数は 46 のまま | 無し |
| MSRV | 1.100 | `msrv-test`、`build-check` が成功 | 無し |
| `unsafe`・`cfg` の場所 | `platform` の中だけ | `nfr_source.rs` が通る | 無し |
| ステートの形 | 時刻のほかを書かない、`format_version` は 1 | `nfr_content.rs` が通る | 無し(`[[job.attempt]]` は決めたとおりの追加) |
| 版とタグ | `main` の上、版と一致 | `verify-tag` が成功 | 無し |

- 作業のやり方のずれ: コードを書く担当が、ファイルの道具を使う決まりに反してシェルや `python3` でファイルを直したことが 3 回あった(結果には影響なし)。`feedback-loop.md` で次からの直し方を決めた。
