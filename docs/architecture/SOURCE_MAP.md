# Source map and maintenance evidence

Companion to the [guide](README.md). This map identifies observed implementation,
normative design locations, and existing validation surfaces. Test links mean
**coverage to consult**, not tests executed by this documentation task.
[inventory.json](inventory.json) holds full SHAs, manifest/lock hashes, selected
resolved dependencies, and manifest patches. Package `explicit_binary_declarations`
records only explicit `[[bin]]` entries, not Cargo's automatically discovered targets
(such as xtask or auxiliary files in src/bin). It is not a complete executable inventory.

## Inspected checkouts

| Repository | Default branch | Source commit | Local packages |
| --- | --- | --- | --- |
| mnemonic-secret | master | [85efbd42](https://github.com/bg002h/mnemonic-secret/tree/85efbd42ea0e5a3da96af4d4a65c4b45ea338ccf) | ms-codec 0.10.0; ms-cli 0.20.1 |
| mnemonic-key | main | [bdb1df7f](https://github.com/bg002h/mnemonic-key/tree/bdb1df7f61e4eef94509e589a5d7b0d1f626cfd6) | mk-codec 0.5.0; mk-cli 0.14.0-dev |
| descriptor-mnemonic | main | [5dfb1fde](https://github.com/bg002h/descriptor-mnemonic/tree/5dfb1fde7dccf3664c6547b065169717452e00ae) | md-codec 0.48.4; md-cli 0.20.3; xtask 0.0.0 |
| mnemonic-toolkit | master | [ef99021a](https://github.com/bg002h/mnemonic-toolkit/tree/ef99021a26f733594ca6b711b9d3b82a494f39b1) | mnemonic-toolkit 0.105.1; wc-codec 0.1.1 |
| mnemonic-transaction | main | [56886b82](https://github.com/bg002h/mnemonic-transaction/tree/56886b826d1c47fdf7ee8246a4a78f5b768bae2d) | mt-codec 0.1.0; mt-cli 0.1.1 |
| mnemonic-engrave | master | [a7571999](https://github.com/bg002h/mnemonic-engrave/tree/a75719996304086e5f293aaf26812ff242a25244) | mnemonic-engrave 0.12.0; mnemonic-io-lib 0.1.0 |
| mnemonic-gui | master | [46e1c0f3](https://github.com/bg002h/mnemonic-gui/tree/46e1c0f31b5688c447309fdae3bcb439c43f7e4a) | mnemonic-gui 0.63.0 |

These are source baselines, before documentation commits. A worktree's later
documentation HEAD is not the source revision in this table. Repositories are
independent Git roots; scratch clones and duplicate worktrees were deduplicated
by Git common directory, not counted as additional products.

## What consumers actually resolve

| Consumer | Selected Cargo.lock resolutions |
| --- | --- |
| ms-cli | local ms-codec 0.10.0; mnemonic-io-lib 0.1.0 from me at 6c24e628 |
| mk-cli | local mk-codec 0.5.0; registry md-codec 0.42.0 and mnemonic-io-lib 0.1.0; registry miniscript 13.0.0 |
| md-cli | local md-codec 0.48.4; registry mk-codec 0.5.0 and mnemonic-io-lib 0.1.0; patched miniscript |
| mnemonic | ms-codec 0.7.0 and mk-codec 0.4.1 from registry; md-codec 0.47.0 at cf35d61a; local wc-codec 0.1.1; registry mnemonic-io-lib 0.1.0; patched miniscript |
| mt-cli | local mt-codec 0.1.0; mnemonic-io-lib 0.1.0 from me at 6c24e628 |
| me | md-codec 0.48.4 at 6cf6599a; ms-codec 0.10.0 at e5dff6f9; mt-codec 0.1.0 at 72b79b87; registry mk-codec 0.5.0; local mnemonic-io-lib; patched miniscript |
| mnemonic-gui | mnemonic-toolkit 0.105.1 via tag mnemonic-toolkit-v0.105.1 at 1402d547; its md0.47/mk0.4.1/ms0.7/wc0.1.1 dependencies; patched miniscript |

Full Git source identities are in the inventory's `resolved_packages`.
The miniscript patch is `rust-bitcoin/rust-miniscript` revision
`ff4732e5f75aa555682343cb180fa72ee3e8e9d5`. A local
`rust-miniscript-fork` clone was discovered, but those consumers currently use the
upstream Git URL. Do not replace the patch with that clone based on its directory name.

GUI binary expectations are separately recorded in
[pinned-upstream.toml](https://github.com/bg002h/mnemonic-gui/blob/46e1c0f31b5688c447309fdae3bcb439c43f7e4a/pinned-upstream.toml): mnemonic-toolkit-v0.105.1,
descriptor-mnemonic-md-cli-v0.20.3, ms-cli-v0.20.1, and mk-cli-v0.13.0.
The GUI's linked toolkit tag does not select the binaries found on the machine.

Resolved-code inspection anchors:
[toolkit's ms payload variants](https://github.com/bg002h/mnemonic-toolkit/blob/ef99021a26f733594ca6b711b9d3b82a494f39b1/vendor/ms-codec/src/payload.rs); [toolkit's mk random framing](https://github.com/bg002h/mnemonic-toolkit/blob/ef99021a26f733594ca6b711b9d3b82a494f39b1/vendor/mk-codec/src/key_card.rs); [toolkit's md wire4/8 decoder](https://github.com/bg002h/mnemonic-toolkit/blob/ef99021a26f733594ca6b711b9d3b82a494f39b1/vendor/md-codec/src/header.rs);
[mk's md wire4-only decoder](https://github.com/bg002h/mnemonic-key/blob/bdb1df7f61e4eef94509e589a5d7b0d1f626cfd6/vendor/md-codec/src/header.rs).
For offline builds, source replacement and provenance are governed by
[vendor-freshness machinery](https://github.com/bg002h/mnemonic-toolkit/blob/ef99021a26f733594ca6b711b9d3b82a494f39b1/ci/repro/vendor-freshness.sh); matching a package
version alone does not prove the vendored bytes came from the intended revision.

## Component reading routes

### ms: secret representation and recovery

Implementation: [exports](https://github.com/bg002h/mnemonic-secret/blob/85efbd42ea0e5a3da96af4d4a65c4b45ea338ccf/crates/ms-codec/src/lib.rs); [Payload](https://github.com/bg002h/mnemonic-secret/blob/85efbd42ea0e5a3da96af4d4a65c4b45ea338ccf/crates/ms-codec/src/payload.rs); [share reconstruction](https://github.com/bg002h/mnemonic-secret/blob/85efbd42ea0e5a3da96af4d4a65c4b45ea338ccf/crates/ms-codec/src/shares.rs); [hashlock primitives](https://github.com/bg002h/mnemonic-secret/blob/85efbd42ea0e5a3da96af4d4a65c4b45ea338ccf/crates/ms-codec/src/hashlock.rs); [CLI derivation](https://github.com/bg002h/mnemonic-secret/blob/85efbd42ea0e5a3da96af4d4a65c4b45ea338ccf/crates/ms-cli/src/cmd/derive.rs); [language selection](https://github.com/bg002h/mnemonic-secret/blob/85efbd42ea0e5a3da96af4d4a65c4b45ea338ccf/crates/ms-cli/src/cmd/payload_lang.rs); [repair candidate reporting](https://github.com/bg002h/mnemonic-secret/blob/85efbd42ea0e5a3da96af4d4a65c4b45ea338ccf/crates/ms-cli/src/cmd/repair.rs).
Dispatch recovered material by payload kind; the current share combine API returns
Tag::ENTR even for a Preimage. Keep separately supplied passphrases outside payload assumptions.

Design: [base format](https://github.com/bg002h/mnemonic-secret/blob/85efbd42ea0e5a3da96af4d4a65c4b45ea338ccf/design/SPEC_ms_v0_1.md); [mnemonic language](https://github.com/bg002h/mnemonic-secret/blob/85efbd42ea0e5a3da96af4d4a65c4b45ea338ccf/design/SPEC_ms_mnem_wordlist_language.md); [sharing](https://github.com/bg002h/mnemonic-secret/blob/85efbd42ea0e5a3da96af4d4a65c4b45ea338ccf/design/SPEC_ms_v0_2_kofn.md); [preimages](https://github.com/bg002h/mnemonic-secret/blob/85efbd42ea0e5a3da96af4d4a65c4b45ea338ccf/design/SPEC_ms_hashlock.md); [API/JSON migration history](https://github.com/bg002h/mnemonic-secret/blob/85efbd42ea0e5a3da96af4d4a65c4b45ea338ccf/MIGRATION.md).

Validation: [base vectors](https://github.com/bg002h/mnemonic-secret/blob/85efbd42ea0e5a3da96af4d4a65c4b45ea338ccf/crates/ms-codec/tests/vectors/v0.1.json); [hashlock vectors](https://github.com/bg002h/mnemonic-secret/blob/85efbd42ea0e5a3da96af4d4a65c4b45ea338ccf/crates/ms-codec/tests/vectors/hashlock-v0.8.json); [recovery regression](https://github.com/bg002h/mnemonic-secret/blob/85efbd42ea0e5a3da96af4d4a65c4b45ea338ccf/crates/ms-codec/tests/codex32_upstream_recovery_regression.rs); [independent hash reproduction](https://github.com/bg002h/mnemonic-secret/blob/85efbd42ea0e5a3da96af4d4a65c4b45ea338ccf/crates/ms-codec/tests/hashlock_repro.rs); [CLI variant coverage](https://github.com/bg002h/mnemonic-secret/blob/85efbd42ea0e5a3da96af4d4a65c4b45ea338ccf/crates/ms-cli/tests/hashlock_other_verbs.rs).

### mk: public key cards

Implementation: [KeyCard](https://github.com/bg002h/mnemonic-key/blob/bdb1df7f61e4eef94509e589a5d7b0d1f626cfd6/crates/mk-codec/src/key_card.rs); [compact xpub reconstruction](https://github.com/bg002h/mnemonic-key/blob/bdb1df7f61e4eef94509e589a5d7b0d1f626cfd6/crates/mk-codec/src/bytecode/xpub_compact.rs); [origin consistency checks](https://github.com/bg002h/mnemonic-key/blob/bdb1df7f61e4eef94509e589a5d7b0d1f626cfd6/crates/mk-codec/src/bytecode/encode.rs); [content IDs and chunks](https://github.com/bg002h/mnemonic-key/blob/bdb1df7f61e4eef94509e589a5d7b0d1f626cfd6/crates/mk-codec/src/string_layer/chunk.rs); [policy stub/membership integration](https://github.com/bg002h/mnemonic-key/blob/bdb1df7f61e4eef94509e589a5d7b0d1f626cfd6/crates/mk-cli/src/cmd/encode.rs).
The top-level decode returns a card without correction details; review lower-level
BCH APIs before promising correction reporting. Wire fields remain version zero
even though crate versions have advanced.

Design: [base format](https://github.com/bg002h/mnemonic-key/blob/bdb1df7f61e4eef94509e589a5d7b0d1f626cfd6/design/SPEC_mk_v0_1.md); [empty-path support](https://github.com/bg002h/mnemonic-key/blob/bdb1df7f61e4eef94509e589a5d7b0d1f626cfd6/design/SPEC_mk_no_path_support.md); [chunk-set ID rules](https://github.com/bg002h/mnemonic-key/blob/bdb1df7f61e4eef94509e589a5d7b0d1f626cfd6/design/SPEC_chunk_set_id_verification.md); [BIP draft](https://github.com/bg002h/mnemonic-key/blob/bdb1df7f61e4eef94509e589a5d7b0d1f626cfd6/bip/bip-mnemonic-key.mediawiki).

Validation: [vectors](https://github.com/bg002h/mnemonic-key/blob/bdb1df7f61e4eef94509e589a5d7b0d1f626cfd6/crates/mk-codec/src/test_vectors/v0.1.json); [external xpub oracle](https://github.com/bg002h/mnemonic-key/blob/bdb1df7f61e4eef94509e589a5d7b0d1f626cfd6/crates/mk-codec/tests/xpub_compact_external_oracle.rs); [deterministic IDs](https://github.com/bg002h/mnemonic-key/blob/bdb1df7f61e4eef94509e589a5d7b0d1f626cfd6/crates/mk-codec/tests/chunk_set_id_determinism.rs); [CLI ID verification](https://github.com/bg002h/mnemonic-key/blob/bdb1df7f61e4eef94509e589a5d7b0d1f626cfd6/crates/mk-cli/tests/csid_verification.rs).

### md: descriptors, identities, and seating

Implementation: [Descriptor and canonical encoding](https://github.com/bg002h/descriptor-mnemonic/blob/5dfb1fde7dccf3664c6547b065169717452e00ae/crates/md-codec/src/encode.rs); [DecodeOpts](https://github.com/bg002h/descriptor-mnemonic/blob/5dfb1fde7dccf3664c6547b065169717452e00ae/crates/md-codec/src/decode.rs); [sparse metadata](https://github.com/bg002h/descriptor-mnemonic/blob/5dfb1fde7dccf3664c6547b065169717452e00ae/crates/md-codec/src/tlv.rs); [identity types](https://github.com/bg002h/descriptor-mnemonic/blob/5dfb1fde7dccf3664c6547b065169717452e00ae/crates/md-codec/src/identity.rs); [SkeletonKey](https://github.com/bg002h/descriptor-mnemonic/blob/5dfb1fde7dccf3664c6547b065169717452e00ae/crates/md-codec/src/skeleton.rs); [composition](https://github.com/bg002h/descriptor-mnemonic/blob/5dfb1fde7dccf3664c6547b065169717452e00ae/crates/md-codec/src/compose/mod.rs); [concrete descriptor route](https://github.com/bg002h/descriptor-mnemonic/blob/5dfb1fde7dccf3664c6547b065169717452e00ae/crates/md-codec/src/descriptor_route.rs); [seating ambiguity](https://github.com/bg002h/descriptor-mnemonic/blob/5dfb1fde7dccf3664c6547b065169717452e00ae/crates/md-cli/src/seat/matching.rs); [stub confirmation levels](https://github.com/bg002h/descriptor-mnemonic/blob/5dfb1fde7dccf3664c6547b065169717452e00ae/crates/md-cli/src/seat/disposition.rs).

Design: [wire format](https://github.com/bg002h/descriptor-mnemonic/blob/5dfb1fde7dccf3664c6547b065169717452e00ae/design/SPEC_v0_30_wire_format.md); [wallet policy](https://github.com/bg002h/descriptor-mnemonic/blob/5dfb1fde7dccf3664c6547b065169717452e00ae/design/SPEC_v0_13_wallet_policy.md); [form conversion](https://github.com/bg002h/descriptor-mnemonic/blob/5dfb1fde7dccf3664c6547b065169717452e00ae/design/SPEC_wallet_form_converter.md); [seating](https://github.com/bg002h/descriptor-mnemonic/blob/5dfb1fde7dccf3664c6547b065169717452e00ae/design/SPEC_seat_auto_partition.md).
The [wire-8 design](https://github.com/bg002h/mnemonic-engrave/blob/a75719996304086e5f293aaf26812ff242a25244/design/SPEC_liana_unspendable_internal_key.md) is hosted in me; its implementation cycle belongs to md, with Rust-primary ordering.
Current source/tests establish implemented behavior; the design's historical status
is not evidence that every described gate or downstream implementation is complete.

Validation: [vector manifest](https://github.com/bg002h/descriptor-mnemonic/blob/5dfb1fde7dccf3664c6547b065169717452e00ae/crates/md-codec/src/test_vectors.rs); [wire8](https://github.com/bg002h/descriptor-mnemonic/blob/5dfb1fde7dccf3664c6547b065169717452e00ae/crates/md-codec/tests/wire_version_8.rs); [mint/recovery separation](https://github.com/bg002h/descriptor-mnemonic/blob/5dfb1fde7dccf3664c6547b065169717452e00ae/crates/md-codec/tests/mint_policy_does_not_reach_decode.rs); [partial origins](https://github.com/bg002h/descriptor-mnemonic/blob/5dfb1fde7dccf3664c6547b065169717452e00ae/crates/md-codec/tests/partial_decode.rs); [seating corpus](https://github.com/bg002h/descriptor-mnemonic/blob/5dfb1fde7dccf3664c6547b065169717452e00ae/crates/md-cli/tests/seating_vectors.rs); [verdict-table generation from recorded evidence](https://github.com/bg002h/descriptor-mnemonic/blob/5dfb1fde7dccf3664c6547b065169717452e00ae/crates/xtask/src/main.rs).

### toolkit and wc-codec

Implementation: [CLI module wiring](https://github.com/bg002h/mnemonic-toolkit/blob/ef99021a26f733594ca6b711b9d3b82a494f39b1/crates/mnemonic-toolkit/src/main.rs); [partial library API](https://github.com/bg002h/mnemonic-toolkit/blob/ef99021a26f733594ca6b711b9d3b82a494f39b1/crates/mnemonic-toolkit/src/lib.rs); [bundle](https://github.com/bg002h/mnemonic-toolkit/blob/ef99021a26f733594ca6b711b9d3b82a494f39b1/crates/mnemonic-toolkit/src/cmd/bundle.rs); [synthesis](https://github.com/bg002h/mnemonic-toolkit/blob/ef99021a26f733594ca6b711b9d3b82a494f39b1/crates/mnemonic-toolkit/src/synthesize.rs); [JSON contract](https://github.com/bg002h/mnemonic-toolkit/blob/ef99021a26f733594ca6b711b9d3b82a494f39b1/crates/mnemonic-toolkit/src/format.rs); [restore](https://github.com/bg002h/mnemonic-toolkit/blob/ef99021a26f733594ca6b711b9d3b82a494f39b1/crates/mnemonic-toolkit/src/cmd/restore.rs); [verification](https://github.com/bg002h/mnemonic-toolkit/blob/ef99021a26f733594ca6b711b9d3b82a494f39b1/crates/mnemonic-toolkit/src/cmd/verify_bundle.rs); [word-card adapter](https://github.com/bg002h/mnemonic-toolkit/blob/ef99021a26f733594ca6b711b9d3b82a494f39b1/crates/mnemonic-toolkit/src/word_card_adapter.rs); [word-card/RAID API](https://github.com/bg002h/mnemonic-toolkit/blob/ef99021a26f733594ca6b711b9d3b82a494f39b1/crates/wc-codec/src/lib.rs).

Design: [toolkit foundation](https://github.com/bg002h/mnemonic-toolkit/blob/ef99021a26f733594ca6b711b9d3b82a494f39b1/design/SPEC_mnemonic_toolkit_v0_3.md); [word-card design history](https://github.com/bg002h/mnemonic-toolkit/blob/ef99021a26f733594ca6b711b9d3b82a494f39b1/design/IMPLEMENTATION_PLAN_word_card_encoding.md).
Use the current implementation and newer amendments alongside the foundation spec.
Detailed explanations already live in the [technical manual](../technical-manual/README.md).

Validation: [full bundles](https://github.com/bg002h/mnemonic-toolkit/blob/ef99021a26f733594ca6b711b9d3b82a494f39b1/crates/mnemonic-toolkit/tests/cli_bundle_full.rs); [taproot restore](https://github.com/bg002h/mnemonic-toolkit/blob/ef99021a26f733594ca6b711b9d3b82a494f39b1/crates/mnemonic-toolkit/tests/cli_restore_taproot.rs); [template multisig](https://github.com/bg002h/mnemonic-toolkit/blob/ef99021a26f733594ca6b711b9d3b82a494f39b1/crates/mnemonic-toolkit/tests/cli_restore_md1_template_multisig.rs); [verification forensics](https://github.com/bg002h/mnemonic-toolkit/blob/ef99021a26f733594ca6b711b9d3b82a494f39b1/crates/mnemonic-toolkit/tests/cli_verify_bundle_forensics.rs); [exit tiers](https://github.com/bg002h/mnemonic-toolkit/blob/ef99021a26f733594ca6b711b9d3b82a494f39b1/crates/mnemonic-toolkit/tests/cli_verify_bundle_exit_tier.rs); [word-card CLI](https://github.com/bg002h/mnemonic-toolkit/blob/ef99021a26f733594ca6b711b9d3b82a494f39b1/crates/mnemonic-toolkit/tests/cli_word_card.rs); [roundtrip properties](https://github.com/bg002h/mnemonic-toolkit/blob/ef99021a26f733594ca6b711b9d3b82a494f39b1/crates/mnemonic-toolkit/tests/prop_backup_restore_roundtrip.rs).

### mt: format versus operational refusal

Implementation: [format API](https://github.com/bg002h/mnemonic-transaction/blob/56886b826d1c47fdf7ee8246a4a78f5b768bae2d/crates/mt-codec/src/lib.rs); [CLI](https://github.com/bg002h/mnemonic-transaction/blob/56886b826d1c47fdf7ee8246a4a78f5b768bae2d/crates/mt-cli/src/main.rs); [input sniffing](https://github.com/bg002h/mnemonic-transaction/blob/56886b826d1c47fdf7ee8246a4a78f5b768bae2d/crates/mt-cli/src/input.rs); [validation](https://github.com/bg002h/mnemonic-transaction/blob/56886b826d1c47fdf7ee8246a4a78f5b768bae2d/crates/mt-cli/src/validate.rs); [read-only node adapter](https://github.com/bg002h/mnemonic-transaction/blob/56886b826d1c47fdf7ee8246a4a78f5b768bae2d/crates/mt-cli/src/node.rs); [operator reporting](https://github.com/bg002h/mnemonic-transaction/blob/56886b826d1c47fdf7ee8246a4a78f5b768bae2d/crates/mt-cli/src/report.rs).

Design owner: [me's mt specification](https://github.com/bg002h/mnemonic-engrave/blob/a75719996304086e5f293aaf26812ff242a25244/design/SPEC_mt_v0_1.md).
Copy policy: [mt provenance](https://github.com/bg002h/mnemonic-transaction/blob/56886b826d1c47fdf7ee8246a4a78f5b768bae2d/design/PROVENANCE.md).
Independent vector source: [me generator](https://github.com/bg002h/mnemonic-engrave/blob/a75719996304086e5f293aaf26812ff242a25244/scripts/gen-mt1-vectors.py);
checked corpus: [mt1 vectors](https://github.com/bg002h/mnemonic-transaction/blob/56886b826d1c47fdf7ee8246a4a78f5b768bae2d/crates/mt-codec/src/test_vectors/mt1_v1.json).

Validation: [provenance](https://github.com/bg002h/mnemonic-transaction/blob/56886b826d1c47fdf7ee8246a4a78f5b768bae2d/scripts/check-provenance.sh); [refusal coverage](https://github.com/bg002h/mnemonic-transaction/blob/56886b826d1c47fdf7ee8246a4a78f5b768bae2d/scripts/check-refusal-coverage.sh); [mutation checks](https://github.com/bg002h/mnemonic-transaction/blob/56886b826d1c47fdf7ee8246a4a78f5b768bae2d/scripts/mutate-refusals.sh); [refusal manifest](https://github.com/bg002h/mnemonic-transaction/blob/56886b826d1c47fdf7ee8246a4a78f5b768bae2d/crates/mt-cli/tests/refusals.toml); [operator journeys](https://github.com/bg002h/mnemonic-transaction/blob/56886b826d1c47fdf7ee8246a4a78f5b768bae2d/scripts/journeys.sh); [optional live-node smoke](https://github.com/bg002h/mnemonic-transaction/blob/56886b826d1c47fdf7ee8246a4a78f5b768bae2d/scripts/live-smoke-test.sh).
Do not run live-node scripts merely to validate documentation.

### me and mnemonic-io-lib

Implementation: [library facade](https://github.com/bg002h/mnemonic-engrave/blob/a75719996304086e5f293aaf26812ff242a25244/crates/me-cli/src/lib.rs); [CLI orchestration](https://github.com/bg002h/mnemonic-engrave/blob/a75719996304086e5f293aaf26812ff242a25244/crates/me-cli/src/main.rs); [public bundle admission](https://github.com/bg002h/mnemonic-engrave/blob/a75719996304086e5f293aaf26812ff242a25244/crates/me-cli/src/bundle.rs); [sealed containers](https://github.com/bg002h/mnemonic-engrave/blob/a75719996304086e5f293aaf26812ff242a25244/crates/me-cli/src/seal/mod.rs); [descriptor intake](https://github.com/bg002h/mnemonic-engrave/blob/a75719996304086e5f293aaf26812ff242a25244/crates/me-cli/src/descriptor/mod.rs); [NDEF framing](https://github.com/bg002h/mnemonic-engrave/blob/a75719996304086e5f293aaf26812ff242a25244/crates/me-cli/src/ndef.rs); [systemwide record classes](https://github.com/bg002h/mnemonic-engrave/blob/a75719996304086e5f293aaf26812ff242a25244/crates/me-cli/src/sysw/record.rs); [systemwide wire](https://github.com/bg002h/mnemonic-engrave/blob/a75719996304086e5f293aaf26812ff242a25244/crates/me-cli/src/sysw/wire.rs); [sidecar protocol](https://github.com/bg002h/mnemonic-engrave/blob/a75719996304086e5f293aaf26812ff242a25244/crates/me-cli/src/preview.rs); [mechanism/policy boundary](https://github.com/bg002h/mnemonic-engrave/blob/a75719996304086e5f293aaf26812ff242a25244/crates/mnemonic-io-lib/src/lib.rs).

Design: [systemwide delivery](https://github.com/bg002h/mnemonic-engrave/blob/a75719996304086e5f293aaf26812ff242a25244/design/SPEC_systemwide_payloads.md); [encrypted delivery](https://github.com/bg002h/mnemonic-engrave/blob/a75719996304086e5f293aaf26812ff242a25244/design/SPEC_encrypted_payload_delivery.md); [descriptor intake](https://github.com/bg002h/mnemonic-engrave/blob/a75719996304086e5f293aaf26812ff242a25244/design/SPEC_descriptor_input.md); [shared CLI mechanisms](https://github.com/bg002h/mnemonic-engrave/blob/a75719996304086e5f293aaf26812ff242a25244/design/SPEC_constellation_cli_uniformity.md).

Validation: [codec seam](https://github.com/bg002h/mnemonic-engrave/blob/a75719996304086e5f293aaf26812ff242a25244/crates/me-cli/tests/codex32_seam.rs); [descriptor seam](https://github.com/bg002h/mnemonic-engrave/blob/a75719996304086e5f293aaf26812ff242a25244/crates/me-cli/tests/descriptor_seam.rs); [descriptor refusals](https://github.com/bg002h/mnemonic-engrave/blob/a75719996304086e5f293aaf26812ff242a25244/crates/me-cli/tests/descriptor_refusals.rs); [sysw CLI](https://github.com/bg002h/mnemonic-engrave/blob/a75719996304086e5f293aaf26812ff242a25244/crates/me-cli/tests/sysw_cli.rs); [seal CLI](https://github.com/bg002h/mnemonic-engrave/blob/a75719996304086e5f293aaf26812ff242a25244/crates/me-cli/tests/seal_cli.rs); [record corpus](https://github.com/bg002h/mnemonic-engrave/blob/a75719996304086e5f293aaf26812ff242a25244/crates/me-cli/tests/record_corpus.rs); [composer records](https://github.com/bg002h/mnemonic-engrave/blob/a75719996304086e5f293aaf26812ff242a25244/crates/me-cli/tests/sysw_composer_records.rs).

### utility GUI

Implementation: [CLI selection](https://github.com/bg002h/mnemonic-gui/blob/46e1c0f31b5688c447309fdae3bcb439c43f7e4a/src/app.rs); [RunPlan construction](https://github.com/bg002h/mnemonic-gui/blob/46e1c0f31b5688c447309fdae3bcb439c43f7e4a/src/form/invocation.rs); [process execution](https://github.com/bg002h/mnemonic-gui/blob/46e1c0f31b5688c447309fdae3bcb439c43f7e4a/src/runner.rs); [schema checks](https://github.com/bg002h/mnemonic-gui/blob/46e1c0f31b5688c447309fdae3bcb439c43f7e4a/src/schema_check.rs); [linked secret taxonomy](https://github.com/bg002h/mnemonic-gui/blob/46e1c0f31b5688c447309fdae3bcb439c43f7e4a/src/secrets.rs); [state persistence](https://github.com/bg002h/mnemonic-gui/blob/46e1c0f31b5688c447309fdae3bcb439c43f7e4a/src/persistence.rs); [structured output consumption](https://github.com/bg002h/mnemonic-gui/blob/46e1c0f31b5688c447309fdae3bcb439c43f7e4a/src/form/tree_form.rs); [descriptor spec model](https://github.com/bg002h/mnemonic-gui/blob/46e1c0f31b5688c447309fdae3bcb439c43f7e4a/src/form/tree_model.rs).

Validation: [schema mirror](https://github.com/bg002h/mnemonic-gui/blob/46e1c0f31b5688c447309fdae3bcb439c43f7e4a/tests/schema_mirror.rs); [pin coherence](https://github.com/bg002h/mnemonic-gui/blob/46e1c0f31b5688c447309fdae3bcb439c43f7e4a/tests/pin_coherence.rs); [taxonomy pin](https://github.com/bg002h/mnemonic-gui/blob/46e1c0f31b5688c447309fdae3bcb439c43f7e4a/tests/secret_taxonomy_pin.rs); [descriptor builder](https://github.com/bg002h/mnemonic-gui/blob/46e1c0f31b5688c447309fdae3bcb439c43f7e4a/tests/build_descriptor_schema.rs); [conditional drift](https://github.com/bg002h/mnemonic-gui/blob/46e1c0f31b5688c447309fdae3bcb439c43f7e4a/tests/gui_schema_conditional_drift.rs); [persistence/environment seam](https://github.com/bg002h/mnemonic-gui/blob/46e1c0f31b5688c447309fdae3bcb439c43f7e4a/tests/persistence_env_seam.rs).
See the [GUI manual](../manual-gui/README.md) for user-facing behavior.

## Cross-repo gates and instructions

Read repository CLAUDE.md explicitly, applicable ancestor guidance, and
[WORKFLOW.md](WORKFLOW.md); an AGENTS file is an entry point, not an automatic loader
for other instruction formats. mt has no local CLAUDE file at this baseline.

Executable CI is the authority for current invocations:
[toolkit Rust](https://github.com/bg002h/mnemonic-toolkit/blob/ef99021a26f733594ca6b711b9d3b82a494f39b1/.github/workflows/rust.yml),
[mt CI](https://github.com/bg002h/mnemonic-transaction/blob/56886b826d1c47fdf7ee8246a4a78f5b768bae2d/.github/workflows/ci.yml),
[me Rust/Go integration](https://github.com/bg002h/mnemonic-engrave/blob/a75719996304086e5f293aaf26812ff242a25244/.github/workflows/release.yml),
[GUI build](https://github.com/bg002h/mnemonic-gui/blob/46e1c0f31b5688c447309fdae3bcb439c43f7e4a/.github/workflows/build.yml),
[GUI schema](https://github.com/bg002h/mnemonic-gui/blob/46e1c0f31b5688c447309fdae3bcb439c43f7e4a/.github/workflows/schema-mirror.yml).
Core codec repositories also carry workflows and `ci/repro/vendor-freshness.sh`.
Keep debug assertions in relevant tests; use each repo's prescribed suite/toolchain.

Shared surfaces:
[manual mirror lint](https://github.com/bg002h/mnemonic-toolkit/blob/ef99021a26f733594ca6b711b9d3b82a494f39b1/docs/manual/tests/lint.sh); [shared CI drift](https://github.com/bg002h/mnemonic-toolkit/blob/ef99021a26f733594ca6b711b9d3b82a494f39b1/ci/shared-drift.sh); [sibling pins](https://github.com/bg002h/mnemonic-toolkit/blob/ef99021a26f733594ca6b711b9d3b82a494f39b1/.github/workflows/sibling-pin-check.yml); [cross-tool differential](https://github.com/bg002h/mnemonic-toolkit/blob/ef99021a26f733594ca6b711b9d3b82a494f39b1/.github/workflows/cross-tool-differential.yml); [GUI pin drift](https://github.com/bg002h/mnemonic-toolkit/blob/ef99021a26f733594ca6b711b9d3b82a494f39b1/.github/workflows/gui-pin-drift-check.yml); [released binary install pins](https://github.com/bg002h/mnemonic-toolkit/blob/ef99021a26f733594ca6b711b9d3b82a494f39b1/scripts/install.sh).
The docs-only CI classifier has conditions beyond path names. Missing/skipped checks
do not establish that documentation, runtime code, or a release passed validation.

## Firmware and Refugium handoff

Host preview uses [Go module replacement](https://github.com/bg002h/mnemonic-engrave/blob/a75719996304086e5f293aaf26812ff242a25244/preview/go.mod) and
[the SeedHammer submodule declaration](https://github.com/bg002h/mnemonic-engrave/blob/a75719996304086e5f293aaf26812ff242a25244/.gitmodules).
The recorded submodule commit is `7b6f2fbbdb909ef812ccc8c90768280c5fa7a4d4`.
Gemini's delivered firmware documentation is a separate snapshot:
[architecture](https://github.com/bg002h/mnemonic-engrave/blob/03776d704d2bf85a32f89c900b0a7368d06e0b5c/design/firmware-architecture/ARCHITECTURE.md),
[source map and Go provenance matrix](https://github.com/bg002h/mnemonic-engrave/blob/03776d704d2bf85a32f89c900b0a7368d06e0b5c/design/firmware-architecture/SOURCE_MAP.md#5-constellation-go-codec-provenance--update-procedure),
[host interfaces](https://github.com/bg002h/mnemonic-engrave/blob/03776d704d2bf85a32f89c900b0a7368d06e0b5c/design/firmware-architecture/HOST_INTERFACES.md),
and [open questions](https://github.com/bg002h/mnemonic-engrave/blob/03776d704d2bf85a32f89c900b0a7368d06e0b5c/design/firmware-architecture/OPEN_QUESTIONS.md).
Its [Gemini entry file](https://github.com/bg002h/seedhammer/blob/ec4fffddcd10b36409ae6d3dba9b8354f6367af2/GEMINI.md)
is in the firmware repository. These documentation commits are distinct from
me's preview gitlink and from the firmware source baseline they describe.

**Review status of these pinned submissions: corrections required.** Read the
[architecture review](https://github.com/bg002h/mnemonic-engrave/blob/7fa58f5e4c371b39ee543b0343a423418cc475b6/design/agent-reports/gemini-firmware-architecture-codex-review.md)
and [interface review](https://github.com/bg002h/mnemonic-engrave/blob/7fa58f5e4c371b39ee543b0343a423418cc475b6/design/agent-reports/gemini-firmware-host-interfaces-codex-review.md)
alongside them. They identify substantive errors in OTP/signing guidance, interface
contracts, commands, navigation and repository ownership. These links locate the
handoff; they do not endorse those disputed claims or copy its unverified codec-pin
summary into this host inventory. Refresh the handoff references after corrections
and review. Use the shared format specs and vector sources above when aligning
host/device behavior; do not infer device support from host acceptance or preview
rendering. Gemini retains ownership of the device guide and initialization file.

The [me manifest](https://github.com/bg002h/mnemonic-engrave/blob/a75719996304086e5f293aaf26812ff242a25244/crates/me-cli/Cargo.toml) and
[Refugium manifest](https://github.com/bg002h/mnemonic-refugium/blob/0e99b034c9ebb25a8cfa2a289e792ac840a43a2d/Cargo.toml) align md-codec at
`6cf6599ab71eb37adef836641095fb3c8756784f`.
This boundary-only inspection does not map Refugium internals.
The inspected [Refugium GUI workspace manifest](https://github.com/bg002h/refugium-wallet/blob/f081c7bea12c9328c118575b8fe0ecdfcf054997/Cargo.toml) and
[lockfile](https://github.com/bg002h/refugium-wallet/blob/f081c7bea12c9328c118575b8fe0ecdfcf054997/Cargo.lock) do not record a mnemonic-engrave dependency,
despite the me manifest comment describing such a consumer.
Keep that comment's intended relationship distinct from the observed graph.

## Refreshing and checking the map

Run `python3 docs/architecture/check.py` from the toolkit checkout with all seven
repositories as siblings, or pass `--workspace /path/to/siblings`. Python 3.11+ is
required. For complete source-link validation, also make boundary-only checkouts
`mnemonic-refugium` at `0e99b034c9ebb25a8cfa2a289e792ac840a43a2d` and
`refugium-wallet` at `f081c7bea12c9328c118575b8fe0ecdfcf054997` available as siblings
or under the workspace's parent. Their recorded Git objects must exist; their HEADs
need not match. The firmware documentation links additionally require a `seedhammer`
Git checkout containing `ec4fffddcd10b36409ae6d3dba9b8354f6367af2`, in either location.
The me Git database must contain the submission and review commits linked above.
The checker fails if recorded objects are unavailable; it does not fetch them.
The preview pin is still checked separately against me's recorded Git tree entry.

`inventory.json` records documentation snapshots and their allowed paths separately
from runtime source baselines. The toolkit snapshot is the standalone fallback;
keep its Git object locally for validation. Each repository also needs cached
`origin/HEAD` metadata matching its recorded default branch. That offline check
proves local consistency, not remote freshness. Source baselines must be ancestors
of the current checkout. See [workflow](WORKFLOW.md) for retaining or repinning
fallback commits during integration.

The checker reads Git objects and current documentation, never builds the
wallet utilities, fetches dependencies, edits source, or accesses real key material.

When adopting a new source baseline, inspect the changed manifests, lockfiles,
selected consumer code and contracts first. Update the inventory fields/hashes from
the chosen Git objects, then update prose and source links in the same change.
Do not merely move SHAs to make a check pass. The checker detects stale recorded
facts; architectural interpretation and deliberate pin alignment need review.
