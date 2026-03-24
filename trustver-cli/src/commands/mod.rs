use anyhow::Result;
use crate::{Commands, HookAction, KeyAction, PadAction};

pub mod init;
pub mod bump;
pub mod validate;
pub mod check_commit;
pub mod audit;
pub mod hook;
pub mod pad;
pub mod key;

pub fn run(cmd: Commands) -> Result<()> {
    match cmd {
        Commands::Init { name, version } => init::run(name, version),
        Commands::Validate { version_string, quiet, json } => validate::run(&version_string, quiet, json),
        Commands::CheckCommit { message, file, json } => check_commit::run(message, file, json),
        Commands::Bump { level, authorship, strict, from_ref, tag, json } => {
            bump::run(&level, authorship, strict, from_ref, tag, json)
        }
        Commands::Audit { range, json } => audit::run(range, json),
        Commands::Hook { action } => match action {
            HookAction::Install { force } => hook::install(force),
        },
        Commands::Pad { action } => match action {
            PadAction::Generate {
                artifact,
                scope,
                build_system,
                build_id,
                reproducible,
                model,
                reviewer,
                contribution_pct,
                output,
            } => pad::generate(
                artifact,
                &scope,
                build_system,
                build_id,
                reproducible,
                model,
                reviewer,
                contribution_pct,
                output,
            ),
            PadAction::Sign {
                pad_file,
                key,
                public_key,
                key_id,
                signer,
                sigstore,
            } => pad::sign(&pad_file, key, public_key, key_id, &signer, sigstore),
            PadAction::Attest {
                pad_file,
                attestation_type,
                attester,
                detail,
                detail_file,
                sign_key,
                unsigned,
            } => pad::attest(&pad_file, &attestation_type, &attester, detail, detail_file, sign_key, unsigned),
            PadAction::Validate {
                pad_file,
                verify,
                public_key,
                json,
            } => pad::validate(&pad_file, verify, public_key, json),
        },
        Commands::Key { action } => match action {
            KeyAction::Generate { output_dir, name } => key::generate(&output_dir, &name),
        },
    }
}
