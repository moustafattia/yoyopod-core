//! `yoyopod target validate` — Round 1 stub.
//!
//! The real implementation depends on on-Pi validation stages (`pi
//! validate smoke`, etc.) being available. Those are deleted as of Round
//! 0 and return in Round 2. Until then this command prints a clear
//! "blocked" message and exits non-zero so automated callers fail loud.

use anyhow::Result;

use crate::cli::ValidateArgs;

use super::TargetContext;

pub fn run(_ctx: &TargetContext, _args: ValidateArgs) -> Result<i32> {
    eprintln!(
        "target validate: blocked on Round 2 of the CLI rebuild.\n\
         See docs/operations/CLI_REBUILD_ROUNDS.md.\n\
         Until then, validate manually after `yoyopod target deploy`:\n\
           journalctl -u yoyopod-dev.service -f\n\
           systemctl status yoyopod-dev.service\n\
         and inspect the device behaviour directly."
    );
    Ok(2)
}
