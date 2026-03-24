use clap::{Parser, Subcommand};

mod commands;

#[derive(Parser)]
#[command(
    name = "trustver",
    version,
    about = "TrustVer CLI — provenance-aware versioning"
)]
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
    /// PAD operations
    Pad {
        #[command(subcommand)]
        action: PadAction,
    },
    /// Key management
    Key {
        #[command(subcommand)]
        action: KeyAction,
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

#[derive(Subcommand)]
pub(crate) enum PadAction {
    /// Generate a PAD from current project state
    Generate {
        #[arg(long, action = clap::ArgAction::Append)]
        artifact: Vec<String>,
        #[arg(long, default_value = "stable")]
        scope: String,
        #[arg(long)]
        build_system: Option<String>,
        #[arg(long)]
        build_id: Option<String>,
        #[arg(long)]
        reproducible: bool,
        #[arg(long)]
        model: Option<String>,
        #[arg(long, action = clap::ArgAction::Append)]
        reviewer: Vec<String>,
        #[arg(long)]
        contribution_pct: Option<u8>,
        #[arg(long, short)]
        output: Option<String>,
    },
    /// Sign an existing PAD
    Sign {
        pad_file: String,
        #[arg(long)]
        key: Option<String>,
        #[arg(long)]
        public_key: Option<String>,
        #[arg(long)]
        key_id: Option<String>,
        #[arg(long)]
        signer: String,
        #[arg(long)]
        sigstore: bool,
    },
    /// Append an attestation to a PAD
    Attest {
        pad_file: String,
        #[arg(long, name = "type")]
        attestation_type: String,
        #[arg(long)]
        attester: String,
        #[arg(long)]
        detail: Option<String>,
        #[arg(long)]
        detail_file: Option<String>,
        #[arg(long)]
        sign_key: Option<String>,
        #[arg(long)]
        unsigned: bool,
    },
    /// Validate PAD structure and optionally verify signatures
    Validate {
        pad_file: String,
        #[arg(long)]
        verify: bool,
        #[arg(long)]
        public_key: Option<String>,
        #[arg(long)]
        json: bool,
    },
}

#[derive(Subcommand)]
pub(crate) enum KeyAction {
    /// Generate an ECDSA P-256 keypair
    Generate {
        #[arg(long, default_value = ".trustver/keys")]
        output_dir: String,
        #[arg(long, default_value = "trustver")]
        name: String,
    },
}

fn main() {
    let cli = Cli::parse();
    if let Err(e) = commands::run(cli.command) {
        eprintln!("Error: {e:#}");
        std::process::exit(2);
    }
}
