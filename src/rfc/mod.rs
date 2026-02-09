//! RFC command execution pipeline.
//!
//! - `rfc init`: create RFC directory and install `create-rfc` skill scaffold.
//! - `rfc new`: render a new RFC markdown file from the resolved template.
//! - `rfc revise`: update an existing RFC in place and append a revision entry.
pub(crate) mod create;
pub(crate) mod init;
mod lookup;
mod reference;
pub(crate) mod revise;
mod template;
mod util;
