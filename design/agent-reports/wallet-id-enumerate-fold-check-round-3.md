FOLD: 10 ISSUES (0 fully-unaddressed Critical/Important; 1 Important PARTIAL; 8 Minor/Nit
non-blocking carryovers; 1 new Minor)

Scope: mechanical fold verification only. Report `wallet-id-enumerate-implementability-lens.md`
(persisted 4dc4268f) vs fold commit `fdf4fd5f` vs current
`design/SPEC_restore_wallet_id_prefix_enumerate.md`. `bash scripts/spec-citation-gate.sh` run:
**GATE: PASS** — 31 citation rows + 6 numeric rows = 37 assertions, matching the report's stated
count; the 8 new rows added by this fold (restore.rs:2377/1436/1793/220/2231/1954,
permutation_search.rs:848/862) all resolved. Not re-hand-verified beyond that per instructions.

## Structural checks (brief items 1-3, 5, 6)

1. **§3 numbering.** `### 3.4a` is inserted as a heading between list item 4 (Exit codes) and
   item 5 (Output cap), splitting the CommonMark ordered list in two. Because item "5." is typed
   explicitly, both GitHub-flavored and standard CommonMark renderers take the first item's own
   number as the list's `start`, so items 5-8 render 5,6,7,8 correctly. No numbering defect.
2. **Cross-references.** Grepped every `§3.[0-9a-z]` occurrence in the file: all `§3.3/§3.4/§3.5/
   §3.6/§3.7/§3.8` references still point at their original list items (unaffected by the heading
   insertion), and every new `§3.4a(x)` reference points at the correct new subsection. No
   reference breakage.
3. **§3.4a "normative over looser phrasing" vs. §3.3/§8 stream table.** §3.4a(b) says: candidate
   rows -> stdout/`candidates[]`; summary line -> stderr/mirrored fields; §3.8 warning ->
   stderr/`warning`. §3.3's own paragraph (unchanged) says the summary line is "on stderr (§8)".
   §8's resolved question says "stdout or stderr for the summary hint? stderr". All three agree —
   refinement, not contradiction.
5. **§3.4a(d) rows-not-groups vs. §3.3 never-merge vs. §3.5 cap-before-address vs. §3.4a(e)
   one-address-per-row — composed into one routine.** Buildable: (i) classify by raw match count
   (§3.3, L4 fix); (ii) sort; (iii) cap to 64 **rows** (§3.5, §3.4a(d) — no groups exist to cap,
   since merging is now forbidden); (iv) compute the sortedmulti annotation from the static
   `is_order_independent_shape(&d.tree)` flag (verified at `restore.rs:1954` — a property of the
   descriptor tree, not of derived addresses), so no address work is needed before the cap; (v)
   derive exactly one first-address per surviving row (§3.4a(e), ignoring `--count`). No
   composition conflict remains — the L5 fix (annotate from the shape flag, never from addresses)
   is what makes this buildable.
6. **Claims the citation gate cannot see, spot-checked against source directly (not just the
   gate):** `restore.rs:1400-1440` confirms the single call site into `emit_completed_multisig`
   from all three modes; `restore.rs:1954` confirms `is_order_independent_shape` is a cheap
   tree-shape check, not an address comparison; `restore.rs:2216-2231`'s `map_search_error`
   confirms `SearchTimeExceedsCeiling` really does route through `bad()` (exit 1), matching
   §3.4a(f) — note the *doc comment* directly above that function (line 2216-2218, "Floor errors
   (duplicate keys, weak prefix, time ceiling) are funds-safety refusals (exit 4)") is itself
   stale/wrong about the ceiling case, which is exactly the confusion L7 was about; this is a
   pre-existing code-comment defect, not something the fold's spec text introduced or needs to
   fix — out of scope here, not double-counted below.

## L1-L9 disposition

- L1 (Critical) — ADDRESSED, §3.4a(a) + §5. `uniqueness_proven` scoped to id-search only, table
  gives all four modes, "absence is not true" stated. §5 requires the enum to carry the mode
  discriminant so the emitter can tell modes apart. Confirmed no contradiction with the original
  §3.4 exit-code table (item 4): that table's "reconstructed, prefix ≥/< threshold" rows only ever
  described the id-search shape (address-search/explicit `@N=` have no "prefix" concept), so
  §3.4a(a) refines rather than contradicts it.
- L2 (Critical) — ADDRESSED, §3.4a(d). `truncated: true` + `match_count` as the true total, both
  channels; new vector 9 guards it.
- L3 (Important) — ADDRESSED, §3.4a(b). Full stream table for every line; agrees with §3.3 and §8
  (see structural check #3 above).
- L4 (Important) — ADDRESSED, §3.3 (amended). "Classification is by MATCH COUNT, never by
  distinct wallets"; "Never merge rows — annotate the set." Resolves the classify-vs-group fork
  definitively in favor of classify-first, never-merge.
- L5 (Important) — ADDRESSED, §3.3 (amended) + §3.4a(d). Annotation is now derived from the static
  `is_order_independent_shape` flag, not from comparing derived addresses, so it no longer
  conflicts with "cap before deriving addresses." §3.4a(d) confirms the cap counts rows, not
  groups (moot now that groups don't exist as a data structure, only as a one-line annotation).
- L6 (Important) — ADDRESSED, §3.4a(e). `--count` applies only to the reconstructed-wallet path;
  a candidate row always shows exactly one address.
- L7 (Important) — ADDRESSED, §3.4a(f). Before/after exit-code table (4 -> 1 for the ceiling case)
  plus the message requirement (name the prefix length, not just the time). Verified against
  source: `map_search_error` really does map `SearchTimeExceedsCeiling` through `bad()` (exit 1),
  distinct from `PrefixTooShort`'s `RestoreMismatch` (exit 4). Minor process note (non-blocking):
  the new row lives in §3.4a(f)'s own table rather than as a literal added row in §3.4's original
  exit-code table; since §3.4a immediately follows §3.4 and is stated normative, this does not
  create a live ambiguity, just a slightly different location than the report's suggested remedy
  (remedies are not authoritative; the underlying defect — no exit-code documentation for this
  case — is fixed).
- L8 (Important) — ADDRESSED, §3.4a(c). Full JSON shape with a worked example; explicit "No
  `candidates[].descriptor` field, ever"; explicit mutual exclusion of `wallets`/`candidates`;
  the worked example omits top-level `wallet_policy_id`/`own_position` for the list case,
  answering the report's third sub-fork.
- L9 (Important) — **PARTIAL**, §3.4a(g). The fold adds a corrected summary-line template with a
  `<kind>` slot and names the three kind strings (`permutations` / `own-anchored subsets` /
  `opt-in subsets`), which is the substance of the fix. But §3.3's **original** "emit exactly one
  line" verbatim template (unchanged by this fold, at the code block immediately following "the
  way out:") still shows the OLD template with no `<kind>` slot, and nothing in §3.3 points to
  §3.4a(g) or marks it superseded. The document now contains two different verbatim templates for
  the same line — the exact contradiction L9 named still exists in §3.3's own text, even though a
  corrected version now exists elsewhere. Remains open: strike or forward-reference the stale
  block in §3.3.

## Minor / Nit disposition (all pre-existing findings; diff does not touch any of these)

- L10 (Minor) — NOT ADDRESSED. §3.4a(b)'s table still just says the warning's JSON value is
  `warning`, without pinning whether it is the six-line block verbatim or a flattened sentence.
- L11 (Minor) — NOT ADDRESSED. §3.4a(b) presents "text mode" / `--json` as parallel columns, which
  reads as mode-exclusive; nothing added states whether the §3.8 warning reaches stderr *in
  addition to* the JSON field when `--json` is passed. §3.8's pre-existing "no flag suppresses it"
  language is suggestive but was already in the document before this finding was raised, and the
  fold added no clarifying sentence.
- L12 (Minor) — NOT ADDRESSED. Bare-xpub `Fingerprint::default()` degenerate `@N=` rendering not
  mentioned anywhere in the fold.
- L13 (Minor) — NOT ADDRESSED. Vector 1's prose (unchanged) still doesn't name the `wsh-multi`
  unsorted shape constraint for the `--own-account-max` fixture-widening advice.
- L14 (Minor) — NOT ADDRESSED. §3.3's sort-key justification (unchanged) still cites
  `address_index` as if it varies on the enumerate path and doesn't note it reverses the engine's
  own tie-break order.
- L15 (Minor) — NOT ADDRESSED. §5's change list still doesn't mention extending
  `search_reference`/the determinism oracle for collect-all mode.
- L16 (Nit) — NOT ADDRESSED. §3.8 (unchanged) still says "before the wallet block" without the
  "on stderr, before any stdout write" clarification.
- L17 (Nit) — NOT ADDRESSED. No mention added of `own_position()`'s re-homing or the
  `verify_bundle.rs:954` defensive-arm question for the enum refactor (§5's enum decision doesn't
  reference these specifics).

## New defect found this round

- **NEW (Minor), §3.4a(c)/(g).** Three different, unreconciled vocabularies now exist in the spec
  for the same 3-way "realized-space kind" concept: the pre-existing prose shorthand ("n!" /
  "s_own" / "s_opt", e.g. line 166), the new JSON field `realized_space_kind` (only one value shown
  in the worked example, `"n_factorial"`; the other two values are never enumerated), and the new
  text-template kind phrases (`permutations` / `own-anchored subsets` / `opt-in subsets`,
  §3.4a(g)). Nothing states that the JSON slug and the text phrase denote the same split, nor what
  the JSON values for the `s_own`/`s_opt` cases actually are. This is fold-introduced (both new
  vocabularies were added by this fold in service of the L9 fix) and is the same class of "must
  guess the exact string" gap L9 originally named — just moved one level down, into the mapping
  between the two new representations the fold itself created.

Full report: design/agent-reports/wallet-id-enumerate-fold-check-round-3.md
