# Dependencies

## 内部の依存

cli → core, runtime, converter / runtime → core(`path` 指定。architecture.md の図を参照)。

## 外部の依存(直接。`crates/*/Cargo.toml`)

| crate | 指定 | ロック済みの版 | 使用箇所 | 備考 |
|-------|------|----------------|----------|------|
| lexopt | 0.3.2 | 0.3.2 | cli | |
| toml | 1.2.1(`default-features = false`、features は crate ごとに異なる) | 1.2.1+spec-1.1.0 | core、runtime、converter | |
| thiserror | 2.0.21 | 2.0.21 | core、runtime、converter | |
| saphyr-parser | 0.3.4 | 0.3.4 | converter | |
| signal-hook | 0.4.6(`default-features = false`、`iterator`) | 0.4.6 | cli(unix) | |
| ctrlc | 3.5.2 | 3.5.2 | cli(windows) | |
| libc | 0.2(範囲指定) | 0.2.191 | runtime(unix) | 唯一の緩い指定 |
| tempfile | 3.27.0 | 3.27.0 | 全 crate の dev-dependencies | |

指定の書式は、`0.2` を除き、`^` を省いた完全な版(Cargo では `^` と同じ互換範囲の意味)。

## 推移的な依存(`Cargo.lock` 抜粋)

bitflags 2.13.2、cfg-if 1.0.5、errno 0.3.15、fastrand 2.5.0、getrandom 0.4.3、hashbrown 0.17.1、indexmap 2.14.2、linux-raw-sys 0.12.1、nix 0.31.3、objc2 0.6.5、once_cell 1.21.4、rustix 1.1.6、serde_core / serde_derive 1.0.229、syn 3.0.10、toml_datetime 1.1.3、toml_parser 1.1.5、toml_writer 1.2.0、winnow 1.0.5、windows-sys 0.61.2、windows-link 0.2.1 ほか(全 50 余り)。`Cargo.lock` は version 4。

## 方針

- `-sys` crate と C ビルド依存(cc、cmake、pkg-config、bindgen)を持たない(`ci.yml` build-check で検査。`linux-raw-sys` と `windows-sys` は許容)
- CI は `--locked` でビルドする
- 脆弱性の検査: `cargo audit`(`audit.yml`、警告のみ)
- 依存の自動更新の設定(Dependabot / Renovate)は無い(`.github/` に該当ファイルなし)
