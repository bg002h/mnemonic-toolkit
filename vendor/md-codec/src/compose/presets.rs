//! The toolkit's five archetypes, plus plain k-of-n multisig, as path lists
//! over THIS grammar (spec §4d, C2). Same spend conditions as
//! `mnemonic build-descriptor`'s goldens, not byte-identical to them: this
//! lowering is one fixed spelling.

use super::{ComposeError, HashLock, KeySet, Lock, PathList, SpendPath, Wrapper, validate};

fn ks(k: u8, n: u8) -> SpendPath {
    SpendPath {
        keys: Some(KeySet { k, n, sorted: true }),
        hash: None,
        lock: None,
    }
}

/// `older` in blocks for the path at `path` (0-based), so a refusal names the
/// tier that carries the bad lock.
fn blocks(b: u32, path: usize) -> Result<Lock, ComposeError> {
    u16::try_from(b)
        .ok()
        .filter(|b| *b >= 1)
        .map(Lock::OlderBlocks)
        .ok_or(ComposeError::LockOutOfRange {
            path,
            why: "older in blocks needs 1..=65535",
        })
}

fn checked(list: PathList) -> Result<PathList, ComposeError> {
    validate(&list)?;
    Ok(list)
}

/// One unlocked k-of-n path: the Multisig Build shape C7 migrates.
pub fn plain_multisig(wrapper: Wrapper, k: u8, n: u8) -> Result<PathList, ComposeError> {
    checked(PathList {
        wrapper,
        paths: vec![ks(k, n)],
    })
}

/// Primary key now; heir after `older_blocks`.
pub fn simple_timelocked_inheritance(
    wrapper: Wrapper,
    older_blocks: u32,
) -> Result<PathList, ComposeError> {
    let mut heir = ks(1, 1);
    heir.lock = Some(blocks(older_blocks, 1)?);
    checked(PathList {
        wrapper,
        paths: vec![ks(1, 1), heir],
    })
}

/// k-of-n now; one recovery key after `older_blocks`.
pub fn kofn_recovery(
    wrapper: Wrapper,
    k: u8,
    n: u8,
    older_blocks: u32,
) -> Result<PathList, ComposeError> {
    let mut recovery = ks(1, 1);
    recovery.lock = Some(blocks(older_blocks, 1)?);
    checked(PathList {
        wrapper,
        paths: vec![ks(k, n), recovery],
    })
}

/// k1-of-n1 now; k2-of-n2 (distinct keys) after `older_blocks`.
pub fn tiered_recovery(
    wrapper: Wrapper,
    k1: u8,
    n1: u8,
    k2: u8,
    n2: u8,
    older_blocks: u32,
) -> Result<PathList, ComposeError> {
    let mut tier2 = ks(k2, n2);
    tier2.lock = Some(blocks(older_blocks, 1)?);
    checked(PathList {
        wrapper,
        paths: vec![ks(k1, n1), tier2],
    })
}

/// A key plus a hash now; a second key after `older_blocks`.
///
/// **`hash` is a `HashLock`, not a bare `[u8; 32]`** (SPEC_hashlock_kinds §9.1).
/// The bare array could only ever mean `sha256`, so the preset could not build
/// the other three fragments at all.
pub fn hashlock_gated(
    wrapper: Wrapper,
    hash: HashLock,
    older_blocks: u32,
) -> Result<PathList, ComposeError> {
    let mut gated = ks(1, 1);
    gated.hash = Some(hash);
    let mut later = ks(1, 1);
    later.lock = Some(blocks(older_blocks, 1)?);
    checked(PathList {
        wrapper,
        paths: vec![gated, later],
    })
}

/// k1-of-n1 after `older1`; a recovery quorum k2-of-n2 (distinct keys) that is
/// NO HARDER to satisfy than the primary (`k2 <= k1`; `n2` is free, since more
/// keys at the same threshold only widen the ways to spend) after
/// `older2 > older1`; one final key after `after_height`. The toolkit's
/// archetype takes the primary and recovery quorums as separate parameters and
/// refuses tiers that do not unlock progressively later; so does this. What
/// "decay" means here is exactly those two guards, nothing more.
#[allow(clippy::too_many_arguments)]
pub fn decaying_multisig(
    wrapper: Wrapper,
    k1: u8,
    n1: u8,
    k2: u8,
    n2: u8,
    older1: u32,
    older2: u32,
    after_height: u32,
) -> Result<PathList, ComposeError> {
    if older2 <= older1 {
        return Err(ComposeError::PresetShape {
            why: "decaying tiers must unlock progressively later (the second older must exceed the first)",
        });
    }
    if k2 > k1 {
        return Err(ComposeError::PresetShape {
            why: "a decaying multisig decays: the recovery threshold cannot exceed the primary threshold",
        });
    }
    let mut t1 = ks(k1, n1);
    t1.lock = Some(blocks(older1, 0)?);
    let mut t2 = ks(k2, n2);
    t2.lock = Some(blocks(older2, 1)?);
    let mut t3 = ks(1, 1);
    t3.lock = Some(Lock::AfterHeight(after_height));
    checked(PathList {
        wrapper,
        paths: vec![t1, t2, t3],
    })
}
