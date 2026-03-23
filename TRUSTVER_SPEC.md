# TrustVer 0.2.0

## Provenance-Aware Versioning for AI-Era Software

**Status:** Draft  
**Authors:** Jascha Wanger (Tarnover LLC / ThirdKey AI)  
**Date:** 2026-03-23  
**License:** CC-BY-SA 4.0  

---

## Abstract

TrustVer is a versioning and provenance specification for software developed in environments where AI agents, human developers, and automated systems co-author code. It combines a clean, human-friendly version string (EffVer-compatible, package-manager-native) with a mandatory **Provenance Attestation Document (PAD)** that carries machine-readable trust metadata — authorship, verification, and artifact identity — as a signed sidecar.

The core design principle: **the version string tells humans and package managers how much effort an update requires. The PAD tells trust infrastructure whether to allow it.**

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
2. Was this change human-authored, AI-generated, AI-generated-and-human-reviewed, or autonomously produced? *(authorship — PAD)*
3. What verification was applied — tests, formal proof, manual audit, none? *(attestation — PAD)*
4. Can I trace this release to exact source and artifacts? *(identity — PAD)*

TrustVer answers question 1 in the version string and questions 2–4 in the PAD.

### Why Not Encode Provenance in the Version String?

Earlier iterations of this spec embedded content hashes and provenance tags directly in the version string (e.g., `2.4.0-a1b2c3d4+hrai.tv.sr`). This was abandoned for three reasons:

1. **The Immutability Trap.** Version strings are immutable, but trust and verification are continuous. A release may be audited, penetration-tested, or formally verified days or weeks after publication. Baking verification status into an immutable identifier means cutting phantom releases just to update metadata. Verification belongs in a mutable attestation layer alongside the release, not welded to the identifier.

2. **Hash Redundancy.** Package ecosystems already handle content integrity via lockfiles (`package-lock.json`, `Cargo.lock`, `poetry.lock`). Forcing a content hash into the primary version string duplicates existing infrastructure and creates version churn for byte-level changes that affect neither effort nor trust.

3. **Human Ergonomics.** Developers talk about releases in stand-ups, documentation, Slack, and commit messages. `2.4.0` is speakable. `2.4.0-a1b2c3d4+hrai.tv_sa_cr.sr` is not. A version string that humans avoid saying is a version string that gets misquoted, truncated, and ultimately ignored.

---

## Specification

### 1. Version String

A TrustVer version string is a pure EffVer version:

```
MACRO.MESO.MICRO
```

**Examples:**

```
2.4.0
0.7.3
1.0.0
```

That's it. The version string contains effort semantics and nothing else. It is fully compatible with SemVer tooling, sorts correctly in every major package manager, and is trivially speakable by humans.

---

### 2. Effort Semantics

TrustVer adopts EffVer semantics:

- **MACRO** — Significant adoption effort expected. Architectural changes, breaking migrations, epoch-level shifts.
- **MESO** — Some effort required. Behavioral changes, deprecation removals, non-trivial adjustments to workflows.
- **MICRO** — No effort expected. Bug fixes, performance improvements, additive features that don't affect existing usage.

Each segment is a non-negative integer. Segments MUST increment numerically. The zero-version case (`0.X.Y`) follows EffVer conventions: `X` behaves as MACRO, `Y` as MESO.

EffVer's "over-bumping" convention applies: bumping a higher segment than strictly necessary (e.g., MESO for a purely additive feature) is acceptable to signal significance to users. Under-bumping (claiming MICRO for a change that actually requires effort) is a violation.

---

### 3. Provenance Attestation Document (PAD)

The PAD is a signed JSON document published alongside each release. It is the authoritative record of a release's trust posture. The PAD is **the core of TrustVer** — the version string is deliberately minimal so that the PAD can carry the real weight.

#### 3.1. PAD Structure

```json
{
  "trustver_spec": "0.2.0",
  "version": "2.4.0",
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

  "scope": "stable",

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

#### 3.2. Key Design Decisions

**Attestations are an append-only array.** This solves the Immutability Trap. The version `2.4.0` is immutable — it always refers to the same code. But the PAD's attestation list grows over time as new verification is applied. The release ships on day 1 with CI test results. A manual audit on day 7 appends a new attestation entry. A formal verification pass on day 30 appends another. Each attestation is individually signed by the attester, so the PAD is a verifiable audit trail, not a point-in-time snapshot.

**Artifact hashes live in the PAD, not the version string.** The PAD carries full SHA-256 hashes of release artifacts, providing the same content-addressable identity that was previously in the version string — but without polluting human-readable identifiers. Trust infrastructure pins to the PAD hash, not the version number.

**Authorship is a release-time claim.** Unlike attestations (which can be added post-release), the authorship tag is set at release time and is immutable. It reflects the predominant mode of production for the changeset. The detail object provides granularity; the tag provides a machine-readable summary.

---

### 4. Authorship Tags

| Tag | Meaning | Description |
|---|---|---|
| `h` | Human-authored | Code written entirely by human developers |
| `ai` | AI-generated | Code generated entirely by AI (any model/agent) |
| `hrai` | Human-reviewed AI | AI-generated code reviewed and approved by a human |
| `aih` | AI-assisted human | Human-authored code with AI assistance (copilot-style) |
| `auto` | Autonomous agent | Produced by an autonomous agent system, no human in the loop |
| `mix` | Mixed/indeterminate | Authorship cannot be cleanly categorized |

**Determination rules:**

- Authorship refers to the *predominant mode of production* for the changeset in this release.
- If >80% of changed lines originated from AI generation, use `ai` or `hrai` (depending on review).
- If authorship is roughly balanced or tooling cannot distinguish, use `mix`.
- When in doubt, use the *lower trust* tag. `mix` is always a safe default.

---

### 5. Attestation Types

Attestations are the verification records appended to a PAD over the release's lifetime. Each attestation is independently signed.

| Type | Meaning | Description |
|---|---|---|
| `formally-verified` | Formal proof | Mathematical proof of correctness (Coq, Lean, vericoding) |
| `test-verified` | Test suite | Passed automated test suite with defined coverage thresholds |
| `static-analysis` | SAST | Passed static analysis / linting without critical findings |
| `manual-audit` | Human audit | Human security or code audit performed |
| `code-review` | Review | Code review (human or AI-assisted) completed |
| `ci-passed` | CI pipeline | Passed CI pipeline (may include tests, but no explicit guarantees) |
| `pentest` | Penetration test | Security penetration testing performed |
| `sbom-verified` | SBOM validated | Software Bill of Materials generated and dependency provenance verified |
| `slsa-attested` | SLSA attestation | Meets a specified SLSA level (detail includes level) |

Attestation types are extensible. Custom types SHOULD use a namespaced format: `org.example.custom-check`.

**Each attestation MUST include:**

- `type` — One of the above or a namespaced custom type.
- `timestamp` — When the attestation was produced (ISO 8601).
- `attester` — Identity of the entity making the attestation.
- `signature` — Cryptographic signature over the attestation content.

**Each attestation SHOULD include:**

- `detail` — Structured data specific to the attestation type.

---

### 6. Scope Tags

| Tag | Meaning | Description |
|---|---|---|
| `stable` | Stable release | Production-ready, intended for general consumption |
| `rc` | Release candidate | Feature-complete, undergoing final validation |
| `preview` | Preview release | Functional but not production-ready |
| `experimental` | Experimental | May be unstable, incomplete, or exploratory |
| `sandbox` | Sandbox-only | Explicitly NOT for production; trust infrastructure SHOULD refuse to promote |

Scope is set at release time and is immutable. A release MUST NOT be re-scoped; instead, cut a new release at the appropriate scope.

---

### 7. PAD Discovery and Distribution

PADs MUST be discoverable by tooling. TrustVer defines three distribution mechanisms (implementors MUST support at least one):

#### 7.1. Sidecar File

Publish the PAD alongside the release artifact:

```
mylib-2.4.0.tar.gz
mylib-2.4.0.tar.gz.pad.json
```

#### 7.2. Well-Known Endpoint

For packages published to registries, the PAD SHOULD be discoverable at:

```
https://<registry-or-domain>/.well-known/trustver/<package>/<version>.pad.json
```

Example:

```
https://registry.npmjs.org/.well-known/trustver/mylib/2.4.0.pad.json
```

#### 7.3. Registry Metadata

Package registries MAY embed PAD data directly in their metadata APIs. For example, an npm registry could include a `trustver_pad` field in the package version metadata.

---

### 8. Trust Infrastructure Integration

#### 8.1. SchemaPin Integration

When a tool or API is versioned with TrustVer, the artifact hash in the PAD SHOULD be cross-referenced against the schema hash registered in SchemaPin's `.well-known/schemapin.json`. This provides independent verification that the artifact content matches the claimed schema.

#### 8.2. AgentPin Integration

When an AI agent publishes a release with authorship tag `ai` or `auto`, the agent SHOULD have a valid AgentPin credential verifiable via domain-anchored trust. The PAD's authorship detail SHOULD include the agent's AgentPin identity URI, enabling downstream consumers to verify the agent's identity independently.

#### 8.3. Symbiont Integration

Symbiont runtime policies reference PAD metadata to enforce zero-trust dependency management:

```yaml
# Example Symbiont policy
dependency_policy:
  require:
    - scope: stable
    - attestations:
        must_include:
          - test-verified
        must_include_one_of:
          - code-review
          - manual-audit
  
  deny:
    - authorship:
        tag: [auto, ai]
        unless_attested:
          - manual-audit

  warn:
    - authorship:
        tag: mix
        missing_attestation:
          - code-review
```

**Policy evaluation flow:**

1. Package manager resolves dependency to `mylib@2.4.0`.
2. Symbiont fetches the PAD for `mylib@2.4.0`.
3. Symbiont verifies PAD signatures.
4. Symbiont evaluates the PAD against the active policy.
5. Install proceeds, blocks, or warns based on policy outcome.

This enables a powerful default: **no dependency enters a production runtime unless it meets minimum provenance requirements, without requiring manual review of every update.**

---

### 9. Comparison with Existing Schemes

| Dimension | SemVer | CalVer | EffVer | HashVer | **TrustVer** |
|---|---|---|---|---|---|
| Version string simplicity | `1.2.3` | `2026.03` | `1.2.3` | `2026.03-a1b2c3` | **`1.2.3`** |
| Adoption effort signal | Indirect | None | Direct | None | **Direct (EffVer)** |
| Temporal signal | None | Primary | None | Date prefix | **PAD timestamp** |
| Source traceability | None | None | None | Git hash | **PAD (full artifact hash)** |
| Authorship metadata | None | None | None | None | **PAD** |
| Verification metadata | None | None | None | None | **PAD (append-only)** |
| Post-release attestation | N/A | N/A | N/A | N/A | **Yes** |
| Trust infrastructure integration | None | None | None | None | **SchemaPin/AgentPin/Symbiont** |
| Package manager compatibility | Native | Partial | Native | Partial | **Native** |
| Human speakability | Good | Good | Good | Poor | **Good** |

---

### 10. Adopting TrustVer

#### 10.1. Minimum Viable Adoption

A project can adopt TrustVer incrementally:

1. **Adopt EffVer semantics.** Use `MACRO.MESO.MICRO` with effort-based increment decisions. If you're already loosely following SemVer, this likely requires no version format changes — just a shift in how you decide which segment to bump.
2. **Generate PADs.** Start producing PAD files with your CI pipeline. Even a minimal PAD (version, artifact hash, authorship tag, CI attestation) is valuable.
3. **Sign PADs.** Add cryptographic signatures. Sigstore/cosign for CI-produced attestations, ECDSA keys for human attestations.
4. **Publish PADs.** Make them discoverable via sidecar files or well-known endpoints.
5. **Enforce policies.** Integrate with trust infrastructure (Symbiont or equivalent) to enforce PAD requirements on your own dependencies.

Each step is independently valuable. Step 1 is free. Step 2 adds transparency. Step 3 adds integrity. Step 4 enables ecosystem consumption. Step 5 closes the loop.

#### 10.2. Tooling

A `trustver` CLI tool SHOULD be provided for:

- Bumping version numbers with EffVer semantics.
- Generating PADs from build context (CI environment variables, git metadata, AI tool logs).
- Signing PADs (integration with Sigstore, local keys, or HSMs).
- Validating PAD structure and signatures.
- Appending attestations to existing PADs.
- Querying PAD metadata for a given package/version.

CI/CD integrations for GitHub Actions, GitLab CI, and similar platforms SHOULD be provided as thin wrappers around the CLI.

---

### 11. Security Considerations

- **PAD signatures are mandatory for enforcement.** Without signatures, a PAD is advisory only. Trust infrastructure MUST verify signatures before making policy decisions. An unsigned PAD is equivalent to no PAD.
- **Authorship tags are self-reported claims.** A malicious publisher can claim `h` (human-authored) on AI-generated code. The PAD's authorship detail provides evidence for auditing, but ultimately authorship verification depends on the integrity of the development environment. Automated authorship classification tooling (see §13) can reduce but not eliminate this risk.
- **Attestation freshness.** Attestations added long after release may reflect outdated analysis (e.g., a SAST scan against a tool version that has since been updated). Consumers SHOULD consider attestation timestamps relative to the release date.
- **PAD mutability scope.** Only the `attestations` array is append-only mutable. The `version`, `identity`, `authorship`, and `scope` fields are set at release time and MUST NOT be modified. Implementations SHOULD enforce this structurally (e.g., the release-time fields are covered by the release signature; attestations carry their own independent signatures).
- **Tag spoofing via PAD omission.** A project that adopts TrustVer but "forgets" to publish PADs for certain releases is effectively evading provenance tracking. Symbiont policies SHOULD treat a missing PAD as equivalent to the lowest trust posture (unknown authorship, no verification).

---

### 12. SLSA Alignment

TrustVer's PAD is designed to complement, not replace, existing supply-chain security frameworks. The following alignment points are defined:

- **SLSA Build Level 1:** A PAD with a `ci-passed` attestation and build provenance (`identity.build`) satisfies SLSA L1 requirements for build provenance.
- **SLSA Build Level 2:** A PAD signed by a hosted CI system (e.g., GitHub Actions with Sigstore) satisfies SLSA L2 for signed provenance.
- **SLSA Build Level 3:** A PAD with `identity.build.reproducible: true` and attestations from an isolated build environment approaches SLSA L3.

PADs MAY include a `slsa-attested` attestation type that explicitly declares the claimed SLSA level, with the detail object referencing the relevant SLSA provenance document.

---

### 13. Future Work

- **Automated authorship classification.** Tooling that integrates with IDE telemetry, AI API call logs, and git metadata to automatically determine authorship tags rather than relying on manual declaration.
- **Differential provenance.** Per-file or per-function provenance tracking within a single PAD, allowing mixed authorship and verification granularity (e.g., "the crypto module is human-authored and formally verified; the CLI wrapper is AI-generated and test-verified").
- **Provenance inheritance.** When a release depends on other TrustVer-versioned packages, computing the effective trust posture of the aggregate. The weakest link in the dependency chain determines the floor.
- **AI model fingerprinting.** Extending authorship metadata to identify specific models, agent configurations, or prompt chains that contributed to a release.
- **Registry-native PAD support.** Working with npm, PyPI, crates.io, and other registries to embed PAD metadata natively in their APIs and enforce PAD requirements at publish time.
- **PAD-aware dependency resolution.** Package manager plugins that factor provenance into dependency resolution — e.g., preferring a slightly older version with stronger attestations over the latest version with no PAD.

---

## Acknowledgments

TrustVer builds directly on the work of:

- **Jacob Tomlinson** — Intended Effort Versioning (EffVer)
- **Tom Preston-Werner** — Semantic Versioning (SemVer)
- **Mahmoud Hashemi** — Calendar Versioning (CalVer)
- **Taylor Brazelton / miniscruff** — Hash Versioning (HashVer)
- **The Sigstore project** — Supply chain attestation patterns
- **SLSA (Supply-chain Levels for Software Artifacts)** — Build provenance framework

---

## License

This specification is released under [CC-BY-SA 4.0](https://creativecommons.org/licenses/by-sa/4.0/). Implementations of TrustVer tooling are encouraged under any OSI-approved open source license.

---

*TrustVer is an independent open specification.*  
*Trust infrastructure integration examples reference the ThirdKey Trust Stack (https://thirdkey.ai).*
