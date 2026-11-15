//! 終了コードの決定(ExitMapper。BR5.1〜BR5.3、NFR2.2)。純粋な関数と定数だけ。
//!
//! 終了コードは、どの OS でも、0〜255 の範囲に合わせる(BR5.2)。

/// 成功、警告、スキップ。
pub const EXIT_OK: i32 = 0;
/// 定義の誤り(ファイルがない・読めない・検証の誤り・command の誤り)。
pub const EXIT_DEFINITION: i32 = 64;
/// ステートの誤り(読めない・壊れている・食い違い・書き込み・初期化の失敗)と、`watch` の出力の失敗。
pub const EXIT_STATE: i32 = 65;
/// 拒否(順序外)。
pub const EXIT_REJECTED: i32 = 66;
/// 変換の誤りと、`convert` の出力の失敗。
pub const EXIT_CONVERSION: i32 = 67;
/// 使い方の誤り。
pub const EXIT_USAGE: i32 = 68;
/// 内部の誤り(パニック、シグナルの準備の失敗)。
pub const EXIT_INTERNAL: i32 = 70;

/// ジョブの結果の `reported_exit_code` を、プロセスの終了コード(0〜255)にする(BR5.2)。
///
/// 範囲内の値は、そのまま。範囲外(255 を超える、または負)は、下位 8 ビット。ただし、
/// エラーのジョブで、下位 8 ビットが 0 になる場合は 1(エラーを成功に見せない)。
pub fn exit_code_for(reported: i32, is_error: bool) -> i32 {
    if (0..=255).contains(&reported) {
        return reported;
    }
    let low = reported & 0xFF;
    if low == 0 && is_error { 1 } else { low }
}

/// 中断(シグナル)の終了コード: 128 + 番号。0〜255 に合わせる。
pub fn signal_exit_code(signal_number: i32) -> i32 {
    exit_code_for(128i32.saturating_add(signal_number), true)
}
