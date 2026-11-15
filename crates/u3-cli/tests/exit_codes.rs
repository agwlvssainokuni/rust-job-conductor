//! 終了コードの表(BR5.1〜BR5.3、NFR2.2)。純粋な関数の全項目。

use conductor_cli::exit::*;

#[test]
fn conductor_自身のエラーの定数() {
    assert_eq!(
        [
            EXIT_OK,
            EXIT_DEFINITION,
            EXIT_STATE,
            EXIT_REJECTED,
            EXIT_CONVERSION,
            EXIT_USAGE,
            EXIT_INTERNAL
        ],
        [0, 64, 65, 66, 67, 68, 70]
    );
}

#[test]
fn 範囲内の値はそのまま() {
    for code in [0, 1, 2, 63, 64, 127, 128, 130, 143, 255] {
        assert_eq!(exit_code_for(code, true), code);
        assert_eq!(exit_code_for(code, false), code);
    }
}

#[test]
fn 範囲外の値は下位_8_ビット() {
    assert_eq!(exit_code_for(256 + 3, true), 3);
    assert_eq!(exit_code_for(511, true), 255);
    assert_eq!(exit_code_for(-1, true), 255);
    assert_eq!(exit_code_for(-2, true), 254);
    // Windows の例外による終了(例: 0xC0000005 を i32 にした値)
    assert_eq!(exit_code_for(0xC000_0005u32 as i32, true), 5);
}

#[test]
fn エラーで_0_になる場合は_1() {
    assert_eq!(exit_code_for(256, true), 1);
    assert_eq!(exit_code_for(512, true), 1);
    assert_eq!(exit_code_for(i32::MIN, true), 1);
    assert_eq!(exit_code_for(-256, true), 1);
}

#[test]
fn エラーでなければ_0_のまま() {
    assert_eq!(exit_code_for(256, false), 0);
    assert_eq!(exit_code_for(0, false), 0);
}

#[test]
fn 中断の終了コードは_128_足す番号() {
    assert_eq!(signal_exit_code(15), 143);
    assert_eq!(signal_exit_code(2), 130);
    assert_eq!(signal_exit_code(1), 129);
    assert_eq!(signal_exit_code(3), 131);
}

#[test]
fn 中断の終了コードは範囲を超えない() {
    assert_eq!(signal_exit_code(i32::MAX), exit_code_for(i32::MAX, true));
    let code = signal_exit_code(300);
    assert!((0..=255).contains(&code));
    assert!((0..=255).contains(&signal_exit_code(-500)));
}
