# Open questions and documentation discrepancies

These are source-backed follow-ups at the [recorded baseline](inventory.json).
They do not authorize dependency, format, or workflow changes. The owner names are
repositories/roles, not people. Resolve a question by linking the deciding change.

| Item / owner | Evidence | Consequence and resolution |
| --- | --- | --- |
| Consumer compatibility — mk + md | [mk's resolved md0.42](https://github.com/bg002h/mnemonic-key/blob/bdb1df7f61e4eef94509e589a5d7b0d1f626cfd6/vendor/md-codec/src/header.rs) accepts wire4; [current md](https://github.com/bg002h/descriptor-mnemonic/blob/5dfb1fde7dccf3664c6547b065169717452e00ae/crates/md-codec/src/header.rs) accepts4/8 | Decide when mk should adopt the newer md dependency and required miniscript patch. Until then, retain the explicit compatibility limit; a sibling version bump is insufficient. |
| Toolkit dependency lag — toolkit | [toolkit lock](https://github.com/bg002h/mnemonic-toolkit/blob/ef99021a26f733594ca6b711b9d3b82a494f39b1/Cargo.lock) versus [checkout versions](SOURCE_MAP.md) | ms0.7 lacks the current Preimage variant and mk0.4.1 retains random framing. Decide upgrades per contract; this guide does not silently change pins. |
| Cross-repo wire8 design link — md / me | The [design is hosted in me](https://github.com/bg002h/mnemonic-engrave/blob/a75719996304086e5f293aaf26812ff242a25244/design/SPEC_liana_unspendable_internal_key.md); its implementation cycle belongs to md. The bare local design path is absent in the md checkout. | Location resolved by this guide. Update older local references to a durable cross-repo link when maintaining them. Do not infer gate completion from the design document's existence. |
| me consumer claim — me / Refugium boundary owner | [me comment](https://github.com/bg002h/mnemonic-engrave/blob/a75719996304086e5f293aaf26812ff242a25244/crates/me-cli/Cargo.toml) describes a Refugium app linking both libraries; [inspected GUI lock](https://github.com/bg002h/refugium-wallet/blob/f081c7bea12c9328c118575b8fe0ecdfcf054997/Cargo.lock) has no me dependency | Treat direct me consumption as intended/unverified. Shared md pin alignment is observed. Confirm the actual consumer/version before drawing a runtime edge. |
| Early-stage descriptions — ms / mk maintainers | [ms guidance](https://github.com/bg002h/mnemonic-secret/blob/85efbd42ea0e5a3da96af4d4a65c4b45ea338ccf/CLAUDE.md), [implemented shares](https://github.com/bg002h/mnemonic-secret/blob/85efbd42ea0e5a3da96af4d4a65c4b45ea338ccf/crates/ms-codec/src/shares.rs); [mk early-stage comments](https://github.com/bg002h/mnemonic-key/blob/bdb1df7f61e4eef94509e589a5d7b0d1f626cfd6/crates/mk-codec/src/lib.rs), [retired shared-crate plan](https://github.com/bg002h/mnemonic-transaction/blob/56886b826d1c47fdf7ee8246a4a78f5b768bae2d/design/PROVENANCE.md) | Correct historical architecture/status prose when maintaining those documents. Preserve applicable process rules; do not revive the retired shared BCH crate plan. |
| Word-card scope comment — toolkit | [wc-codec API](https://github.com/bg002h/mnemonic-toolkit/blob/ef99021a26f733594ca6b711b9d3b82a494f39b1/crates/wc-codec/src/lib.rs) exports RAID despite its introductory future-work wording | Align the comment with the implemented API; the guide maps the exports. |
| GUI output interpretation — utility GUI | [README description](https://github.com/bg002h/mnemonic-gui/blob/46e1c0f31b5688c447309fdae3bcb439c43f7e4a/README.md) versus [JSON result handlers](https://github.com/bg002h/mnemonic-gui/blob/46e1c0f31b5688c447309fdae3bcb439c43f7e4a/src/form/tree_form.rs) | Update the broad no-output-interpretation description. Preserve tests for emitted specifications and validation JSON, not only flag names. |

## Maintenance/process questions

Existing CLAUDE files include dated workflow and model-routing instructions, and mt's
provenance document contains a historical statement about branch protection.
[WORKFLOW.md](WORKFLOW.md) preserves reading routes and inherited requirements.
This documentation task does not rewrite those policies, treat old remote-protection
claims as current, or claim a Codex review fulfills an Opus-specific code-release gate.

The core manuals, flags, GUI mirrors, package releases and actual installed binaries
advance separately. A future coordinated release should establish its own compatible
matrix from exact artifact revisions. This guide's snapshot is source evidence,
not a release certification.

Firmware ownership remains separate. The delivered guide, provenance matrix,
initialization file and correction reports are connected in the
[firmware handoff](SOURCE_MAP.md#firmware-and-refugium-handoff). The linked submission
still needs factual corrections; its port-pin summary is not imported as verified
inventory. The firmware owner should update these pinned links after the corrections
are reviewed. Device parsing, rendering, physical engraving and recovery on hardware
remain unvalidated by this host-only documentation work.
