# TrustVer

Provenance-aware versioning for AI-era software.

TrustVer combines [EffVer](https://effver.org/) effort semantics with an authorship tag and a signed Provenance Attestation Document (PAD) to answer: **how much effort does this update require, who or what wrote it, and what verification was applied?**

```
2.4.0+hrai
```

That's a TrustVer version. Effort level `2.4.0`. Authorship: AI-generated, human-reviewed.

[Full Specification](TRUSTVER_SPEC.md)

## CLI Tool

The `trustver` CLI implements the spec's tooling requirements. Install from source:

```bash
cargo install --path trustver-cli
```

### Version & Commit Operations

```bash
# Initialize a project
trustver init --name mylib

# Validate a version string
trustver validate "2.4.0+hrai"

# Validate a commit message
trustver check-commit "feat(auth): add OAuth2 PKCE flow [hrai]

Authorship: hrai
Model: claude-opus-4-6
Reviewer: jascha@tarnover.com"

# Bump version with auto-derived authorship from commit history
trustver bump meso

# Provenance audit for a release range
trustver audit v1.0.0..v2.0.0

# Install commit-msg git hook
trustver hook install
```

### PAD Operations

```bash
# Generate a signing keypair
trustver key generate

# Generate a PAD from current project state
trustver pad generate --artifact dist/mylib-2.4.0.tar.gz --scope stable

# Sign the PAD
trustver pad sign mylib-2.4.0+hrai.pad.json \
  --key .trustver/keys/trustver-private.pem \
  --public-key .trustver/keys/trustver-public.pem \
  --signer jascha@tarnover.com

# Append an attestation
trustver pad attest mylib-2.4.0+hrai.pad.json \
  --type test-verified \
  --attester ci@github.com \
  --detail '{"suite":"cargo test","passed":84,"failed":0}' \
  --sign-key .trustver/keys/trustver-private.pem

# Validate PAD structure and verify signatures
trustver pad validate mylib-2.4.0+hrai.pad.json \
  --verify --public-key .trustver/keys/trustver-public.pem
```

### Authorship Tags

| Tag | Meaning |
|---|---|
| `h` | Human-authored |
| `ai` | AI-generated (no human review) |
| `hrai` | Human-reviewed AI |
| `aih` | AI-assisted human |
| `auto` | Autonomous agent |
| `mix` | Mixed/indeterminate |

### Commit Convention

TrustVer extends [Conventional Commits](https://www.conventionalcommits.org/) with an authorship tag:

```
feat(auth): add OAuth2 PKCE flow [hrai]

AI-generated implementation reviewed by human.

Authorship: hrai
Model: claude-opus-4-6
Reviewer: jascha@tarnover.com
```

## Trust Stack Integration

TrustVer is part of the [ThirdKey Trust Stack](https://thirdkey.ai):

- **[SchemaPin](https://schemapin.org)** — ECDSA P-256 signing primitives and `.well-known` key discovery used for PAD signatures
- **[AgentPin](https://agentpin.org)** — Verifiable agent identity for autonomous releases
- **[ToolClad](https://toolclad.org)** — Declarative tool contracts for secure command execution in agentic systems
- **[Symbiont](https://symbiont.dev)** — Runtime policy enforcement against PAD attestations

## License

Specification: [CC-BY-SA 4.0](https://creativecommons.org/licenses/by-sa/4.0/)
Tooling: MIT

Comments, ideas, and PRs welcome.
