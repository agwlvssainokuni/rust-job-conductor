# Dependencies

## 外部の依存
| クレート | 版 | 使うクレート | 条件 |
|----------|-----|--------------|------|
| lexopt | 0.3.2 | cli | 全 OS |
| signal-hook | 0.4.6(iterator のみ) | cli | cfg(unix) |
| ctrlc | 3.5.2 | cli | cfg(windows) |
| libc | 0.2 | runtime | cfg(unix) |
| toml | 1.2.1 | core、converter、runtime | 全 OS |
| saphyr-parser | 0.3.4 | converter | 全 OS |
| thiserror | 2.0.21 | core、converter、runtime | 全 OS |
| tempfile | 3.27.0 | 全クレート(dev) | テストのみ |

## 内部の依存
cli → core、runtime、converter / runtime → core / converter は core に依存しない。

## 制約
CI が、C のビルドを要する依存(cc、cmake、pkg-config、bindgen)と、`windows-sys`・`linux-raw-sys` 以外の `-sys` クレートを禁止している(`ci.yml` の build-check)。`cargo audit` を毎週月曜と PR で実行(警告のみ)。
