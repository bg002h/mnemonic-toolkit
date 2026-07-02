# gui_example.pdf cycle — RECON report C (Examples fixtures + GUI flow coverage)

> Provenance: recon subagent report, persisted verbatim per house convention.
> Cycle: `gui_example_tutorial` (spec: `docs/manual-gui/design/SPEC_gui_example_tutorial.md`).
> Dispatched 2026-07-02 by the RECON+SPEC author. Sources verified against
> toolkit `master@4c401b16` working tree (`.examples-build/`, all 1623 lines of
> Examples.md read end-to-end) and mnemonic-gui `master@0d4429d` (v0.55.0).

---

# Recon Agent C — GUI Tutorial Book (gui_example.pdf) findings

Scope: `/scratch/code/shibboleth/mnemonic-toolkit/.examples-build/` (Examples.md + fixtures) and `/scratch/code/shibboleth/mnemonic-gui/`. All 1623 lines of Examples.md read end-to-end.

## Critical upfront fact: display-name vs fixture-name indirection

Examples.md shows files under *cosmetic* names (`policy.desc`, `taproot.desc`, `seed0.txt`, `policy.json`…) that do NOT exist as fixtures. `gen.sh` copies the real checked-in fixtures into the working dir under those display names (`.examples-build/gen.sh:30-37`):

- `policy.desc` ← `degrade2.desc` (`gen.sh:30`)
- `taproot.desc` ← `tr2.desc` (`gen.sh:31`)
- `taproot-4leaf.desc` ← `tr4.desc` (`gen.sh:32`)
- `policy.json` ← `degrade2-spec.json`, with the placeholder sha256 rewritten to the `opensessame` digest via `sed` (`gen.sh:33-37`)
- `multisig.desc` / `taproot-multi.desc` are generated inline from keys derived **live** by `mnemonic convert` (not from any fixture) (`gen.sh:252-261`, `gen.sh:617-619`)
- `seed{0,1,2}.txt` written inline from `S0/S1/S2` string literals (`gen.sh:27-29`, `198`, `224-226`)

So a GUI tutorial must ship pre-generated fixtures under whatever names it chooses; the underlying static assets are `degrade2.desc`, `tr2.desc`, `tr4.desc`, `degrade2-spec.json`.

---

## 1 + 2. Per-journey table (step → CLI → fixtures → GUI form? → secret fields)

Journey→section map: J1=§2, J2=§3 (+§3.4), J3=§5, J4=§6 (+Appendix B), J5=§4 (+ the §3/§5/§6 `export-wallet --format …` steps).

### J1 — Single-sig card set (Examples §2, lines 202-260)
| Step | CLI (subcommand + flags) | Input fixture | GUI form? | Secret field |
|---|---|---|---|---|
| Write seed | `printf … > seed0.txt` (shell) | `S0` literal `gen.sh:27` | n/a (type into field) | seed phrase (SECRET) |
| Emit bundle | `mnemonic bundle --template bip84 --network mainnet --slot @0.phrase=- < seed0.txt` (`Examples.md:223`) | seed0.txt | **bundle** `schema/mnemonic.rs:4312` | `--slot @0.phrase` (SECRET, slot-subkey) |

Seed phrase (public vector `abandon…about`, fp `73c5da0a`) — **SECRET-bearing**.

### J2 — Conventional 2-of-3 multisig (Examples §3, lines 264-437; §3.4 lines 439-549; restore in §4)
| Step | CLI | Fixture | GUI form? | Secret field |
|---|---|---|---|---|
| Per-device fingerprint | `mnemonic convert --from phrase=- --to fingerprint --template wsh-sortedmulti --network mainnet < seedN.txt` (`Examples.md:298,316,334`) | seed{0,1,2}.txt | **convert** `4328` | `--from phrase=` (SECRET, node-dynamic) |
| Per-device xpub | `mnemonic convert --from phrase=- --to xpub …` (`303,321,339`) | seedN.txt | convert `4328` | `--from phrase=` (SECRET) |
| Canonicalise desc | `mnemonic export-wallet --descriptor "$(cat multisig.desc)" --format descriptor --network mainnet` (`356`) | multisig.desc (generated) | **export-wallet** `4336` | none (xpubs only) |
| First addr (BSMS) | `mnemonic export-wallet --descriptor … --format bsms` (`365`) | multisig.desc | export-wallet `4336` | none |
| Engrave watch-only | `mnemonic bundle --descriptor-file multisig.desc --network mainnet` (`379`) | multisig.desc | bundle `4312` | none |
| §3.4 all-seeds | `mnemonic bundle --template wsh-sortedmulti --threshold 2 --network mainnet --slot "@0.phrase=$(cat seed0.txt)" --slot "@1…" --slot "@2…"` (`466`) | seed{0,1,2}.txt | bundle `4312` | 3× `--slot @N.phrase` (SECRET) |
| Restore from md1 | `mnemonic restore --network mainnet --md1 …` (`585`, via `sed`) | multisig.md1 (generated) | **restore** `4370` | none |

Three public seed vectors (`73c5da0a`, `b8688df1`, `28645006`) — **SECRET-bearing**. All descriptor/xpub/BSMS/md1 material is watch-only (non-secret).

### J3 — Pathological 4-tier degrading wsh vault (Examples §5, lines 652-1162)
| Step | CLI | Fixture | GUI form? | Secret field |
|---|---|---|---|---|
| Hashlock preimage/hash | `python3 -c "import hashlib…"` (`Examples.md:685`) | word `opensessame` (public) | **NO GUI** (pure shell) | none (public word) |
| Guided-builder refusal | `mnemonic build-descriptor --spec policy.json --network mainnet` (`831`) | `policy.json` ← `degrade2-spec.json` | **build-descriptor** `4353` | none (xpubs only) |
| Canonicalise | `mnemonic export-wallet --descriptor "$(cat policy.desc)" --format descriptor` (`850`) | `policy.desc`←`degrade2.desc` | export-wallet `4336` | none |
| First addr | `mnemonic export-wallet --descriptor … --format bsms` (`858`) | policy.desc | export-wallet `4336` | none |
| Engrave | `mnemonic bundle --descriptor-file policy.desc --network mainnet` (`881`) | policy.desc | bundle `4312` | none |
| Restore round-trip | `mnemonic bundle … --json | jq -r ".md1[]" > policy.md1` then `mnemonic restore … --md1 …` (`1125,1130`) | policy.md1 (generated) | bundle + restore `4312/4370` | none |

**No secrets** — the whole vault is built from 11 watch-only xpubs (from `degrade2.desc`); the `opensessame` hashlock is a public word / public hash `a84dce…08ad`.

### J4 — Taproot twin (Examples §6, lines 1166-1483; Appendix B lines 1502-1623)
| Step | CLI | Fixture | GUI form? | Secret |
|---|---|---|---|---|
| Depth-2 refusal | `mnemonic export-wallet --descriptor "$(cat taproot-4leaf.desc)" --format descriptor` (`1190`) | `taproot-4leaf.desc`←`tr4.desc` | export-wallet `4336` | none |
| Canonicalise depth-1 | `mnemonic export-wallet --descriptor "$(cat taproot.desc)" --format descriptor` (`1212`) | `taproot.desc`←`tr2.desc` | export-wallet `4336` | none |
| Engrave + restore | `bundle … --json | jq … > taproot.md1`; `restore … --md1 …` (`1234,1268`) | taproot.md1 | bundle+restore | none |
| Core export | `export-wallet --descriptor … --format bitcoin-core` (`1305`) | taproot.desc | export-wallet | none |
| BSMS unsupported | `export-wallet --descriptor … --format bsms` → error (`1335`) | taproot.desc | export-wallet | none |
| §6.5 cost compare | `mnemonic compare-cost --miniscript "…ripemd160(06d05e…)"` (`1365,1385`) | Hp hash (public); helper python `1355` | **compare-cost** `4581` | none |
| §6.6 NUMS multisig | `export-wallet --descriptor "$(cat taproot-multi.desc)" …`; `bundle`; `restore` (`1443,1452,1457`) | taproot-multi.desc (generated, NUMS `gen.sh:617`) | export-wallet/bundle/restore | none |
| §6.6 Core cross-check | `bitcoin-cli … deriveaddresses …` (`1474`) | — | **NO GUI** (external) | none |
| Appendix B | `git clone`/`checkout`/`cargo build` `mnemonic-depth2`; `restore` (`1521-1597`) | `taproot-4leaf.desc`←`tr4.desc` | **NO GUI** (build steps) | none |

**No secrets.** Both hashlocks (`opensessame`→`a84dce…`, `please`→`06d05e…`) are public.

### J5 — Watch-only export (Examples §4, lines 553-648; formats reused in §5.5, §6.4)
| Step | CLI | Fixture | GUI form? | Secret |
|---|---|---|---|---|
| md1 chunks → wallet | `mnemonic bundle --descriptor-file multisig.desc --network mainnet --json | jq -r ".md1[]" > multisig.md1` (`565`) | multisig.desc | bundle `4312` | none |
| Restore descriptor | `mnemonic restore --network mainnet --md1 …` (`585`) | multisig.md1 | restore `4370` | none |
| Core importdescriptors | `mnemonic restore … --format bitcoin-core` (`601`) | multisig.md1 | restore `4370` | none |
| Import into Core | `bitcoin-cli … createwallet / importdescriptors / getnewaddress` (`640-642`) | wallet.json | **NO GUI** (external) | none |

**No secrets** (md1 card carries watch-only policy only).

Secret inventory summary: the ONLY secret-bearing inputs across all journeys are the **three public BIP-39 seed phrases** (J1, J2). They appear verbatim in Examples.md at lines 209/215, 279/312, 284/329 and in Appendix A (`Examples.md:1491-1493`), and as `S0/S1/S2` literals in `gen.sh:27-29`. **No secret exists in any fixture that isn't already printed in Examples.md.** No ms1-secret strings, no xprvs anywhere (every `ms1` shown is watch-only-mode "omitted/placeholder"; every key is an `xpub`). `keys.env` (untracked) holds only public xpubs `K0..K10`. UNVERIFIED/ambiguous: none — the seed material is unambiguously the three world-known test vectors.

---

## Fixture checked-in status (`git ls-files` + `.gitignore`)

The `.gitignore` (`.examples-build/.gitignore`) explicitly documents intent: only `gen.sh` + 4 static assets + 5 helper scripts are tracked; everything else is a build artifact or "earlier draft".

**TRACKED (checked in):** `degrade2.desc`, `degrade2-spec.json`, `tr2.desc`, `tr4.desc`, `derive_keys.sh`, `gen_spec.py`, `gen_spec2.py`, `render_desc.py`, `tr_render.py`, `gen.sh`, `.gitignore`.

**UNTRACKED (gitignored scratch / build output):** `degrade.desc`, `degrade-spec.json` (superseded v1), `t_nums.desc`, `t_real.desc`, `ex3.json`, `tr2-bundle.json`, `keys.env`, `H.txt`, `Hp.txt`, `knew.txt`, `degrade2.out`, `Examples.md`, `Examples.pdf`, `section{5,6,6_5}.md`, `preamble.tex`.

Per-journey fixture dependency & tracked status:
- **J1/J2:** seed phrases (literals in tracked `gen.sh`) — OK. `multisig.desc`/`.md1` generated live — OK.
- **J3:** needs `degrade2.desc` (TRACKED) + `degrade2-spec.json` (TRACKED). The hash-rewrite `sed` is in `gen.sh` (TRACKED). Fully reproducible.
- **J4:** needs `tr2.desc` (TRACKED) + `tr4.desc` (TRACKED). NUMS hex hard-coded in `gen.sh:617`. Reproducible.
- **J5:** derived from J2/J3/J4 outputs — OK.

Note: `ex3.json`, `tr2-bundle.json`, `t_nums.desc`, `t_real.desc`, `degrade.desc`, `H.txt`, `Hp.txt`, `knew.txt` named in the recon prompt are NOT referenced by the current `gen.sh` and are gitignored scratch/earlier drafts (confirmed by grepping `gen.sh` — zero hits). The hashes they contain (`a84dce…`, `06d05e…`) and the `knew` key are instead hard-coded inline in `gen.sh`/Examples.md prose. They are safe to ignore for the tutorial.

---

## 3. GUI form existence for every journey subcommand

Every subcommand used by the journeys exists as a GUI form (`schema/mnemonic.rs` subcommand table, lines 4300-4603):

| Journey subcommand | GUI form | line |
|---|---|---|
| bundle | YES | 4312 |
| convert | YES | 4328 |
| export-wallet | YES | 4336 |
| build-descriptor | YES | 4353 |
| restore | YES | 4370 |
| verify-bundle | YES | 4320 |
| inspect | YES | 4517 |
| import-wallet | YES | 4527 |
| xpub-search (4 modes) | YES | 4544-4568 |
| compare-cost | YES | 4581 |

**build-descriptor tree/spec modes (the J3 pivot):** The build-descriptor form supports THREE modes — Generic (dropdown), Archetype preset, and **Tree builder** (`form/tree_form.rs:8-13,53`). Two ways to feed a spec:
- **`--spec` is a FILE-PATH field**, not inline JSON: `FlagKind::Path { stdio_sentinel: true }` (`schema/mnemonic.rs:3921-3937`). The schema comment is explicit: "`--spec` is a FILE PATH (or `-` = stdin)… NEVER inline JSON, so it is `Path{stdio_sentinel:true}`… a Text widget would emit raw JSON → ENOENT." So **the J3 tutorial passes `--spec policy.json` (a pre-generated fixture) via the file-path field** — this is directly supported.
- **Tree-builder mode** pipes an in-GUI-built spec via stdin as `--spec -` (`form/tree_form.rs:76-86` `spec_stdin_bytes`; host appends `--spec -`). Also `--spec-schema` boolean (`3939`), `--archetype` dropdown (`3958`), `--emit-spec`/"Edit as tree…" bridge (`tree_form.rs:176-221`).

Bundle/export-wallet's `--descriptor-file` is also `FlagKind::Path` (`schema/mnemonic.rs:294`, `851`), so the journeys' `--descriptor-file policy.desc` and the `"$(cat …)"` command-substitutions map onto file-path / Text fields (`--descriptor` is `FlagKind::Text` `284/841`). restore's `--md1` is a repeating flag (`679`, `1072`, `2374`) — the many `--md1 <chunk>` args become repeating rows.

---

## 4. Secret masking in the GUI — mechanism, coverage, and the I3 64-list

**Classification mechanism (the "Secret flag"):** three parallel secret-class sets, all imported from the toolkit as the single source of truth (`secrets.rs:34-36`):
- `SECRET_NODE_TYPES` / `_ARGV` — node-type secrecy (`secrets.rs:44-56`, narrow set = phrase/entropy/xprv/wif/ms1/bip38/electrum-phrase/seedqr; `_ARGV` adds `minikey`).
- `SECRET_SLOT_SUBKEYS` = `[phrase, seedqr, entropy, ms1, xprv, wif]` (`secrets.rs:69-70`).
- `FlagSchema.secret` bool + hand-maintained `SECRET_FLAG_NAMES` (`--passphrase`, `--bip38-passphrase`, `--passphrase-stdin`) → `flag_is_secret()` (`secrets.rs:143-153`).

**Password-mask char / redaction:**
- Live widgets render `egui::TextEdit::singleline(...).password(true)` (bullet-dots). Secret Text flags: `SecretLineEdit::show` `form/secret_widget.rs:58`. SlotEditor rows: gated on `row.subkey.is_secret_bearing()` `form/slot_editor.rs:46-52`. Composite value fields: `.password(node_type_is_argv_secret(node))` `form/widget.rs:604-606`.
- Argv/preview redaction sentinel `SECRET_MASK = "••••"` (`form/invocation.rs:137`); the masked assembler masks all four secret-value sources incl. secret slot rows and argv-secret composite nodes (`invocation.rs:139-151`).

**Coverage for the exact entry paths the journeys need:**
1. **Plain seed-phrase text field (convert `--from phrase=`, J2 per-device):** masked dynamically — `--from` is `secret:false` because "secrecy is node-dependent" (`schema/mnemonic.rs:1206-1211`); masking/paste-warn/argv-mask/persist-redact all key on `node_type_is_argv_secret("phrase")` (`widget.rs:604`, `invocation.rs:148`, `persistence.rs:96-101`). MASKED.
2. **SlotEditor rows (bundle `--slot @N.phrase`, J1 + J2 §3.4 multi-slot):** `.password(true)` when subkey is secret-bearing (`slot_editor.rs:51`); persist-redaction DROPS slot rows whose subkey ∈ `SECRET_SLOT_SUBKEYS` (`persistence.rs:111-121`). MASKED + never-persisted. Dedicated tests: `tests/slot_secret_mask_v0_38_0.rs`, `tests/argv_assembler_slot.rs`.
3. **Composite / nested tree editors (build-descriptor tree keys, J3/J4 if built via tree instead of `--spec` file):** tree `key`/`keys` persist-then-redact via `blank_non_extended_public_keys` (private-like blanked, watch-only xpub survives) — proven in `tests/ui_harness_i3_secret_nopersist.rs:485-520`; tree secret key stays off the `--spec -` masked preview and the stdin holder scrubs on drop (`:526-598`). COVERED.

**The I3 census (64 secrets) — honest verdict:** `tests/ui_harness_i3_secret_nopersist.rs:274-279` pins `(value_bearing, narrowed_boolean, total) = (40, 24, 64)` **classified secret FLAGS** across all 4 CLI tabs, plus a SEPARATE census of **5 secret positionals** (`:288-298`). The test **explicitly scopes to classified FLAGS only** and states it "CANNOT catch an UNclassified secret rendering as a normal Text widget" (`:9-12`).

Therefore, be precise: the journeys' seed-entry fields are **NOT literally members of the 64-flag census**, because (a) `bundle --slot @N.phrase` is a **slot-subkey**, not a flag, and (b) `convert --from phrase=` is a **node-dynamic composite** with `secret:false`. Neither is one of the 64 classified FLAGS or the 5 positionals. They are instead covered by the **parallel** `SECRET_SLOT_SUBKEYS` and `SECRET_NODE_TYPES_ARGV` mechanisms — which mask, paste-warn, run-confirm, and persist-redact identically — and are pinned by their own dedicated tests (`slot_secret_mask_v0_38_0.rs`, `argv_assembler_slot.rs`, `secret_mask_preview_v0_39_0.rs`, `h3_minikey_secret_surfaces.rs`, `widget_secret.rs`, `secret_taxonomy_pin.rs`). Net masking-coverage verdict: **all three journey secret entry paths are fully masked + never-persisted**, but a tutorial spec should not claim they ride the I3 64-flag list — they ride the sibling slot/node taxonomies.

(The `--passphrase` flag that appears on bundle/restore/convert forms IS on the 64-flag list, but no journey uses a BIP-39 passphrase.)

---

## 5. Shell-only steps with NO GUI equivalent (need tutorial-level adaptation)

Enumerated from `gen.sh` / Examples.md:

1. **Python hashlock helpers** (`Examples.md:685` §5.1, `:1355` §6.5; `gen.sh:393,577`): `python3 -c "import hashlib…"` computing `H=sha256(sha256("opensessame"))=a84dce…08ad` and `Hp=ripemd160(sha256("please"))=06d05e…aff4`. No GUI. → **Adaptation: pre-computed constants baked into the descriptor fixtures** (they already are, inside `degrade2.desc`/`tr2.desc`; H.txt/Hp.txt hold them).
2. **Spec/descriptor generators** `gen_spec.py`, `gen_spec2.py`, `render_desc.py`, `tr_render.py`, `derive_keys.sh` (all TRACKED but pure shell/python, not run in Examples.md; they *produced* the fixtures). No GUI. → **Adaptation: ship the pre-generated `policy.json` / `.desc` fixtures.**
3. **jq + sed + command-substitution glue for the bundle→restore md1 hop:** `mnemonic bundle … --json | jq -r ".md1[]" > policy.md1` then `mnemonic restore … $(sed "s/^/--md1 /" policy.md1)` (`Examples.md:565,585,1125,1130,1234,1268,1452,1457`). No single GUI form spans two invocations. → **Adaptation: tutorial step "copy the md1 chunks from the bundle output into the restore form's repeating `--md1` rows", or provide a pre-generated `.md1` fixture.**
4. **Command-substitution `--descriptor "$(cat X.desc)"`** (`:356,365,850,858,1190,1212,1305,1443`). → **HAS a GUI equivalent**: use the `--descriptor-file` Path field (`schema/mnemonic.rs:294,851`) instead of `$(cat …)`. (Adaptation is trivial, not a gap.)
5. **`printf … > seedN.txt` + `< seedN.txt` stdin redirect** (`gen.sh:198,224-226`; `Examples.md:209,223`). → GUI types the phrase into the masked field; the runner handles stdin routing internally. (Not a gap — but the tutorial narrative changes from "file redirect" to "type into field".)
6. **Inline `--slot "@N.phrase=$(cat seedN.txt)"` (argv, with warnings)** §3.4 (`:466`). → GUI equivalent = 3 masked SlotEditor rows; the GUI's own paste-warn/run-confirm replaces the CLI's `secret material on argv` warning.
7. **`bitcoin-cli` external steps** (§4 `:640-642`, §6.6 `:1474`): `createwallet` / `importdescriptors` / `getnewaddress` / `deriveaddresses`. External tool, no GUI form. → **Adaptation: show as external follow-on, or paste pre-captured output.**
8. **Install (`§1`) and Appendix B build steps** (`Examples.md:90,1521-1524`): `install.sh`, `git clone`/`checkout`/`cargo build` of `mnemonic-depth2`. No GUI. → **Adaptation: out-of-band setup prose; Appendix B is EXPERIMENTAL and likely out of tutorial scope.**

One structural note for the spec: J3's headline moment is `build-descriptor` **refusing** the 11-key policy (`Examples.md:831-834`, over_envelope) and redirecting to the raw `--descriptor` path. The GUI build-descriptor form can reproduce the refusal (fill `--spec` file field → Run → toolkit prints the diagnostic), so the "guided builder caps complexity" teaching moment survives as a GUI step — but the actual vault is then built by feeding `policy.desc` to the bundle/export-wallet **file-path** fields, exactly as the CLI does.
