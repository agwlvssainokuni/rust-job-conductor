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

//! UNIX 系の `platform`(Linux、macOS)。
//!
//! このクレートで、`unsafe` を許す場所は、このモジュールだけである(NFR4.8)。許すのは、
//! シグナルの送信の `libc::kill` を 1 回呼ぶ 1 か所に限る。

#![allow(unsafe_code)]

use std::fs::{File, OpenOptions};
use std::os::unix::fs::OpenOptionsExt;
use std::os::unix::process::{CommandExt, ExitStatusExt};
use std::path::Path;
use std::process::{Command, ExitStatus};

/// シグナルで終了したとき、そのシグナルの番号(BR1.4)。
pub(crate) fn termination_signal(status: &ExitStatus) -> Option<i32> {
    status.signal()
}

/// ディレクトリを同期する(置き換えの記録を、ディスクに確かめる)。失敗しても、エラーにしない。
pub(crate) fn sync_dir(dir: &Path) {
    let dir = crate::temp_file::dir_or_current(dir);
    let _ = File::open(dir).and_then(|f| f.sync_all());
}

/// 新しく作るファイルの権限を、本人だけが読み書きできる 0600 にする(ログのファイル。v0.10.0 の BR3.4)。
pub(crate) fn restrict_to_owner(options: &mut OpenOptions) {
    options.mode(0o600);
}

/// ジョブを、conductor とは別の、新しいプロセスグループで起動する(BR1.7)。
///
/// 端末の Ctrl+C などは conductor だけに届き、conductor が受け取ったシグナルを、ジョブに 1 回だけ
/// 転送する(二重に届かない)。
pub(crate) fn configure_spawn(command: &mut Command) {
    command.process_group(0);
}

/// 転送してよいシグナル。SIGKILL、SIGSTOP、SIGCONT、その他は転送しない(ジョブを止めたままにしたり、
/// 強制終了したりしないため)。
const FORWARDABLE: [i32; 4] = [libc::SIGHUP, libc::SIGINT, libc::SIGQUIT, libc::SIGTERM];

/// 送ってよいかの判断(純粋)。送ってよければ、`kill` に渡す値を返す。
///
/// - pid が `i32` に収まり、0 より大きいこと(0 や負にならない。プロセスグループには送らない)
/// - シグナルが、転送してよい集合のどれかであること
fn plan_send(pid: u32, signal: i32) -> Option<(libc::pid_t, libc::c_int)> {
    let pid = libc::pid_t::try_from(pid).ok().filter(|p| *p > 0)?;
    FORWARDABLE.contains(&signal).then_some((pid, signal))
}

/// ジョブの子プロセスに、シグナルを 1 回送る。失敗(プロセスが存在しないなど)は無視する。
///
/// 呼び出し側は、その子プロセスを、まだ回収(wait)していないあいだにだけ呼ぶ(pid の再利用で、
/// 無関係のプロセスに届かないようにするため。NFR4.8 の条件 3)。
pub(crate) fn forward_signal(pid: u32, signal: i32) {
    if let Some((pid, signal)) = plan_send(pid, signal) {
        // SAFETY: `kill` は、整数 2 つを受け取るだけで、メモリに触れない。pid は、0 より大きく
        // `pid_t` に収まる値(プロセスグループへの送信にならない)、signal は許可した集合の値に
        // 確かめてある。失敗は戻り値で返り、無視してよい。
        let _ = unsafe { libc::kill(pid, signal) };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn 許可したシグナルと正の_pid_は送る() {
        for s in FORWARDABLE {
            assert_eq!(plan_send(4242, s), Some((4242, s)));
        }
    }

    #[test]
    fn pid_が_0_なら送らない() {
        assert_eq!(plan_send(0, libc::SIGTERM), None);
    }

    #[test]
    fn pid_が_i32_に収まらなければ送らない() {
        assert_eq!(plan_send(i32::MAX as u32 + 1, libc::SIGTERM), None);
        assert_eq!(plan_send(u32::MAX, libc::SIGTERM), None);
        // 負の pid(プロセスグループ)になりうる値は、変換に失敗するため送られない。
        assert_eq!(plan_send(-1i32 as u32, libc::SIGTERM), None);
        assert_eq!(
            plan_send(i32::MAX as u32, libc::SIGTERM).map(|p| p.0),
            Some(i32::MAX)
        );
    }

    #[test]
    fn 許可されないシグナルは送らない() {
        for s in [
            libc::SIGKILL,
            libc::SIGSTOP,
            libc::SIGCONT,
            libc::SIGUSR1,
            0,
            -1,
            9999,
        ] {
            assert_eq!(plan_send(4242, s), None, "signal {s}");
        }
    }

    #[test]
    fn 拒否された送信は何も起こさない() {
        // pid 0 を渡しても、呼び出し側のプロセスグループには、何も届かない(拒否される)。
        forward_signal(0, libc::SIGTERM);
        forward_signal(4242, libc::SIGKILL);
    }

    #[test]
    fn 新しいプロセスグループで起動できる() {
        let mut cmd = Command::new("sleep");
        cmd.arg("0");
        configure_spawn(&mut cmd);
        let status = cmd.status().expect("起動できる");
        assert!(status.success());
    }
}
