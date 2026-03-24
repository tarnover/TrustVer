# TrustVer 0.3.0

## Provenance-Aware Versioning for AI-Era Software

**Status:** Draft  
**Authors:** Jascha Wanger (Tarnover LLC / ThirdKey AI)  
**Date:** 2026-03-23  
**License:** CC-BY-SA 4.0  

---

## Abstract

TrustVer is a versioning and provenance specification for software developed in environments where AI agents, human developers, and automated systems co-author code. It combines a human-friendly version string — EffVer effort semantics plus an authorship tag — with a **commit convention** for capturing ground-truth provenance at the point of creation, and a mandatory **Provenance Attestation Document (PAD)** that carries the full trust metadata as a signed sidecar.

The version string answers two questions at a glance: **how much effort does this update require?** and **who or what wrote it?** Commits carry the ground-truth authorship data. The PAD answers everything else: verification status, artifact identity, attestation history, and build provenance.

```
2.4.0+hrai
```

That's a TrustVer version. Effort level `2.4.0`. Authorship: AI-generated, human-reviewed.

---

## Motivation

Existing versioning schemes were designed for a world where humans write code at human speed:

- **SemVer** communicates API compatibility but assumes sequential, deliberate releases and a clear human judgment call at each boundary. At AI-assisted development velocity, the cognitive overhead of "major vs. minor vs. patch" becomes untenable, and version numbers inflate meaninglessly.
- **CalVer** communicates *when* but not *what* or *how much effort*. It tells you nothing about the nature or safety of a change.
- **EffVer** improves on SemVer by honestly communicating adoption effort rather than pretending to guarantee compatibility. But it says nothing about *provenance* — who or what authored the change, and what verification was applied.
- **HashVer** provides exact source traceability but no semantic signal about effort or trust posture.

None of these schemes address the fundamental new question introduced by AI-assisted and AI-autonomous development: **what is the trust posture of this release?**

When 40–60% of code is AI-generated, consumers need to know:

1. How much effort will this update cost me? *(effort — version string)*
2. Who or what wrote this? *(authorship — version string)*
3. What verification was applied — tests, formal proof, manual audit, none? *(attestation — PAD)*
4. Can I trace this release to exact source and artifacts? *(identity — PAD)*

TrustVer answers questions 1–2 in the version string and questions 3–4 in the PAD.

---

## Design Principles

### What Goes in the Version String

A datum belongs in the version string only if it satisfies **all three** criteria:

1. **Immutable at release time.** It will never need to be updated after publication.
2. **Novel signal.** No existing versioning scheme or package manager infrastructure already provides it.
3. **Human-speakable.** A developer can say it aloud in a stand-up without stumbling.

**Effort level** passes all three: it's decided at release time, EffVer is not yet standard, and "two four oh" is natural speech.

**Authorship** passes all three: the code's origin doesn't change post-release, no existing scheme encodes it, and "two four oh plus h-r-a-i" is short enough for conversation.

**Verification** fails criterion 1. A release may be audited, pentested, or formally verified weeks after publication. Baking verification into an immutable string means cutting phantom releases to update metadata.

**Content hashes** fail criterion 2. Lockfiles (`package-lock.json`, `Cargo.lock`, `poetry.lock`) already provide artifact integrity. They also fail criterion 3 — nobody says hex digests in a stand-up.

### What Goes in the PAD

Everything else. Verification status, artifact hashes, build provenance, source traceability, SLSA level, dependency provenance. The PAD is a signed, append-only attestation document that evolves over a release's lifetime. It is the authoritative trust record.

---

## Specification

### 1. Version String

A TrustVer version string takes the form:

```
MACRO.MESO.MICRO+AUTHORSHIP
```

**Examples:**

```
2.4.0+hrai       # Meso effort, AI-generated code reviewed by a human
1.0.0+h          # Macro effort, entirely human-authored
0.7.3+auto       # Pre-1.0, produced by an autonomous agent
3.1.1+aih        # Micro effort, human-authored with AI assistance
0.2.0+mix        # Mixed or indeterminate authorship
```

The effort segment (`MACRO.MESO.MICRO`) follows EffVer semantics and is fully compatible with SemVer tooling. The authorship tag occupies the SemVer build metadata position (`+`), which is **explicitly ignored for version precedence** per SemVer §10. This means:

- Package managers sort TrustVer versions correctly: `2.4.0+hrai` and `2.4.0+auto` have identical precedence.
- No custom plugins, no ecosystem changes, no new registry support required for basic adoption.
- The authorship tag is visible to humans and to trust-aware tooling, but invisible to dependency resolution.

The authorship tag is REQUIRED for a conformant TrustVer string. A bare `MACRO.MESO.MICRO` without an authorship tag is valid EffVer/SemVer but is NOT conformant TrustVer.

---

### 2. Effort Semantics

TrustVer adopts EffVer semantics:

- **MACRO** — Significant adoption effort expected. Architectural changes, breaking migrations, epoch-level shifts.
- **MESO** — Some effort required. Behavioral changes, deprecation removals, non-trivial adjustments to workflows.
- **MICRO** — No effort expected. Bug fixes, performance improvements, additive features that don't affect existing usage.

Each segment is a non-negative integer. Segments MUST increment numerically. The zero-version case (`0.X.Y`) follows EffVer conventions: `X` behaves as MACRO, `Y` as MESO.

EffVer's "over-bumping" convention applies: bumping a higher segment than strictly necessary (e.g., MESO for a purely additive feature) is acceptable to signal significance. Under-bumping (claiming MICRO for a change that actually requires effort) is a violation.

---

### 3. Authorship Tags

The authorship tag encodes the predominant mode of production for the changeset in a release.

| Tag | Meaning | Description |
|---|---|---|
| `h` | Human-authored | Code written entirely by human developers |
| `ai` | AI-generated | Code generated entirely by AI (any model/agent) |
| `hrai` | Human-reviewed AI | AI-generated code reviewed and approved by a human |
| `aih` | AI-assisted human | Human-authored code with AI assistance (copilot-style) |
| `auto` | Autonomous agent | Produced by an autonomous agent with no human in the loop |
| `mix` | Mixed/indeterminate | Authorship cannot be cleanly categorized |

**Determination rules:**

- Authorship refers to the *predominant mode of production* for the changeset in this release, not the entire codebase.
- If >80% of changed lines originated from AI generation, use `ai` or `hrai` (depending on review status).
- If authorship is roughly balanced or tooling cannot distinguish, use `mix`.
- When in doubt, use the *lower trust* tag. `mix` is always a safe default.
- The authorship tag is **immutable after release**. If you discover the tag was wrong, document the correction in the PAD's notes field; do not cut a new release solely to change the tag.

**The distinction between `ai` and `hrai`** is whether a human reviewed the AI-generated output before release. This is a critical trust signal: `ai` means the code went from model output to release without human eyes on it; `hrai` means a human explicitly approved it.

**The distinction between `aih` and `hrai`** is about the direction of initiative. In `aih`, a human is writing code and using AI as a tool (autocomplete, suggestion, generation of boilerplate). In `hrai`, the AI produced the substantive logic and a human reviewed it. The mental model: who would you blame if there's a bug?

---

### 4. Commit Convention

Authorship is most accurately captured at the commit level — when the developer or agent actually knows how the code was produced. The release-level authorship tag (§3) is then *derived* from the commit history rather than declared by hand.

TrustVer defines a commit message convention compatible with [Conventional Commits](https://www.conventionalcommits.org/) that embeds authorship as ground-truth metadata.

#### 4.1. Commit Message Format

```
<type>(<scope>): <description> [<authorship-tag>]

<optional body>

Authorship: <tag>
Model: <model identifier, if applicable>
Contribution: <percentage or qualitative description>
Reviewer: <identity, if applicable>
Agent-Id: <AgentPin URI, if applicable>
```

**Examples:**

```
feat(auth): add OAuth2 PKCE flow [hrai]

AI-generated implementation of PKCE extension for the OAuth2 auth module.
Reviewed and modified by jascha@tarnover.com.

Authorship: hrai
Model: claude-opus-4-6
Contribution: ~85% AI-generated
Reviewer: jascha@tarnover.com
```

```
fix(parser): handle nested brackets in config values [h]

Edge case found during manual testing. Hand-written fix.

Authorship: h
```

```
feat(api): implement rate limiting middleware [auto]

Autonomously generated and tested by deployment agent.

Authorship: auto
Model: claude-opus-4-6
Agent-Id: did:web:agents.thirdkey.ai#deploy-bot-1
Contribution: 100% AI-generated
```

```
refactor(db): migrate connection pooling to bb8 [aih]

Human-driven refactor using AI for boilerplate generation
and test scaffolding.

Authorship: aih
Model: claude-sonnet-4-6
Contribution: ~30% AI-generated (boilerplate, tests)
Reviewer: jascha@tarnover.com
```

#### 4.2. Subject Line Tag

The `[authorship-tag]` in the subject line is REQUIRED for TrustVer-conformant commits. It uses the same tag vocabulary as §3 (`h`, `ai`, `hrai`, `aih`, `auto`, `mix`).

The subject line tag is the human-readable signal. It shows up in `git log --oneline`, PR titles, and changelog generators without any special tooling:

```
$ git log --oneline v2.3.0..v2.4.0
f4a2c1e feat(auth): add OAuth2 PKCE flow [hrai]
b7d9e3a fix(parser): handle nested brackets [h]
3c8f1d2 feat(api): rate limiting middleware [auto]
a1e5b4f refactor(db): migrate to bb8 pool [aih]
9d2c7f8 docs: update API reference [hrai]
```

At a glance, you can see the provenance profile of a release range. No tooling required — just `git log`.

#### 4.3. Footer Metadata

The commit body footer carries structured metadata using [git-trailer](https://git-scm.com/docs/git-interpret-trailers) conventions. The following fields are defined:

| Field | Required | Description |
|---|---|---|
| `Authorship` | REQUIRED | Tag from §3 vocabulary. MUST match the subject line tag. |
| `Model` | RECOMMENDED when AI-involved | Identifier of the AI model used (e.g., `claude-opus-4-6`, `gpt-4o`) |
| `Contribution` | RECOMMENDED when AI-involved | Approximate AI contribution (e.g., `~85% AI-generated`, `boilerplate only`) |
| `Reviewer` | REQUIRED for `hrai` | Identity of the human reviewer |
| `Agent-Id` | RECOMMENDED for `auto` | AgentPin URI or other verifiable agent identity |

Footer fields are machine-parseable via `git log --format='%(trailers)'` and standard trailer-parsing libraries.

#### 4.4. Deriving Release Authorship from Commits

The `trustver bump` command SHOULD derive the release-level authorship tag by scanning commits since the last release. The aggregation algorithm:

1. **Collect** all commits in the range `<last-release-tag>..HEAD`.
2. **Extract** the `Authorship` trailer (or subject line tag as fallback) from each commit.
3. **Weight** by lines changed (insertions + deletions) per commit. Merge commits and commits touching only non-code files (docs, CI config) MAY be excluded or down-weighted.
4. **Apply threshold rules:**

| Condition | Release Tag |
|---|---|
| ≥95% of weighted commits are `h` | `h` |
| ≥80% weighted are `ai` or `hrai`, and all AI commits have `Reviewer` | `hrai` |
| ≥80% weighted are `ai` or `auto`, missing human review | `ai` |
| ≥80% weighted are `aih` | `aih` |
| ≥80% weighted are `auto` | `auto` |
| No single tag reaches 80% | `mix` |

5. **The maintainer MAY override** the computed tag when cutting the release, but overrides MUST be documented in the PAD with a rationale. This handles edge cases like "90% of the lines changed were AI-generated test fixtures, but the 10% human-written code is the actual logic."

#### 4.5. Provenance Auditing via Git

The commit convention enables powerful provenance queries using standard git tooling:

```bash
# Authorship summary for a release range
git log v2.3.0..v2.4.0 --format='%(trailers:key=Authorship,valueonly)' | sort | uniq -c | sort -rn

# All autonomous agent commits
git log --all --grep='\[auto\]' --oneline

# AI-involved commits by a specific model
git log --all --format='%H %(trailers:key=Model,valueonly)' | grep 'claude-opus'

# Commits with no human review (potential risk)
git log v2.3.0..v2.4.0 --grep='\[ai\]\|\[auto\]' --oneline

# Lines of code by authorship type in a range
for tag in h ai hrai aih auto mix; do
  echo -n "$tag: "
  git log v2.3.0..v2.4.0 --grep="\[$tag\]" --numstat --format='' | \
    awk '{s+=$1+$2} END {print s+0}'
done
```

#### 4.6. Integration with Existing Tooling

The commit convention is designed to layer on top of Conventional Commits without breaking existing tools:

- **commitlint:** Add a custom rule to validate the `[tag]` suffix and footer fields.
- **semantic-release / release-please:** The `[tag]` is inside the description field and does not interfere with type/scope parsing. A plugin can extract it for PAD generation.
- **changelog generators:** The tag naturally flows into changelogs. A TrustVer-aware generator can group entries by authorship.
- **GitHub / GitLab:** The tag shows up in PR titles, merge commit messages, and commit lists without any platform changes.

#### 4.7. Commit-Level PAD Records

For projects requiring fine-grained provenance, the PAD MAY include a `commit_provenance` array that records per-commit authorship data, providing a complete audit trail from individual commits through to the release:

```json
{
  "commit_provenance": [
    {
      "commit": "f4a2c1e...",
      "authorship": "hrai",
      "model": "claude-opus-4-6",
      "contribution_pct": 85,
      "reviewer": "jascha@tarnover.com",
      "lines_changed": 342
    },
    {
      "commit": "b7d9e3a...",
      "authorship": "h",
      "lines_changed": 12
    },
    {
      "commit": "3c8f1d2...",
      "authorship": "auto",
      "model": "claude-opus-4-6",
      "agent_id": "did:web:agents.thirdkey.ai#deploy-bot-1",
      "lines_changed": 187
    }
  ],
  "derived_tag": "mix",
  "override": null
}
```

This bridges the gap between commit-level ground truth and release-level summary, and directly supports the differential provenance goal described in §14.

---

### 5. Provenance Attestation Document (PAD)

The PAD is a signed JSON document published alongside each release. It is the authoritative record of a release's full trust posture. The version string carries effort and authorship; the PAD carries everything else.

#### 5.1. PAD Structure

```json
{
  "trustver_spec": "0.3.0",
  "version": "2.4.0+hrai",
  "package": "mylib",
  "timestamp": "2026-03-23T14:22:00Z",

  "identity": {
    "artifact_hashes": {
      "sha256": "e3b0c44298fc1c149afbf4c8996fb924..."
    },
    "source": {
      "repository": "https://github.com/example/mylib",
      "commit": "abc123def456789...",
      "branch": "main"
    },
    "build": {
      "system": "github-actions",
      "build_id": "run-98765",
      "reproducible": true
    }
  },

  "authorship": {
    "tag": "hrai",
    "detail": {
      "ai_model": "claude-opus-4-6",
      "ai_contribution_pct": 72,
      "human_reviewers": ["jascha@tarnover.com"],
      "review_timestamp": "2026-03-23T13:45:00Z"
    }
  },

  "scope": "stable",

  "attestations": [
    {
      "type": "test-verified",
      "timestamp": "2026-03-23T14:00:00Z",
      "detail": {
        "suite": "pytest",
        "coverage_pct": 94,
        "passed": 847,
        "failed": 0
      },
      "attester": "ci@github.com",
      "signature": "eyJhbGci..."
    },
    {
      "type": "static-analysis",
      "timestamp": "2026-03-23T14:05:00Z",
      "detail": {
        "tool": "semgrep",
        "critical_findings": 0,
        "high_findings": 0
      },
      "attester": "ci@github.com",
      "signature": "eyJhbGci..."
    },
    {
      "type": "manual-audit",
      "timestamp": "2026-03-30T10:00:00Z",
      "detail": {
        "auditor": "jascha@tarnover.com",
        "scope": "security-review",
        "report_url": "https://example.com/audits/mylib-2.4.0.pdf"
      },
      "attester": "jascha@tarnover.com",
      "signature": "MEUCIQDk..."
    }
  ],

  "signatures": [
    {
      "signer": "jascha@tarnover.com",
      "algorithm": "ECDSA-P256",
      "key_id": "did:web:tarnover.com#release-key-1",
      "signature": "MEUCIQDk..."
    }
  ]
}
```

#### 5.2. Key Design Decisions

**Attestations are append-only.** The version `2.4.0+hrai` is immutable — it always refers to the same code with the same authorship. But the PAD's attestation list grows over time as new verification is applied. The release ships on day 1 with CI test results. A manual audit on day 7 appends a new entry. A formal verification pass on day 30 appends another. Each attestation is independently signed by the attester, making the PAD a verifiable audit trail.

**Artifact hashes live in the PAD, not the version string.** The PAD carries full SHA-256 hashes of release artifacts, providing content-addressable identity without polluting the human-readable version. Trust infrastructure pins to the PAD hash, not the version number.

**Authorship appears in both the string and the PAD.** The version string carries the tag (`hrai`) for at-a-glance visibility. The PAD carries the tag plus structured detail (model, contribution percentage, reviewers). When the commit convention (§4) is in use, the release-level tag is auto-derived from commit history, and the PAD MAY include per-commit provenance records (§4.7) for full audit traceability. The PAD is the source of truth; the version string is the summary.

**Scope is release-time metadata in the PAD.** Unlike earlier iterations of this spec, scope (stable, rc, preview, experimental, sandbox) is not in the version string. SemVer already has conventions for pre-release identifiers (`-alpha`, `-rc.1`), and duplicating them in a new tag adds no value.

**Canonical signing format.** When computing signatures over PAD content, the signable content is the PAD document with the `signatures` array removed, serialized as canonical JSON: keys sorted lexicographically at all nesting levels, no whitespace, no trailing commas. This ensures signature stability regardless of JSON formatting or field ordering in the source document.

---

### 6. Attestation Types

Attestations are verification records appended to a PAD over a release's lifetime. Each is independently signed.

| Type | Description |
|---|---|
| `formally-verified` | Mathematical proof of correctness (Coq, Lean, vericoding) |
| `test-verified` | Passed automated test suite with defined coverage thresholds |
| `static-analysis` | Passed SAST / linting without critical findings |
| `manual-audit` | Human security or code audit performed |
| `code-review` | Code review (human or AI-assisted) completed |
| `ci-passed` | Passed CI pipeline (may include tests, but no explicit guarantees) |
| `pentest` | Security penetration testing performed |
| `sbom-verified` | Software Bill of Materials generated and dependency provenance verified |
| `slsa-attested` | Meets a specified SLSA level (detail includes level) |

Attestation types are extensible. Custom types SHOULD use a namespaced format: `org.example.custom-check`.

**Each attestation MUST include:**

- `type` — One of the above or a namespaced custom type.
- `timestamp` — When the attestation was produced (ISO 8601).
- `attester` — Identity of the entity making the attestation.
- `signature` — Cryptographic signature over the attestation content.

**Each attestation SHOULD include:**

- `detail` — Structured data specific to the attestation type.

---

### 7. Scope

The PAD's `scope` field indicates the release's intended deployment context:

| Scope | Description |
|---|---|
| `stable` | Production-ready, intended for general consumption |
| `rc` | Release candidate, feature-complete, undergoing final validation |
| `preview` | Functional but not production-ready |
| `experimental` | May be unstable, incomplete, or exploratory |
| `sandbox` | Explicitly NOT for production; trust infrastructure SHOULD refuse to promote |

Scope is set at release time and is immutable within the PAD.

---

### 8. PAD Discovery and Distribution

PADs MUST be discoverable by tooling. TrustVer defines three distribution mechanisms (implementors MUST support at least one):

#### 8.1. Sidecar File

Publish the PAD alongside the release artifact:

```
mylib-2.4.0.tar.gz
mylib-2.4.0.tar.gz.pad.json
```

#### 8.2. Well-Known Endpoint

```
https://<registry-or-domain>/.well-known/trustver/<package>/<version>.pad.json
```

Example:

```
https://registry.npmjs.org/.well-known/trustver/mylib/2.4.0.pad.json
```

#### 8.3. Registry Metadata

Package registries MAY embed PAD data directly in their metadata APIs (e.g., a `trustver_pad` field in npm package version metadata).

---

### 9. Trust Infrastructure Integration

#### 9.1. SchemaPin Integration

PAD signatures SHOULD use SchemaPin's ECDSA P-256 signing primitives for cryptographic operations. Specifically:

- **Signing:** Use SchemaPin's `sign_data()` to sign the canonical JSON representation of the PAD (see §5.2). The resulting base64-encoded signature is stored in the PAD's `signatures` array with `algorithm: "ECDSA-P256"`.
- **Key identity:** The `key_id` field in PAD signatures SHOULD be the SHA-256 fingerprint of the signer's public key, computed via SchemaPin's `calculate_key_id()`. This fingerprint is stable and verifiable without fetching the full key.
- **Key discovery:** The signer's public key SHOULD be discoverable via the `.well-known/schemapin.json` endpoint (RFC 8615) hosted at the signer's domain. This enables automated key retrieval for PAD verification.
- **Verification:** Use SchemaPin's `verify_signature()` to verify PAD signatures against the signer's public key, obtained either locally or via `.well-known` discovery.
- **Artifact cross-referencing:** When a tool or API is versioned with TrustVer, the artifact hash in the PAD SHOULD be cross-referenced against the schema hash registered in SchemaPin's `.well-known/schemapin.json`. This provides independent verification that the artifact content matches the claimed schema.

SchemaPin integration is RECOMMENDED but not required. PADs MAY use alternative signing mechanisms (e.g., Sigstore/cosign for CI-produced attestations) provided the `algorithm` field in the signature accurately identifies the scheme used.

#### 9.2. AgentPin Integration

When an AI agent publishes a release with authorship tag `ai` or `auto`, the agent SHOULD have a valid AgentPin credential verifiable via domain-anchored trust. The PAD's authorship detail SHOULD include the agent's AgentPin identity URI, enabling downstream consumers to verify the agent's identity independently.

#### 9.3. Symbiont Integration

Symbiont runtime policies reference both the version string's authorship tag (for fast filtering) and the PAD (for full policy evaluation):

```yaml
# Example Symbiont policy
dependency_policy:
  # Fast-path: version string authorship check
  # (before fetching PAD)
  quick_deny:
    - authorship_tag: [auto, ai]

  # Full policy: evaluated against PAD
  require:
    - scope: stable
    - attestations:
        must_include:
          - test-verified
        must_include_one_of:
          - code-review
          - manual-audit

  allow_with_attestation:
    - authorship_tag: [auto, ai]
      required_attestations:
        - manual-audit

  warn:
    - authorship_tag: mix
      missing_attestation:
        - code-review
```

**Policy evaluation flow:**

1. Package manager resolves dependency to `mylib@2.4.0+hrai`.
2. Symbiont reads the authorship tag from the version string for fast-path filtering.
3. If not quick-denied, Symbiont fetches the PAD for `mylib@2.4.0`.
4. Symbiont verifies PAD signatures.
5. Symbiont evaluates the PAD against the full policy.
6. Install proceeds, blocks, or warns based on policy outcome.

The authorship tag in the version string enables **step 2** — a cheap, local check before any network call to fetch the PAD. This is particularly valuable in air-gapped or latency-sensitive environments.

---

### 10. Comparison with Existing Schemes

| Dimension | SemVer | CalVer | EffVer | HashVer | **TrustVer** |
|---|---|---|---|---|---|
| Version string | `1.2.3` | `2026.03` | `1.2.3` | `2026.03-a1b2` | **`1.2.3+hrai`** |
| Effort signal | Indirect | None | Direct | None | **Direct (EffVer)** |
| Authorship signal | None | None | None | None | **In version string** |
| Commit-level provenance | None | None | None | None | **Commit convention** |
| Temporal signal | None | Primary | None | Date prefix | **PAD timestamp** |
| Source traceability | None | None | None | Git hash | **PAD** |
| Verification metadata | None | None | None | None | **PAD (append-only)** |
| Post-release attestation | N/A | N/A | N/A | N/A | **Yes** |
| Trust infra integration | None | None | None | None | **SchemaPin/AgentPin/Symbiont** |
| Pkg manager compatibility | Native | Partial | Native | Partial | **Native** |
| Human speakability | Good | Good | Good | Poor | **Good** |

---

### 11. Adopting TrustVer

#### 11.1. Minimum Viable Adoption

1. **Adopt EffVer + authorship tag.** Start versioning releases as `MACRO.MESO.MICRO+AUTHORSHIP`. If you're already loosely following SemVer, this requires only adding the `+tag` suffix and shifting to effort-based increment decisions. Zero tooling changes; zero ecosystem friction.
2. **Adopt the commit convention.** Start tagging commits with `[authorship]` in subject lines and structured footers. This is the ground-truth layer — everything else derives from it.
3. **Generate PADs.** Produce PAD files in your CI pipeline. Use `trustver bump` to auto-derive release authorship from the commit history. Even a minimal PAD (version, artifact hash, authorship detail, CI attestation) is valuable.
4. **Sign PADs.** Add cryptographic signatures. Sigstore/cosign for CI-produced attestations, ECDSA keys for human attestations.
5. **Publish PADs.** Make them discoverable via sidecar files or well-known endpoints.
6. **Enforce policies.** Integrate with trust infrastructure (Symbiont or equivalent) to enforce PAD requirements on your own dependencies.

Each step is independently valuable. Step 1 is free. Step 2 builds the provenance habit. Step 3 adds transparency. Step 4 adds integrity. Step 5 enables ecosystem consumption. Step 6 closes the loop.

#### 11.2. Tooling

A `trustver` CLI tool SHOULD be provided for:

- Bumping version numbers with EffVer semantics and auto-derived authorship tagging from commit history.
- Validating commit messages against the TrustVer commit convention.
- Generating provenance summaries from commit ranges (`trustver audit v2.3.0..v2.4.0`).
- Generating PADs from build context (`trustver pad generate`), including auto-detection of CI environment, artifact hashing, and authorship detail from commit history.
- Signing PADs with local ECDSA keys via SchemaPin (`trustver pad sign`) or Sigstore/cosign (`trustver pad sign --sigstore`).
- Validating version strings and PAD structure, with optional cryptographic signature verification (`trustver pad validate --verify`).
- Appending attestations to existing PADs with optional per-attestation signatures (`trustver pad attest`).
- Generating ECDSA P-256 keypairs for PAD signing (`trustver key generate`).
- Querying PAD metadata for a given package/version.

Companion integrations SHOULD be provided for:

- **Git hooks:** A `commit-msg` hook that validates TrustVer commit convention compliance.
- **commitlint plugin:** Custom rules for subject line tag and footer field validation.
- **CI/CD platforms:** GitHub Actions, GitLab CI wrappers around the CLI for automated PAD generation at release time.

---

### 12. Security Considerations

- **PAD signatures are mandatory for enforcement.** Without signatures, a PAD is advisory only. Trust infrastructure MUST verify signatures before making policy decisions. An unsigned PAD is equivalent to no PAD.
- **Authorship tags are self-reported claims.** A malicious publisher can claim `h` (human-authored) on AI-generated code. The PAD's authorship detail provides evidence for auditing, but ultimately authorship verification depends on the integrity of the development environment. The version string tag is a convenience signal, not a security boundary — the PAD is the evidence.
- **The authorship tag enables fast filtering, not enforcement.** Symbiont's quick-deny based on the version string authorship tag is an optimization. Policy enforcement MUST be based on the signed PAD, not the unsigned version string tag. A mismatch between the version string tag and the PAD authorship is itself a policy violation.
- **Attestation freshness.** Attestations added long after release may reflect outdated analysis. Consumers SHOULD consider attestation timestamps relative to the release date.
- **PAD mutability scope.** Only the `attestations` array is append-only mutable. The `version`, `identity`, `authorship`, and `scope` fields are set at release time and MUST NOT be modified. The release-time fields are covered by the release signature; attestations carry their own independent signatures.
- **Missing PADs.** A project that adopts TrustVer but omits PADs for certain releases is evading provenance tracking. Symbiont policies SHOULD treat a missing PAD as the lowest trust posture (unknown authorship, no verification).

---

### 13. SLSA Alignment

TrustVer's PAD is designed to complement, not replace, existing supply-chain security frameworks:

- **SLSA Build Level 1:** A PAD with a `ci-passed` attestation and build provenance (`identity.build`) satisfies SLSA L1 requirements.
- **SLSA Build Level 2:** A PAD signed by a hosted CI system (e.g., GitHub Actions with Sigstore) satisfies SLSA L2 for signed provenance.
- **SLSA Build Level 3:** A PAD with `identity.build.reproducible: true` and attestations from an isolated build environment approaches SLSA L3.

PADs MAY include a `slsa-attested` attestation type that explicitly declares the claimed SLSA level.

---

### 14. Future Work

- **Automated authorship tagging.** The commit convention (§4) defines the data model; the next step is tooling that auto-detects authorship by integrating with IDE telemetry (Cursor/Windsurf session logs), AI API call records, and git diff heuristics — pre-populating the commit footer so developers only need to confirm rather than declare.
- **Differential provenance.** The commit-level PAD records (§4.7) provide per-commit granularity. The next frontier is per-file or per-function tracking within a single commit, allowing statements like "the crypto module is `h` + formally verified; the CLI wrapper is `hrai` + test-verified" at file-level resolution.
- **Provenance inheritance.** When a release depends on other TrustVer-versioned packages, computing the effective trust posture of the aggregate. The weakest link determines the floor.
- **AI model fingerprinting.** Extending authorship metadata to identify specific models, agent configurations, or prompt chains that contributed to a release.
- **Registry-native PAD support.** Working with npm, PyPI, crates.io, and other registries to embed PAD metadata natively in their APIs and enforce PAD requirements at publish time.
- **PAD-aware dependency resolution.** Package manager plugins that factor provenance into resolution — e.g., preferring a slightly older version with stronger attestations over the latest with no PAD.

---

## Acknowledgments

TrustVer builds directly on the work of:

- **Jacob Tomlinson** — Intended Effort Versioning (EffVer)
- **Tom Preston-Werner** — Semantic Versioning (SemVer)
- **Mahmoud Hashemi** — Calendar Versioning (CalVer)
- **Taylor Brazelton / miniscruff** — Hash Versioning (HashVer)
- **The Sigstore project** — Supply chain attestation patterns
- **SLSA (Supply-chain Levels for Software Artifacts)** — Build provenance framework
- **Conventional Commits** — Commit message convention

---

## License

This specification is released under [CC-BY-SA 4.0](https://creativecommons.org/licenses/by-sa/4.0/). Implementations of TrustVer tooling are encouraged under any OSI-approved open source license.

---

*TrustVer is an independent open specification.*  
*Trust infrastructure integration examples reference the ThirdKey Trust Stack (https://thirdkey.ai).*
