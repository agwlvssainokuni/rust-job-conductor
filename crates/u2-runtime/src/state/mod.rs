//! ステート(StateStore)。ファイルに触れるのは、`reader`、`writer` だけで、
//! `consistency` と `record` は、入出力のない純粋な処理である。

pub(crate) mod consistency;
pub(crate) mod reader;
pub(crate) mod record;
pub(crate) mod writer;
