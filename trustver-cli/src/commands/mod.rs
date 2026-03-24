use anyhow::Result;
use crate::{Commands, HookAction};

pub mod init;
pub mod bump;
pub mod validate;
pub mod check_commit;
pub mod audit;
pub mod hook;

pub fn run(cmd: Commands) -> Result<()> {
    match cmd {
        Commands::Init { name, version } => init::run(name, version),
        Commands::Validate { version_string, quiet, json } => validate::run(&version_string, quiet, json),
        Commands::CheckCommit { message, file, json } => check_commit::run(message, file, json),
        Commands::Bump { .. } => todo!("bump"),
        Commands::Audit { .. } => todo!("audit"),
        Commands::Hook { action } => match action {
            HookAction::Install { .. } => todo!("hook install"),
        },
    }
}
