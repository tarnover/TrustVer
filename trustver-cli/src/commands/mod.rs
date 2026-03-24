use anyhow::Result;
use crate::Commands;

pub mod init;
pub mod bump;
pub mod validate;
pub mod check_commit;
pub mod audit;
pub mod hook;

pub fn run(cmd: Commands) -> Result<()> {
    match cmd {
        Commands::Init => todo!("init"),
        Commands::Bump => todo!("bump"),
        Commands::Validate => todo!("validate"),
        Commands::CheckCommit => todo!("check-commit"),
        Commands::Audit => todo!("audit"),
        Commands::Hook => todo!("hook"),
    }
}
