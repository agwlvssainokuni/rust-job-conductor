//! この Unit が足す型(Types)。共有する型(`JobExecution`、`PipelineState` など)は、
//! conductor-core(`conductor-core`)が定める。ここには、入出力と OS に依存しない値の形だけを置く。

use std::fmt;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicI32, Ordering};

/// command の文字列を、コマンド名と引数に分けたもの(BR1.1)。シェルを介さずに起動する。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandLine {
    /// コマンド名(空でない)。
    pub program: String,
    /// 引数。1 つずつ、そのままの文字列で渡される(メタ文字を解釈しない)。
    pub args: Vec<String>,
}

/// 中断の合図。Cli が用意する、読み取り専用の値(契約 C3 の `interruptSignal`)。
///
/// 立ったら下りない(レベルで持つ)。立っているあいだ、最初に受け取ったシグナルの番号を運ぶ。
/// スレッド間で共有できる(`Sync`)。Cli は、シグナルを受け取るたびに [`InterruptSignal::raise`]
/// を呼び、この Unit は [`InterruptSignal::is_raised`] などで読むだけである。
pub struct InterruptSignal {
    /// `NOT_RAISED` なら立っていない。そうでなければ、最初のシグナルの番号。
    state: AtomicI32,
}

const NOT_RAISED: i32 = i32::MIN;

impl InterruptSignal {
    /// 立っていない合図を作る。
    pub const fn new() -> Self {
        Self {
            state: AtomicI32::new(NOT_RAISED),
        }
    }

    /// 合図を立てる。すでに立っていれば、最初の番号を保つ(何もしない)。
    pub fn raise(&self, signal_number: i32) {
        // 番号が番号の予約値と重なったときも、立っていることは変わらない。
        let value = if signal_number == NOT_RAISED {
            NOT_RAISED + 1
        } else {
            signal_number
        };
        let _ = self
            .state
            .compare_exchange(NOT_RAISED, value, Ordering::SeqCst, Ordering::SeqCst);
    }

    /// 合図が立っているか。
    pub fn is_raised(&self) -> bool {
        self.state.load(Ordering::SeqCst) != NOT_RAISED
    }

    /// 立っているとき、最初に受け取ったシグナルの番号。
    pub fn signal_number(&self) -> Option<i32> {
        match self.state.load(Ordering::SeqCst) {
            NOT_RAISED => None,
            n => Some(n),
        }
    }
}

impl Default for InterruptSignal {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for InterruptSignal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("InterruptSignal")
            .field("raised", &self.is_raised())
            .field("signal_number", &self.signal_number())
            .finish()
    }
}

impl PartialEq for InterruptSignal {
    fn eq(&self, other: &Self) -> bool {
        self.signal_number() == other.signal_number()
    }
}

impl Eq for InterruptSignal {}

/// ステートファイルの場所(BR2.1)。定義ファイルと同じディレクトリの `<定義ファイル名>.state`。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StateLocation {
    path: PathBuf,
}

impl StateLocation {
    /// 定義ファイルのパスから、ステートファイルの場所を決める。
    pub fn of(definition_path: &Path) -> Self {
        let mut name = definition_path
            .file_name()
            .map(|n| n.to_os_string())
            .unwrap_or_default();
        name.push(".state");
        let path = match definition_path.parent() {
            Some(dir) => dir.join(name),
            None => PathBuf::from(name),
        };
        Self { path }
    }

    /// ステートファイルのパス。
    pub fn path(&self) -> &Path {
        &self.path
    }
}

/// ステートと定義の食い違いの種類(BR2.5)。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MismatchKind {
    /// パイプライン名の違い。
    PipelineName,
    /// ジョブIDの追加・削除・改名。
    JobIds,
    /// 記述順だけの違い。
    JobOrder,
}

/// ステートと定義の照合の結果(契約 C3 の `check_consistency`)。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConsistencyResult {
    Ok,
    Mismatch(MismatchKind),
}

/// ステートの書き込みの耐久性の指定(BR2.4 の差分)。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Durability {
    /// ファイルの内容を同期してから置き換える(結果・スキップ・中断の書き込み)。
    #[default]
    Durable,
    /// 同期を省く(起動前の初期ステートの書き込み。NFR1.1)。
    Fast,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn 合図は最初は立っていない() {
        let s = InterruptSignal::new();
        assert!(!s.is_raised());
        assert_eq!(s.signal_number(), None);
        assert_eq!(s, InterruptSignal::default());
    }

    #[test]
    fn 合図は立ったら下りず最初の番号を保つ() {
        let s = InterruptSignal::new();
        s.raise(15);
        s.raise(2);
        assert!(s.is_raised());
        assert_eq!(s.signal_number(), Some(15));
        assert_ne!(s, InterruptSignal::new());
    }

    #[test]
    fn 予約値の番号でも立つ() {
        let s = InterruptSignal::new();
        s.raise(i32::MIN);
        assert!(s.is_raised());
        assert!(s.signal_number().is_some());
        let n = InterruptSignal::new();
        n.raise(-1);
        assert_eq!(n.signal_number(), Some(-1));
    }

    #[test]
    fn 合図はスレッド間で共有できる() {
        let s = std::sync::Arc::new(InterruptSignal::new());
        let t = {
            let s = s.clone();
            std::thread::spawn(move || s.raise(1))
        };
        t.join().expect("スレッドが終わる");
        assert_eq!(s.signal_number(), Some(1));
        assert!(format!("{s:?}").contains("raised: true"));
    }

    #[test]
    fn ステートの場所は定義ファイルと同じディレクトリ() {
        let loc = StateLocation::of(Path::new("dir/conductor.toml"));
        assert_eq!(loc.path(), Path::new("dir/conductor.toml.state"));
        let rel = StateLocation::of(Path::new("conductor.toml"));
        assert_eq!(rel.path(), Path::new("conductor.toml.state"));
    }

    #[test]
    fn ファイル名の無いパスでもパニックしない() {
        let loc = StateLocation::of(Path::new("/"));
        assert_eq!(
            loc.path().file_name().and_then(|n| n.to_str()),
            Some(".state")
        );
        let empty = StateLocation::of(Path::new(""));
        assert_eq!(empty.path(), Path::new(".state"));
    }

    #[test]
    fn 耐久性の既定は_durable() {
        assert_eq!(Durability::default(), Durability::Durable);
    }
}
