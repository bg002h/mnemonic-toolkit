# SPEC — `mnemonic slip39` (v0.13.0)

**Status:** Phase 0 — SPEC author + R0 reviewer-loop.
**Cycle:** v0.13.0 (toolkit-only minor bump).
**Predecessor:** v0.12.0 seed-xor splitter shipped at `mnemonic-toolkit-v0.12.0` (tag at `63b4503`, 2026-05-14).
**Driving FOLLOWUP:** `slip39-shamir-secret-sharing` at `design/FOLLOWUPS.md:1006` (open since 2026-05-06, three-cycle deferral — v0.6.1 surface, v0.7 dropped after lib-audit, v0.8 re-tiered to v1+). Closes at PE.
**Brainstorm + plan:** consolidated plan-mode artifact `~/.claude/plans/radiant-seeking-teacup.md` (BRAINSTORM + PLAN sections); this document renders plan §B + corresponding entries from §C.

External authority:
- [SLIP-0039 specification](https://github.com/satoshilabs/slips/blob/master/slip-0039.md)
- [`python-shamir-mnemonic` reference impl](https://github.com/trezor/python-shamir-mnemonic) (MIT)
- [`python-shamir-mnemonic/vectors.json`](https://github.com/trezor/python-shamir-mnemonic/blob/master/vectors.json) — 45 canonical test vectors (15 positive + 30 negative)

---

## §1 Purpose

Add Trezor SLIP-0039 hierarchical K-of-N Shamir Secret Sharing as the `mnemonic slip39` subcommand (with `split` + `combine` sub-subcommands). Companion to v0.12.0's `seed-xor` (all-or-nothing); SLIP-39 is the threshold-based alternative used by Trezor Model T as its native backup format.

**Use cases:**
1. K-of-N paper-backup recovery — any K of N shares reconstitute the master; lose any (N − K) shares without losing the wallet.
2. Two-level hierarchy: `group_threshold` of `group_count` groups; each group has its own `member_threshold` of `member_count`. Models e.g. "any 2 of 3 family branches, where each branch needs 3 of 5 cosigners" backup policies.
3. Trezor hardware interop — combine shares generated on a Trezor Model T to recover the entropy off-device.
4. Optional passphrase wraps the master via 4-round Feistel encryption before splitting (recovery requires the same passphrase OR returns silently-wrong material — caught by the 4-byte digest verification step).

**Toolkit-only minor bump.** No cross-repo work. Closes deferred FOLLOWUP `slip39-shamir-secret-sharing`.

## §2 Functional surface

### §2.1 Library entry point

New multi-module subdirectory `crates/mnemonic-toolkit/src/slip39/` (~2000 LOC across the layer split):

```
crates/mnemonic-toolkit/src/slip39/
├── mod.rs           — public surface (split, combine, types)
├── gf256.rs         — GF(256) Rijndael field arithmetic
├── lagrange.rs      — Lagrange interpolation over GF(256) at point 0
├── feistel.rs       — 4-round Feistel encryption + PBKDF2-derived round keys
├── rs1024.rs        — Reed-Solomon-1024 BCH checksum (custom generator per spec §3.2)
├── share.rs         — Share struct + bit-packing + mnemonic encode/decode
├── wordlist.rs      — 1024-word SLIP-39 wordlist (embedded via include_str!)
└── error.rs         — Slip39Error variants
```

**Public surface:**

```rust
#[derive(Debug, Clone, Copy)]
pub struct GroupSpec {
    pub member_count: u8,
    pub member_threshold: u8,
}

#[derive(zeroize::Zeroize, zeroize::ZeroizeOnDrop)]
pub struct Share {
    // share-value bytes — secret-bearing, zeroized on drop
    #[zeroize]
    value: Vec<u8>,
    // metadata — public; #[zeroize(skip)] (non-secret)
    #[zeroize(skip)]
    pub identifier: u16,           // 15-bit
    #[zeroize(skip)]
    pub extendable: bool,
    #[zeroize(skip)]
    pub iteration_exponent: u8,    // 4-bit, 0..=15
    #[zeroize(skip)]
    pub group_index: u8,
    #[zeroize(skip)]
    pub group_threshold: u8,
    #[zeroize(skip)]
    pub group_count: u8,
    #[zeroize(skip)]
    pub member_index: u8,
    #[zeroize(skip)]
    pub member_threshold: u8,
}

pub fn slip39_split(
    master_secret: &[u8],
    passphrase: &[u8],
    group_threshold: u8,
    groups: &[GroupSpec],
    iteration_exponent: u8,
    identifier: Option<u16>,           // None = OS-RNG-derived random
    rng: &mut (impl rand_core::CryptoRng + rand_core::RngCore),
) -> Result<Vec<Vec<Share>>, Slip39Error>;

pub fn slip39_combine(
    shares: &[Share],
    passphrase: &[u8],
) -> Result<zeroize::Zeroizing<Vec<u8>>, Slip39Error>;

pub fn parse_slip39_share(s: &str) -> Result<Share, Slip39Error>;
pub fn render_slip39_share(s: &Share) -> String;
```

**Crypto primitives** (all RustCrypto, MIT/Apache-2.0):
- `sha2 = "0.10"` — SHA-256
- `hmac = "0.12"` — HMAC-SHA-256
- `pbkdf2 = "0.12"` — PBKDF2-HMAC-SHA-256
- `rand_core = "0.6"` (already a dep from v0.12.0) — RNG trait

**NO AES dep.** Feistel uses pure XOR + PBKDF2-derived round keys per SLIP-0039 §"Encryption". Library-local `Slip39Error` per the v0.11.0 final-word + v0.12.0 seed-xor precedent (avoids pulling in binary-private `ToolkitError`; tracked under FOLLOWUP `library-error-and-language-surface-promotion`).

**Round-key buffer hygiene.** Each Feistel round derives an `n/2`-byte round-key via PBKDF2. The impl reuses a SINGLE `Zeroizing<Vec<u8>>` of length `master_secret.len() / 2`, refilled across 4 rounds — **one** `pin_pages_for` call per encryption pass, not four. Matches the `bip85.rs:52,84` precedent.

**Per-share output pin discipline (O(N), not O(1)).** Updated at v0.13.0 P2.2 GREEN (Q6 fold): the original "O(1) single-pin" claim is structurally unachievable for the `Vec<Zeroizing<String>>` shape. The top-level Vec's backing buffer holds `String` headers (24 bytes each on 64-bit; non-secret); each `String`'s UTF-8 bytes live in a SEPARATE heap allocation (the actual share value). Pinning the Vec backing pins only the headers, NOT the per-share secret bytes. Production discipline is therefore an O(N) per-rendered-share loop: each iteration calls `mlock::pin_pages_for(share.as_bytes())` and the guard drops at end of iteration (pages stay locked until the next iteration completes). For 256 shares × ~32 bytes/share value = ~8 KB pinned at peak, well within Linux default `ulimit -l` (64 KB). Lint anchor in `lint_zeroize_discipline.rs` checks `cmd/slip39.rs` contains the `Zeroizing::new` wraps; the SPEC §5 row-12 `seed_xor.rs:157-164` cite is removed (seed-xor uses the same `Vec<Zeroizing<String>>` shape and is also NOT a single-pin precedent).

**Algorithm summary** (per SLIP-0039 spec):
1. **Encrypt** master secret S via 4-round Feistel with PBKDF2(passphrase, salt=identifier-derived) round keys → encrypted master secret (EMS).
2. **Compute digest** = `HMAC-SHA256(key=R, msg=S)[0:4]` where R is a random 4-byte key.
3. **Bundle** `(digest || R)` as the "digest payload" alongside EMS.
4. **Group-level Shamir share** — emit `group_count` raw group shares; reconstruction needs `group_threshold` of them. One of the shares carries the digest payload, indexed by the group_threshold-th-coefficient.
5. **Member-level Shamir share** — for each group_index, take that group's raw share and Shamir-split it into `member_count` member shares with `member_threshold`.
6. **Encode** each member share as a SLIP-39 mnemonic string: identifier + iteration_exponent + group_index + group_threshold + group_count + member_index + member_threshold + share_value (bit-packed) + 30-bit RS1024 BCH checksum, mapped to the 1024-word wordlist (10 bits per word).

### §2.2 CLI subcommand grammar

New file `crates/mnemonic-toolkit/src/cmd/slip39.rs`. Nested clap subcommand:

```rust
#[derive(clap::Subcommand)]
pub enum Slip39Command {
    /// Split a master secret into K-of-N SLIP-39 shares (2-level hierarchy).
    Split(Slip39SplitArgs),
    /// Combine SLIP-39 shares back into the master secret.
    Combine(Slip39CombineArgs),
}
```

#### `mnemonic slip39 split`

| Flag | Required | Default | Purpose |
|---|---|---|---|
| `--from <phrase=<v-or->>` OR `--from <entropy=<hex-or->>` | yes | — | Master secret as BIP-39 phrase OR raw entropy hex |
| `--passphrase <P>` | no | `""` | SLIP-39 passphrase (NOT BIP-39 passphrase); empty string = SLIP-39 default |
| `--passphrase-stdin` | no | false | Read passphrase from stdin (single-stdin-per-invocation rule) |
| `--group-threshold <G>` | yes | — | Groups required to reconstruct (`1 <= G <= group_count`) |
| `--group <N,T>` | yes (repeating; `ArgAction::Append`) | — | Group spec: N=member_count, T=member_threshold per group |
| `--iteration-exponent <E>` | no | `0` | PBKDF2 cost exponent; `0 <= E <= 15`; iterations = 10000 × 2^E |
| `--language <LANG>` | no | `english` | BIP-39 language of input phrase; ignored for `entropy=` |
| `--json-out <PATH>` | no | — | Side-effect JSON envelope |

`--group N,T` parsing uses `value_parser = parse_group_spec` (defined in `cmd/slip39.rs`), accepting `N,T` decimal pairs. Syntactic errors yield clap exit code 64; out-of-bound values surface as semantic refusals per §2.5 row 4.

Example: `mnemonic slip39 split --from phrase=- --group-threshold 2 --group 3,2 --group 3,2 --group 5,3` → 3 groups (3+3+5 = 11 shares total); reconstruction needs 2 of 3 groups, each group needing its own member threshold.

**Stdout:** each share on its own line; groups separated by a blank line; trailing newline.

#### `mnemonic slip39 combine`

| Flag | Required | Default | Purpose |
|---|---|---|---|
| `--share <slip39-mnemonic-or->` | yes (repeating; >= group_threshold) | — | Share strings; at most ONE may be `-` (stdin) |
| `--passphrase <P>` | no | `""` | SLIP-39 passphrase used at split time |
| `--passphrase-stdin` | no | false | Read passphrase from stdin (incompatible with any `--share -`) |
| `--to phrase --language LANG` OR `--to entropy` | no | `--to entropy` | Output shape |
| `--json-out <PATH>` | no | — | Side-effect JSON envelope |

**Stdin contention.** SLIP-39 `combine` has up to N `--share` slots PLUS optional `--passphrase-stdin` — N+1 potential stdin candidates. AT MOST ONE total stdin consumer (single-stdin-per-invocation per `convert.rs:637-651` precedent). Three pairwise refusals: (a) `--passphrase-stdin` + any `--share value=-` → refuse; (b) two distinct `--share value=-` slots → refuse; (c) two `--share` slots both with `-` → refuse.

**Stdout:** a single line — hex entropy (default) or BIP-39 phrase (with `--to phrase`).

### §2.3 JSON envelope schema

Schema `v1`. Discriminated by `operation`.

**Split:**
```json
{
  "schema_version": "1",
  "operation": "split",
  "identifier": 12345,
  "iteration_exponent": 0,
  "group_threshold": 2,
  "groups": [
    {"member_count": 3, "member_threshold": 2, "shares": ["...", "...", "..."]},
    {"member_count": 3, "member_threshold": 2, "shares": ["...", "...", "..."]},
    {"member_count": 5, "member_threshold": 3, "shares": ["...", "...", "...", "...", "..."]}
  ]
}
```

**Combine:**
```json
{
  "schema_version": "1",
  "operation": "combine",
  "identifier": 12345,
  "iteration_exponent": 0,
  "output_shape": "phrase",
  "phrase": "...",
  "entropy_hex": null
}
```

(For `output_shape: "entropy"`: `phrase: null`, `entropy_hex: "..."`.)

Field order is part of the schema (SHA-pinned in `tests/cli_slip39_json.rs`).

### §2.4 Exit codes

- `0` on success.
- `1` for runtime refusals (`ToolkitError::BadInput` / `Slip39` per `src/error.rs:244` precedent).
- `64` reserved for clap parse errors.

### §2.5 Refusals (24 classes; expanded 2026-05-14 from `python-shamir-mnemonic/vectors.json` audit covering negative-vector categories the original SPEC enumeration elided; further expanded at v0.13.0 P1c-E.1 GREEN with 5 combine-driver + parse-layer refusal rows for empty share lists, per-share + cross-share value-length consistency, extendable-bit consistency, and the python-parser `group_count >= group_threshold` mirror; row 24 `MemberThresholdMismatch` added at v0.13.0 P2.2 GREEN per Q3 fold)

| # | Input class | Exit | Stderr message stem |
|---|---|---|---|
| 1 | `--from phrase` word-count not in {12,15,18,21,24} | 1 | `slip39 split: input phrase must be 12/15/18/21/24 words; got K` |
| 2 | `--from entropy=` hex not parseable / odd length / length not in {16,20,24,28,32} bytes | 1 | `slip39 split: entropy hex must decode to 16/20/24/28/32 bytes; got K bytes` |
| 3 | `--group-threshold` outside `1..=group_count` | 1 | `slip39 split: --group-threshold must be in 1..=K (number of --group flags); got G` |
| 4 | `--group N,T` with `T > N` OR `T < 1` OR `N > 16` (SLIP-39 spec max) | 1 | `slip39 split: --group N,T requires 1 <= T <= N <= 16; got group <idx>=N,T` |
| 5 | Any `--group 1,1` (toolkit usability policy; spec permits but recommends N=1 → T=1 is no-op) | 1 | `slip39 split: 1-of-1 group offers no recovery benefit; use --group N,T with N >= 2 (toolkit policy)` |
| 6 | `--iteration-exponent` outside 0..=15 | 1 | `slip39 split: --iteration-exponent must be 0..=15 (4-bit field); got E` |
| 7 | `combine` shares: identifier mismatch across shares (vectors.json #6, #25) | 1 | `slip39 combine: shares disagree on identifier (got {I1, I2, ...}); shares must come from the same secret` |
| 8 | `combine` shares: iteration-exponent mismatch (vectors.json #7, #26) | 1 | `slip39 combine: shares disagree on iteration-exponent` |
| 9 | `combine` shares: RS1024 checksum failure on share I (vectors.json #2, #21) | 1 | `slip39 combine: share at position I has invalid SLIP-39 checksum (RS1024)` |
| 10 | `combine` shares: unknown SLIP-39 word at position I in share J | 1 | `slip39 combine: share at position J: word at index I not in SLIP-39 wordlist` |
| 11 | `combine` shares: digest verification failure (4-byte HMAC-SHA256(key=R, msg=decrypted-S) mismatch) (vectors.json #13, #32) | 1 | `slip39 combine: reconstructed master digest mismatch — wrong --passphrase OR a share was substituted` |
| 12 | `combine` shares: insufficient share count for one or more required groups (vectors.json #5, #14, #15, #16, #24, #33, #34, #35) | 1 | `slip39 combine: insufficient shares for group <idx>: need <member_threshold>, got <K>` |
| 13 | `combine` shares: mismatching group thresholds across shares (vectors.json #8, #27) | 1 | `slip39 combine: shares disagree on group_threshold` |
| 14 | `combine` shares: mismatching group counts across shares (vectors.json #9, #28) | 1 | `slip39 combine: shares disagree on group_count` |
| 15 | `combine` shares: duplicate member index within a single group (vectors.json #11, #30) | 1 | `slip39 combine: duplicate member index <I> in group <G>` |
| 16 | Invalid padding bits in encoded share (vectors.json #3, #22) | 1 | `slip39 combine: share at position I has non-zero padding bits (encoding violation)` |
| 17 | `--from` variant other than `phrase=` / `entropy=` | 1 | `slip39 split: --from only accepts phrase=<value-or-> or entropy=<hex-or->; got <node>=` |
| 18 | Multi-stdin contention (passphrase-stdin + share-stdin OR two share-stdin) | 1 | `slip39: at most one stdin consumer per invocation (across --share, --from, and --passphrase-stdin)` |
| 19 | `combine` called with empty share list (no `--share` and `--share-stdin` produced 0 shares) | 1 | `slip39 combine: at least one share required` |
| 20 | `combine` shares: share at position I has value-byte length L not in {16,20,24,28,32} (vectors.json #40) | 1 | `slip39 combine: share at position I has value length L (must be 16/20/24/28/32 bytes)` |
| 21 | `combine` shares: shares disagree on value-byte length (cross-share length divergence) | 1 | `slip39 combine: shares disagree on value length` |
| 22 | `combine` shares: shares disagree on the `extendable` (ext) bit | 1 | `slip39 combine: shares disagree on the extendable bit` |
| 23 | `combine` shares: parse-time refusal — share at position J encodes `group_count < group_threshold` (vectors.json #10, #29; mirrors `python-shamir-mnemonic/share.py:216-219` @ 17fcce14) | 1 | `slip39 combine: share at position J: group_threshold T exceeds group_count N` |
| 24 | `combine` shares: shares within a single group disagree on `member_threshold` (vectors.json #12, #31; lib variant `MemberThresholdMismatch`; added at v0.13.0 P2.2 GREEN per Q3 fold — structurally distinct from row 15 `DuplicateMemberIndex` and row 12's member-level `InsufficientShares` branch) | 1 | `slip39 combine: shares within a group disagree on member_threshold` |

(Refusal 18 covers the N+1 pairwise candidates explicitly.)

### §2.6 Advisories (stderr, non-fatal)

| Trigger | Stderr advisory |
|---|---|
| Any inline secret on argv (`--from`, `--share`, `--passphrase`) | Per-occurrence `secret_in_argv_warning(stderr, flag, alternative)` |
| `split` AND stdout is TTY | **New advisory class (K-of-N parameterized — extends v0.12.0's multi-secret-on-stdout):** `warning: SLIP-39 shares on stdout — N=<n> shares emitted across <g> groups (group-threshold <G>); each share is independently secret material; distribute per your group/member-threshold policy; do not paste this output into a single untrusted tool` |
| `combine` AND stdout is TTY | Reuse v0.11.0 pattern: `warning: reconstructed secret material on stdout — verify the recovered wallet's expected derived address before trusting` |
| `--json-out` to a world-readable path | Reuse v0.11.0 `#[cfg(unix)]` permission-mode advisory at `cmd/final_word.rs:175-200` |
| `--iteration-exponent E` where E >= 5 (PBKDF2 iterations >= 320K, ≈ 200–500ms wall-clock on commodity x86) | `warning: --iteration-exponent E=<E> yields <iters> × PBKDF2-HMAC-SHA-256 iterations; split + combine performance may be observably slow (sub-second to multi-second). Trezor's reference uses E=1 (20000 iters) as default; the SLIP-0039 spec gives no recommended values. E >= 10 may exceed 30s on weak hardware.` |
| Either `MNEMONIC_SLIP39_TEST_RNG` OR `MNEMONIC_SLIP39_TEST_IDENTIFIER` env-var set on a `split` invocation (always-on; NOT suppressible; added at v0.13.0 P2.2 GREEN per Q2 fold — see §6 for the env-vars' definitions) | `warning: MNEMONIC_SLIP39_TEST_RNG set — output is deterministic and INSECURE; do not use for real shares` |

## §3 Out-of-scope (filed for explicit closure)

| OOS class | Rationale | Where it goes |
|---|---|---|
| `OOS-slip39-multilanguage-wordlist` | Spec defines only an English wordlist; SLIP-39 has no multi-language counterpart to BIP-39's 10-language coverage. | Not applicable; spec-locked |
| `OOS-slip39-three-level-hierarchy` | Spec defines exactly 2-level (groups × members) — no supergroup/metagroup extension. | Not applicable; spec-locked |
| `OOS-slip39-codex32-interop` | Different format; no shared wordlist or checksum. Companion `ms-codec` v0.2 K-of-N is the codex32 path. | Future `mnemonic-secret` v0.2 cycle |
| `OOS-slip39-share-reshare-cli` | Convenience subcommand to re-split N-of-M into K-of-N — requires combine + split chain; user does the two steps manually. | Documented in manual; defer subcommand sugar |
| `OOS-slip39-import-trezor-onev-format` | Trezor One predates SLIP-39 (uses raw BIP-39). | Not applicable |

## §4 Acceptance gates

| Gate | Criterion |
|---|---|
| G1 — SLIP-0039 spec test vectors | Vendor `python-shamir-mnemonic/vectors.json` (45 canonical test vectors; verified at fetch 2026-05-14) into `crates/mnemonic-toolkit/tests/fixtures/slip39_vectors.json` (SHA-pinned against the upstream-commit SHA at fetch time). **15 positive vectors** (must recover the expected hex secret + match the expected BIP-32 xprv) + **30 negative vectors** (must refuse with the appropriate `Slip39Error` variant per the §2.5 refusal mapping). ALL 45 must pass byte-for-byte in `tests/lib_slip39_vectors.rs`. Vector shape: 4-tuple `[description, mnemonics_list, hex_secret, expected_xprv]`; positive vectors have non-empty `hex_secret`, negative have empty. |
| G2 — Round-trip property tests | For each of 5 entropy sizes (16/20/24/28/32 bytes) × N group configurations: split → combine → byte-equal. Property test ≥ 50 vectors per shape. |
| G3 — Plain stdout shape | `slip39 split ... --group 3,2 --group 3,2` emits exactly 6 lines + 1 blank separator; each line parseable as a SLIP-39 share. |
| G4 — JSON envelope stability | SHA-pinned over 2 anchor vectors. Determinism is achieved at test time via the env-vars `MNEMONIC_SLIP39_TEST_RNG` (32-byte hex seed for ChaCha20Rng) and `MNEMONIC_SLIP39_TEST_IDENTIFIER` (decimal u16); both are inert in production (env-vars unset → `OsRng` + library-generated random identifier) and trigger an always-on insecurity advisory in stderr when set. The production `--help` surface does NOT mention these vars; they are documented in §6 only. |
| G5 — Refusal coverage | All 23 refusal classes (§2.5) have CLI tests asserting exit code 1 + pinned stderr stem. The 30 negative vectors from G1 are exercised at the lib layer; CLI-level tests verify each stem surfaces byte-faithfully. |
| G6 — Cycle A/B discipline | Cycle A: argv-leakage advisory + `Zeroizing<String>` wraps + new `lint_argv_secret_flags.rs` rows (`slip39 split --from phrase=`, `slip39 split --from entropy=`, `slip39 split --passphrase`, `slip39 combine --share`, `slip39 combine --passphrase`) — count 23 → 28 (Q1 fold preserves the lint convention's one-row-per-(subcommand,flag) shape; v0.13.0 P2.2 GREEN). Cycle B: mlock Site 1 pins on parsed inputs + Feistel round-key buffer (single-buffer, single pin per encryption pass) + per-rendered-share output pins (O(N), one `mlock::pin_pages_for` call per share inside the stdout-emit loop per §2.1 patch). New `lint_zeroize_discipline.rs` rows. |
| G7 — Manual chapter | `## mnemonic slip39` section in `41-mnemonic.md`; `cli-subcommands.list` adds `mnemonic slip39 split` + `mnemonic slip39 combine`; chapter intro bumps from 8 to 9 subcommands (8 user-facing + introspection-only `gui-schema`). |
| G8 — Trezor interop smoke test (manual, post-tag) | Generate a SLIP-39 backup on a Trezor Model T (or via `python-shamir-mnemonic` CLI), combine via our CLI, verify byte-equal entropy recovery. Recipe lives at `docs/manual/src/40-cli-reference/41-mnemonic.md` under a new `### Trezor interop (manual smoke test)` H3 within the `## mnemonic slip39` section (authored at P3, validated at PE). NOT a CI gate (no Trezor in CI). |
| G9 — Iteration-exponent advisory threshold | `E >= 5` advisory fires; below E=5, no advisory. Pinned via CLI test. (Threshold rationale: ≈ 200ms wall-clock on commodity x86; Trezor's reference default is E=1.) |

## §5 Cross-refs

Existing utilities reused (paths verified at grep-against-source ground truth post-v0.12.0):

| Utility | Path |
|---|---|
| `FromInput` + `parse_from_input` + `NodeType` | `src/cmd/convert.rs:30-151` |
| `read_stdin_to_string<R: Read>` | `src/cmd/convert.rs:566-572` |
| `secret_advisory::secret_in_argv_warning` | `src/secret_advisory.rs:25-30` |
| `mnemonic_toolkit::mlock::pin_pages_for` | `src/mlock.rs:90-127` |
| `CliLanguage` + `From<CliLanguage> for bip39::Language` | `src/language.rs:6-57` |
| `ToolkitError::BadInput` / `exit_code()` | `src/error.rs:11,242-280` |
| `bip39::Mnemonic::parse_in` + `from_entropy_in` | `bip39 = "2"` |
| Wordlist embedding pattern | `src/wordlists/mod.rs` (Electrum precedent); SLIP-39 mirrors with `slip39/wordlist.rs` |
| `std::io::IsTerminal` | std (v0.11.0 first use) |
| `#[cfg(unix)]` permission-mode helper | v0.11.0 `cmd/final_word.rs:175-200` |
| JSON envelope `schema_version: "1"` + serde struct field-order | v0.11.0 + v0.12.0 precedent |
| Manual chapter pattern | v0.11.0 `## mnemonic final-word`, v0.12.0 `## mnemonic seed-xor` (sub-subcommand-aware shape) |
| Lint anchors | `tests/lint_argv_secret_flags.rs` (baseline 23 rows post-v0.12.0) + `tests/lint_zeroize_discipline.rs` (loose-bound 18..=35) |
| K-of-N stdout-on-TTY advisory pattern | `cmd/seed_xor.rs:184-198` (v0.12.0 introduced; SLIP-39 extends with `<g> groups (group-threshold <G>)` interpolation) |
| `Vec<Zeroizing<String>>` for multi-secret output | `cmd/seed_xor.rs:157-164` (v0.12.0 per-share shape) |

New crate deps (v0.13.0):
- `sha2 = "0.10"` — RustCrypto, MIT/Apache-2.0
- `hmac = "0.12"` — RustCrypto
- `pbkdf2 = "0.12"` — RustCrypto

External SPEC references:
- [SLIP-0039 specification](https://github.com/satoshilabs/slips/blob/master/slip-0039.md) — Encryption, Decryption, Checksum, Mnemonic encoding, Member threshold, Group threshold
- [`python-shamir-mnemonic`](https://github.com/trezor/python-shamir-mnemonic) — MIT reference impl; algorithm-correctness oracle
- [`vectors.json`](https://github.com/trezor/python-shamir-mnemonic/blob/master/vectors.json) — 45 test vectors (G1 acceptance gate fixture)

## §6 Test-only environment variables

Added at v0.13.0 P2.2 GREEN (per Q2 fold + Opus G4-architect consult). Two env-vars override `slip39 split`'s otherwise-non-deterministic identifier + RNG sources, enabling the §4 G4 SHA-pin tests in `tests/cli_slip39_json.rs` to assert byte-identical envelope output across runs. Both are runtime-gated (NOT compile-time `#[cfg]`-gated) so they exercise the same production code paths `cargo run` / `cargo install` users invoke. The production `--help` surface does NOT mention these vars — they live in the SPEC only.

| Env-var | Type | Effect | Default (unset) |
|---|---|---|---|
| `MNEMONIC_SLIP39_TEST_RNG` | 32-byte hex (64 chars) | Seeds a `ChaCha20Rng::from_seed(…)` instead of `OsRng`; invalid hex / wrong length → refuse with a clear stem (NOT silent fallback) | `OsRng` |
| `MNEMONIC_SLIP39_TEST_IDENTIFIER` | decimal u16 in 0..=32767 (15-bit) | Used as the `identifier` argument to `slip39_split`, overriding the RNG-derived random; out-of-range → refuse | library generates random |

**Always-on insecurity advisory.** When EITHER env-var is set, `run_split` emits the SPEC §2.6 row 6 advisory `warning: MNEMONIC_SLIP39_TEST_RNG set — output is deterministic and INSECURE; do not use for real shares` to stderr (NOT suppressible; loud-by-design — env-var misuse must be self-disclosing in any captured terminal log).

**Stability disclaimer.** These env-vars are NOT part of the v0.13.0 user-facing contract. Names, semantics, default-when-unset behavior, and presence may change at any minor or patch release without a semver bump. They exist solely to enable CI-stable SHA-pin tests; external consumers MUST NOT rely on them.
