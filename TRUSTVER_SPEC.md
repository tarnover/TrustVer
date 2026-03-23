# TrustVer 0.1.0

## Provenance-Aware Versioning for AI-Era Software

**Status:** Draft  
**Authors:** Jascha Wanger (Tarnover LLC / ThirdKey AI)  
**Date:** 2026-03-23  
**License:** CC-BY-SA 4.0  

---

## Abstract

TrustVer is a versioning specification designed for software developed in environments where AI agents, human developers, and automated systems co-author code. It extends effort-based versioning with content-addressable identity and machine-readable provenance metadata, enabling downstream consumers — including trust infrastructure, package managers, and autonomous agents — to reason about *what changed*, *how much work adoption requires*, and *who or what produced the change under what verification regime*.

---

## Motivation

Existing versioning schemes were designed for a world where humans write code at human speed:

- **SemVer** communicates API compatibility but assumes sequential, deliberate releases and a clear human judgment call at each boundary. At AI-assisted development velocity, the cognitive overhead of "major vs. minor vs. patch" becomes untenable, and version numbers inflate meaninglessly.
- **CalVer** communicates *when* but not *what* or *how much effort*. It tells you nothing about the nature or safety of a change.
- **EffVer** improves on SemVer by honestly communicating adoption effort rather than pretending to guarantee compatibility. But it still says nothing about *provenance* — who or what authored the change, and what verification was applied.
- **HashVer** provides exact source traceability but no semantic signal about effort or trust posture.

None of these schemes address the fundamental new question introduced by AI-assisted and AI-autonomous development: **what is the trust posture of this release?**

When 40–60% of code is AI-generated, consumers need to know:

1. How much effort will this update cost me? *(effort)*
2. Can I trace this release to exact source? *(identity)*
3. Was this change human-authored, AI-generated, AI-generated-and-human-reviewed, or autonomously produced? *(authorship)*
4. What verification was applied — tests, formal proof, manual audit, none? *(attestation)*

TrustVer encodes all four dimensions in a single version string.

---

## Specification

### 1. Version Format

A TrustVer version string takes the form:

```
MACRO.MESO.MICRO-HASH+PROVENANCE
```

**Full example:**

```
2.4.0-a1b2c3d4+hrai.tv.sr
```

Broken down:

| Segment | Example | Meaning |
|---|---|---|
| `MACRO.MESO.MICRO` | `2.4.0` | Effort level (EffVer-compatible) |
| `-HASH` | `-a1b2c3d4` | Content identity (truncated content hash) |
| `+PROVENANCE` | `+hrai.tv.sr` | Provenance tag (authorship.verification.scope) |

All three segments are REQUIRED for a conformant TrustVer string. Partial forms (effort-only, effort+hash) are valid SemVer/EffVer and MAY be used in contexts where provenance is not yet tracked, but such strings are NOT conformant TrustVer.

---

### 2. Effort Segment: `MACRO.MESO.MICRO`

TrustVer adopts EffVer semantics for the numeric portion:

- **MACRO** — Significant adoption effort expected. Architectural changes, breaking migrations, epoch-level shifts.
- **MESO** — Some effort required. Behavioral changes, deprecation removals, non-trivial adjustments to workflows.
- **MICRO** — No effort expected. Bug fixes, performance improvements, additive features that don't affect existing usage.

Each segment is a non-negative integer. Segments MUST increment numerically. The zero-version case (`0.X.Y`) follows EffVer conventions: `X` behaves as MACRO and `Y` as MESO.

**Rationale:** EffVer is forward/backward compatible with SemVer tooling and package manager resolution. This means TrustVer versions sort correctly in npm, pip, cargo, and other ecosystems without modification. The effort framing is more honest and more useful than compatibility promises that are routinely violated in practice.

---

### 3. Content Identity Segment: `-HASH`

The HASH segment is a truncated content hash of the release artifact(s), prefixed with `-` per SemVer pre-release syntax positioning.

**Requirements:**

- The hash MUST be derived from the content of the release, NOT from the git commit hash (which reflects repository state, not artifact state).
- The hash algorithm MUST be SHA-256. Truncation to the first 8 hex characters (32 bits) is the default for display. The full hash MUST be available in release metadata.
- For single-artifact releases: hash the artifact directly.
- For multi-artifact releases: hash a deterministic manifest of individual artifact hashes (sorted lexicographically by artifact path, one `SHA256:path` entry per line, then hash the manifest).

**Content-addressable identity means:**

- Two releases with identical content produce identical hashes regardless of when or where they were built (reproducible builds).
- The hash serves as a checksum for integrity verification.
- Downstream trust infrastructure (e.g., Symbiont) can pin to content hashes rather than version numbers, providing a cryptographic binding between "the version I evaluated" and "the artifact I'm running."

**Example manifest for multi-artifact release:**

```
e3b0c442...98fb:lib/core.wasm
a7ffc6f8...4adb:lib/plugin-api.js
b5bb9d80...8ee9:bin/cli-linux-x64
```

SHA-256 of the manifest → truncated to `a1b2c3d4` for the version string.

---

### 4. Provenance Segment: `+PROVENANCE`

The provenance tag is a structured, dot-separated string following the `+` delimiter (occupying the SemVer build metadata position). It encodes three sub-fields:

```
+AUTHORSHIP.VERIFICATION.SCOPE
```

#### 4.1. Authorship Tags

Describes *who or what* produced the change.

| Tag | Meaning | Description |
|---|---|---|
| `h` | Human-authored | Code written entirely by human developers |
| `ai` | AI-generated | Code generated entirely by AI (any model/agent) |
| `hrai` | Human-reviewed AI | AI-generated code that has been reviewed and approved by a human |
| `aih` | AI-assisted human | Human-authored code with AI assistance (copilot-style) |
| `auto` | Autonomous agent | Code produced by an autonomous agent system with no human in the loop |
| `mix` | Mixed/indeterminate | Authorship cannot be cleanly categorized; release includes both human and AI contributions with varying review levels |

**Determination rules:**

- Authorship refers to the *predominant mode of production* for the changeset in this release.
- If >80% of changed lines originated from AI generation, use `ai` or `hrai` (depending on review).
- If authorship is roughly balanced or the tooling cannot distinguish, use `mix`.
- When in doubt, use the *lower trust* tag. `mix` is always a safe default.

#### 4.2. Verification Tags

Describes *what assurance process* was applied before release.

| Tag | Meaning | Description |
|---|---|---|
| `fv` | Formally verified | Mathematical proof of correctness (e.g., vericoding, Coq/Lean proofs) |
| `tv` | Test-verified | Passed automated test suite with defined coverage thresholds |
| `sa` | Static analysis | Passed static analysis / linting / SAST without critical findings |
| `ma` | Manual audit | Human security/code audit performed |
| `cr` | Code review | Standard code review (human or AI-assisted) |
| `ci` | CI-passed | Passed CI pipeline (may include tests, but no explicit coverage guarantees) |
| `nv` | Not verified | No verification beyond "it compiles" |

**Composition:** Multiple verification tags MAY be combined with `_` (underscore) to indicate layered verification:

```
+hrai.tv_sa_cr.sr    # AI-generated, human-reviewed, test-verified + static analysis + code review
+auto.ci.sr          # Autonomous agent, CI-passed only
+h.fv_ma.sr          # Human-authored, formally verified + manual audit
```

**Ordering:** Verification tags SHOULD be ordered from strongest to weakest assurance.

#### 4.3. Scope Tags

Describes *what the release covers* in terms of deployment trust.

| Tag | Meaning | Description |
|---|---|---|
| `sr` | Stable release | Production-ready, intended for general consumption |
| `rc` | Release candidate | Feature-complete, undergoing final validation |
| `pr` | Preview release | Functional but not production-ready |
| `ex` | Experimental | May be unstable, incomplete, or exploratory |
| `sb` | Sandbox-only | Explicitly NOT for production; trust infrastructure SHOULD refuse to promote |

---

### 5. Version Ordering

For sorting and dependency resolution, TrustVer follows SemVer precedence rules:

1. Versions are compared by MACRO, then MESO, then MICRO (numerically).
2. The HASH segment is treated as a pre-release identifier per SemVer — this means `2.4.0-a1b2c3d4` has *lower* precedence than `2.4.0` in strict SemVer sorting. Tooling that is TrustVer-aware SHOULD treat the hash as metadata (not affecting precedence).
3. The PROVENANCE segment occupies the build metadata position and is IGNORED for precedence per SemVer §10.

**Practical implication:** Standard package managers (npm, pip, cargo) will sort TrustVer versions correctly by effort level. The hash and provenance are metadata that ride along without breaking existing tooling.

---

### 6. Trust Infrastructure Integration

TrustVer is designed to interoperate with cryptographic trust systems. The following integration points are defined:

#### 6.1. SchemaPin Integration

When a tool or API is versioned with TrustVer, the content hash in the version string SHOULD match the schema hash registered in SchemaPin's `.well-known/schemapin.json`. This creates a verifiable binding: the version string claims a content identity, and SchemaPin independently attests to the schema's integrity.

#### 6.2. AgentPin Integration

When an AI agent publishes a release, the authorship tag (`ai`, `auto`, etc.) SHOULD correspond to a verifiable AgentPin credential. An agent claiming `+auto.tv.sr` MUST have a valid AgentPin identity that can be independently verified via the agent's domain-anchored trust chain.

#### 6.3. Symbiont Integration

Symbiont runtime policies can reference TrustVer provenance tags directly:

```yaml
# Example Symbiont policy
dependency_policy:
  allow:
    - provenance.authorship: [h, hrai, aih]
      provenance.verification: [tv, fv, tv_sa, tv_cr]
      provenance.scope: [sr]
  deny:
    - provenance.authorship: [auto, ai]
      provenance.verification: [nv, ci]
  quarantine:
    - provenance.authorship: [auto]
      provenance.verification: [tv]
      # Auto-generated + test-verified: allow but flag for review
```

This enables zero-trust dependency management: Symbiont can enforce that no dependency enters a production runtime unless it meets minimum provenance requirements, without requiring manual review of every update.

---

### 7. Provenance Attestation Document

The version string is a *claim*. Claims require *evidence*. Each TrustVer release SHOULD be accompanied by a **Provenance Attestation Document** (PAD), a signed JSON document that provides the full evidentiary backing for the provenance tag.

```json
{
  "trustver": "2.4.0-a1b2c3d4+hrai.tv_sa.sr",
  "artifact_hash_full": "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
  "hash_algorithm": "sha256",
  "timestamp": "2026-03-23T14:22:00Z",
  "authorship": {
    "tag": "hrai",
    "detail": {
      "ai_model": "claude-opus-4-6",
      "ai_contribution_pct": 72,
      "human_reviewers": ["jascha@tarnover.com"],
      "review_timestamp": "2026-03-23T13:45:00Z"
    }
  },
  "verification": {
    "tags": ["tv", "sa"],
    "detail": {
      "test_suite": "pytest",
      "test_coverage_pct": 94,
      "tests_passed": 847,
      "tests_failed": 0,
      "sast_tool": "semgrep",
      "sast_critical_findings": 0,
      "sast_high_findings": 0
    }
  },
  "scope": {
    "tag": "sr"
  },
  "source": {
    "repository": "https://github.com/thirdkeyai/example",
    "commit": "abc123def456...",
    "branch": "main",
    "build_system": "github-actions",
    "build_id": "run-98765"
  },
  "signatures": [
    {
      "signer": "jascha@tarnover.com",
      "algorithm": "ECDSA-P256",
      "signature": "MEUCIQDk..."
    },
    {
      "signer": "ci@github.com",
      "algorithm": "sigstore-cosign",
      "signature": "eyJhbGci..."
    }
  ]
}
```

The PAD SHOULD be:
- Published alongside the release artifact (e.g., as `<artifact>.trustver.json`).
- Signed by at least one human identity and (if applicable) the CI/build system identity.
- Discoverable via a `.well-known/trustver/` endpoint for published packages.

---

### 8. Velocity Management

TrustVer addresses version number inflation (the core concern for AI-velocity development) through two mechanisms:

#### 8.1. Content Hash Deduplication

If two successive builds produce identical artifacts, they produce identical content hashes. Tooling SHOULD recognize this and suppress redundant version bumps. This is particularly relevant for AI-generated code that may be regenerated without semantic change.

#### 8.2. Epoch Resets

When a project undergoes a fundamental transformation (full rewrite, new architecture, new trust regime), TrustVer permits an **epoch reset** using the format:

```
E:MACRO.MESO.MICRO-HASH+PROVENANCE
```

Where `E` is a monotonically increasing epoch integer. Epoch 1 is implicit and SHOULD be omitted. Example:

```
2:1.0.0-f4e5d6c7+auto.fv.sr
```

This is epoch 2, a formally verified release produced by an autonomous agent. The effort counter has reset.

**Note:** Epoch prefixes are not SemVer-compatible. Package managers that do not support epochs will see `2:1.0.0` as an invalid version. For ecosystem compatibility, the epoch SHOULD be encoded in the package name or as metadata (e.g., `mylib-e2` at version `1.0.0-f4e5d6c7+auto.fv.sr`), or the epoch prefix should be stripped for registry publication and preserved only in the PAD.

---

### 9. Comparison with Existing Schemes

| Dimension | SemVer | CalVer | EffVer | HashVer | **TrustVer** |
|---|---|---|---|---|---|
| Adoption effort signal | Indirect (compatibility) | None | Direct | None | **Direct (EffVer-based)** |
| Temporal signal | None | Primary | None | Yes (date prefix) | **Via PAD timestamp** |
| Source traceability | None | None | None | Git hash | **Content hash (artifact-level)** |
| Authorship metadata | None | None | None | None | **Structured tags** |
| Verification metadata | None | None | None | None | **Structured tags** |
| Trust infrastructure integration | None | None | None | None | **SchemaPin/AgentPin/Symbiont** |
| Package manager compatibility | Native | Partial | Native | Partial | **Native (effort segment)** |

---

### 10. Adopting TrustVer

#### 10.1. Minimum Viable Adoption

A project can adopt TrustVer incrementally:

1. **Start with EffVer:** Use `MACRO.MESO.MICRO` with effort semantics. This alone is an improvement.
2. **Add content hash:** Integrate artifact hashing into your build pipeline. Append `-HASH` to versions.
3. **Add provenance tags:** Begin tracking authorship and verification. Append `+PROVENANCE`.
4. **Publish PADs:** Generate and sign Provenance Attestation Documents.
5. **Integrate with trust infrastructure:** Connect to SchemaPin/AgentPin/Symbiont for end-to-end verification.

#### 10.2. Tooling Requirements

- A `trustver` CLI tool SHOULD be provided for:
  - Generating version strings from build metadata.
  - Validating TrustVer strings.
  - Generating and signing PADs.
  - Querying provenance from version strings.
- CI/CD integrations for GitHub Actions, GitLab CI, and similar platforms.
- Package manager plugins/wrappers that preserve provenance metadata through publish/install cycles.

---

### 11. Security Considerations

- **Provenance tags are claims, not proofs.** Without a signed PAD and independent verification (via SchemaPin, Sigstore, etc.), provenance tags should be treated as advisory. Trust infrastructure MUST verify PAD signatures before enforcing policy based on provenance.
- **Authorship determination is imprecise.** The boundary between "AI-assisted human" and "human-reviewed AI" is fuzzy. Projects SHOULD document their threshold criteria and apply them consistently.
- **Hash truncation for display.** The 8-character truncated hash in the version string provides collision resistance of ~2^32. This is sufficient for display and quick reference but NOT for security-critical identity verification. Always verify against the full hash in the PAD.
- **Tag spoofing.** A malicious publisher could claim `+h.fv.sr` on unverified AI-generated code. This is why the PAD exists and why Symbiont policies should require signed attestations, not just tag parsing.

---

### 12. Future Work

- **AI model fingerprinting:** Extending authorship metadata to identify specific models or agent configurations that contributed to a release, enabling downstream trust decisions based on model provenance chains.
- **Differential provenance:** Per-file or per-function provenance tracking, allowing a single release to carry mixed provenance (e.g., "the crypto module is human-authored and formally verified; the CLI wrapper is AI-generated and test-verified").
- **Provenance inheritance:** When a release depends on other TrustVer-versioned packages, the effective trust posture of the aggregate should be computable. The weakest link in the dependency chain determines the floor.
- **Automated authorship classification:** Tooling that integrates with IDE telemetry, AI API logs, and git metadata to automatically determine authorship tags rather than relying on manual declaration.

---

## Acknowledgments

TrustVer builds directly on the work of:

- **Jacob Tomlinson** — Intended Effort Versioning (EffVer)
- **Tom Preston-Werner** — Semantic Versioning (SemVer)  
- **Mahmoud Hashemi** — Calendar Versioning (CalVer)
- **Taylor Brazelton / miniscruff** — Hash Versioning (HashVer)
- **The Sigstore project** — Supply chain attestation patterns

---

## License

This specification is released under [CC-BY-SA 4.0](https://creativecommons.org/licenses/by-sa/4.0/). Implementations of TrustVer tooling are encouraged under any OSI-approved open source license.

---

*TrustVer is a component of the ThirdKey Trust Stack ecosystem.*
*https://thirdkey.ai*
