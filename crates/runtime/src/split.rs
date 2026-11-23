// Copyright 2026 agwlvssainokuni
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! command の分け方(Split。BR1.1)。
//!
//! 3 つの OS で同じ規則で、文字を 1 つずつ読む状態機械(再帰なし)で実装する。
//! 変数・`*`・`|`・`;`・`&&`・`>` などは解釈しない(普通の文字として引数に入る)。

use conductor_core::Command;

use crate::error::{CommandError, CommandErrorKind};
use crate::types::CommandLine;

#[derive(Clone, Copy)]
enum State {
    /// 引用符の外。
    Plain,
    /// `'...'` の中。すべてそのまま。
    Single,
    /// `"..."` の中。
    Double,
    /// `"..."` の中で、バックスラッシュを 1 つ読んだ直後(次の文字で意味が決まる)。
    DoubleBackslash,
}

fn is_separator(c: char) -> bool {
    matches!(c, ' ' | '\t' | '\n' | '\r')
}

/// command の文字列を、コマンド名と引数に分ける。
///
/// - 空白(スペース・タブ・改行)で区切る。
/// - `'...'` の中は、すべてそのまま。
/// - `"..."` の中は、左から順に読み、バックスラッシュの次が二重引用符のときだけ、その 2 文字を
///   二重引用符 1 つにする。ほかのバックスラッシュは、そのままの文字。
/// - 引用符の外のバックスラッシュも、そのままの文字。
/// - 引用符で囲んだ部分と囲んでいない部分は、つなげて 1 つの引数にする。`''` と `""` は空の引数。
///
/// 閉じていない引用符は [`CommandErrorKind::UnclosedQuote`]、コマンド名がなければ
/// [`CommandErrorKind::Empty`]。
pub(crate) fn split(command: &str) -> Result<CommandLine, CommandError> {
    let mut words: Vec<String> = Vec::new();
    let mut current = String::new();
    let mut in_word = false;
    let mut state = State::Plain;

    for c in command.chars() {
        state = match state {
            State::Plain => {
                if is_separator(c) {
                    if in_word {
                        words.push(std::mem::take(&mut current));
                        in_word = false;
                    }
                    State::Plain
                } else if c == '\'' {
                    in_word = true;
                    State::Single
                } else if c == '"' {
                    in_word = true;
                    State::Double
                } else {
                    in_word = true;
                    current.push(c);
                    State::Plain
                }
            }
            State::Single => {
                if c == '\'' {
                    State::Plain
                } else {
                    current.push(c);
                    State::Single
                }
            }
            State::Double => match c {
                '"' => State::Plain,
                '\\' => State::DoubleBackslash,
                _ => {
                    current.push(c);
                    State::Double
                }
            },
            State::DoubleBackslash => match c {
                '"' => {
                    current.push('"');
                    State::Double
                }
                // 保留していたバックスラッシュはそのままの文字。今読んだ文字が次の判断の元になる。
                '\\' => {
                    current.push('\\');
                    State::DoubleBackslash
                }
                _ => {
                    current.push('\\');
                    current.push(c);
                    State::Double
                }
            },
        };
    }

    if !matches!(state, State::Plain) {
        return Err(CommandError::new(CommandErrorKind::UnclosedQuote));
    }
    if in_word {
        words.push(current);
    }

    let mut iter = words.into_iter();
    match iter.next() {
        Some(program) if !program.is_empty() => Ok(CommandLine {
            program,
            args: iter.collect(),
        }),
        _ => Err(CommandError::new(CommandErrorKind::Empty)),
    }
}

/// command を、起動せずに検査し、プログラムと引数の列にする(契約 C3 の `check_command`。BR1.1、NFR4.2)。
///
/// - [`Command::Line`] は、[`split`] で分ける。分けられるなら `Ok`(分けた結果を返す)、閉じていない
///   引用符や空の command は [`CommandError`]。
/// - [`Command::Argv`] は、要素をそのまま引数の列にする(分割も展開もしない)。先頭の要素が空の
///   とき(定義の検査で防ぐが、二重の検査として)と、要素がないときは [`CommandErrorKind::Empty`]。
///
/// 呼び出し側(Cli)は、実行の前に全ジョブの command を検査して、定義の誤りとして報告する。
pub fn check_command(command: &Command) -> Result<CommandLine, CommandError> {
    match command {
        Command::Line(line) => split(line),
        Command::Argv(argv) => {
            let mut iter = argv.iter();
            match iter.next() {
                Some(program) if !program.trim().is_empty() => Ok(CommandLine {
                    program: program.clone(),
                    args: iter.cloned().collect(),
                }),
                _ => Err(CommandError::new(CommandErrorKind::Empty)),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ok(input: &str) -> Vec<String> {
        let line = split(input).expect("分けられる");
        let mut v = vec![line.program];
        v.extend(line.args);
        v
    }

    fn err(input: &str) -> CommandErrorKind {
        split(input).expect_err("分けられない").kind
    }

    #[test]
    fn 空白で区切る() {
        assert_eq!(ok("echo a b"), ["echo", "a", "b"]);
        assert_eq!(ok("  echo \t a \n b  "), ["echo", "a", "b"]);
        assert_eq!(ok("echo\r\na"), ["echo", "a"]);
    }

    #[test]
    fn 一重引用符の中はそのまま() {
        assert_eq!(
            ok(r#"echo 'a b' 'c\d' '"x"'"#),
            ["echo", "a b", r"c\d", "\"x\""]
        );
    }

    #[test]
    fn 二重引用符の中の引用符のエスケープだけ解釈する() {
        assert_eq!(ok(r#"echo "a \" b""#), ["echo", "a \" b"]);
        assert_eq!(ok(r#"echo "a\nb" "c\\d""#), ["echo", r"a\nb", r"c\\d"]);
    }

    #[test]
    fn バックスラッシュ2つと二重引用符は左から読む() {
        // 最初のバックスラッシュは次がバックスラッシュなのでそのまま。2つ目と引用符で引用符1つ。
        assert_eq!(ok(r#"echo "\\"x""#), ["echo", r#"\"x"#]);
        // 「バックスラッシュ2つ + 二重引用符」は引用符のエスケープなので、引用符は閉じない。
        assert_eq!(err(r#"echo "a\\" b"#), CommandErrorKind::UnclosedQuote);
    }

    #[test]
    fn 引用符の外のバックスラッシュはそのまま() {
        assert_eq!(ok(r"C:\dir\x.exe a\ b"), [r"C:\dir\x.exe", r"a\", "b"]);
        assert_eq!(ok(r"\\server\share"), [r"\\server\share"]);
    }

    #[test]
    fn 末尾がバックスラッシュのパスは一重引用符で書ける() {
        assert_eq!(ok(r"prog 'C:\dir\'"), ["prog", r"C:\dir\"]);
        assert_eq!(err(r#"prog "C:\dir\""#), CommandErrorKind::UnclosedQuote);
    }

    #[test]
    fn 引用符の部分と外の部分はつなげて1つの引数() {
        assert_eq!(ok("echo --msg='a b'"), ["echo", "--msg=a b"]);
        assert_eq!(ok(r#"echo a"b c"'d e'f"#), ["echo", "ab cd ef"]);
    }

    #[test]
    fn 空の引数() {
        assert_eq!(ok("echo '' \"\" x"), ["echo", "", "", "x"]);
        assert_eq!(ok("echo ''"), ["echo", ""]);
    }

    #[test]
    fn メタ文字は普通の文字() {
        assert_eq!(
            ok("echo a;b && c | d > e $HOME * `x` (y)"),
            [
                "echo", "a;b", "&&", "c", "|", "d", ">", "e", "$HOME", "*", "`x`", "(y)"
            ]
        );
    }

    #[test]
    fn 閉じていない引用符() {
        assert_eq!(err("echo 'a"), CommandErrorKind::UnclosedQuote);
        assert_eq!(err("echo \"a"), CommandErrorKind::UnclosedQuote);
        assert_eq!(err("echo \"a\\"), CommandErrorKind::UnclosedQuote);
    }

    #[test]
    fn コマンド名がない() {
        assert_eq!(err(""), CommandErrorKind::Empty);
        assert_eq!(err(" \t\n"), CommandErrorKind::Empty);
        assert_eq!(err("''"), CommandErrorKind::Empty);
    }

    #[test]
    fn マルチバイトの文字() {
        assert_eq!(ok("echo 'あ い' う"), ["echo", "あ い", "う"]);
    }

    #[test]
    fn check_command_は分けた結果を返す() {
        let line = check_command(&"a 'b c'".into()).expect("検査を通る");
        assert_eq!(line.program, "a");
        assert_eq!(line.args, ["b c"]);
        assert!(check_command(&"a 'b".into()).is_err());
    }

    #[test]
    fn 配列の_command_は要素をそのまま引数にする() {
        let argv = Command::Argv(vec![
            "my prog".into(),
            "a b".into(),
            "'q'".into(),
            "$HOME".into(),
            String::new(),
        ]);
        let line = check_command(&argv).expect("検査を通る");
        assert_eq!(line.program, "my prog");
        assert_eq!(line.args, ["a b", "'q'", "$HOME", ""]);
    }

    #[test]
    fn 配列の先頭が空または要素がなければ_empty() {
        for argv in [vec![], vec![String::new()], vec!["  ".into(), "x".into()]] {
            let e = check_command(&Command::Argv(argv)).expect_err("空");
            assert_eq!(e.kind, CommandErrorKind::Empty);
        }
    }

    #[test]
    fn エラーの表示に入力の値が入らない() {
        let e = split("secret-token 'unclosed").expect_err("閉じていない");
        assert!(!e.to_string().contains("secret-token"));
        assert!(!e.to_string().contains("unclosed"));
    }
}
