 I have completed an independent review of GPT-6 Astra’s constellation architecture delivery across all seven repositories under
  /scratch/code/shibboleth/architecture-worktrees/ on branch docs/constellation-architecture, as well as the installed workspace root AGENTS.md.
  ──────
  ### Overall Assessment

  Verdict: Approve (0 Critical · 0 Important · 4 Minor / Integration Notes).

  Astra’s documentation suite is thorough, well-structured, and evidence-backed:

  • Word count & structure: README.md is 1,831 words (within the 1,500–2,500 word target), includes a clear Mermaid dependency diagram, traces backup/recovery,
  word-card, signed-transaction, host-delivery, and GUI flows, and provides a change-ownership table.
  • Checkout vs. consumer lockfile separation: SOURCE_MAP.md:11-57 and inventory.json cleanly distinguish repository HEAD package versions (ms-codec 0.10.0, mk-
  codec 0.5.0, md-codec 0.48.4) from what downstream consumers actually resolve in Cargo.lock (e.g., mnemonic-toolkit resolving ms-codec 0.7.0, mk-codec 0.4.1,
  md-codec 0.47.0 at cf35d61a; mk-cli resolving registry md-codec 0.42.0, which accepts wire v4 only).
  • Discrepancy capture: OPEN_QUESTIONS.md accurately documents real code-vs-doc discrepancies without inventing unauthorized requirements (including the
  crates/me-cli/Cargo.toml comment claiming a Refugium GUI links mnemonic_engrave vs. refugium-wallet/Cargo.lock having no mnemonic-engrave dependency, wc-codec
  exporting RAID despite its stale top-of-file comment, and mnemonic-gui/README.md claiming no CLI output parsing despite src/form/tree_form.rs consuming
  structured JSON).
  • Agent entry points: All seven repository AGENTS.md files (AGENTS.md, AGENTS.md, AGENTS.md, AGENTS.md, AGENTS.md, AGENTS.md, AGENTS.md) are compact (< 2.5
  KiB each), explicitly instruct agents to load CLAUDE.md and WORKFLOW.md, and handle mnemonic-transaction’s lack of a local CLAUDE.md.
  ──────
  ### Independent Verification Executed

  1. Static architecture checker (check.py):
      • Ran python3 docs/architecture/check.py --workspace /scratch/code/shibboleth/architecture-worktrees → PASS: {"gitlinks": 1, "hashes": 35, "links": 278,
      "markdown": 13, "repositories": 7}.
  2. CI code-docs-check.sh guard compatibility:
      • Ran bash ci/code-docs-check.sh in all six repositories that carry the guard (mnemonic-secret, mnemonic-key, descriptor-mnemonic, mnemonic-toolkit,
      mnemonic-transaction, mnemonic-engrave) → all 6 exited 0 with 0 hits.
  3. Git state & remote ref parity:
      • Confirmed all seven worktrees in /scratch/code/shibboleth/architecture-worktrees/ are clean on docs/constellation-architecture and match
      origin/docs/constellation-architecture and DELIVERY.md:16-24.
      • Confirmed /scratch/code/shibboleth/AGENTS.md is byte-for-byte identical to WORKSPACE_AGENTS.md.

  ──────
  ### Findings (0 Critical · 0 Important · 4 Minor)

  #### M1 — SOURCE_MAP.md:114-121 and AGENTS.md:18-28 omit implementation links for seal, ndef, and descriptor

  • Evidence: README.md:135-139 and AGENTS.md:23-24 emphasize that me converts admitted text to NDEF, validates supported descriptors, and keeps seal (MNEMBLOB)
  and sysw (MNEMSYSW) container formats separate. SOURCE_MAP.md:118-120 links the design specs (SPEC_encrypted_payload_delivery.md, SPEC_descriptor_input.md)
  and integration tests (tests/seal_cli.rs, tests/descriptor_seam.rs), and links crates/me-cli/src/sysw/record.rs and crates/me-cli/src/sysw/wire.rs under
  Implementation:, but omits the corresponding implementation entry points for:
      • Encrypted containers: crates/me-cli/src/seal/mod.rs (or seal/record.rs and seal/wire.rs)
      • Descriptor intake: crates/me-cli/src/descriptor/mod.rs
      • NDEF framing: crates/me-cli/src/ndef.rs
  • Recommendation: Add crates/me-cli/src/seal/mod.rs, crates/me-cli/src/descriptor/mod.rs, and crates/me-cli/src/ndef.rs to the me and mnemonic-io-lib
  implementation line in SOURCE_MAP.md:116 (and optionally link crates/me-cli/src/seal/mod.rs alongside sysw/record.rs in AGENTS.md:23-24).

  #### M2 — check.py:74-177 has minor static-validation blind spots (default_branch, blob/tree kind on branch links, and directory #fragment links)

  • Evidence:
      1. In check_inventory, row["default_branch"] from inventory.json is never validated, and row["source_commit"] is checked only with git cat-file -e
      <rev>^{commit} rather than verifying git merge-base --is-ancestor <rev> HEAD.
      2. In check_link, links targeting documentation_branch (https://github.com/bg002h/<name>/<kind>/docs/constellation-architecture/...) check only path.
      exists() in the working tree: unlike commit-SHA links (lines 162–163), they do not verify kind (blob vs. tree against path.is_file() / path.is_dir()) or
      that path is tracked in Git.
      3. In check_link and lines 175–176, if parsed.fragment and path.is_file(): silently passes if a #fragment is attached to a directory link.
  • Recommendation: Tighten check_link to require path.is_file() when kind == "blob" (or when parsed.fragment is present) and path.is_dir() when kind == "tree",
  and optionally verify git merge-base --is-ancestor <source_commit> HEAD in check_inventory.

  #### M3 — Post-merge branch-deletion hazard for docs/constellation-architecture fallback URLs

  • Evidence: Six repository AGENTS.md files (line 10 in mnemonic-secret, mnemonic-key, descriptor-mnemonic, mnemonic-transaction, mnemonic-engrave, mnemonic-
  gui) and WORKSPACE_AGENTS.md:11-12 link to https://github.com/bg002h/mnemonic-toolkit/blob/docs/constellation-architecture/docs/architecture/{README,
  WORKFLOW}.md. WORKFLOW.md:24-26 and DELIVERY.md:39 note that these links should be updated when integrating to default branches. However, per RULES.md:141-144,
  head branches are normally auto-deleted immediately upon PR merge. Furthermore, check.py:191-192 hardcodes require(inventory["documentation_branch"] ==
  "docs/constellation-architecture", ...).
  • Recommendation: When integrating these seven branches into master/main:
      1. Either keep docs/constellation-architecture alive on bg002h/mnemonic-toolkit until all seven PRs have switched their fallback URLs to blob/master/...,
      or update the fallback URLs in the six sibling AGENTS.md files, WORKSPACE_AGENTS.md:11-12, inventory.json:4 ("documentation_branch": "master"), and
      check.py:191 in the integration fold before merging.


  #### M4 — Connecting the delivered Firmware Architecture handoff (OPEN_QUESTIONS.md:30-33, SOURCE_MAP.md:149-158)

  • Evidence: OPEN_QUESTIONS.md:30-31 notes: "Once Gemini's map is delivered, connect its entry point and provenance matrix here without duplicating the device
  architecture."
  • Handoff coordinates now available for cross-linking:
      • mnemonic-engrave branch docs/firmware-architecture (commit 03776d704d2bf85a32f89c900b0a7368d06e0b5c, based on the same mnemonic-engrave source baseline
      a75719996304086e5f293aaf26812ff242a25244):
          • Entry point: ARCHITECTURE.md
          • Firmware source map & Go codec provenance matrix (§4): SOURCE_MAP.md
          • Host-to-device contracts (MNEMBLOB, MNEMSYSW, NDEF, UR, me-preview sidecar, UF2/OTP): HOST_INTERFACES.md
          • Firmware open questions & drift: OPEN_QUESTIONS.md
      • seedhammer branch docs/gemini-init (commit ec4fffddcd10b36409ae6d3dba9b8354f6367af2, based on seedhammer origin/main
      abdcb1825ed3bde5e01ff0e4da33f76f08f604c2):
          • Gemini initialization file: GEMINI.md
      • Notable cross-boundary finding from the firmware audit that complements Astra’s consumer-pin table:
          • Just as mnemonic-toolkit lags sibling codec HEADs in Cargo.lock, seedhammer (origin/main at abdcb182) pins Go ports at specific upstream Rust
          commits (md at md-codec v0.48.3 1a88c584 + fe4fa311; mk at mk-codec v0.5.0 480366c; codex32/hashlock at ms-codec v0.8.0 68400fb; mt at mt-codec v0.1.0
          72b79b8; seal vectors at me 4df43d6; sysw vectors at me 83b9e82), while mnemonic-engrave’s third_party/seedhammer submodule (7b6f2fbb) lags seedhammer
          origin/main by 24 commits (F-213).
