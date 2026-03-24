---
name: trustver
title: TrustVer
description: Provenance-aware versioning — EffVer effort semantics + authorship tags + signed Provenance Attestation Documents (PADs) for AI-era software
version: 0.3.0
---

# TrustVer Development Skills Guide

**Purpose**: This guide helps AI assistants integrate TrustVer versioning, commit conventions, and PAD operations into software projects.

**For Full Documentation**: See the [Specification](https://github.com/tarnover/TrustVer/blob/main/TRUSTVER_SPEC.md) and [README](https://github.com/tarnover/TrustVer/blob/main/README.md).

## What TrustVer Does

TrustVer answers three questions about every software release:

1. **How much effort does this update require?** — EffVer semantics (Macro/Meso/Micro)
2. **Who or what wrote it?** — Authorship tag in the version string
3. **What verification was applied?** — Signed Provenance Attestation Document (PAD)

```
2.4.0+hrai
│ │ │  └── authorship: human-reviewed AI
│ │ └──── micro: no effort expected
│ └────── meso: some effort required
└──────── macro: significant effort
```

---

## Quick Start

### Install

```bash
cargo install --git https://github.com/tarnover/TrustVer trustver-cli
```

### Initialize a Project

```bash
trustver init --name mylib
```

Creates `trustver.toml`:

```toml
package_name = "mylib"
current_version = "0.1.0+mix"
strict = false
```

### Version String Format

```
MACRO.MESO.MICRO+AUTHORSHIP
```

Valid authorship tags:

| Tag | Meaning | Who to blame for bugs? |
|---|---|---|
| `h` | Human-authored | The human |
| `ai` | AI-generated, no review | Nobody reviewed it |
| `hrai` | Human-reviewed AI | The reviewer |
| `aih` | AI-assisted human | The human |
| `auto` | Autonomous agent | The agent operator |
| `mix` | Mixed/indeterminate | It's complicated |

### Validate a Version String

```bash
trustver validate "2.4.0+hrai"
# Valid TrustVer: 2.4.0+hrai
#   effort: 2.4.0
#   authorship: hrai

trustver validate "2.4.0"
# Invalid: missing '+' authorship separator
# (exit code 1)
```

### Bump Versions

```bash
# Auto-derives authorship from commit history
trustver bump micro
trustver bump meso
trustver bump macro

# Override authorship manually
trustver bump meso --authorship hrai

# Strict mode: fail if any commit lacks a TrustVer tag
trustver bump meso --strict
```

---

## Commit Convention

TrustVer extends [Conventional Commits](https://www.conventionalcommits.org/) with authorship metadata.

### Format

```
<type>(<scope>): <description> [<authorship-tag>]

<optional body>

Authorship: <tag>
Model: <model identifier>
Contribution: <percentage or description>
Reviewer: <identity>
```

### Examples

```bash
# Human-written fix
git commit -m "fix(parser): handle nested brackets [h]

Authorship: h"

# AI-generated feature, human-reviewed
git commit -m "feat(auth): add OAuth2 PKCE flow [hrai]

AI-generated implementation reviewed by jascha.

Authorship: hrai
Model: claude-opus-4-6
Contribution: ~85% AI-generated
Reviewer: jascha@tarnover.com"

# Autonomous agent
git commit -m "feat(api): rate limiting middleware [auto]

Authorship: auto
Model: claude-opus-4-6
Agent-Id: did:web:agents.thirdkey.ai#deploy-bot-1"
```

### Validate Commits

```bash
# Inline message
trustver check-commit "feat: add feature [hrai]

Authorship: hrai
Reviewer: dev@example.com"

# From file (used by git hooks)
trustver check-commit --file .git/COMMIT_EDITMSG

# Install git hook
trustver hook install
```

### Audit a Release Range

```bash
trustver audit v1.0.0..v2.0.0
# Provenance Audit: v1.0.0..v2.0.0
#   Derived tag: hrai
#   Commits: 12 (0 untagged)
#   Lines changed: 1847
#
#   Authorship breakdown (by lines):
#     hrai: 72.3%
#     h: 18.1%
#     aih: 9.6%

trustver audit --json v1.0.0..v2.0.0
```

---

## PAD Operations

### Generate a Signing Keypair

```bash
trustver key generate
# Generated ECDSA P-256 keypair:
#   Private key: .trustver/keys/trustver-private.pem
#   Public key:  .trustver/keys/trustver-public.pem
#   Key ID:      sha256:a1b2c3...
```

Uses [SchemaPin](https://schemapin.org) ECDSA P-256 crypto.

### Generate a PAD

```bash
trustver pad generate \
  --artifact dist/mylib-2.4.0.tar.gz \
  --scope stable \
  --model claude-opus-4-6 \
  --reviewer jascha@tarnover.com \
  --contribution-pct 72
```

Auto-detects: git commit, branch, remote URL, CI environment (GitHub Actions, GitLab CI, CircleCI, Jenkins).

### Sign a PAD

```bash
# Local ECDSA key
trustver pad sign mylib-2.4.0+hrai.pad.json \
  --key .trustver/keys/trustver-private.pem \
  --public-key .trustver/keys/trustver-public.pem \
  --signer jascha@tarnover.com

# Sigstore/cosign
trustver pad sign mylib-2.4.0+hrai.pad.json \
  --signer ci@github.com \
  --sigstore
```

### Append Attestations

```bash
# Signed attestation
trustver pad attest mylib-2.4.0+hrai.pad.json \
  --type test-verified \
  --attester ci@github.com \
  --detail '{"suite":"cargo test","passed":84,"failed":0}' \
  --sign-key .trustver/keys/trustver-private.pem

# Unsigned (draft)
trustver pad attest mylib-2.4.0+hrai.pad.json \
  --type code-review \
  --attester dev@example.com \
  --unsigned
```

Standard attestation types: `test-verified`, `static-analysis`, `manual-audit`, `code-review`, `ci-passed`, `pentest`, `sbom-verified`, `slsa-attested`, `formally-verified`.

### Validate a PAD

```bash
# Structure only
trustver pad validate mylib-2.4.0+hrai.pad.json

# With signature verification
trustver pad validate mylib-2.4.0+hrai.pad.json \
  --verify \
  --public-key .trustver/keys/trustver-public.pem

# JSON output
trustver pad validate --json mylib-2.4.0+hrai.pad.json
```

---

## PAD Structure

```json
{
  "trustver_spec": "0.3.0",
  "version": "2.4.0+hrai",
  "package": "mylib",
  "timestamp": "2026-03-23T14:22:00Z",
  "identity": {
    "artifact_hashes": { "sha256": "e3b0c44..." },
    "source": { "repository": "https://github.com/...", "commit": "abc123...", "branch": "main" },
    "build": { "system": "github-actions", "build_id": "run-98765", "reproducible": true }
  },
  "authorship": {
    "tag": "hrai",
    "detail": { "ai_model": "claude-opus-4-6", "ai_contribution_pct": 72, "human_reviewers": ["jascha@tarnover.com"] }
  },
  "scope": "stable",
  "attestations": [],
  "signatures": []
}
```

---

## Authorship Derivation Rules

When running `trustver bump`, the release authorship tag is auto-derived from commits since the last tag, weighted by lines changed:

| Condition | Release Tag |
|---|---|
| >= 95% human | `h` |
| >= 80% ai/hrai, all reviewed | `hrai` |
| >= 80% aih | `aih` |
| >= 80% auto | `auto` |
| >= 80% ai/auto, no review | `ai` |
| No tag reaches threshold | `mix` |

---

## Pro Tips for AI Assistants

1. **Always include `[tag]` in commit subjects** — it's required for TrustVer conformance and shows up in `git log --oneline`
2. **The `Authorship` trailer must match the subject tag** — `[hrai]` in subject means `Authorship: hrai` in the footer
3. **`Reviewer` is required for `hrai`** — omitting it is a validation error
4. **`mix` is always a safe default** when authorship is unclear
5. **Version strings are SemVer-compatible** — the `+tag` goes in the build metadata position, ignored by package managers
6. **PADs are append-only for attestations** — you can add attestations after release but never modify identity or authorship
7. **Sign PADs with SchemaPin keys** — same ECDSA P-256 keys work across the ThirdKey trust stack
8. **Use `--json` flags** for machine-readable output in CI pipelines
9. **Pre-release versions work** — `1.0.0-rc.1+hrai` is valid TrustVer
10. **Install the git hook early** — `trustver hook install` catches non-conformant commits before they're pushed
