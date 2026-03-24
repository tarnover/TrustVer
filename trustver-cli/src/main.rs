use clap::{Parser, Subcommand};

mod commands;

#[derive(Parser)]
#[command(name = "trustver", version, about = "TrustVer CLI — provenance-aware versioning")]
struct Cli {
    /// Disable colored output
    #[arg(long, global = true)]
    no_color: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new trustver.toml
    Init,
    /// Bump the version
    Bump,
    /// Validate a TrustVer version string
    Validate,
    /// Validate a commit message against TrustVer convention
    CheckCommit,
    /// Show provenance audit for a git range
    Audit,
    /// Manage git hooks
    Hook,
}

fn main() {
    let cli = Cli::parse();
    if let Err(e) = commands::run(cli.command) {
        eprintln!("Error: {e:#}");
        std::process::exit(2);
    }
}
