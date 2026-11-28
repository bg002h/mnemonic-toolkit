# R2 review: `design/PLAN_export_wallet_coldcard_edge.md` (plan r2), scoped to the R1 I-A / I-B fold

Reviewer: Claude Opus 5.5 (R2, fold-scoped). Date 2026-11-28.
Plan: commit 16800b56 on `claude/project-thread-0411jd`. Code baseline: `origin/master` d72a18b1 (this checkout).
Firmware: the R0 copies of Coldcard `new_edge` @ `825ab7d` (scratch `ccfw-r0/ne/`), read as data.
Out of scope, as briefed: the R0 findings, the R1 Minors, and the `new_edge` facts.

**Verdict: GREEN. 0 Critical, 0 Important, 4 Minor.** I-A and I-B fold correctly. The r2 fold introduces no
Critical or Important defect. The four Minors below are wording or test-spec fixes and do not need a re-review.

---

## (1) Do I-A and I-B fold correctly?

### I-A: the checks run in the emitter on the parsed canonical descriptor. Fixed.

- **Can the emitter see the canonical descriptor?** Yes. `EmitInputs.canonical_descriptor: CheckedDescriptor<'a>`
  (`src/wallet_export/mod.rs:509`) derefs to `&str` with the `#csum`. Both export arms build it from the same
  `canonical` that the gate reads (`cmd/export_wallet.rs:803,807`; `:1080`). `parse_descriptor_lenient`
  (`src/parse_descriptor.rs:432`) is available to parse it back. Every input checks 1-6 need is in that text:
  - key origins (fp + path), the xpub (depth, parent fp, child number, own fingerprint), the suffix;
  - the internal key versus leaf keys, from `Descriptor::Tr`.

  `--taproot-internal-key nums` renders as the bare `NUMS_XONLY_HEX` (`wallet_export/pipeline.rs:141-143`), a
  `Single` key, so check 1 refuses it with no special case, as r2 says.
- **Does `every_format_meets_the_same_gate_before_its_own_verdict` stay true?** Yes. `emit_payload`
  (`cmd/export_wallet.rs:136`) is called only after `export_admission_gate` on both export arms (`:803`→`:830`,
  `:1080`→`:1112`). Its `collect_missing` match and the emitter both sit downstream of the gate. The test
  (`tests/cli_export_wallet_allow.rs:941-975`) drives `--descriptor <sigless wsh>`. A `coldcard-edge` refusal inside
  the emitter is never reached there, so the gate's exit 2 comes first. Adding `coldcard-edge` to `FORMATS`
  (12 → 13, §4) makes the test assert this rather than assume it.
- **Is the depth-0 rule right?** Yes. At depth 0 Edge asserts `swab32(node.my_fp()) == xfp`, "master xfp mismatch"
  (`ne/shared/desc_utils.py:318-322`). It does so unconditionally, whether or not a derivation is present. r2's
  "at depth 0 fp == the xpub's own fingerprint" matches that. The depth-1 rule (`:323-325`) also matches.
  Labelling caveat in M-1.
- **Is anything still checked on `ResolvedSlot`?** No check reads it. The "Where" paragraph forbids it for checks
  1-6, and check 2 drops the raw-`SlotInput` read. Because an omitted `.fingerprint=` becomes `00000000`
  (`cmd/bundle.rs:641-646`), a non-zero test on the canonical origin covers both the missing and the zero case. The
  only remaining tie to slots is wording: check 2 names the key "by its `@N` (template path)". Naming only, see M-3.

### I-B: the restore and `--from-import-json` refusals are placed where every route passes. Fixed.

- **Is "top of `restore::run`" before both the md1 dispatch and the single-sig path?** Yes. `restore::run` starts at
  `cmd/restore.rs:312`. Its first statement is the `if !args.md1.is_empty()` block (`:340-375`), which fans out to
  three routes:
  - `run_singlesig_template_completion` (`:347`), which already refuses `--format` at `:849`;
  - `run_multisig_template_completion` (`:372`);
  - `run_multisig` (`:375`), which reaches `emit_payload` via `:4337`→`:3582`.

  The single-sig code follows from `:377`. Its `--format` gate is at `:421` and it reaches `emit_payload` via
  `:665`→`:3257`. A refusal placed before `:340` therefore precedes all four restore routes. Those include the two
  that reach `emit_payload`, as §3.1 states.
- **Is "top of `run_from_import_json`" reachable before any `emit_payload` call?** Yes. `run` dispatches to it as its
  first action (`cmd/export_wallet.rs:506-508`), before any slot, template or descriptor work. Inside
  `run_from_import_json` (`:853`) the only `emit_payload` call is `:1112`, after gate 2 (`:1080`). A refusal at
  function entry precedes the Fix-α taproot refusal (`:961-973`), `format_requires_template` (`:1048`) and
  `emit_payload`. The wsh-envelope hole is closed.
- Row 13 is split as R1 asked. `restore_md1_multisig_refuses_coldcard_edge` with a NUMS md1 card pins the C2 path.

---

## (2) New defects introduced by r2

No Critical. No Important.

### Minor

- **M-1. Check 3's length rule is labelled "Edge E8", but Edge exempts an empty derivation.** Edge checks
  `len(derivation) == depth` only `if self.origin.derivation` ("we now allow blinded keys that have depth X but
  derivation len is 0", `ne/shared/desc_utils.py:312-317`). On `--descriptor`, `[fp]xpub` with depth 4 and an empty
  path satisfies r2's check 2 (an origin is present), passes Edge, and fails r2's check 3. The behaviour is
  fail-closed and fine.
  *Fix:* label "origin length == depth for an empty path" as toolkit policy. Alternatively, say in check 2 that the
  origin must carry a path, which makes the policy explicit. (On the template path an empty path never reaches the
  text, `pipeline.rs:46-62`.)
- **M-2. The new row `…_refuses_depth0_fingerprint_mismatch` can pass for the wrong reason.** On the template path,
  a depth-0 xpub always gets the 3-component fallback origin (`pipeline.rs:46-62`), so the length rule refuses it
  before the depth-0 rule is reached.
  *Fix:* specify `--descriptor` with `[<wrong fp>]<depth-0 xpub>/<0;1>/*` (empty path, so the length check passes).
  Assert the master-fingerprint wording, not just exit 1. Also say that the `child_number` clause applies at
  depth ≥ 1 only: at depth 0 there is no last element.
- **M-3. Check 2's "@N (template path)" naming needs a key→slot mapping that the "Where" paragraph does not give.**
  The canonical text does not carry slot indices. The internal key is lifted out of the leaf, and leaf order follows
  slot order minus `@N`. Matching keys by xpub against `inputs.resolved_slots` is ambiguous in exactly check 4's
  case (the same xpub under two fps).
  *Fix:* derive `@N` positionally from the template's construction (internal = `inputs.taproot_internal_key`, leaf
  = remaining slots in order). Otherwise name keys by `[fp/path]` on both paths. Either reads `ResolvedSlot` only
  for wording, never for a verdict.
- **M-4. The "Where" paragraph places §3.2 checks 1-6 but not §3.1's shape admission.** If an implementer places
  the `--descriptor`/`--template` shape refusal in `run` before `:803`, exit 1 would precede the gate's exit 2.
  The 13-entry `FORMATS` list now catches this, so it is self-detecting.
  *Fix:* say "§3.1's export shape admission also runs in the emitter". The restore and `--from-import-json` refusals
  stay at their function tops.

### Noted, not new (outside the r2 fold, not re-derived)

§3.3's "only an explicit `--wallet-name` selects JSON" has no input in `EmitInputs` today. The struct carries only
`wallet_name_is_non_default`, which is `args.wallet_name.is_some() || bip388_policy_name.is_some()`
(`cmd/export_wallet.rs:819`). That conflates an explicit name with a name lifted from a BIP-388 `--descriptor`
(`:594`). The build must add a field such as `wallet_name_user_supplied`, set in both `EmitInputs` constructions.
This is a one-line note for the build, not a plan defect.

---

## Result

GREEN (0C/0I). The loop closes. Fold M-1 to M-4 as wording and test spec without re-review.
