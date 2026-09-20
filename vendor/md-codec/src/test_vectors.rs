//! Canonical `md` test-vector corpus.
//!
//! Used by `md-codec`'s own integration tests, by `md-cli`'s `vectors`
//! subcommand, and by `md-cli`'s `tests/json_snapshots.rs` /
//! `tests/template_roundtrip.rs`. Single source of truth: any vector
//! addition / removal / rename happens here.
//!
//! `Vector` is `#[non_exhaustive]` so future fields can be added without a
//! breaking-change bump: external consumers construct nothing — they only
//! read `MANIFEST` entries.

/// One entry of the canonical test-vector corpus.
#[non_exhaustive]
pub struct Vector {
    /// Vector identifier — used in test failure messages and as a stable
    /// handle for cross-suite filtering. Convention: snake_case mirroring
    /// the wallet-policy template's distinguishing structure.
    pub name: &'static str,
    /// BIP-388 wallet-policy template string the vector encodes. Parsed
    /// by `parse::template`; round-tripped through `encode` and `decode`.
    pub template: &'static str,
    /// `(@N, xpub)` pairs binding each `@N` placeholder in `template`. Empty
    /// when the vector exercises template-only paths (no key binding).
    pub keys: &'static [(u8, &'static str)],
    /// `(@N, 4-byte master fingerprint)` pairs aligned with `keys`. Empty
    /// when the vector does not exercise fingerprint round-tripping.
    pub fingerprints: &'static [(u8, [u8; 4])],
    /// When true, force the encoder onto the chunked wire path even if the
    /// payload would fit in a single chunk. Exercises chunk-boundary logic
    /// without padding the template artificially.
    pub force_chunked: bool,
    /// Explicit shared origin path applied via the encoder's `--path`
    /// override (`m/...` literal, or a named `bip44|48|49|84|86` form).
    /// `None` = elided origin — the encoder infers the canonical origin via
    /// `canonical_origin`. `Some(..)` is REQUIRED for non-canonical shapes
    /// (`tr()` + TapTree, NUMS-taproot) whose `canonical_origin` returns
    /// `None`: without an explicit origin they mint a card the decoder
    /// rejects with `MissingExplicitOrigin`. Carried into the emitted
    /// `.descriptor.json` via `path_decl` (the `.template` file alone does
    /// not determine it), so the BIP §Test Vectors table pins the
    /// template+path pair for path-carrying rows.
    pub path: Option<&'static str>,
}

/// The wallet-policy journey's four cosigners, master fingerprint 73c5da0a at
/// `m/48'/0'/{0..3}'/2'` -- the same public keys the keyed entries above bind,
/// named here because every `keyed_compose_*` vector below binds them in slot
/// order. NEVER put funds behind them.
const XPUB_JOURNEY_0: &str = "xpub6DkFAXWQ2dHxq2vatrt9qyA3bXYU4ToWQwCHbf5XB2mSTexcHZCeKS1VZYcPoBd5X8yVcbXFHJR9R8UCVpt82VX1VhR28mCyxUFL4r6KFrf";
const XPUB_JOURNEY_1: &str = "xpub6DzhyrnFFYQ1HimDiM388xHnDiRPNdZJFBmmxge3Y1WWcHLtMJLfRuhRHqnQCPbTj3fGKTuKFLHzzwpJkp5Dtc3UtLKZKaVZe1yqMBXd6Vk";
const XPUB_JOURNEY_2: &str = "xpub6EGx8sPr9FxPPE1rbZazhqWwpMXA3Hf5DYKtZbL7c4BSddzmQktp96UaTvecEkoCZysuaj79GMCFZYT1KKk7Ph2M3Kf5g8B82KZ8TZ9SKQR";
const XPUB_JOURNEY_3: &str = "xpub6E6Z3Ss5TXJYNJp4U1q3NZ3pCn82i7KXQAKUtNnzLJ3cCdchQeSdFvXemizaHUF7wNwRQAB8mPdoZhGHLiv49cWPtCnoJY3Az3E8JKxH9Mq";

/// The canonical 15-entry corpus.
///
/// Part-3 additions (BIP-alignment cycle): `sh_wpkh`, `tr_with_leaf`,
/// `nums_taproot`, `wsh_sortedmulti_2chunk`, and `single_string_boundary`.
///
/// * `sh_wpkh` — un-omitted: since F-A1, `sh(wpkh)` round-trips symmetrically
///   in ELIDED form (`canonical_origin(sh(wpkh))` = `m/49'/0'/0'`), so it is a
///   corpus ADDITION (`path: None`), not an asymmetric omission.
/// * `tr_with_leaf` / `nums_taproot` — non-canonical `tr()` shapes
///   (`canonical_origin` = `None`); expressible now via the `path` field,
///   which supplies the explicit origin the decoder requires.
/// * `wsh_sortedmulti_2chunk` — a genuine 2-member chunk set: a 2-of-8
///   sortedmulti with a master fingerprint on every cosigner makes a 376-bit
///   payload that still FITS a single string (below the 400-bit / 80-symbol
///   regular-code cap), so it is `force_chunked` to route it through `split`,
///   whose 320-bit per-chunk budget then yields two chunks. (Contrast
///   `wsh_multi_chunked`, also `force_chunked` but a chunk-set-of-one.)
/// * `single_string_boundary` (F-V2) — a 2-of-9 sortedmulti with fingerprints
///   on 8 of the 9 cosigners, sized so its single-string regular-code emit
///   lands at 79 of the 80 max data symbols (95 chars total = `md1` plus 79
///   data plus 13 checksum). Proves the regular-only single-string boundary
///   holds right at the codex32 BCH(93,80) cap — NOT chunked, NOT long-code.
#[rustfmt::skip]
pub const MANIFEST: &[Vector] = &[
    // ── KEYED CONFORMANCE VECTORS (R3, 2026-08-20) ─────────────────────────────
    //
    // Every entry above is KEYLESS, and `Vector::keys` was read by no code at all
    // -- `cmd/vectors.rs` passed `&[]` unconditionally -- so the corpus pinned
    // template bytes and nothing else. A Go port could agree with Rust about every
    // byte on the wire and still derive a different ADDRESS, and no vector would
    // say so. These entries close that: each one carries real xpubs, so the export
    // can emit descriptor strings, both wallet ids and per-chain addresses.
    //
    // THE KEYS ARE PUBLIC BY CONSTRUCTION and reproducible in one command. They are
    // BIP-39's own published test mnemonic --
    // "abandon abandon ... about" -- at `bip48-p2wsh` accounts 0..3:
    //
    // printf 'abandon abandon abandon abandon abandon abandon abandon \
    // abandon abandon abandon abandon about' \
    // | ms derive --phrase - --template bip48-p2wsh --account N
    //
    // Master fingerprint 73c5da0a. NEVER put funds behind them.
    //
    // ALL KEYED ENTRIES ARE `force_chunked`, and not as a style choice: real
    // xpubs push the payload past the codex32 regular code's 80-data-symbol cap
    // for a single string. A keyed 2-of-3 measures 474 symbols. Chunking is what a
    // full-policy card actually is.
    //
    // The keyless entries above are deliberately NOT converted: they pin the
    // template-mode wire bytes, which is a different contract, and rewriting them
    // would churn the whole committed corpus to test one more thing.
    Vector { name: "keyed_wpkh",          template: "wpkh(@0/<0;1>/*)",
        keys: &[(0, "xpub6DkFAXWQ2dHxq2vatrt9qyA3bXYU4ToWQwCHbf5XB2mSTexcHZCeKS1VZYcPoBd5X8yVcbXFHJR9R8UCVpt82VX1VhR28mCyxUFL4r6KFrf")],
        fingerprints: &[(0, [0x73, 0xc5, 0xda, 0x0a])], force_chunked: true, path: None },
    Vector { name: "keyed_wsh_multi_2of3", template: "wsh(multi(2,@0/48'/0'/0'/2'/<0;1>/*,@1/48'/0'/1'/2'/<0;1>/*,@2/48'/0'/2'/2'/<0;1>/*))",
        keys: &[(0, "xpub6DkFAXWQ2dHxq2vatrt9qyA3bXYU4ToWQwCHbf5XB2mSTexcHZCeKS1VZYcPoBd5X8yVcbXFHJR9R8UCVpt82VX1VhR28mCyxUFL4r6KFrf"), (1, "xpub6DzhyrnFFYQ1HimDiM388xHnDiRPNdZJFBmmxge3Y1WWcHLtMJLfRuhRHqnQCPbTj3fGKTuKFLHzzwpJkp5Dtc3UtLKZKaVZe1yqMBXd6Vk"), (2, "xpub6EGx8sPr9FxPPE1rbZazhqWwpMXA3Hf5DYKtZbL7c4BSddzmQktp96UaTvecEkoCZysuaj79GMCFZYT1KKk7Ph2M3Kf5g8B82KZ8TZ9SKQR")],
        fingerprints: &[(0, [0x73, 0xc5, 0xda, 0x0a]), (1, [0x73, 0xc5, 0xda, 0x0a]), (2, [0x73, 0xc5, 0xda, 0x0a])], force_chunked: true, path: None },
    Vector { name: "keyed_wsh_sortedmulti_2of3", template: "wsh(sortedmulti(2,@0/48'/0'/0'/2'/<0;1>/*,@1/48'/0'/1'/2'/<0;1>/*,@2/48'/0'/2'/2'/<0;1>/*))",
        keys: &[(0, "xpub6DkFAXWQ2dHxq2vatrt9qyA3bXYU4ToWQwCHbf5XB2mSTexcHZCeKS1VZYcPoBd5X8yVcbXFHJR9R8UCVpt82VX1VhR28mCyxUFL4r6KFrf"), (1, "xpub6DzhyrnFFYQ1HimDiM388xHnDiRPNdZJFBmmxge3Y1WWcHLtMJLfRuhRHqnQCPbTj3fGKTuKFLHzzwpJkp5Dtc3UtLKZKaVZe1yqMBXd6Vk"), (2, "xpub6EGx8sPr9FxPPE1rbZazhqWwpMXA3Hf5DYKtZbL7c4BSddzmQktp96UaTvecEkoCZysuaj79GMCFZYT1KKk7Ph2M3Kf5g8B82KZ8TZ9SKQR")],
        fingerprints: &[(0, [0x73, 0xc5, 0xda, 0x0a]), (1, [0x73, 0xc5, 0xda, 0x0a]), (2, [0x73, 0xc5, 0xda, 0x0a])], force_chunked: true, path: None },
    Vector { name: "keyed_tr_keyonly",     template: "tr(@0/<0;1>/*)",
        keys: &[(0, "xpub6DkFAXWQ2dHxq2vatrt9qyA3bXYU4ToWQwCHbf5XB2mSTexcHZCeKS1VZYcPoBd5X8yVcbXFHJR9R8UCVpt82VX1VhR28mCyxUFL4r6KFrf")],
        fingerprints: &[(0, [0x73, 0xc5, 0xda, 0x0a])], force_chunked: true, path: None },
    // THE SHAPE THE WHOLE CYCLE IS ABOUT, and the one no keyless vector could
    // price: a taproot script tree. It needs an explicit `path` because its
    // `canonical_origin` is None -- exactly the case R4's `--path` on
    // address/verify exists to reach.
    Vector { name: "keyed_tr_with_leaf",   template: "tr(@0/48'/0'/0'/2'/<0;1>/*,pk(@1/48'/0'/1'/2'/<0;1>/*))",
        keys: &[(0, "xpub6DkFAXWQ2dHxq2vatrt9qyA3bXYU4ToWQwCHbf5XB2mSTexcHZCeKS1VZYcPoBd5X8yVcbXFHJR9R8UCVpt82VX1VhR28mCyxUFL4r6KFrf"), (1, "xpub6DzhyrnFFYQ1HimDiM388xHnDiRPNdZJFBmmxge3Y1WWcHLtMJLfRuhRHqnQCPbTj3fGKTuKFLHzzwpJkp5Dtc3UtLKZKaVZe1yqMBXd6Vk")],
        fingerprints: &[(0, [0x73, 0xc5, 0xda, 0x0a]), (1, [0x73, 0xc5, 0xda, 0x0a])], force_chunked: true, path: None },
    // DEPTH-2, unbalanced: leaf depths (2,2,1). This is the shape the pre-#953
    // renderer flattened, so before the ff4732e pin it could not have had a
    // conformance record at all -- the descriptor string would have been
    // unparseable.
    // R5: `sortedmulti_a` at a taproot LEAF -- legal per BIP-386/387 and, until
    // Stage 3, encodable but not derivable. Order-invariant by construction,
    // which is what distinguishes it from `multi_a` and what the address in this
    // record pins.
    // THE PATHOLOGICAL JOURNEY'S OWN FRAGMENT SET, at three keys instead of
    // eleven: or_i + and_v + the `v:` wrapper + after + older + sha256 + multi.
    // That policy is a real wsh() wallet this repo already engraves and
    // documents, so it is the honest target for a segwit-v0 script emitter --
    // and every fragment here appears in it.
    //
    // BOTH multis use k != n (2-of-3 and 1-of-2), and that is deliberate. With
    // 2-of-2 and 1-of-1 a mutation that emits `n` before `k` is INVISIBLE --
    // measured: it passed. A fixture has to distinguish the parameters it
    // claims to cover.
    //
    // The `v:` wrapper is the subtle one: it MERGES into the wrapped fragment's
    // terminator (CHECKSIG -> CHECKSIGVERIFY, EQUAL -> EQUALVERIFY) rather than
    // appending OP_VERIFY, so an emitter that always appends produces a script
    // that still looks plausible and hashes to a different address.
    // THE DEGRADING-MULTISIG IDIOM: a 2-of-2 today, or one key after a
    // relative timelock. `or_d` is the fragment that makes it work, and its
    // script form (OP_IFDUP OP_NOTIF) is unlike every other or_*.
    Vector { name: "keyed_wsh_or_d_degrading", template: "wsh(or_d(multi(2,@0/48'/0'/0'/2'/<0;1>/*,@1/48'/0'/1'/2'/<0;1>/*),and_v(v:older(65535),pk(@2/48'/0'/2'/2'/<0;1>/*))))",
        keys: &[(0, "xpub6DkFAXWQ2dHxq2vatrt9qyA3bXYU4ToWQwCHbf5XB2mSTexcHZCeKS1VZYcPoBd5X8yVcbXFHJR9R8UCVpt82VX1VhR28mCyxUFL4r6KFrf"), (1, "xpub6DzhyrnFFYQ1HimDiM388xHnDiRPNdZJFBmmxge3Y1WWcHLtMJLfRuhRHqnQCPbTj3fGKTuKFLHzzwpJkp5Dtc3UtLKZKaVZe1yqMBXd6Vk"), (2, "xpub6EGx8sPr9FxPPE1rbZazhqWwpMXA3Hf5DYKtZbL7c4BSddzmQktp96UaTvecEkoCZysuaj79GMCFZYT1KKk7Ph2M3Kf5g8B82KZ8TZ9SKQR")],
        fingerprints: &[(0, [0x73, 0xc5, 0xda, 0x0a]), (1, [0x73, 0xc5, 0xda, 0x0a]), (2, [0x73, 0xc5, 0xda, 0x0a])], force_chunked: true, path: None },
    // `thresh` over sub-policies, with the `s:` (OP_SWAP) wrapper on all but
    // the first -- the canonical thresh shape. k=2 of 3 branches, so a
    // mutation that emits the branch count instead of the threshold shows.
    Vector { name: "keyed_wsh_thresh", template: "wsh(thresh(2,pk(@0/48'/0'/0'/2'/<0;1>/*),s:pk(@1/48'/0'/1'/2'/<0;1>/*),s:pk(@2/48'/0'/2'/2'/<0;1>/*)))",
        keys: &[(0, "xpub6DkFAXWQ2dHxq2vatrt9qyA3bXYU4ToWQwCHbf5XB2mSTexcHZCeKS1VZYcPoBd5X8yVcbXFHJR9R8UCVpt82VX1VhR28mCyxUFL4r6KFrf"), (1, "xpub6DzhyrnFFYQ1HimDiM388xHnDiRPNdZJFBmmxge3Y1WWcHLtMJLfRuhRHqnQCPbTj3fGKTuKFLHzzwpJkp5Dtc3UtLKZKaVZe1yqMBXd6Vk"), (2, "xpub6EGx8sPr9FxPPE1rbZazhqWwpMXA3Hf5DYKtZbL7c4BSddzmQktp96UaTvecEkoCZysuaj79GMCFZYT1KKk7Ph2M3Kf5g8B82KZ8TZ9SKQR")],
        fingerprints: &[(0, [0x73, 0xc5, 0xda, 0x0a]), (1, [0x73, 0xc5, 0xda, 0x0a]), (2, [0x73, 0xc5, 0xda, 0x0a])], force_chunked: true, path: None },
    // `or_b` (OP_BOOLOR) with `s:`. Kept separate from thresh because they
    // share the `s:` wrapper but differ in terminator, so one vector could
    // pass while the other's rule was wrong.
    Vector { name: "keyed_wsh_or_b", template: "wsh(or_b(pk(@0/48'/0'/0'/2'/<0;1>/*),s:pk(@1/48'/0'/1'/2'/<0;1>/*)))",
        // TWO keys, because the template uses two. The first draft reused a
        // three-key block and produced a card carrying a pubkey for @2 that the
        // template never references -- which Rust encoded and the Go port
        // REFUSED on re-encode ("override order violation"). Filed as F-213;
        // the vector is not the place to exercise that disagreement.
        keys: &[(0, "xpub6DkFAXWQ2dHxq2vatrt9qyA3bXYU4ToWQwCHbf5XB2mSTexcHZCeKS1VZYcPoBd5X8yVcbXFHJR9R8UCVpt82VX1VhR28mCyxUFL4r6KFrf"), (1, "xpub6DzhyrnFFYQ1HimDiM388xHnDiRPNdZJFBmmxge3Y1WWcHLtMJLfRuhRHqnQCPbTj3fGKTuKFLHzzwpJkp5Dtc3UtLKZKaVZe1yqMBXd6Vk")],
        fingerprints: &[(0, [0x73, 0xc5, 0xda, 0x0a]), (1, [0x73, 0xc5, 0xda, 0x0a])], force_chunked: true, path: None },
    // TWO SPENDING CLAUSES OVER FIVE DISTINCT KEYS. Until the mdcli-mini cycle
    // the recovery clause reused @1 and @2 from the primary clause at the
    // IDENTICAL path expression -- BIP 388's own forbidden example, and what
    // R-N1a (`design/SPEC_mdcli_mini.md`) now refuses to mint. The recovery
    // clause therefore takes FRESH placeholders. The vector's role is
    // unchanged: it is still the degrading-multisig shape, a timelock and a
    // hashlock over one `or_i` with different thresholds per clause, and it is
    // still the corpus's only `sha256` + `after`/`older` combination.
    //
    // Operator ruling 2026-08-31, verbatim: "No carve out for reused keys
    // unless different origin paths." A degrading vault that hands the SAME
    // cosigner the recovery path is exactly the shape that ruling declines to
    // carve out, so the corpus may not pin one.
    //
    // @4 is under the SECOND master (b8688df1) because 73c5da0a's records stop
    // at 48'/0'/3'/2'. Every [fingerprint/path] in the descriptor is bound to
    // exactly one xpub, which `md-cli/tests/corpus_origin_consistency.rs`
    // checks (F-217: a corpus may not pin a wallet that cannot exist).
    Vector { name: "keyed_wsh_timelock_hashlock", template: "wsh(or_i(and_v(v:after(1000000),and_v(v:sha256(a84dce40975727c398023cfbd50d5db3b9662375521d0f1ac62dbd829b9a08ad),multi(2,@0/48'/0'/0'/2'/<0;1>/*,@1/48'/0'/1'/2'/<0;1>/*,@2/48'/0'/2'/2'/<0;1>/*))),and_v(v:older(65535),multi(1,@3/48'/0'/3'/2'/<0;1>/*,@4/48'/0'/0'/2'/<0;1>/*))))",
        keys: &[(0, "xpub6DkFAXWQ2dHxq2vatrt9qyA3bXYU4ToWQwCHbf5XB2mSTexcHZCeKS1VZYcPoBd5X8yVcbXFHJR9R8UCVpt82VX1VhR28mCyxUFL4r6KFrf"), (1, "xpub6DzhyrnFFYQ1HimDiM388xHnDiRPNdZJFBmmxge3Y1WWcHLtMJLfRuhRHqnQCPbTj3fGKTuKFLHzzwpJkp5Dtc3UtLKZKaVZe1yqMBXd6Vk"), (2, "xpub6EGx8sPr9FxPPE1rbZazhqWwpMXA3Hf5DYKtZbL7c4BSddzmQktp96UaTvecEkoCZysuaj79GMCFZYT1KKk7Ph2M3Kf5g8B82KZ8TZ9SKQR"), (3, "xpub6E6Z3Ss5TXJYNJp4U1q3NZ3pCn82i7KXQAKUtNnzLJ3cCdchQeSdFvXemizaHUF7wNwRQAB8mPdoZhGHLiv49cWPtCnoJY3Az3E8JKxH9Mq"), (4, "xpub6FQya7zGhR92kacYsNnjreouvnHJMpXYsUXnW6NJJAJRCKsa26TzDy4LdnGhEurr3d6y1J8PJ7EEMKQp74XTqYvmGJNogYXSKDszYHtF8mX")],
        fingerprints: &[(0, [0x73, 0xc5, 0xda, 0x0a]), (1, [0x73, 0xc5, 0xda, 0x0a]), (2, [0x73, 0xc5, 0xda, 0x0a]), (3, [0x73, 0xc5, 0xda, 0x0a]), (4, [0xb8, 0x68, 0x8d, 0xf1])], force_chunked: true, path: None },
    // `sortedmulti_a` AT A TAPROOT LEAF, with a key-path spend beside it.
    //
    // The internal key gets a placeholder that appears NOWHERE in the leaf.
    // Until the mdcli-mini cycle it was @0 in both positions at the identical
    // path expression -- one placeholder at two use sites, which BIP 388 names
    // among its forbidden templates and R-N1a
    // (`design/SPEC_mdcli_mini.md`) now refuses to mint. The vector's role is
    // untouched: a keyed `tr` with BOTH a key path and a script path, and the
    // leaf still holds two distinct keys.
    Vector { name: "keyed_tr_sortedmulti_a", template: "tr(@0/48'/0'/0'/2'/<0;1>/*,sortedmulti_a(2,@1/48'/0'/1'/2'/<0;1>/*,@2/48'/0'/2'/2'/<0;1>/*))",
        keys: &[(0, "xpub6DkFAXWQ2dHxq2vatrt9qyA3bXYU4ToWQwCHbf5XB2mSTexcHZCeKS1VZYcPoBd5X8yVcbXFHJR9R8UCVpt82VX1VhR28mCyxUFL4r6KFrf"), (1, "xpub6DzhyrnFFYQ1HimDiM388xHnDiRPNdZJFBmmxge3Y1WWcHLtMJLfRuhRHqnQCPbTj3fGKTuKFLHzzwpJkp5Dtc3UtLKZKaVZe1yqMBXd6Vk"), (2, "xpub6EGx8sPr9FxPPE1rbZazhqWwpMXA3Hf5DYKtZbL7c4BSddzmQktp96UaTvecEkoCZysuaj79GMCFZYT1KKk7Ph2M3Kf5g8B82KZ8TZ9SKQR")],
        fingerprints: &[(0, [0x73, 0xc5, 0xda, 0x0a]), (1, [0x73, 0xc5, 0xda, 0x0a]), (2, [0x73, 0xc5, 0xda, 0x0a])], force_chunked: true, path: None },
    // UNSORTED multi_a, and it is the ONLY order-SENSITIVE tap leaf in the
    // corpus. Every other multi-key leaf here is sortedmulti_a, which sorts on
    // the derived keys and therefore reads the same whichever order a consumer
    // walks them in -- so until this vector existed, "preserve the WRITTEN key
    // order in a tap leaf" was asserted by nothing. Found by mutation: reversing
    // a leaf's key indices in the Go port passed the entire suite.
    //
    // Two keys IN THE LEAF is enough. Any permutation of two is a reversal, and
    // a reversal changes the emitted script, so a wrong order cannot round-trip
    // to the same address.
    //
    // The internal key is a THIRD placeholder, appearing nowhere in the leaf.
    // It was @0 in both positions until the mdcli-mini cycle, which R-N1a
    // (`design/SPEC_mdcli_mini.md`) now refuses to mint. The order-sensitivity
    // role survives untouched -- what it needs is two DISTINCT keys in the leaf
    // in a written order, and the leaf carried exactly that before and after.
    Vector { name: "keyed_tr_multi_a", template: "tr(@0/48'/0'/0'/2'/<0;1>/*,multi_a(2,@1/48'/0'/1'/2'/<0;1>/*,@2/48'/0'/2'/2'/<0;1>/*))",
        keys: &[(0, "xpub6DkFAXWQ2dHxq2vatrt9qyA3bXYU4ToWQwCHbf5XB2mSTexcHZCeKS1VZYcPoBd5X8yVcbXFHJR9R8UCVpt82VX1VhR28mCyxUFL4r6KFrf"), (1, "xpub6DzhyrnFFYQ1HimDiM388xHnDiRPNdZJFBmmxge3Y1WWcHLtMJLfRuhRHqnQCPbTj3fGKTuKFLHzzwpJkp5Dtc3UtLKZKaVZe1yqMBXd6Vk"), (2, "xpub6EGx8sPr9FxPPE1rbZazhqWwpMXA3Hf5DYKtZbL7c4BSddzmQktp96UaTvecEkoCZysuaj79GMCFZYT1KKk7Ph2M3Kf5g8B82KZ8TZ9SKQR")],
        fingerprints: &[(0, [0x73, 0xc5, 0xda, 0x0a]), (1, [0x73, 0xc5, 0xda, 0x0a]), (2, [0x73, 0xc5, 0xda, 0x0a])], force_chunked: true, path: None },
    // RIGHT-SPINE depth-2: {A,{B,C}}, leaf depths (1,2,2) -- the MIRROR of
    // keyed_tr_depth2's (2,2,1). Both are needed because a tree-rebuilding bug
    // can be chirality-dependent: mutation testing showed that "combine the top
    // two nodes unconditionally, ignoring depth" produces the CORRECT root for a
    // left-heavy tree and a wrong one here. With only the left-heavy vector the
    // mutation passed; the pair catches it.
    // THE PATHOLOGICAL WALLET, in taproot form -- the most demanding vector in
    // this corpus and the reason it exists. Four tiers of a degrading vault as
    // a DEPTH-3 taproot tree: two `sha256` hashlocks, all four Bitcoin timelock
    // flavours across `after`/`older`, `multi_a` at thresholds 3/2/2/1, and the
    // NUMS internal key (script paths only, no key-path spend).
    //
    // Every leaf is `and_v(v:...)` wrapping a timelock or hashlock, which is
    // precisely the shape F-214 was filed for: the fork's tap-leaf DESCRIBER
    // named only pk / multi_a / sortedmulti_a, so it refused all four while the
    // primary derived them without complaint. This vector is what stops that
    // divergence coming back.
    //
    // Depth 3 also matters on its own: every other tr vector here is depth <= 2,
    // so the tree-shape arithmetic was untested one level down.
    //
    // ELEVEN DISTINCT accounts, 48'/0'/N'/2' for N in 0..10, each declared in
    // the TEMPLATE rather than via --path -- eleven keys under one master is
    // exactly where F-217's flattening would bind one origin to eleven keys.
    Vector { name: "keyed_tr_pathological", template: "tr(50929b74c1a04954b78b4b6035e97a5e078a5a0f28ec96d547bfee9ace803ac0,{and_v(v:after(1000000),and_v(v:sha256(a84dce40975727c398023cfbd50d5db3b9662375521d0f1ac62dbd829b9a08ad),multi_a(3,@0/48'/0'/0'/2'/<0;1>/*,@1/48'/0'/1'/2'/<0;1>/*,@2/48'/0'/2'/2'/<0;1>/*))),{and_v(v:after(1893456000),and_v(v:sha256(a84dce40975727c398023cfbd50d5db3b9662375521d0f1ac62dbd829b9a08ad),multi_a(2,@3/48'/0'/3'/2'/<0;1>/*,@4/48'/0'/4'/2'/<0;1>/*,@5/48'/0'/5'/2'/<0;1>/*))),{and_v(v:older(65535),multi_a(2,@6/48'/0'/6'/2'/<0;1>/*,@7/48'/0'/7'/2'/<0;1>/*)),and_v(v:older(4255898),multi_a(1,@8/48'/0'/8'/2'/<0;1>/*,@9/48'/0'/9'/2'/<0;1>/*,@10/48'/0'/10'/2'/<0;1>/*))}}})",
        keys: &[(0, "xpub6DkFAXWQ2dHxq2vatrt9qyA3bXYU4ToWQwCHbf5XB2mSTexcHZCeKS1VZYcPoBd5X8yVcbXFHJR9R8UCVpt82VX1VhR28mCyxUFL4r6KFrf"), (1, "xpub6DzhyrnFFYQ1HimDiM388xHnDiRPNdZJFBmmxge3Y1WWcHLtMJLfRuhRHqnQCPbTj3fGKTuKFLHzzwpJkp5Dtc3UtLKZKaVZe1yqMBXd6Vk"), (2, "xpub6EGx8sPr9FxPPE1rbZazhqWwpMXA3Hf5DYKtZbL7c4BSddzmQktp96UaTvecEkoCZysuaj79GMCFZYT1KKk7Ph2M3Kf5g8B82KZ8TZ9SKQR"), (3, "xpub6E6Z3Ss5TXJYNJp4U1q3NZ3pCn82i7KXQAKUtNnzLJ3cCdchQeSdFvXemizaHUF7wNwRQAB8mPdoZhGHLiv49cWPtCnoJY3Az3E8JKxH9Mq"), (4, "xpub6EhpCqtVqedgGvswhRdYH3pTh3z7SXMKQWX5LWiAafipEJXvZsoH5RbtQcj2QZV2sT77KmUHpHF9Yh72N47vCqYGuqpw9bjBoFcdeiV7kyM"), (5, "xpub6EzwjvpuQiAFs6x3n1CKaBy8ZSdDyRsJWAMYzSLLbJ1PWZa7CHREmJVQZrixNrYsZ2gwEcPpQGAHXpYzf7evYbKxGL8hBBw5uma8tM6JTos"), (6, "xpub6EoPBmSpHj5nnnRje1Y6LHRGKgG16FSL4HKwDaiY1sKBJecAi6UTtbWU7ZF7G5HeHxgd6Dn7XbHqmbAVMBCFogZFVw7WDVRfWxqVeNwPf3x"), (7, "xpub6ErD7N4g3RwvUsj9zk5SseA2JYpR9izv9SDQMst26jL1htYXgQaLUNDdx7uK4tjYa5m5ztYMJX53njGKZYv9iRQ7Ninwjwc7URhXefMDhV6"), (8, "xpub6EQuyDGuE2bnFfgUcSSUBgfc5Xj4bZZ3qJLdHYDsXGBPiwm7ux2zpZurcVV7C4WS9kSjxetdsNDn4znDC9LogP8e9WBeA5zRPPEPZ6e2Vb9"), (9, "xpub6Dx8Loh39ugPD29DFTkre9XoVHBnQt9Knzx7ZN5deYd4oQxKrveQgS9sUQtJ1M8W6nFmQVwayV1t9QD5oQYAym96MHu6TY4Snm7LkYS1GgD"), (10, "xpub6DkyqSfHVB9mjk1MyTSUcHH1YCu6tL18b2sD36stDRdd8TfX6gL2r2scLsc42MZqZxVFJhvh5mzZyCQve4WbeKGUhzkvxuvqnWiWkB4ZNNE")],
        fingerprints: &[(0, [0x73, 0xc5, 0xda, 0x0a]), (1, [0x73, 0xc5, 0xda, 0x0a]), (2, [0x73, 0xc5, 0xda, 0x0a]), (3, [0x73, 0xc5, 0xda, 0x0a]), (4, [0x73, 0xc5, 0xda, 0x0a]), (5, [0x73, 0xc5, 0xda, 0x0a]), (6, [0x73, 0xc5, 0xda, 0x0a]), (7, [0x73, 0xc5, 0xda, 0x0a]), (8, [0x73, 0xc5, 0xda, 0x0a]), (9, [0x73, 0xc5, 0xda, 0x0a]), (10, [0x73, 0xc5, 0xda, 0x0a])], force_chunked: true, path: None },
    Vector { name: "keyed_tr_depth2_rightspine", template: "tr(@0/48'/0'/0'/2'/<0;1>/*,{pk(@1/48'/0'/1'/2'/<0;1>/*),{pk(@2/48'/0'/2'/2'/<0;1>/*),pk(@3/48'/0'/3'/2'/<0;1>/*)}})",
        keys: &[(0, "xpub6DkFAXWQ2dHxq2vatrt9qyA3bXYU4ToWQwCHbf5XB2mSTexcHZCeKS1VZYcPoBd5X8yVcbXFHJR9R8UCVpt82VX1VhR28mCyxUFL4r6KFrf"), (1, "xpub6DzhyrnFFYQ1HimDiM388xHnDiRPNdZJFBmmxge3Y1WWcHLtMJLfRuhRHqnQCPbTj3fGKTuKFLHzzwpJkp5Dtc3UtLKZKaVZe1yqMBXd6Vk"), (2, "xpub6EGx8sPr9FxPPE1rbZazhqWwpMXA3Hf5DYKtZbL7c4BSddzmQktp96UaTvecEkoCZysuaj79GMCFZYT1KKk7Ph2M3Kf5g8B82KZ8TZ9SKQR"), (3, "xpub6E6Z3Ss5TXJYNJp4U1q3NZ3pCn82i7KXQAKUtNnzLJ3cCdchQeSdFvXemizaHUF7wNwRQAB8mPdoZhGHLiv49cWPtCnoJY3Az3E8JKxH9Mq")],
        fingerprints: &[(0, [0x73, 0xc5, 0xda, 0x0a]), (1, [0x73, 0xc5, 0xda, 0x0a]), (2, [0x73, 0xc5, 0xda, 0x0a]), (3, [0x73, 0xc5, 0xda, 0x0a])], force_chunked: true, path: None },
    Vector { name: "keyed_tr_depth2",      template: "tr(@0/48'/0'/0'/2'/<0;1>/*,{{pk(@1/48'/0'/1'/2'/<0;1>/*),pk(@2/48'/0'/2'/2'/<0;1>/*)},pk(@3/48'/0'/3'/2'/<0;1>/*)})",
        keys: &[(0, "xpub6DkFAXWQ2dHxq2vatrt9qyA3bXYU4ToWQwCHbf5XB2mSTexcHZCeKS1VZYcPoBd5X8yVcbXFHJR9R8UCVpt82VX1VhR28mCyxUFL4r6KFrf"), (1, "xpub6DzhyrnFFYQ1HimDiM388xHnDiRPNdZJFBmmxge3Y1WWcHLtMJLfRuhRHqnQCPbTj3fGKTuKFLHzzwpJkp5Dtc3UtLKZKaVZe1yqMBXd6Vk"), (2, "xpub6EGx8sPr9FxPPE1rbZazhqWwpMXA3Hf5DYKtZbL7c4BSddzmQktp96UaTvecEkoCZysuaj79GMCFZYT1KKk7Ph2M3Kf5g8B82KZ8TZ9SKQR"), (3, "xpub6E6Z3Ss5TXJYNJp4U1q3NZ3pCn82i7KXQAKUtNnzLJ3cCdchQeSdFvXemizaHUF7wNwRQAB8mPdoZhGHLiv49cWPtCnoJY3Az3E8JKxH9Mq")],
        fingerprints: &[(0, [0x73, 0xc5, 0xda, 0x0a]), (1, [0x73, 0xc5, 0xda, 0x0a]), (2, [0x73, 0xc5, 0xda, 0x0a]), (3, [0x73, 0xc5, 0xda, 0x0a])], force_chunked: true, path: None },
    Vector { name: "wpkh_basic",         template: "wpkh(@0/<0;1>/*)",                                   keys: &[], fingerprints: &[], force_chunked: false, path: None },
    Vector { name: "pkh_basic",          template: "pkh(@0/<0;1>/*)",                                    keys: &[], fingerprints: &[], force_chunked: false, path: None },
    Vector { name: "wsh_multi_2of2",     template: "wsh(multi(2,@0/<0;1>/*,@1/<0;1>/*))",                keys: &[], fingerprints: &[], force_chunked: false, path: None },
    Vector { name: "wsh_multi_2of3",     template: "wsh(multi(2,@0/<0;1>/*,@1/<0;1>/*,@2/<0;1>/*))",     keys: &[], fingerprints: &[], force_chunked: false, path: None },
    Vector { name: "wsh_sortedmulti",    template: "wsh(sortedmulti(2,@0/<0;1>/*,@1/<0;1>/*,@2/<0;1>/*))", keys: &[], fingerprints: &[], force_chunked: false, path: None },
    Vector { name: "tr_keyonly",         template: "tr(@0/<0;1>/*)",                                     keys: &[], fingerprints: &[], force_chunked: false, path: None },
    Vector { name: "sh_wsh_multi",       template: "sh(wsh(multi(2,@0/<0;1>/*,@1/<0;1>/*)))",            keys: &[], fingerprints: &[], force_chunked: false, path: None },
    Vector { name: "wsh_divergent_paths", template: "wsh(multi(2,@0/<0;1>/*,@1/<2;3>/*))",               keys: &[], fingerprints: &[], force_chunked: false, path: None },
    Vector { name: "wsh_with_fingerprints", template: "wsh(multi(2,@0/<0;1>/*,@1/<0;1>/*))",
        keys: &[],
        fingerprints: &[(0, [0xDE,0xAD,0xBE,0xEF]), (1, [0xCA,0xFE,0xBA,0xBE])],
        force_chunked: false, path: None },
    Vector { name: "wsh_multi_chunked",  template: "wsh(multi(3,@0/<0;1>/*,@1/<0;1>/*,@2/<0;1>/*))",     keys: &[], fingerprints: &[], force_chunked: true, path: None },
    // --- Part-3 additions ---
    // F-A1: elided sh(wpkh) now round-trips (canonical origin m/49'/0'/0').
    Vector { name: "sh_wpkh",            template: "sh(wpkh(@0/<0;1>/*))",                               keys: &[], fingerprints: &[], force_chunked: false, path: None },
    // Non-canonical tr()+leaf — explicit origin via the new `path` field.
    Vector { name: "tr_with_leaf",       template: "tr(@0/<0;1>/*,pk(@1/<0;1>/*))",                      keys: &[], fingerprints: &[], force_chunked: false, path: Some("48'/0'/0'/2'") },
    // NUMS-taproot (`is_nums = 1` wire path) — script-path-only tr, explicit origin.
    Vector { name: "nums_taproot",       template: "tr(50929b74c1a04954b78b4b6035e97a5e078a5a0f28ec96d547bfee9ace803ac0,multi_a(2,@0/<0;1>/*,@1/<0;1>/*,@2/<0;1>/*))",
        keys: &[], fingerprints: &[], force_chunked: false, path: Some("48'/0'/0'/2'") },
    // 2-of-8 sortedmulti + fingerprint on every cosigner: a 376-bit payload
    // that FITS a single string (< the 400-bit regular-code cap), so
    // `force_chunked` routes it through `split`, whose 320-bit per-chunk budget
    // yields a genuine 2-member chunk set.
    Vector { name: "wsh_sortedmulti_2chunk",
        template: "wsh(sortedmulti(2,@0/<0;1>/*,@1/<0;1>/*,@2/<0;1>/*,@3/<0;1>/*,@4/<0;1>/*,@5/<0;1>/*,@6/<0;1>/*,@7/<0;1>/*))",
        keys: &[],
        fingerprints: &[
            (0, [0x01,0x02,0x03,0x04]), (1, [0x02,0x03,0x04,0x05]),
            (2, [0x03,0x04,0x05,0x06]), (3, [0x04,0x05,0x06,0x07]),
            (4, [0x05,0x06,0x07,0x08]), (5, [0x06,0x07,0x08,0x09]),
            (6, [0x07,0x08,0x09,0x0A]), (7, [0x08,0x09,0x0A,0x0B]),
        ],
        force_chunked: true, path: None },
    // F-V2: single-string regular-code boundary. 2-of-9 sortedmulti with a
    // fingerprint on 8 of the 9 cosigners sizes the payload to 79 of the 80
    // max data symbols → a single 95-char md1 string (NOT chunked, NOT long).
    Vector { name: "single_string_boundary",
        template: "wsh(sortedmulti(2,@0/<0;1>/*,@1/<0;1>/*,@2/<0;1>/*,@3/<0;1>/*,@4/<0;1>/*,@5/<0;1>/*,@6/<0;1>/*,@7/<0;1>/*,@8/<0;1>/*))",
        keys: &[],
        fingerprints: &[
            (0, [0x01,0x02,0x03,0x04]), (1, [0x02,0x03,0x04,0x05]),
            (2, [0x03,0x04,0x05,0x06]), (3, [0x04,0x05,0x06,0x07]),
            (4, [0x05,0x06,0x07,0x08]), (5, [0x06,0x07,0x08,0x09]),
            (6, [0x07,0x08,0x09,0x0A]), (7, [0x08,0x09,0x0A,0x0B]),
        ],
        force_chunked: false, path: None },
    // THE COMPOSER'S OWN OUTPUT, keyed and derivable: three spend paths under
    // one wsh wrapper carrying `pkh`, a RELATIVE timelock, an ABSOLUTE timelock
    // and a `sha256` hashlock. It exists because the device can now BUILD this
    // shape (hashlock H6 + the composer's lock editor) and nothing pinned that
    // an address derived from it agrees with this crate's -- the fork's
    // `TestDeviceDerivesTheTimelockHashlockPolicy` asserts exactly that against
    // this vector's conformance record.
    //
    // The digest is the hashlock corpus's own anchor: sha256 of sha256 of
    // "correct horse battery staple" (hashlock-v0.8.json derivation[0].sha256_h),
    // so a preimage for it exists and is written down rather than being a
    // 32-byte constant nobody can open.
    Vector { name: "keyed_compose_wsh_timelock_hashlock",
        template: "wsh(or_i(pkh(@0/48'/0'/0'/2'/<0;1>/*),or_i(and_v(v:pkh(@1/48'/0'/1'/2'/<0;1>/*),older(144)),and_v(v:pkh(@2/48'/0'/2'/2'/<0;1>/*),and_v(v:sha256(b867db875479bcc0287352cdaa4a1755689b8338777d0915e9acd9f6edbc96cb),after(800000))))))",
        keys: &[(0, "xpub6DkFAXWQ2dHxq2vatrt9qyA3bXYU4ToWQwCHbf5XB2mSTexcHZCeKS1VZYcPoBd5X8yVcbXFHJR9R8UCVpt82VX1VhR28mCyxUFL4r6KFrf"), (1, "xpub6DzhyrnFFYQ1HimDiM388xHnDiRPNdZJFBmmxge3Y1WWcHLtMJLfRuhRHqnQCPbTj3fGKTuKFLHzzwpJkp5Dtc3UtLKZKaVZe1yqMBXd6Vk"), (2, "xpub6EGx8sPr9FxPPE1rbZazhqWwpMXA3Hf5DYKtZbL7c4BSddzmQktp96UaTvecEkoCZysuaj79GMCFZYT1KKk7Ph2M3Kf5g8B82KZ8TZ9SKQR")],
        fingerprints: &[(0, [0x73, 0xc5, 0xda, 0x0a]), (1, [0x73, 0xc5, 0xda, 0x0a]), (2, [0x73, 0xc5, 0xda, 0x0a])],
        force_chunked: true, path: None },

    // ── COMPOSE VECTORS (composer S0, 2026-09-02) ──────────────────────────────
    //
    // The FIXED lowering's own corpus (SPEC_wallet_policy_composer.md §12 item 1).
    // Every template below is pasted VERBATIM from
    // `compose_vectors::print_family_templates_for_the_manifest` -- the inline-origin
    // form `compose::template_with_origins` emits -- never retyped. The tag table that
    // says what each one covers lives in `tests/compose_support.rs::family()`, and
    // `compose_vectors.rs` asserts this list and that one agree in both directions.
    //
    // The two keyless-wsh family entries are deliberately ABSENT: the exporter and the
    // corpus tests parse under the MINTING disposition, which refuses a signature-free
    // spend path without --experimental. They stay pinned by `family()` and the §5b
    // cross-check instead.
    Vector { name: "keyed_compose_wsh_sole_sortedmulti",
        template: "wsh(sortedmulti(2,@0/48'/0'/0'/2'/<0;1>/*,@1/48'/0'/1'/2'/<0;1>/*,@2/48'/0'/2'/2'/<0;1>/*))",
        keys: &[(0, XPUB_JOURNEY_0), (1, XPUB_JOURNEY_1), (2, XPUB_JOURNEY_2)],
        fingerprints: &[(0, [0x73, 0xc5, 0xda, 0x0a]), (1, [0x73, 0xc5, 0xda, 0x0a]), (2, [0x73, 0xc5, 0xda, 0x0a])],
        force_chunked: true, path: None },
    Vector { name: "keyed_compose_wsh_two_path_or_d",
        template: "wsh(or_d(multi(2,@0/48'/0'/0'/2'/<0;1>/*,@1/48'/0'/1'/2'/<0;1>/*,@2/48'/0'/2'/2'/<0;1>/*),and_v(v:pkh(@3/48'/0'/3'/2'/<0;1>/*),older(26280))))",
        keys: &[(0, XPUB_JOURNEY_0), (1, XPUB_JOURNEY_1), (2, XPUB_JOURNEY_2), (3, XPUB_JOURNEY_3)],
        fingerprints: &[(0, [0x73, 0xc5, 0xda, 0x0a]), (1, [0x73, 0xc5, 0xda, 0x0a]), (2, [0x73, 0xc5, 0xda, 0x0a]), (3, [0x73, 0xc5, 0xda, 0x0a])],
        force_chunked: true, path: None },
    Vector { name: "keyed_compose_wsh_two_path_distinct_fingerprints",
        template: "wsh(or_d(multi(2,@0/48'/0'/0'/2'/<0;1>/*,@1/48'/0'/1'/2'/<0;1>/*,@2/48'/0'/2'/2'/<0;1>/*),and_v(v:pkh(@3/48'/0'/3'/2'/<0;1>/*),older(26280))))",
        keys: &[(0, XPUB_JOURNEY_0), (1, XPUB_JOURNEY_1), (2, XPUB_JOURNEY_2), (3, XPUB_JOURNEY_3)],
        fingerprints: &[(0, [0x11; 4]), (1, [0x22; 4]), (2, [0x33; 4]), (3, [0x44; 4])],
        force_chunked: true, path: None },
    Vector { name: "keyed_compose_wsh_single_head_or_i",
        template: "wsh(or_i(pkh(@0/48'/0'/0'/2'/<0;1>/*),and_v(v:pkh(@1/48'/0'/1'/2'/<0;1>/*),older(4209492))))",
        keys: &[(0, XPUB_JOURNEY_0), (1, XPUB_JOURNEY_1)],
        fingerprints: &[(0, [0x73, 0xc5, 0xda, 0x0a]), (1, [0x73, 0xc5, 0xda, 0x0a])],
        force_chunked: true, path: None },
    Vector { name: "keyed_compose_wsh_locked_head_or_i",
        template: "wsh(or_i(and_v(v:multi(2,@0/48'/0'/0'/2'/<0;1>/*,@1/48'/0'/1'/2'/<0;1>/*),after(905000)),pkh(@2/48'/0'/2'/2'/<0;1>/*)))",
        keys: &[(0, XPUB_JOURNEY_0), (1, XPUB_JOURNEY_1), (2, XPUB_JOURNEY_2)],
        fingerprints: &[(0, [0x73, 0xc5, 0xda, 0x0a]), (1, [0x73, 0xc5, 0xda, 0x0a]), (2, [0x73, 0xc5, 0xda, 0x0a])],
        force_chunked: true, path: None },
    Vector { name: "keyed_compose_wsh_hash_and_time",
        template: "wsh(or_i(pkh(@0/48'/0'/0'/2'/<0;1>/*),and_v(v:multi(2,@1/48'/0'/1'/2'/<0;1>/*,@2/48'/0'/2'/2'/<0;1>/*),and_v(v:sha256(a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8),after(1893456000)))))",
        keys: &[(0, XPUB_JOURNEY_0), (1, XPUB_JOURNEY_1), (2, XPUB_JOURNEY_2)],
        fingerprints: &[(0, [0x73, 0xc5, 0xda, 0x0a]), (1, [0x73, 0xc5, 0xda, 0x0a]), (2, [0x73, 0xc5, 0xda, 0x0a])],
        force_chunked: true, path: None },
    Vector { name: "keyed_compose_wsh_three_paths",
        template: "wsh(or_i(pkh(@0/48'/0'/0'/2'/<0;1>/*),or_i(and_v(v:pkh(@1/48'/0'/1'/2'/<0;1>/*),older(4032)),and_v(v:pkh(@2/48'/0'/2'/2'/<0;1>/*),after(1000000)))))",
        keys: &[(0, XPUB_JOURNEY_0), (1, XPUB_JOURNEY_1), (2, XPUB_JOURNEY_2)],
        fingerprints: &[(0, [0x73, 0xc5, 0xda, 0x0a]), (1, [0x73, 0xc5, 0xda, 0x0a]), (2, [0x73, 0xc5, 0xda, 0x0a])],
        force_chunked: true, path: None },
    Vector { name: "keyed_compose_wsh_unsorted_sole",
        template: "wsh(multi(2,@0/48'/0'/0'/2'/<0;1>/*,@1/48'/0'/1'/2'/<0;1>/*,@2/48'/0'/2'/2'/<0;1>/*))",
        keys: &[(0, XPUB_JOURNEY_0), (1, XPUB_JOURNEY_1), (2, XPUB_JOURNEY_2)],
        fingerprints: &[(0, [0x73, 0xc5, 0xda, 0x0a]), (1, [0x73, 0xc5, 0xda, 0x0a]), (2, [0x73, 0xc5, 0xda, 0x0a])],
        force_chunked: true, path: None },
    Vector { name: "keyed_compose_sh_wsh_sole",
        template: "sh(wsh(sortedmulti(2,@0/48'/0'/0'/1'/<0;1>/*,@1/48'/0'/1'/1'/<0;1>/*,@2/48'/0'/2'/1'/<0;1>/*)))",
        keys: &[(0, XPUB_JOURNEY_0), (1, XPUB_JOURNEY_1), (2, XPUB_JOURNEY_2)],
        fingerprints: &[(0, [0x73, 0xc5, 0xda, 0x0a]), (1, [0x73, 0xc5, 0xda, 0x0a]), (2, [0x73, 0xc5, 0xda, 0x0a])],
        force_chunked: true, path: None },
    Vector { name: "keyed_compose_sh_wsh_one_of_two",
        template: "sh(wsh(sortedmulti(1,@0/48'/0'/0'/1'/<0;1>/*,@1/48'/0'/1'/1'/<0;1>/*)))",
        keys: &[(0, XPUB_JOURNEY_0), (1, XPUB_JOURNEY_1)],
        fingerprints: &[(0, [0x73, 0xc5, 0xda, 0x0a]), (1, [0x73, 0xc5, 0xda, 0x0a])],
        force_chunked: true, path: None },
    Vector { name: "keyed_compose_sh_sole",
        template: "sh(sortedmulti(2,@0/48'/0'/0'/2'/<0;1>/*,@1/48'/0'/1'/2'/<0;1>/*))",
        keys: &[(0, XPUB_JOURNEY_0), (1, XPUB_JOURNEY_1)],
        fingerprints: &[(0, [0x73, 0xc5, 0xda, 0x0a]), (1, [0x73, 0xc5, 0xda, 0x0a])],
        force_chunked: true, path: None },
    Vector { name: "keyed_compose_sh_two_of_four",
        template: "sh(sortedmulti(2,@0/48'/0'/0'/2'/<0;1>/*,@1/48'/0'/1'/2'/<0;1>/*,@2/48'/0'/2'/2'/<0;1>/*,@3/48'/0'/3'/2'/<0;1>/*))",
        keys: &[(0, XPUB_JOURNEY_0), (1, XPUB_JOURNEY_1), (2, XPUB_JOURNEY_2), (3, XPUB_JOURNEY_3)],
        fingerprints: &[(0, [0x73, 0xc5, 0xda, 0x0a]), (1, [0x73, 0xc5, 0xda, 0x0a]), (2, [0x73, 0xc5, 0xda, 0x0a]), (3, [0x73, 0xc5, 0xda, 0x0a])],
        force_chunked: true, path: None },
    Vector { name: "keyed_compose_tr_two_path_nums",
        template: "tr(50929b74c1a04954b78b4b6035e97a5e078a5a0f28ec96d547bfee9ace803ac0,{multi_a(2,@0/48'/0'/0'/3'/<0;1>/*,@1/48'/0'/1'/3'/<0;1>/*,@2/48'/0'/2'/3'/<0;1>/*),and_v(v:pk(@3/48'/0'/3'/3'/<0;1>/*),older(26280))})",
        keys: &[(0, XPUB_JOURNEY_0), (1, XPUB_JOURNEY_1), (2, XPUB_JOURNEY_2), (3, XPUB_JOURNEY_3)],
        fingerprints: &[(0, [0x73, 0xc5, 0xda, 0x0a]), (1, [0x73, 0xc5, 0xda, 0x0a]), (2, [0x73, 0xc5, 0xda, 0x0a]), (3, [0x73, 0xc5, 0xda, 0x0a])],
        force_chunked: true, path: None },
    Vector { name: "keyed_compose_tr_two_path_distinct_fingerprints",
        template: "tr(50929b74c1a04954b78b4b6035e97a5e078a5a0f28ec96d547bfee9ace803ac0,{multi_a(2,@0/48'/0'/0'/3'/<0;1>/*,@1/48'/0'/1'/3'/<0;1>/*,@2/48'/0'/2'/3'/<0;1>/*),and_v(v:pk(@3/48'/0'/3'/3'/<0;1>/*),older(26280))})",
        keys: &[(0, XPUB_JOURNEY_0), (1, XPUB_JOURNEY_1), (2, XPUB_JOURNEY_2), (3, XPUB_JOURNEY_3)],
        fingerprints: &[(0, [0x11; 4]), (1, [0x22; 4]), (2, [0x33; 4]), (3, [0x44; 4])],
        force_chunked: true, path: None },
    Vector { name: "keyed_compose_tr_extracted_first",
        template: "tr(@0/48'/0'/0'/3'/<0;1>/*,and_v(v:pk(@1/48'/0'/1'/3'/<0;1>/*),older(65535)))",
        keys: &[(0, XPUB_JOURNEY_0), (1, XPUB_JOURNEY_1)],
        fingerprints: &[(0, [0x73, 0xc5, 0xda, 0x0a]), (1, [0x73, 0xc5, 0xda, 0x0a])],
        force_chunked: true, path: None },
    Vector { name: "keyed_compose_tr_extracted_later_four_paths",
        template: "tr(@0/48'/0'/0'/3'/<0;1>/*,{and_v(v:pk(@1/48'/0'/1'/3'/<0;1>/*),older(10)),{and_v(v:pk(@2/48'/0'/2'/3'/<0;1>/*),after(1000000)),and_v(v:pk(@3/48'/0'/3'/3'/<0;1>/*),older(4194404))}})",
        keys: &[(0, XPUB_JOURNEY_0), (1, XPUB_JOURNEY_1), (2, XPUB_JOURNEY_2), (3, XPUB_JOURNEY_3)],
        fingerprints: &[(0, [0x73, 0xc5, 0xda, 0x0a]), (1, [0x73, 0xc5, 0xda, 0x0a]), (2, [0x73, 0xc5, 0xda, 0x0a]), (3, [0x73, 0xc5, 0xda, 0x0a])],
        force_chunked: true, path: None },
    Vector { name: "keyed_compose_tr_three_paths_extracted_later",
        template: "tr(@0/48'/0'/0'/3'/<0;1>/*,{and_v(v:pk(@1/48'/0'/1'/3'/<0;1>/*),older(10)),and_v(v:pk(@2/48'/0'/2'/3'/<0;1>/*),older(4194309))})",
        keys: &[(0, XPUB_JOURNEY_0), (1, XPUB_JOURNEY_1), (2, XPUB_JOURNEY_2)],
        fingerprints: &[(0, [0x73, 0xc5, 0xda, 0x0a]), (1, [0x73, 0xc5, 0xda, 0x0a]), (2, [0x73, 0xc5, 0xda, 0x0a])],
        force_chunked: true, path: None },
    Vector { name: "keyed_compose_tr_nums_three_leaves",
        template: "tr(50929b74c1a04954b78b4b6035e97a5e078a5a0f28ec96d547bfee9ace803ac0,{and_v(v:pk(@0/48'/0'/0'/3'/<0;1>/*),older(1)),{and_v(v:pk(@1/48'/0'/1'/3'/<0;1>/*),older(2)),and_v(v:multi_a(2,@2/48'/0'/2'/3'/<0;1>/*,@3/48'/0'/3'/3'/<0;1>/*),after(2))}})",
        keys: &[(0, XPUB_JOURNEY_0), (1, XPUB_JOURNEY_1), (2, XPUB_JOURNEY_2), (3, XPUB_JOURNEY_3)],
        fingerprints: &[(0, [0x73, 0xc5, 0xda, 0x0a]), (1, [0x73, 0xc5, 0xda, 0x0a]), (2, [0x73, 0xc5, 0xda, 0x0a]), (3, [0x73, 0xc5, 0xda, 0x0a])],
        force_chunked: true, path: None },
    Vector { name: "keyed_compose_tr_sole_sortedmulti_a",
        template: "tr(50929b74c1a04954b78b4b6035e97a5e078a5a0f28ec96d547bfee9ace803ac0,sortedmulti_a(2,@0/48'/0'/0'/3'/<0;1>/*,@1/48'/0'/1'/3'/<0;1>/*,@2/48'/0'/2'/3'/<0;1>/*))",
        keys: &[(0, XPUB_JOURNEY_0), (1, XPUB_JOURNEY_1), (2, XPUB_JOURNEY_2)],
        fingerprints: &[(0, [0x73, 0xc5, 0xda, 0x0a]), (1, [0x73, 0xc5, 0xda, 0x0a]), (2, [0x73, 0xc5, 0xda, 0x0a])],
        force_chunked: true, path: None },
    Vector { name: "keyed_compose_tr_key_path_only",
        template: "tr(@0/48'/0'/0'/3'/<0;1>/*)",
        keys: &[(0, XPUB_JOURNEY_0)],
        fingerprints: &[(0, [0x73, 0xc5, 0xda, 0x0a])],
        force_chunked: true, path: None },
    Vector { name: "keyed_compose_tr_unsorted_sole_leaf",
        template: "tr(50929b74c1a04954b78b4b6035e97a5e078a5a0f28ec96d547bfee9ace803ac0,multi_a(2,@0/48'/0'/0'/3'/<0;1>/*,@1/48'/0'/1'/3'/<0;1>/*))",
        keys: &[(0, XPUB_JOURNEY_0), (1, XPUB_JOURNEY_1)],
        fingerprints: &[(0, [0x73, 0xc5, 0xda, 0x0a]), (1, [0x73, 0xc5, 0xda, 0x0a])],
        force_chunked: true, path: None },
    Vector { name: "keyed_compose_tr_hash_leaf",
        template: "tr(50929b74c1a04954b78b4b6035e97a5e078a5a0f28ec96d547bfee9ace803ac0,{multi_a(2,@0/48'/0'/0'/3'/<0;1>/*,@1/48'/0'/1'/3'/<0;1>/*),and_v(v:pk(@2/48'/0'/2'/3'/<0;1>/*),and_v(v:sha256(a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8),after(1893456000)))})",
        keys: &[(0, XPUB_JOURNEY_0), (1, XPUB_JOURNEY_1), (2, XPUB_JOURNEY_2)],
        fingerprints: &[(0, [0x73, 0xc5, 0xda, 0x0a]), (1, [0x73, 0xc5, 0xda, 0x0a]), (2, [0x73, 0xc5, 0xda, 0x0a])],
        force_chunked: true, path: None },
    Vector { name: "compose_wsh_eight_paths",
        template: "wsh(or_i(and_v(v:pkh(@0/48'/0'/0'/2'/<0;1>/*),older(100)),or_i(and_v(v:pkh(@1/48'/0'/1'/2'/<0;1>/*),older(101)),or_i(and_v(v:pkh(@2/48'/0'/2'/2'/<0;1>/*),older(102)),or_i(and_v(v:pkh(@3/48'/0'/3'/2'/<0;1>/*),older(103)),or_i(and_v(v:pkh(@4/48'/0'/4'/2'/<0;1>/*),older(104)),or_i(and_v(v:pkh(@5/48'/0'/5'/2'/<0;1>/*),older(105)),or_i(and_v(v:pkh(@6/48'/0'/6'/2'/<0;1>/*),older(106)),and_v(v:pkh(@7/48'/0'/7'/2'/<0;1>/*),older(107))))))))))",
        keys: &[],
        fingerprints: &[],
        force_chunked: true, path: None },
    Vector { name: "compose_tr_seven_leaves",
        template: "tr(@0/48'/0'/0'/3'/<0;1>/*,{and_v(v:pk(@1/48'/0'/1'/3'/<0;1>/*),older(101)),{and_v(v:pk(@2/48'/0'/2'/3'/<0;1>/*),older(102)),{and_v(v:pk(@3/48'/0'/3'/3'/<0;1>/*),older(103)),{and_v(v:pk(@4/48'/0'/4'/3'/<0;1>/*),older(104)),{and_v(v:pk(@5/48'/0'/5'/3'/<0;1>/*),older(105)),{and_v(v:pk(@6/48'/0'/6'/3'/<0;1>/*),older(106)),and_v(v:pk(@7/48'/0'/7'/3'/<0;1>/*),older(107))}}}}}})",
        keys: &[],
        fingerprints: &[],
        force_chunked: true, path: None },
    Vector { name: "compose_wsh_thirty_two_slots",
        template: "wsh(or_d(multi(9,@0/48'/0'/0'/2'/<0;1>/*,@1/48'/0'/1'/2'/<0;1>/*,@2/48'/0'/2'/2'/<0;1>/*,@3/48'/0'/3'/2'/<0;1>/*,@4/48'/0'/4'/2'/<0;1>/*,@5/48'/0'/5'/2'/<0;1>/*,@6/48'/0'/6'/2'/<0;1>/*,@7/48'/0'/7'/2'/<0;1>/*,@8/48'/0'/8'/2'/<0;1>/*),or_d(multi(9,@9/48'/0'/9'/2'/<0;1>/*,@10/48'/0'/10'/2'/<0;1>/*,@11/48'/0'/11'/2'/<0;1>/*,@12/48'/0'/12'/2'/<0;1>/*,@13/48'/0'/13'/2'/<0;1>/*,@14/48'/0'/14'/2'/<0;1>/*,@15/48'/0'/15'/2'/<0;1>/*,@16/48'/0'/16'/2'/<0;1>/*,@17/48'/0'/17'/2'/<0;1>/*),or_d(multi(9,@18/48'/0'/18'/2'/<0;1>/*,@19/48'/0'/19'/2'/<0;1>/*,@20/48'/0'/20'/2'/<0;1>/*,@21/48'/0'/21'/2'/<0;1>/*,@22/48'/0'/22'/2'/<0;1>/*,@23/48'/0'/23'/2'/<0;1>/*,@24/48'/0'/24'/2'/<0;1>/*,@25/48'/0'/25'/2'/<0;1>/*,@26/48'/0'/26'/2'/<0;1>/*),multi(5,@27/48'/0'/27'/2'/<0;1>/*,@28/48'/0'/28'/2'/<0;1>/*,@29/48'/0'/29'/2'/<0;1>/*,@30/48'/0'/30'/2'/<0;1>/*,@31/48'/0'/31'/2'/<0;1>/*)))))",
        keys: &[],
        fingerprints: &[],
        force_chunked: true, path: None },
    Vector { name: "compose_tr_thirty_two_slots",
        template: "tr(50929b74c1a04954b78b4b6035e97a5e078a5a0f28ec96d547bfee9ace803ac0,{multi_a(4,@0/48'/0'/0'/3'/<0;1>/*,@1/48'/0'/1'/3'/<0;1>/*,@2/48'/0'/2'/3'/<0;1>/*,@3/48'/0'/3'/3'/<0;1>/*),{multi_a(4,@4/48'/0'/4'/3'/<0;1>/*,@5/48'/0'/5'/3'/<0;1>/*,@6/48'/0'/6'/3'/<0;1>/*,@7/48'/0'/7'/3'/<0;1>/*),{multi_a(4,@8/48'/0'/8'/3'/<0;1>/*,@9/48'/0'/9'/3'/<0;1>/*,@10/48'/0'/10'/3'/<0;1>/*,@11/48'/0'/11'/3'/<0;1>/*),{multi_a(4,@12/48'/0'/12'/3'/<0;1>/*,@13/48'/0'/13'/3'/<0;1>/*,@14/48'/0'/14'/3'/<0;1>/*,@15/48'/0'/15'/3'/<0;1>/*),{multi_a(4,@16/48'/0'/16'/3'/<0;1>/*,@17/48'/0'/17'/3'/<0;1>/*,@18/48'/0'/18'/3'/<0;1>/*,@19/48'/0'/19'/3'/<0;1>/*),{multi_a(4,@20/48'/0'/20'/3'/<0;1>/*,@21/48'/0'/21'/3'/<0;1>/*,@22/48'/0'/22'/3'/<0;1>/*,@23/48'/0'/23'/3'/<0;1>/*),{multi_a(4,@24/48'/0'/24'/3'/<0;1>/*,@25/48'/0'/25'/3'/<0;1>/*,@26/48'/0'/26'/3'/<0;1>/*,@27/48'/0'/27'/3'/<0;1>/*),multi_a(4,@28/48'/0'/28'/3'/<0;1>/*,@29/48'/0'/29'/3'/<0;1>/*,@30/48'/0'/30'/3'/<0;1>/*,@31/48'/0'/31'/3'/<0;1>/*)}}}}}}})",
        keys: &[],
        fingerprints: &[],
        force_chunked: true, path: None },
    Vector { name: "keyed_compose_preset_plain_multisig",
        template: "wsh(sortedmulti(2,@0/48'/0'/0'/2'/<0;1>/*,@1/48'/0'/1'/2'/<0;1>/*,@2/48'/0'/2'/2'/<0;1>/*))",
        keys: &[(0, XPUB_JOURNEY_0), (1, XPUB_JOURNEY_1), (2, XPUB_JOURNEY_2)],
        fingerprints: &[(0, [0x73, 0xc5, 0xda, 0x0a]), (1, [0x73, 0xc5, 0xda, 0x0a]), (2, [0x73, 0xc5, 0xda, 0x0a])],
        force_chunked: true, path: None },
    Vector { name: "keyed_compose_preset_simple_timelocked_inheritance",
        template: "wsh(or_i(pkh(@0/48'/0'/0'/2'/<0;1>/*),and_v(v:pkh(@1/48'/0'/1'/2'/<0;1>/*),older(26280))))",
        keys: &[(0, XPUB_JOURNEY_0), (1, XPUB_JOURNEY_1)],
        fingerprints: &[(0, [0x73, 0xc5, 0xda, 0x0a]), (1, [0x73, 0xc5, 0xda, 0x0a])],
        force_chunked: true, path: None },
    Vector { name: "keyed_compose_preset_kofn_recovery",
        template: "tr(50929b74c1a04954b78b4b6035e97a5e078a5a0f28ec96d547bfee9ace803ac0,{multi_a(2,@0/48'/0'/0'/3'/<0;1>/*,@1/48'/0'/1'/3'/<0;1>/*,@2/48'/0'/2'/3'/<0;1>/*),and_v(v:pk(@3/48'/0'/3'/3'/<0;1>/*),older(26280))})",
        keys: &[(0, XPUB_JOURNEY_0), (1, XPUB_JOURNEY_1), (2, XPUB_JOURNEY_2), (3, XPUB_JOURNEY_3)],
        fingerprints: &[(0, [0x73, 0xc5, 0xda, 0x0a]), (1, [0x73, 0xc5, 0xda, 0x0a]), (2, [0x73, 0xc5, 0xda, 0x0a]), (3, [0x73, 0xc5, 0xda, 0x0a])],
        force_chunked: true, path: None },
    Vector { name: "keyed_compose_preset_tiered_recovery",
        template: "wsh(or_d(multi(2,@0/48'/0'/0'/2'/<0;1>/*,@1/48'/0'/1'/2'/<0;1>/*),and_v(v:multi(1,@2/48'/0'/2'/2'/<0;1>/*,@3/48'/0'/3'/2'/<0;1>/*),older(26280))))",
        keys: &[(0, XPUB_JOURNEY_0), (1, XPUB_JOURNEY_1), (2, XPUB_JOURNEY_2), (3, XPUB_JOURNEY_3)],
        fingerprints: &[(0, [0x73, 0xc5, 0xda, 0x0a]), (1, [0x73, 0xc5, 0xda, 0x0a]), (2, [0x73, 0xc5, 0xda, 0x0a]), (3, [0x73, 0xc5, 0xda, 0x0a])],
        force_chunked: true, path: None },
    Vector { name: "keyed_compose_preset_hashlock_gated",
        template: "wsh(or_i(and_v(v:pkh(@0/48'/0'/0'/2'/<0;1>/*),sha256(a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8)),and_v(v:pkh(@1/48'/0'/1'/2'/<0;1>/*),older(26280))))",
        keys: &[(0, XPUB_JOURNEY_0), (1, XPUB_JOURNEY_1)],
        fingerprints: &[(0, [0x73, 0xc5, 0xda, 0x0a]), (1, [0x73, 0xc5, 0xda, 0x0a])],
        force_chunked: true, path: None },
    Vector { name: "keyed_compose_preset_hashlock_gated_hash256",
        template: "wsh(or_i(and_v(v:pkh(@0/48'/0'/0'/2'/<0;1>/*),hash256(98a20fc25dbcdf236fb0307e3f82cad47fca2e807f3ef82c31993549641cd488)),and_v(v:pkh(@1/48'/0'/1'/2'/<0;1>/*),older(26280))))",
        keys: &[(0, XPUB_JOURNEY_0), (1, XPUB_JOURNEY_1)],
        fingerprints: &[(0, [0x73, 0xc5, 0xda, 0x0a]), (1, [0x73, 0xc5, 0xda, 0x0a])],
        force_chunked: true, path: None },
    Vector { name: "keyed_compose_preset_hashlock_gated_ripemd160",
        template: "wsh(or_i(and_v(v:pkh(@0/48'/0'/0'/2'/<0;1>/*),ripemd160(09e7bb5051d89788fb4e4b374126721dbcc2946b)),and_v(v:pkh(@1/48'/0'/1'/2'/<0;1>/*),older(26280))))",
        keys: &[(0, XPUB_JOURNEY_0), (1, XPUB_JOURNEY_1)],
        fingerprints: &[(0, [0x73, 0xc5, 0xda, 0x0a]), (1, [0x73, 0xc5, 0xda, 0x0a])],
        force_chunked: true, path: None },
    Vector { name: "keyed_compose_preset_hashlock_gated_hash160",
        template: "wsh(or_i(and_v(v:pkh(@0/48'/0'/0'/2'/<0;1>/*),hash160(b5b72c0e6896ff59dfa99e0d1052a9c0214cd0bd)),and_v(v:pkh(@1/48'/0'/1'/2'/<0;1>/*),older(26280))))",
        keys: &[(0, XPUB_JOURNEY_0), (1, XPUB_JOURNEY_1)],
        fingerprints: &[(0, [0x73, 0xc5, 0xda, 0x0a]), (1, [0x73, 0xc5, 0xda, 0x0a])],
        force_chunked: true, path: None },
    Vector { name: "keyed_compose_preset_decaying_multisig",
        template: "wsh(or_i(and_v(v:multi(2,@0/48'/0'/0'/2'/<0;1>/*,@1/48'/0'/1'/2'/<0;1>/*),older(13140)),or_i(and_v(v:pkh(@2/48'/0'/2'/2'/<0;1>/*),older(26280)),and_v(v:pkh(@3/48'/0'/3'/2'/<0;1>/*),after(1000000)))))",
        keys: &[(0, XPUB_JOURNEY_0), (1, XPUB_JOURNEY_1), (2, XPUB_JOURNEY_2), (3, XPUB_JOURNEY_3)],
        fingerprints: &[(0, [0x73, 0xc5, 0xda, 0x0a]), (1, [0x73, 0xc5, 0xda, 0x0a]), (2, [0x73, 0xc5, 0xda, 0x0a]), (3, [0x73, 0xc5, 0xda, 0x0a])],
        force_chunked: true, path: None },
];
