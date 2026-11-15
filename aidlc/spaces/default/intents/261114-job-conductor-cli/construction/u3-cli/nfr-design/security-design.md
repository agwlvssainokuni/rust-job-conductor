# Security Design(u3-cli)

上流: security-requirements(NFR4.1〜NFR4.8、既存の成果物との差分)、tech-stack-decisions(T1〜T6)、functional-spec(W1〜W6)、rules(BR1.1〜BR7.3)、contract-summary(E1)。`conductor` は、認証・認可・暗号化・ネットワークを持たない。入力は、引数、環境変数、他の Unit が読むファイル、シグナルである。コードは、短い疑似コード(15行以下)に限る。

**この設計は、functional-design と nfr-requirements の関連する記述より優先する**(下の「既存の成果物との差分」を参照)。

## 設計の要点

### 1. 引数の解釈(ArgParser。NFR4.1)

1. `lexopt` で、引数を、字句(オプションの名前、値、位置引数)に分ける。シェルの展開や解釈はしない(引数は、すでに、OS から、文字列の並びとして届く)。
1a. **文法の細目:** オプションの値は、`--config path` と `--config=path` の両方を受け付ける(`--to` も同じ)。`--` は、オプションの終わりを示し、以降を、すべて位置引数とする(先頭が `-` のジョブIDは、`--` のあとなら使える)。短いオプションのまとめ書き(`-vv` など)は、受け付けない(`-v` は、ちょうど1回。2回以上、または `-vv` は、`UsageError(invalidValue)`。`-v` は1段階)。値のないオプション(`--config` で値が続かない)は、`UsageError(missingValue)`。`--help`・`-h`・`--version`・`-V` は、`--` より前に現れたときだけ、最初に処理する。
2. 最初の位置引数を、コマンド名として、`run`、`pipeline`、`watch`、`reset`、`convert` のどれかに対応づける。コマンドごとに、受け付けるオプションと、位置引数の数を、表で持ち、表に無いものは、`UsageError` にする(`-v` は run・pipeline だけ、`--config` は run・pipeline・watch・reset だけ、`--to` は convert だけで必須)。`--help`・`-h`・`--version`・`-V` は、先に処理する(BR1.6)。
3. 引数が文字列として読めない(UNIX 系で、UTF-8 でないバイト列)ときは、`UsageError(invalidValue)`。パスの値(`--config`、convert の入力)は、`OsString` のまま、保つ(UTF-8 でないパスでも、使える)。
4. 使い方の誤りは、`UsageError` の種類と、固定の文で、標準エラーに出し、終了コード 68。ジョブは実行しない。

### 2. 表示の安全(Reporter。NFR4.2)

- 入力に由来する文字列は、`escape_display(s, limit)` を通して、表示する。制御文字(U+0000〜U+001F、U+007F〜U+009F)、書式・双方向の制御文字(U+200B〜U+200F、U+2028、U+2029、U+202A〜U+202E、U+2060〜U+2064、U+2066〜U+2069、U+FEFF)を、`\u{..}` の形にして、64 文字で切る(パスは、切らず、エスケープだけ)。
- `escape_display` は、この Unit の中の、小さな関数で、u1-core・u4-converter にある、同じ規則の関数(security-requirements の NFR4.6)と、同じ規則で書く。同じ入力と期待値の表を、各 Unit のテストに置いて、揃えを確かめる(**同じ規則の関数が、4つの Unit に重複している。共通のクレートへまとめることを、将来の検討事項として、code-summary に記録する**)。
- ジョブ本体の出力は、そのまま通し、エスケープしない。

### 3. シグナル(SignalWatcher。NFR4.3)

```
fn start_signal_watcher(sig: Arc<InterruptSignal>) {
  // UNIX: let mut signals = Signals::new([SIGHUP, SIGINT, SIGQUIT, SIGTERM])?;
  //       thread::spawn(move || for n in signals.forever() { sig.raise(n); })
  // Windows: ctrlc::set_handler(move || sig.raise(2))
}
```

- 専用のスレッドで受け取り、最初の番号だけを、合図(u2-runtime の `InterruptSignal`。立ったら下りず、最初の番号を保つ)に書く。OS が直接呼ぶ部分(ハンドラ)は、ライブラリの内部で、この Unit は、`unsafe` を使わない(`#![forbid(unsafe_code)]`。`low_level` は使わない)。
- 準備に失敗したとき(ライブラリが、ハンドラを登録できない)は、`conductor` 自身のエラー(内部の誤り。終了コード 70。固定の文「シグナルの準備に失敗した」)として、ジョブを実行せずに終わる。
- 合図は、定義の検査のあいだに立っても、検査と判断は、最後まで進める。判断が Reject・Skip・各種のエラーなら、その終了コードが優先し、Run なら、`run_job` が、起動前の合図を見て、起動しない(BR4.2a)。

### 4. 出力の安全(NFR4.4、NFR4.7)

- **convert:** 変換の結果の文字列を、全部作ってから、標準出力に、1回で書く。失敗のとき、標準出力には、何も書かない。標準出力への書き込みの失敗(閉じたパイプなど)は、パニックにせず、終了コード 67(convert)・65(watch)で終わる。Rust の実行環境は、SIGPIPE を無視する設定で動くため、書き込みの失敗は、エラーの値として返る。
- **標準エラーへの書き込みの失敗:** 表示の失敗は、無視する(表示できなくても、終了コードは、本来の値で返す)。
- **パニック:** パニックのフックを、`main` の最初に設定する。フックは、固定の文(英語・日本語の判定は、行わず、英語の固定の文)を標準エラーに書き、終了コード 70 で、プロセスを終了する(終了の呼び出しは、テストで差し替えられる作り)。リリースの実行ファイルに、テスト専用の経路(パニックを起こすコマンド)を持たせない。

### 5. ファイルとパス(NFR4.5、NFR4.6)

- ネットワークのコードを持たない(依存にも、ネットワークのライブラリがない)。読み書きするファイルは、定義ファイル(u1-core が読む)、ステートファイル(u2-runtime が読み書きする)、convert の入力(u4-converter が読む)だけ。
- パスは、`OsString`/`PathBuf` のまま、u1-core・u2-runtime・u4-converter に渡す(この Unit で、文字列に変換・展開しない)。

### 6. 終了コード(ExitMapper。NFR2.2)

- 終了コードの決定は、純粋な関数(`exit_code_for(outcome_or_error) -> u8`)に分ける。`reportedExitCode`(`i32`)は、BR5.2 で、0〜255 に合わせる(範囲外は、下位 8 ビット。エラーで 0 になるなら 1)。conductor 自身のエラーは、64〜68、70 の定数。
- 終了は、`main.rs` の最後で、1回だけ、`std::process::exit` を呼ぶ。

## リリースの完全性(NFR4.8。この Unit の設計の範囲)

- NFR4.8(タグを付けたときの、3つの OS 向けのビルドと、SHA-256 のチェックサムの公開)は、**この Unit のコードの設計の対象外**で、CI Pipeline と Deployment Pipeline のステージで、設計・実装する。この Unit に求める設計上の取り決めは、(1) 依存を `Cargo.lock` で固定し、`--locked` でビルドできること、(2) 実行ファイルの名前が `conductor` であること、(3) テスト専用の補助の実行ファイルが、配布のビルドに含まれないこと(`-p conductor-cli` のビルド)の3つで、tech-stack-decisions と cicd-pipeline に記録してある。

## 他の NFR の設計(NFR2.1、NFR3.1、NFR5.1、NFR6.1〜NFR6.4)

- **NFR2.1(単一バイナリ):** `lib.rs` と `main.rs` を、1つのパッケージ(`conductor-cli`)にし、u1-core・u2-runtime・u4-converter を、パス依存で静的に組み込む。C のコンパイルを要しない依存だけを使う。
- **NFR3.1(OS 固有のコード):** この Unit では、シグナルの監視(`cfg(unix)`/`cfg(windows)`)と、端末の判定のモジュールの中だけ。
- **NFR5.1(テスト):** Q1: A により、全部の処理が `lib.rs` にあり、同じプロセスのテストで、行カバレッジ 80% 以上(Linux)を測れる。バイナリを起動するテストは、終了コードと、標準出力・標準エラーの分け方を確かめる少数に絞る。Q2: A により、流れの失敗の経路(ジョブのあとの書き込みの失敗、初期化の失敗、シグナル、閉じたパイプ)を、偽の実装で確かめる。
- **NFR6.1〜NFR6.4(表示):** observability-design に設計した(状態の文字表示、`-v` の1段階、言語の切り替え、診断のしやすさ)。

## 既存の成果物との差分(この設計を優先する)

| 対象 | 変更の内容 |
|------|------------|
| functional-design の rules(BR7.1)と契約 E1 | `watch` の標準出力への書き込みの失敗(閉じたパイプなど)は、終了コード 65 とする(「ステートの誤り」の 65 と同じ値だが、ステートの誤りではない。表示は、標準エラーに、固定の文で出す)。E1 の、`watch` の終了コードに足す |
| functional-design の rules(BR2.4) | `apply_skip` が、ステートを変えなかったとき(`alreadyExecuted`)は、`write_state` を呼ばない(変更のない `Durable` の書き込みを避ける) |
| nfr-requirements の NFR4.7 | パニックのフックは、`main` の最初に設定し、終了の呼び出しを差し替えられる作りにする。フックの表示は、英語の固定の文 |
| functional-design の rules(BR6.1) | パニックのフックと、シグナルの準備の失敗(70)の表示は、ロケールを判定せず、英語の固定の文にする(ロケールの判定は、通常の表示のときだけ) |

## 設計の判断(代替案との比較)

### D1 `escape_display` を、この Unit に持つ(4つ目の複製)

- **Context:** ジョブID、パス、キー名などを、安全に表示する必要がある。同じ規則の関数が、他の Unit にもある。
- **Decision:** この Unit にも、同じ規則の小さな関数を持ち、同じ入力と期待値の表で、揃えを確かめる。
- **Consequences:** 良い点は、Unit の独立と、起動の速さ(追加の依存なし)。悪い点は、同じ規則が4か所にあり、規則を変えるとき、すべてを直す必要がある。
- **Alternatives Rejected:** (1) u1-core の関数を、公開して使う案: 重複は減るが、u1-core の表示用の関数を、公開の API にし、u2-runtime・u4-converter との揃えは、別に必要なため、いまは退けた。(2) 共通のクレートを作る案: 重複は無くなるが、4つの Unit の変更を伴い、いまの完了済みの Unit に手を入れるため、将来の検討とする。

### D2 シグナルは、専用のスレッドで受け、最初の番号だけを書く

- **Context:** 「最初のシグナルの番号が、合図に残る」(u2-runtime の `InterruptSignal`)と、この Unit に `unsafe` を置かないことを、両立させる。
- **Decision:** `signal-hook` の `Signals`(イテレータ)を、専用のスレッドで回し、`ctrlc`(Windows)のハンドラも、合図の `raise` を呼ぶだけにする。
- **Consequences:** 良い点は、`unsafe` が要らず、番号が分かる。悪い点は、スレッドが1つ増える(約 100 マイクロ秒の見込み)。
- **Alternatives Rejected:** `signal-hook` の `flag::register`(番号ごとのフラグ): スレッドは要らないが、到着順が分からず、「最初の番号」を実現できない。`low_level::register`: 最小の遅れだが、`unsafe` が要る。

### D3 パニックは、70 で終わる(ジョブの実行中も)

- **Context:** 内部の誤りを、見分けられるようにしたい。
- **Decision:** フックで、固定の文を出し、70 で終了する。ジョブの実行中のパニックは、子プロセスを待たずに終わり、孤児が残りうる(at-least-once の範囲)。
- **Consequences:** 良い点は、原因が見える。悪い点は、孤児のジョブが残りうる。
- **Alternatives Rejected:** パニックの既定(101): 実装は不要だが、文言が Rust の既定で固定でなく、利用者に見せたい形(固定の英語の文)にならない。パニック時に、子プロセスを終了させる案: 子の pid を、パニックのフックから知る仕組みが要り、複雑で、この版では行わない。

## Assumptions & Open Questions

- `InterruptSignal`(`raise` と「最初の番号を保つ」)は、u2-runtime の entities.md が定義し、u2-runtime が所有する型である。この Unit は、`raise` を呼ぶだけ。 [assumption]
- `signal-hook` の `Signals` が、最小の Rust のバージョン(1.85)で使えること、`ctrlc` が Windows で Ctrl+C を受け取ることは、Code Generation の最初に確かめる(止めて人に確認する手順は、tech-stack-decisions の通り)。 [assumption]
- `escape_display` の4か所の複製の整理(共通のクレート)は、この版の範囲外で、将来の検討とする。 [assumption]
- 認証・認可・暗号化は、この Unit に該当しない。 [assumption]
