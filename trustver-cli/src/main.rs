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
pub(crate) enum Commands {
    /// Initialize a new trustver.toml
    Init {
        #[arg(long)]
        name: Option<String>,
        #[arg(long)]
        version: Option<String>,
    },
    /// Bump the version
    Bump {
        level: String,
        #[arg(long)]
        authorship: Option<String>,
        #[arg(long)]
        strict: bool,
        #[arg(long, name = "from")]
        from_ref: Option<String>,
        #[arg(long)]
        tag: bool,
        #[arg(long)]
        json: bool,
    },
    /// Validate a TrustVer version string
    Validate {
        version_string: String,
        #[arg(long)]
        quiet: bool,
        #[arg(long)]
        json: bool,
    },
    /// Validate a commit message against TrustVer convention
    #[command(name = "check-commit")]
    CheckCommit {
        message: Option<String>,
        #[arg(long)]
        file: Option<String>,
        #[arg(long)]
        json: bool,
    },
    /// Show provenance audit for a git range
    Audit {
        range: Option<String>,
        #[arg(long)]
        json: bool,
    },
    /// Manage git hooks
    Hook {
        #[command(subcommand)]
        action: HookAction,
    },
}

#[derive(Subcommand)]
pub(crate) enum HookAction {
    /// Install the commit-msg hook
    Install {
        #[arg(long)]
        force: bool,
    },
}

fn main() {
    let cli = Cli::parse();
    if let Err(e) = commands::run(cli.command) {
        eprintln!("Error: {e:#}");
        std::process::exit(2);
    }
}
