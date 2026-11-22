//! The coordinators, their verified versions, and their rules.
//!
//! **Adding a version** is a string in `verified` plus, if the rules change
//! there, a new [`RuleSet`]. **Adding a coordinator** is a new entry. Neither
//! touches [`super::verdicts`] (design §1).
//!
//! Every clause is a REFUSAL. A clause may under-approximate — miss a
//! refusal, which leaves the verdict silent or to evidence — but must never
//! refuse what the coordinator imports: that is a D1 build failure against
//! the evidence (`super::build`).

use super::{
    Clause, Coordinator, CoordinatorId, Description, Form, FormRefusal, ReadSet, Reason, RuleSet,
    Span, Version, any_lock, is_nums, is_tr, keyless_gated, unlocked,
};
use crate::policy_shape::{KeyPathKind, LockKind, RootKind};
use crate::skeleton::Skeleton;

const S: ReadSet = ReadSet::STRUCTURE;
const K: ReadSet = ReadSet::KEY_IDENTITY;
const SV: ReadSet = ReadSet::STRUCTURE.with(ReadSet::LOCK_VALUES);

const fn span(since: &'static str, until: &'static str) -> Span {
    Span {
        since: Version(since),
        until: Version(until),
    }
}

/// Every coordinator the registry knows, in the order notices print them.
pub static REGISTRY: &[Coordinator] = &[LIANA, NUNCHUK, CORE];

// ---------------------------------------------------------------------------
// Liana. Classes in Liana's own order of refusal: miniscript PARSE errors
// first (they fire before Liana sees a policy), then key checks, then the
// policy-model classes of the fork's `composerLianaOutsideModelClass`
// (fork `2c9eed3`, `gui/composer_consent.go:396`), which is ported here and
// becomes the convergence port.
// ---------------------------------------------------------------------------

const LIANA_CLAUSES: &[Clause] = &[
    Clause {
        reason: Reason {
            class: "a sortedmulti_a leaf",
            cite: "Liana v8.0 and v15.0 parse: \"unexpected «sortedmulti_a(4 args) while parsing Miniscript»\" (evidence plain-2of3-tr)",
        },
        reads: S,
        fires: |s| is_tr(s) && s.shape.branches.iter().any(|b| b.k > 0 && b.sorted),
    },
    Clause {
        reason: Reason {
            class: "a path with no key",
            cite: "Liana v15.0 parse: \"All spend paths must require a signature\" (evidence keyless-hash-path-wsh; F-633)",
        },
        reads: S,
        fires: |s| s.shape.branches.iter().any(keyless_gated),
    },
    Clause {
        reason: Reason {
            class: "one signer twice in a path",
            cite: "Liana: \"is derived from the same origin as another key present in the same spending path\" (evidence X11-wsh-samefp-in-path)",
        },
        reads: K,
        fires: |s| s.fp_partition.iter().flatten().any(|g| g.len() >= 2),
    },
    Clause {
        reason: Reason {
            class: "a key used twice",
            cite: "Liana DuplicateKey (design §1A key_partition)",
        },
        reads: K,
        fires: |s| s.key_partition.iter().any(|g| g.len() >= 2),
    },
    Clause {
        reason: Reason {
            class: "legacy wrapper",
            cite: "liana analysis.rs:586-587 (wsh or tr only)",
        },
        reads: S,
        fires: |s| !matches!(s.root, RootKind::Wsh | RootKind::Tr),
    },
    Clause {
        reason: Reason {
            class: "NUMS key path",
            cite: "liana analysis.rs:568-569",
        },
        reads: S,
        fires: is_nums,
    },
    Clause {
        reason: Reason {
            class: "no locked path",
            cite: "liana analysis.rs:554-558, :472-474, :583",
        },
        reads: S,
        fires: |s| s.shape.branches.iter().all(unlocked),
    },
    Clause {
        reason: Reason {
            class: "a hash lock",
            cite: "liana analysis.rs:186-199, :212-257",
        },
        reads: S,
        fires: |s| s.shape.branches.iter().any(|b| !b.hashlocks.is_empty()),
    },
    Clause {
        reason: Reason {
            class: "a relative lock Liana cannot use",
            cite: "liana csv_check :139-145 (a u16 block count only); message \"Timelock value … isn't valid or safe to use\" (older-units-wsh; x24-older70000, plan 1b r0 M-1)",
        },
        // BEFORE "an absolute lock", unlike the Go (its class 6 after class
        // 5): Liana 8.0 and 15.0 refuse mixed-lock-bases-{wsh,tr}, which carry
        // both, with this class's message. The Go order is a D3 the table
        // build reports on three evidence rows.
        //
        // Wider than the Go's "older in time units": `csv_check` takes only a
        // u16 BLOCK count, so an `older` in blocks above 65535 is refused too
        // (measured, `older(70000)`), and it reads the lock VALUE (r0 M-1, M-4).
        reads: SV,
        fires: |s| {
            s.shape.branches.iter().any(|b| {
                b.locks.iter().any(|l| {
                    l.kind == LockKind::OlderUnits
                        || (l.kind == LockKind::OlderBlocks && l.value > 0xFFFF)
                })
            })
        },
    },
    Clause {
        reason: Reason {
            class: "an absolute lock",
            cite: "liana analysis.rs:212-257",
        },
        reads: S,
        fires: |s| any_lock(s, &[LockKind::AfterHeight, LockKind::AfterTime]),
    },
    Clause {
        reason: Reason {
            class: "no unlocked path",
            cite: "liana analysis.rs:633",
        },
        // NOT the Go's `if KeyPath == Spendable { unlocked++ }`: the Rust
        // walk already pushes a real internal key as its own unlocked branch
        // 0, so adding it again double-counts (recon §1d). A Liana key and a
        // NUMS key push no branch, and count as nothing.
        reads: S,
        fires: |s| !s.shape.branches.iter().any(unlocked),
    },
    Clause {
        reason: Reason {
            class: "two paths with one lock",
            cite: "liana analysis.rs:624-626",
        },
        reads: SV,
        fires: |s| {
            let mut seen: Vec<u32> = Vec::new();
            for b in &s.shape.branches {
                for l in &b.locks {
                    if l.kind == LockKind::OlderBlocks {
                        if seen.contains(&l.value) {
                            return true;
                        }
                        seen.push(l.value);
                    }
                }
            }
            false
        },
    },
    Clause {
        reason: Reason {
            class: "a second unlocked k-of-n path with k >= 2",
            cite: "liana analysis.rs:611-616 after rust-miniscript semantic.rs:359-426 normalized(); evidence X24 and cx-multi-then-1of2multi (folded, imported) vs X25/X26 and cx-single-then-multi (refused)",
        },
        // NARROWER than the Go's class 9, which names every second unlocked
        // path. Liana lifts the policy and `normalized()` flattens every
        // 1-of-n into bare keys, which it then FOLDS into the primary path
        // and imports (X24's single key; plan 1b r0 I-1's `multi(1,C,D)`,
        // read as 2-of-4) -- refusing those would be a D1. What it refuses is
        // a second unlocked path that stays a threshold: `k >= 2`. A second
        // path that is an `and` of keys (k = 0, two slots) is refused by
        // Liana too and missed here, which errs toward silence.
        reads: S,
        fires: |s| {
            s.shape
                .branches
                .iter()
                .filter(|b| unlocked(b))
                .skip(1)
                .any(|b| b.k >= 2)
        },
    },
];

/// Liana's primary path is the ONE unlocked path, read as `k`-of-`n`. It
/// folds a second single-key or 1-of-n unlocked path into it (X24: built as
/// 2-of-3 plus 1-of-1, read as 2-of-4), which is exactly an import "not as
/// built".
fn liana_reads_as_built(s: &Skeleton, d: &Description) -> bool {
    let mut primaries = s.shape.branches.iter().filter(|b| unlocked(b));
    let (Some(p), None) = (primaries.next(), primaries.next()) else {
        return false;
    };
    let built = match (p.k, p.slots.len()) {
        (0, 1) => Some((1, 1)),
        (0, _) => None,
        (k, n) => u8::try_from(n).ok().map(|n| (k, n)),
    };
    built.is_some() && built == d.threshold
}

fn liana_class_of_message(m: &str) -> Option<&'static str> {
    if m.contains("sortedmulti_a") {
        Some("a sortedmulti_a leaf")
    } else if m.contains("All spend paths must require a signature") {
        Some("a path with no key")
    } else if m.contains(
        "is derived from the same origin as another key present in the same spending path",
    ) {
        Some("one signer twice in a path")
    } else if m.starts_with("Timelock value") {
        Some("a relative lock Liana cannot use")
    } else if m.starts_with("A Liana policy requires at least one recovery path") {
        Some("no locked path")
    } else {
        None
    }
}

const LIANA: Coordinator = Coordinator {
    id: CoordinatorId("liana"),
    name: "Liana",
    verified: &["8.0", "15.0"],
    rules: &[RuleSet {
        span: span("8.0", "15.0"),
        source_verified_at: &["8.0"],
        clauses: LIANA_CLAUSES,
        form_refusals: &[],
        measured_refusals_admitted: false,
    }],
    reads_as_built: liana_reads_as_built,
    class_of_message: liana_class_of_message,
};

// ---------------------------------------------------------------------------
// Nunchuk. Two source-derived rules -- a keyless `wsh` path, and ruling
// R-1's key-order rule for Liana's key; every other refusal is measured, because Nunchuk's acceptance is a byte round trip of the
// descriptor text (libnunchuk `a7cfb49` `src/descriptor.cpp:633-648`), so a
// refusal is a claim about a spelling, not a policy (design §1).
// ---------------------------------------------------------------------------

const NUNCHUK_CLAUSES: &[Clause] = &[
    Clause {
        reason: Reason {
            class: "a path with no key",
            cite: "libnunchuk a7cfb49 src/descriptor.cpp:573 (ParseWshDescriptor -> IsValidMiniscriptTemplate), src/nunchukutils.cpp:1276 (IsSane), contrib/bitcoin 57b47c4 src/script/miniscript.h:1617 (IsSane includes NeedsSignature)",
        },
        // `wsh` only: that is the path whose source was read. A `tr` policy's
        // leaves go through `ParseTrDescriptor`, not read for this rule.
        reads: S,
        fires: |s| s.root == RootKind::Wsh && s.shape.branches.iter().any(keyless_gated),
    },
    Clause {
        reason: Reason {
            class: "Liana's key is not the one Nunchuk derives (leaf keys not in sorted order)",
            cite: "libnunchuk a7cfb49 src/descriptor.cpp:689-712 (GetUnspendableXpub sorts and dedups, :700-701) and :646 (re-render must equal the input); ruling R-1",
        },
        // Reads key MATERIAL (`leaf_keys_ascending`), so a template is
        // `Unproven { KeysAbsent }` for Nunchuk (R-1).
        reads: K,
        fires: |s| {
            s.shape.key_path == KeyPathKind::LianaUnspendable
                && s.leaf_keys_ascending == Some(false)
        },
    },
];

/// F-626: Nunchuk shows an unsorted single-path `multi` as `MINISCRIPT
/// 0-of-3` rather than a `k`-of-`n` multisig. The comparison is scoped to
/// that measured case: a single-branch threshold must read as `MULTI_SIG`
/// with the same `k`-of-`n`.
fn nunchuk_reads_as_built(s: &Skeleton, d: &Description) -> bool {
    match s.shape.branches.as_slice() {
        [b] if b.k > 0 => {
            d.wallet_kind == "MULTI_SIG"
                && u8::try_from(b.slots.len()).ok().map(|n| (b.k, n)) == d.threshold
        }
        _ => true,
    }
}

fn no_class(_: &str) -> Option<&'static str> {
    None
}

const NUNCHUK: Coordinator = Coordinator {
    id: CoordinatorId("nunchuk"),
    name: "Nunchuk",
    verified: &["2.1.1"],
    rules: &[RuleSet {
        span: span("2.1.1", "2.1.1"),
        source_verified_at: &["2.1.1"],
        clauses: NUNCHUK_CLAUSES,
        form_refusals: &[],
        measured_refusals_admitted: true,
    }],
    reads_as_built: nunchuk_reads_as_built,
    class_of_message: no_class,
};

// ---------------------------------------------------------------------------
// Bitcoin Core. Boundaries MEASURED on official release binaries
// (`design/agent-reports/coord-compat-core-boundary.md`): tapscript
// miniscript from 26.0, the `<0;1>` multipath spelling from between 28.4
// and 29.4. Source line numbers are `src/script/descriptor.cpp` at the tag.
// ---------------------------------------------------------------------------

const CORE_NO_KEY: Clause = Clause {
    reason: Reason {
        class: "a path with no key",
        cite: "bitcoin src/script/descriptor.cpp NeedsSignature: v24.2:1525, v25.2:1528, v26.0:1800, v27.2:1800, v29.2:2102, v30.0:2483",
    },
    reads: S,
    fires: |s| s.shape.branches.iter().any(keyless_gated),
};

const CORE_MULTIPATH: FormRefusal = FormRefusal {
    form: Form::Multipath,
    reason: Reason {
        class: "the <0;1> multipath spelling (use --chain 0 and --chain 1)",
        cite: "measured 24.2-28.4: \"Key path value '<0;1>' is not a valid uint32\" (coord-compat-core-boundary); bitcoin v26.0 src/script/descriptor.cpp:1290",
    },
};

const CORE_PRE_26: &[Clause] = &[
    Clause {
        reason: Reason {
            class: "miniscript under tr",
            cite: "bitcoin src/script/descriptor.cpp \"Miniscript expressions can only be used in wsh\": v24.2:1507, v25.2:1510; measured 24.2-25.2",
        },
        reads: S,
        // A tap leaf that is anything but `pk(K)` or a bare `multi_a` is
        // miniscript. Under-approximates: a lone `pkh(K)` leaf is missed.
        fires: |s| {
            is_tr(s)
                && s.shape.branches.iter().any(|b| {
                    !b.locks.is_empty()
                        || !b.hashlocks.is_empty()
                        || (b.k == 0 && b.slots.len() >= 2)
                })
        },
    },
    CORE_NO_KEY,
];

fn core_reads_as_built(_: &Skeleton, d: &Description) -> bool {
    d.wallet_kind != "ADDRESSES_DIFFER"
}

fn core_class_of_message(m: &str) -> Option<&'static str> {
    if m.contains("Miniscript expressions can only be used in wsh") || m.contains("multi_a(") {
        Some("miniscript under tr")
    } else if m.contains("is not a valid uint32") {
        Some("the <0;1> multipath spelling (use --chain 0 and --chain 1)")
    } else if m.contains("witnesses without signature exist") {
        Some("a path with no key")
    } else {
        None
    }
}

const CORE: Coordinator = Coordinator {
    id: CoordinatorId("core"),
    name: "Bitcoin Core",
    verified: &[
        "24.2", "25.0", "25.2", "26.0", "26.2", "27.2", "28.4", "29.4", "30.3", "31.1",
    ],
    rules: &[
        RuleSet {
            span: span("24.2", "25.2"),
            source_verified_at: &["24.2", "25.2"],
            clauses: CORE_PRE_26,
            form_refusals: &[CORE_MULTIPATH],
            measured_refusals_admitted: false,
        },
        RuleSet {
            span: span("26.0", "28.4"),
            source_verified_at: &["26.0", "27.2"],
            clauses: &[CORE_NO_KEY],
            form_refusals: &[CORE_MULTIPATH],
            measured_refusals_admitted: false,
        },
        RuleSet {
            span: span("29.4", "31.1"),
            source_verified_at: &["29.2", "30.0"],
            clauses: &[CORE_NO_KEY],
            form_refusals: &[],
            measured_refusals_admitted: false,
        },
    ],
    reads_as_built: core_reads_as_built,
    class_of_message: core_class_of_message,
};
