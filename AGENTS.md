---
name: trustver
title: TrustVer
description: Provenance-aware versioning specification and CLI for AI-era software — EffVer effort semantics, authorship tags, commit convention, and signed Provenance Attestation Documents (PADs)
version: 0.3.0
license: CC-BY-SA-4.0 (spec), MIT (tooling)
repository: https://github.com/tarnover/TrustVer
---

# TrustVer

## Project

| Field | Value |
|---|---|
| Name | TrustVer |
| Type | Versioning specification + CLI tool |
| Language | Rust |
| Version | 0.3.0 (spec), 0.1.0 (CLI) |
| License | CC-BY-SA 4.0 (spec), MIT (tooling) |
| Repository | https://github.com/tarnover/TrustVer |
| Website | https://trustver.org |
| Author | Tarnover, LLC / ThirdKey AI |

## Capabilities

- Parse, validate, and bump provenance-aware version strings (`MACRO.MESO.MICRO+AUTHORSHIP`)
- Validate commit messages against the TrustVer commit convention (Conventional Commits + authorship tags)
- Auto-derive release authorship from commit history using weighted threshold rules
- Generate Provenance Attestation Documents (PADs) from git and CI context
- Sign PADs with ECDSA P-256 keys (via SchemaPin) or Sigstore/cosign
- Append signed attestations to PADs (test results, audits, code reviews)
- Validate PAD structure and verify cryptographic signatures
- Generate ECDSA P-256 signing keypairs
- Install git hooks for commit convention enforcement
- Audit provenance across git commit ranges

## Available Tools

| Command | Description |
|---|---|
| `trustver init` | Initialize project with `trustver.toml` |
| `trustver validate <version>` | Validate a TrustVer version string |
| `trustver check-commit [msg]` | Validate commit message against convention |
| `trustver bump <level>` | Bump version (macro/meso/micro) with auto-derived authorship |
| `trustver audit [range]` | Provenance summary for a git range |
| `trustver hook install` | Install commit-msg git hook |
| `trustver key generate` | Generate ECDSA P-256 keypair |
| `trustver pad generate` | Create PAD from project state |
| `trustver pad sign <file>` | Sign PAD with local key or cosign |
| `trustver pad attest <file>` | Append attestation to PAD |
| `trustver pad validate <file>` | Validate PAD structure and signatures |

## Installation

```bash
# From source
cargo install --git https://github.com/tarnover/TrustVer trustver-cli

# Pre-built binaries
# See https://github.com/tarnover/TrustVer/releases
```

## Authorship Tags

| Tag | Meaning |
|---|---|
| `h` | Human-authored |
| `ai` | AI-generated (no human review) |
| `hrai` | Human-reviewed AI |
| `aih` | AI-assisted human |
| `auto` | Autonomous agent |
| `mix` | Mixed/indeterminate |

## Integration Points

| System | Integration |
|---|---|
| [SchemaPin](https://schemapin.org) | ECDSA P-256 signing primitives and `.well-known` key discovery for PAD signatures |
| [AgentPin](https://agentpin.org) | Verifiable agent identity for `auto` authorship releases |
| [ToolClad](https://toolclad.org) | Secure tool contracts for agentic build systems |
| [Symbiont](https://symbiont.dev) | Runtime policy enforcement against PAD attestations |
| [Sigstore/cosign](https://docs.sigstore.dev/) | Keyless CI signing via `--sigstore` flag |
| Git | Commit convention, hooks, tag-based version tracking |
| GitHub Actions | Auto-detected CI context in PAD generation |
| GitLab CI | Auto-detected CI context in PAD generation |

## Security Model

- **Version string authorship is a convenience signal, not a security boundary** — the signed PAD is the evidence
- **PAD signatures use ECDSA P-256** via SchemaPin, with key discovery via `.well-known/schemapin.json`
- **Attestations are independently signed** — each attester signs their own attestation
- **Canonical JSON** ensures signature stability regardless of JSON formatting
- **Missing PADs should be treated as lowest trust posture** by consuming tools

## Specification

The full TrustVer specification is at [TRUSTVER_SPEC.md](https://github.com/tarnover/TrustVer/blob/main/TRUSTVER_SPEC.md) covering:

- Version string format (§1-3)
- Commit convention (§4)
- Provenance Attestation Documents (§5-8)
- Trust infrastructure integration (§9)
- Adoption guide (§11)
- Security considerations (§12)
