# Code Generation Plan(u4-converter)

## Summary

- Builds: 定義ファイルを YAML と TOML のあいだで変換するライブラリ(`convert`)。YAML はイベントで読み、展開と数えを自分で行う
- Touches: `crates/u4-converter/`(パッケージ `conductor-converter`。ソースとテスト)。ワークスペースの `Cargo.toml` と `Cargo.lock`(依存の追加のため)
- Tests: 約 50 件(単体、往復、失敗の順序、暴走の入力、統合。Standard の方針と、行カバレッジ 80% の下限)

## 前提(設計から)

- 上流: functional-spec(W1)、rules(BR1.1〜BR1.12)、entities、contract-summary(C4)、nfr-design の security-design(**変換の流れ、`Value`、D1〜D4、既存の成果物との差分**。差分は、nfr-requirements の関連する記述より優先する)、logical-components、tech-stack-decisions(T1、T2)、cicd-pipeline。
- 設計の成果物は書き換えない。設計との差は、`code-summary.md` に記録する。
- テストの順序は、チームの実践(test-after)に従い、**層ごとに、実装してから、その層のテストを書いて実行する**。
- コードは、`crates/u4-converter/`(パッケージ名 `conductor-converter`)に置く。ワークスペースは、`crates/*` を含んでいるため、`Cargo.toml` のメンバーの変更は要らない。
- 公開する関数(契約 C4): `convert(request: &ConversionRequest) -> Result<ConvertedText, ConversionError>`。入力のファイルを読む以外の入出力を持たず、結果は文字列で返す(書き出しは u3-cli)。ファイルに触れずに試せる内部の入口 `convert_text(text, target, file_label)` も公開する。

## 実装の手順

### 層 0: 準備

- [x] **Step 1: クレートの準備と、ライブラリの確認(テストの実行環境の準備を含む)**
  - `crates/u4-converter/Cargo.toml`(edition・rust-version・license は、ワークスペースから継ぐ)、`src/lib.rs`(空のクレート)を作る。依存: `toml`(u1-core と同じバージョンと機能)、`thiserror`(同じ)、開発用に `tempfile`(同じ)。YAML のライブラリは、下の確認で決める。バージョンは、その時点の安定版を選び、`Cargo.lock` で固定する。
  - **YAML のライブラリの確認(tech-stack-decisions の T1 と、security-design の「既存の成果物との差分」)**: 次の条件を、実際のコードで確かめて、満たすものを選ぶ。(a) 展開しないイベントの列(ノード、アンカー、エイリアスの参照、位置)を、呼び出し側が、再帰を使わずに読める、(b) YAML 1.2 のコアスキーマに沿って、スカラーを自分で解決できる(スカラーが文字列として得られる)、(c) タグの情報を得られる、(d) 文法の誤りの位置(行)が得られる、(e) Rust だけで書かれ、`cargo tree` に `-sys` がない、(f) rust-version 1.85 でビルドできる(できなければ、**止めて人に確認する**)、(g) 保守が続いている。
  - **条件を満たすライブラリがなければ、止めて、人に確認する**(選択肢: 条件を緩める、Functional Design の規則(BR1.2、BR1.7)を見直す、アンカー・エイリアス・マージキーを `unrepresentable` で拒否する縮小)。勝手に決めて進めない。
  - TOML の確認(T2): (a) 深さ 1000 と 100,000 の入力で、異常終了せず失敗が返る、(b) 日時の値を判別できる、(c) 整数の範囲(`i64`)が扱える、(d) 文字列の引用符を正しく出力できる。確かめられなければ、止めて人に確認する(T2)。
  - 確認の結果と選んだライブラリを、`code-summary.md` に記録する。
  - 実行できることを確かめる: `cargo test -p conductor-converter --locked`(テスト 0 件で成功)。
  - 関わる: NFR2.1、NFR3.1、NFR5.1、NFR4.3、NFR4.4。

### 層 1: 型とエラー

- [x] **Step 2: 型(`types.rs`)**
  - entities.md の型: `TargetFormat`(`Yaml`/`Toml`)、`ConversionRequest`(input_path、target_format)、`ConvertedText`(text)。内部の `Value`(security-design の D1。文字列、整数 `i64`、浮動小数 `f64`、真偽、配列、マップ(キーは文字列、入力に現れる順を保つ並び))。入出力に依存しない。
  - 関わる: contract C4、BR1.1、BR1.2、BR1.11。
- [x] **Step 3: エラーの型と表示(`error.rs`、Describe)**
  - `ConversionErrorKind`: fileMissing、unreadable、tooLarge、syntax、unrepresentable、rootNotMap。`ConversionError`(file、line、key_path、kind、reason)。`thiserror` で、kind ごとに固定の表示の文を付ける(ライブラリのエラー文、ソースの抜粋、入力の値は含めない)。キー名など入力に由来する文字列は、制御文字と書式・双方向の制御文字をエスケープし、64 文字までに切る。`key_path` の書式(例: `jobs[2].transitions[0]`)は、ここで決め、`code-summary.md` に記録する。非文字列キーは、そのマップの経路に `(非文字列キー)` を付ける。
  - 関わる: NFR4.6、NFR4.1、BR1.10。
- [x] **Step 4: 層 1 のテスト**: 表示の文に入力の値が入らないこと(全 kind)、キー名のエスケープと切り詰め(制御文字、書式文字)、`key_path` の書式、`Value` の等しさの比較(NaN どうしは等しく、`0.0` と `-0.0` は区別)。

### 層 2: 読み取り

- [x] **Step 5: 取り込み(`intake.rs`、Intake)**
  - ファイルを開き、通常のファイルか確かめ、「上限(1 MiB)+ 1 バイト」までの読み取りを、`impl Read` を取る関数に分けて行う(終わりのない入力でも、上限 + 1 バイトで止まることを、単体で確かめられるように)。順序は BR1.12 の 1〜4(fileMissing → unreadable(権限・通常でない)→ tooLarge → unreadable(UTF-8 でない))。
  - 関わる: BR1.5、BR1.12、NFR4.2、AC7.1.3。
- [x] **Step 6: YAML の読み取り(`yaml_read.rs`、Read)**
  - 選んだライブラリのイベントの列を、**再帰を使わず、明示のスタックで**読み、展開しない構造(素の木。アンカーとエイリアスの参照を持つ)を作る。この段階で、文法の誤り、重複するキー、再帰するアンカー(閉じる前のアンカーを参照するエイリアス)、展開前の構造の深さ 32 の超過(最上位のマップを 1 とし、33 段目で打ち切る)を `syntax`(行つき)で返す。文書の数を数える。スカラーは、YAML 1.2 のコアスキーマで解決する(`true`/`false` だけが真偽値、`null`/`~`/空は null、整数、浮動小数、それ以外は文字列。`yes`/`no`/`on`/`off` は文字列)。タグつきの値を、`unrepresentable` の記録の対象として印をつける(`!!str` など、標準のスカラーのタグで型が変わらないものは、タグなしと同じ)。
  - 関わる: BR1.2、BR1.4、BR1.6、BR1.7、BR1.12、NFR4.4、NFR4.5、NFR4.7。
- [x] **Step 7: TOML の読み取り(`toml_read.rs`、Read)**
  - `toml` のライブラリで読み、素の木(日時の値に印)に写す。読み取り後に、深さを数え、32 を超えたら `syntax`。文法の誤りは `syntax`(行つき)。空の TOML は空のマップ。
  - 関わる: BR1.4、BR1.8、NFR4.4。
- [x] **Step 8: 文書の確認と展開(`resolve.rs`、RootCheck と Resolve)**
  - 複数の文書は `unrepresentable`、最上位がマップでなければ `rootNotMap`(空の YAML も)。素の木を、明示のスタックで、深さ優先・入力に現れる順に、`Value` へ写す。アンカー・エイリアスの展開、マージキー(`<<`。引用符つきの `"<<"` は通常のキー。値がマップ、またはマップのみの並びでなければ `unrepresentable`。並びは先頭が優先、マップ自身のキーが優先)の展開を行う。展開のために訪れたノードを、すべて数え(捨てられる項目、複製も含む)、100,000 を超えたら `tooLarge`、展開後の深さが 32 を超えたら `syntax` で、その時点で打ち切る。表せない値(null、文字列でないキー、タグつきの値、TOML の日時、`i64` を超える整数)の最初の1つを記録し、打ち切りがなければ、そのあとで `unrepresentable`(key_path つき)を返す。
  - 関わる: BR1.6、BR1.7、BR1.8、BR1.12、NFR4.3、NFR4.4。
- [x] **Step 9: 層 2 のテスト**(各モジュールの単体テストと `tests/read_*.rs`)
  - 取り込み: 存在しない、1 MiB ちょうどは受理、1 MiB + 1 は `tooLarge`、終わりのない入力が上限 + 1 で止まる、UTF-8 でない、ディレクトリ。
  - YAML の読み取り: 文法の誤り、重複するキー、再帰するアンカー、深さ 32 は受理、33 は `syntax`、深さ 1000 と 100,000(配列、インラインのマップ、インデント)で異常終了せず `syntax`、`yes`/`on`/`null`/`~`/整数/浮動小数/文字列のスカラーの解決。
  - TOML の読み取り: 文法の誤り、深さ 32/33/1000/100,000、日時の印、空の TOML。
  - 展開と確認: 複数の文書、最上位が配列・空の YAML(`rootNotMap`)、アンカーとエイリアスの展開、マージキー(単独、並び、優先順位、引用符つき `"<<"`、マップでない値)、展開で膨らむ入力(100,000 ちょうどは受理、100,001 は `tooLarge`。捨てられる項目も数える)、展開後の深さの超過、表せない値の最初の1つ、**失敗の順序(BR1.12)**(複数の失敗が同時にあるとき、決められた kind が返る)。

### 層 3: 書き出しと入口

- [x] **Step 10: 書き出し(`yaml_write.rs`、`toml_write.rs`、Write)**
  - `Value` を、YAML と TOML の文字列にする。文字列は、値もキーも、読み戻したときに文字列のままになるように、必要なら引用符で囲む(`yes`、`1`、`null`、`true`、空文字列、先頭や末尾の空白、特殊な文字)。キー `<<` は、YAML へ出力するとき、必ず引用符で囲む。浮動小数は、整数と区別できる形(`1.0`)で、`-0.0`、`nan`、`inf`、`-inf` を保つ。整数は `i64`。マップの順序は、`Value` の並び(読み取りの順)。TOML は、`toml` のライブラリの書き出しを使い、整列などは、ライブラリの既定でよい。
  - 関わる: BR1.2、BR1.3、BR1.9、BR1.11。
- [x] **Step 11: 公開の入口(`convert`、`convert_text`)**
  - `convert(request)`: Intake → Read(`--to` と反対の形式)→ RootCheck → Resolve → Write の流れを組み立て、`ConvertedText` を返す。入出力に触れるのは Intake だけ。`convert_text(text, target, file_label)` は、ファイルに触れずに試せる入口。
  - 関わる: contract C4(`convert`)、BR1.1、BR1.9、NFR4.5。
- [x] **Step 12: 層 3 のテスト**(`tests/convert_*.rs`、`tests/fixtures/*.yaml|toml`)
  - **往復(BR1.11)**: 定義ファイルらしい TOML と YAML(3〜4 ジョブ、`on` キーを含む)、スカラーの境界(`"yes"`、`"1"`、`"null"`、`"true"`、`"<<"` をキーと値に持つ)、浮動小数(`1.0`、`-0.0`、`nan`、`inf`)、`i64` の端、空のマップ、入れ子の配列とマップ、で、変換した結果を読み戻した木が、入力の木と等しい(TOML → YAML → TOML、YAML → TOML → YAML)。
  - AC7.1.1(TOML → YAML)、AC7.1.2(YAML → TOML。アンカー・エイリアスつき)、AC7.1.3(文法が壊れた入力は、変換されず、種類と行が返る。出力がない)。
  - 暴走の入力のテスト(深さ 100,000、展開で膨らむ YAML)は、各 30 秒の時間の上限(別のスレッド + `recv_timeout`)を持つ。

### 層 4: 横断

- [x] **Step 13: 公開の API の文書(`lib.rs`、`///` の説明)**: 公開する型と `convert`・`convert_text` に、入出力・失敗・規則の ID を書く。モジュールの構成(logical-components の呼び名との対応)を `lib.rs` に書く。関わる: contract C4。
- [x] **Step 14: 横断のテスト(`tests/nfr_*.rs`)**:
  - NFR4.5: この Unit のソースに `std::process`、`std::net` が現れないこと(ソースの文字列の検索)。環境変数を変えても結果が変わらないこと。一時ディレクトリの外にファイルが作られないこと。`!!binary` と独自のタグは `unrepresentable`、`!!str "1"` は文字列 `1`。
  - NFR4.6: すべての `ConversionErrorKind` で、表示の文に、入力の値が現れないこと(キー名は、エスケープされた形で現れてよい)。
  - NFR4.7: 範囲を超える整数、壊れた UTF-8、空の入力でパニックしないこと。
  - NFR1.1: この Unit に、静的な初期化(起動時に動く処理、`lazy` な大域の変数)がないこと(ソースの検索)。
- [x] **Step 15: 確認(Build and Test の前の、この Unit の自己確認)**: `cargo fmt --all -- --check`、`cargo clippy -p conductor-converter --all-targets --locked -- -D warnings`、`cargo test -p conductor-converter --locked` が通ること。`cargo llvm-cov` が使えれば、このクレートの行カバレッジを測って記録する(使えなければ、「測っていない」と `code-summary.md` に書く。80% の判定は Build and Test が、全体のテストで行う)。`cargo +1.85 build -p conductor-converter --locked` が、使える環境なら、通ること(使えなければ、記録する)。

## テストの方針(Standard)

- 部品ごとに 5〜8 件を目安に、規則は規則ごとに確かめる。単体テストは `#[cfg(test)]` のモジュールと `tests/`、入力の例は `tests/fixtures/`。
- 行カバレッジ 80% 以上(下限。設定でしきい値を下げない)。
- ファイルを使うテストは `tempfile` で一時ディレクトリを作る。外部のサービスやネットワークは使わない。

## 設計からの追跡(ストーリー → 手順)

| ストーリー | 手順 |
|------------|------|
| US7.1(AC7.1.1、AC7.1.2、AC7.1.3) | Step 5〜12 |
| NFR1.1 / NFR2.1 / NFR3.1 | Step 1、14 |
| NFR4.1〜NFR4.7 | Step 3、5〜9、12、14 |
| NFR5.1 | Step 4、9、12、15 |

## Testing Contract

```json
{
  "version": 1,
  "methodology": "test-after",
  "source": "team",
  "ordering": "各テスト可能な層を実装してから、その層のテストを書いて実行する。",
  "scope": "mvp",
  "test_strategy": "standard",
  "project_type": "greenfield",
  "applicable_notes": [
    {
      "layer": "org",
      "text": "We treat tests as a first-class deliverable in every Bolt. The specific\nmethodology (TDD, BDD, ATDD, or classic test-after) is affirmed at\npractices-discovery and recorded in `team.md` under this heading with explicit\n`Methodology` and `Ordering` fields; Code Generation resolves those fields\nindependently from coverage, tooling, and scope notes.\n\nWhen no posture has been affirmed, our default per scope is:\n- **Methodology**: test-after\n- **Ordering**: implement each applicable testable layer, then write and run\n  that layer's tests.\n- `mvp`, `enterprise`, `feature`, `infra` add an 80% line-coverage\n  floor and CI execution before merge.\n- `bugfix`, `security-patch` add a targeted regression for the specific\n  bug/vulnerability and require the existing suite to remain green.\n- `express` uses the Minimal strategy: requirement-driven unit tests (one per\n  requirement, with a happy-path floor per component); existing tests remain\n  green.\n- `poc`, `refactor`, `workshop`, `classic` add no extra new-test floor and require the\n  existing suite to remain green.\n\nThe active `Test Strategy` still applies in every scope and determines test\nvolume/types. Scope floors are additive; they never reduce or replace the\nselected strategy.\n\nBuild and Test verifies defined coverage floors and affirmed quality targets;\nthey may not be weakened to make a step pass.\n\nAffirm a stricter posture in `team.md` if the team commits to one."
    },
    {
      "layer": "team",
      "text": "- **Methodology**: test-after\n- **Methodology evidence**: インタビュー Q3 で、チームは test-after(実装してからテストを書く)を選んだ。\n- **Ordering**: 各テスト可能な層を実装してから、その層のテストを書いて実行する。\n- `cargo test` による単体テストと、バイナリを実行する統合テストを含める。\n- 行カバレッジ 80% を下限とする(インタビュー Q4: A)。\n- 主な開発環境の OS では、ローカルでテストを実行する。それ以外の OS(Linux・macOS・Windows のうち該当するもの。制約 C3)のテストは、GitHub Actions の CI で実行する。マージ前に CI を通す。"
    }
  ],
  "obligations": {
    "strategy": "standard",
    "strategy_volume": [
      "Five to eight tests per component.",
      "Unit tests plus integration tests for key boundaries.",
      "Add E2E, performance, or security tests when requirements demand them."
    ],
    "scope_floor": [
      "Meet an 80% line-coverage floor.",
      "Run the selected tests in CI before merge."
    ],
    "combination_rule": "Apply every selected-strategy obligation and every scope-floor obligation; neither replaces the other, and a targeted scope regression may add the narrowest necessary test type beyond the strategy default."
  },
  "plan_profile": {
    "methodology": "test-after",
    "runner_step": "Bootstrap the minimal test runner/configuration and record the exact unit-scoped command.",
    "runner_ready_before_first_test": true,
    "testable_layers": [
      "Data model / database behavior",
      "Repository / data access",
      "Business logic",
      "API / endpoint",
      "Frontend behavior"
    ],
    "steps": [
      "Project structure and production configuration skeleton.",
      "Bootstrap the minimal test runner/configuration and record the exact unit-scoped command.",
      "Data model / database behavior - implement.",
      "Data model / database behavior - write and run its tests after implementation.",
      "Repository / data access - implement.",
      "Repository / data access - write and run its tests after implementation.",
      "Business logic - implement.",
      "Business logic - write and run its tests after implementation.",
      "API / endpoint - implement.",
      "API / endpoint - write and run its tests after implementation.",
      "Frontend behavior - implement.",
      "Frontend behavior - write and run its tests after implementation.",
      "Environment/build configuration.",
      "Documentation and traceability."
    ]
  },
  "input_sha256": "sha256:ce0f69628897190511501dd3c515b27d7bab3d0e15b2a2103f574b66ace49bcb",
  "contract_sha256": "sha256:6bd9ef79a3010e02afd33449a0c5b558cbba318b21e4e7e1f917ad9a9138e14f"
}
```
