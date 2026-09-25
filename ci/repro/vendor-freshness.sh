#!/usr/bin/env bash
# vendor/ freshness + CONTENT guard — the LEADING (PR-time) gate.
#
# Four checks, in ascending cost. Any one RED fails the gate.
#
#   (1) RESOLUTION  — can the committed vendor/ tree satisfy Cargo.lock under the
#       reproducible build's `--offline --locked` source-replacement config?
#   (2) INTEGRITY   — does every vendored file match the sha256 recorded for it in
#       its crate's `.cargo-checksum.json`?
#   (3) REGISTRY PROVENANCE — for every crates.io crate, does that manifest's
#       `package` digest (the sha256 of the published .crate) equal the
#       `checksum` Cargo.lock pins, and is the SET of crates lacking such an
#       anchor exactly the one this gate is grounded for?
#       Honest scope note, measured: the digest comparison itself is REDUNDANT
#       with cargo. Tampering a `package` digest REDs in check (1) already
#       ("checksum for `bitcoin v0.32.8` changed between lock files"), because
#       cargo validates it during resolution. What is NOT redundant, and the
#       reason (3) stays, is the unanchored-SET assertion: cargo is perfectly
#       happy with a new git or path dependency that no digest can vouch for,
#       and that is the shape F-354 arrived in.
#   (4) GIT-FORK PROVENANCE — check (3) is impossible for a git dependency: a git
#       source has no published tarball, so BOTH `package` and Cargo.lock's
#       `checksum` are null. That hole is anchored here instead, against a
#       digest grounded on upstream by hand (see GROUNDING below).
#
# WHY (2)-(4) EXIST — F-354. Check (1) alone was green for two months over a
# vendor/miniscript tree vendored from the WRONG REVISION (95fdd1c5, missing
# rust-miniscript PR #953) while Cargo.toml/Cargo.lock pinned ff4732e5. Check (1)
# cannot see this: it compares Cargo.lock resolution, never the vendored BYTES,
# and name+version+source-id all matched. Every networked build fetched the
# correct rev from git and was fine; only the `--offline` vendored path — which
# is what the reproducible RELEASE binary is built from — compiled the old
# formatter. Depth->=2 taproot restore was measurably broken in that binary while
# every gate reported green.
#
# NOTE check (2) alone would NOT have caught F-354 either, and this was measured,
# not assumed: the mis-vendored tree was internally SELF-CONSISTENT (168 crates,
# 7479 files, 0 checksum mismatches) because `cargo vendor` wrote the manifest
# from the same wrong rev it wrote the files from. Integrity catches corruption
# and hand-edits; only (3)/(4) catch "vendored from the wrong source".
#
# ── BLIND SPOTS. Read these before trusting a green. ────────────────────────
#
#  * Check (2) proves the tree matches what `cargo vendor` wrote. It does NOT
#    prove what `cargo vendor` wrote came from the pinned rev. For crates.io
#    crates that gap is closed by the published-tarball digest (enforced by cargo
#    in check (1), restated in (3)). For the git fork there is no such digest,
#    and (4) closes it only as far as its grounding reaches.
#  * Check (4) is a TRUST-ON-FIRST-USE anchor, not a live verification. It
#    compares the vendored tree against a digest a human grounded against GitHub
#    once. It proves the tree has not CHANGED since that grounding; it cannot by
#    itself prove the grounding was right. Re-ground it (command below) whenever
#    the pin moves — the gate REDs and tells you to, rather than passing silently.
#  * NOTHING here re-proves bit-for-bit build reproducibility. That stays with
#    repro-drift.yml (scheduled) and the release `repro` gate (tag).
#  * A dependency added from a NEW git source would have no anchor at all, so
#    check (3) fails closed on any unanchored crate it does not already know
#    about, rather than skipping it. A git source in Cargo.lock with no
#    GROUNDED_GIT_SOURCES row also fails closed, before check (1).
#  * Check (1) runs under an EMPTY, throwaway CARGO_HOME (F-675). Without that, a
#    git source the stanzas miss resolves from ~/.cargo/git on any machine that
#    has built the tree, so a local run passed while CI failed. The mutation
#    tests in ci/repro/vendor-freshness.test.sh pin every RED, and pin the
#    hermetic home directly: they run the gate with a caller CARGO_HOME whose
#    config.toml is malformed, which only passes if that home is ignored.
#
# Spec: design/SPEC_vendor_freshness_ci_guard.md
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$REPO_ROOT"

# ── GROUNDING for check (4) — one row per tolerated git source ─────────────
# Format: <vendor dir>|<git URL exactly as in Cargo.lock>|<grounded rev>|<sha256 of
# vendor/<dir>/.cargo-checksum.json>. Check (1) builds one [source] replacement
# stanza per row, keyed on the rev CARGO.LOCK carries for that URL; check (4)
# REDs if that rev is not the grounded one or the manifest digest moved. A git
# source in Cargo.lock with NO row fails closed before check (1).
#
# miniscript — grounded 2026-08-27 while fixing F-354: 97 of the 98 vendored
#   files were confirmed byte-identical to https://github.com/rust-bitcoin/
#   rust-miniscript at ff4732e5 by git blob hash (GitHub trees API, recursive).
#   The 98th is Cargo.toml, which `cargo vendor` rewrites ("THIS FILE IS
#   AUTOMATICALLY GENERATED BY CARGO").
# md-codec — grounded 2026-09-24 for F-675 (md-codec became a git dependency in
#   F-642): 395 of the 396 vendored files are byte-identical, by git blob hash,
#   to crates/md-codec/ of https://github.com/bg002h/descriptor-mnemonic at
#   cf35d61a (tag descriptor-mnemonic-md-cli-v0.19.0; the commit was confirmed
#   present on GitHub via the commits API). The 396th is Cargo.toml, rewritten
#   by `cargo vendor` as above. No upstream file of that crate is missing.
#
# To RE-GROUND after moving a pin:
#   cargo vendor --locked vendor/
#   sha256sum vendor/<dir>/.cargo-checksum.json
# and paste the rev + digest below. Verify the vendored files against upstream
# at the new rev (git blob hashes) before you do.
GROUNDED_GIT_SOURCES=(
  "miniscript|https://github.com/rust-bitcoin/rust-miniscript|ff4732e5f75aa555682343cb180fa72ee3e8e9d5|30cc80f5ea57305f09790b661805b58cfdcd16aaaddd26c3769078eccd9a1277"
  "md-codec|https://github.com/bg002h/descriptor-mnemonic|cf35d61af0f058029df841f4882945e53f86cd4b|717b40e17d1a32acb7d1ac23aeb538baa08b7e72256efaef7f5a4598aeb41fb5"
)

# Every git source Cargo.lock pins, as "<url> <rev>" (authoritative, comment-free).
# Deriving the stanzas' revs from the lock is what makes them auto-track a pin;
# check (4) is what stops that tracking from quietly accepting an unverified rev.
mapfile -t LOCK_GIT_SOURCES < <(
  sed -nE 's/^source = "git\+([^?"#]+)\?rev=([0-9a-f]{40})#[0-9a-f]{40}"$/\1 \2/p' Cargo.lock | sort -u
)
if [ "${#LOCK_GIT_SOURCES[@]}" -eq 0 ]; then
  echo "::error::vendor-freshness: Cargo.lock names no 'git+<url>?rev=<40-hex>' source, but this" \
       "gate is grounded for ${#GROUNDED_GIT_SOURCES[@]}. Either the pins moved to crates.io (drop" \
       "the rows) or the lock format changed under this parser. Failing closed." >&2
  exit 1
fi

# A git source in Cargo.lock that no row grounds would get no [source] stanza.
# Say so by name here, instead of leaving it to check (1)'s cargo error.
SRC_CONFIG=( --config 'source.crates-io.replace-with="vendored-sources"' )
LOCK_REVS=""      # "<dir>=<rev>" per grounded row, for check (4)
for line in "${LOCK_GIT_SOURCES[@]}"; do
  url="${line% *}"; rev="${line#* }"
  row=""
  for g in "${GROUNDED_GIT_SOURCES[@]}"; do
    IFS='|' read -r gdir gurl _ _ <<<"$g"
    if [ "$gurl" = "$url" ]; then row="$gdir"; fi
  done
  if [ -z "$row" ]; then
    echo "::error::vendor-freshness: Cargo.lock pins git source $url?rev=$rev, which has no" \
         "row in GROUNDED_GIT_SOURCES. A git source has no published tarball digest, so the" \
         "vendored bytes cannot be verified. Ground it (see the header) or drop it." >&2
    exit 1
  fi
  SRC_CONFIG+=(
    --config "source.\"git+${url}?rev=${rev}\".git=\"${url}\""
    --config "source.\"git+${url}?rev=${rev}\".rev=\"${rev}\""
    --config "source.\"git+${url}?rev=${rev}\".replace-with=\"vendored-sources\""
  )
  LOCK_REVS+="${row}=${rev} "
done
SRC_CONFIG+=( --config 'source.vendored-sources.directory="vendor"' )

# ── (1) RESOLUTION ──────────────────────────────────────────────────────────
# The replacement stanzas mirror the release build (ci/repro/double-build.sh,
# man-pages.yml): crates-io + one block per git source + vendored-sources.
#
# HERMETIC CARGO_HOME. Under `--offline`, a git source with no replacement stanza
# resolves from ~/.cargo/git whenever that rev is cached there — which it always
# is on a developer box that has built the tree. That is how check (1) passed
# locally while CI (empty cache) failed "can't checkout ... you are in the
# offline mode" after F-642 made md-codec a git dependency (F-675). An empty,
# throwaway CARGO_HOME gives every run CI's empty cache, and also keeps a
# developer's ~/.cargo/config.toml out of the resolution. Toolchains live under
# RUSTUP_HOME, so the pinned toolchain is unaffected.
HERMETIC_CARGO_HOME="$(mktemp -d)"
trap 'rm -rf "$HERMETIC_CARGO_HOME"' EXIT

echo "vendor-freshness: (1/4) resolving Cargo.lock against committed vendor/ (offline, locked, empty CARGO_HOME; git sources: ${LOCK_REVS% }) ..."
if CARGO_HOME="$HERMETIC_CARGO_HOME" cargo metadata --format-version 1 --locked --offline "${SRC_CONFIG[@]}" >/dev/null; then
  echo "vendor-freshness: (1/4) OK — vendor/ satisfies Cargo.lock."
else
  echo "::error::vendor/ is out of sync with Cargo.lock — the --offline --locked reproducible build" \
       "cannot resolve a dependency from the committed vendor/ tree. Run 'cargo vendor vendor/' and" \
       "commit the result (see docs/verify-reproducibility.md). This is the v0.74.0 release-CI failure" \
       "class, now caught at PR time." >&2
  exit 1
fi

# ── (2) INTEGRITY, (3) REGISTRY PROVENANCE, (4) GIT-FORK PROVENANCE ─────────
# Pure content checks: no compile, no network, no toolchain. Measured at ~0.2s
# for 168 crates / 7479 files, so every file is verified — nothing is sampled.
GROUNDED="$(printf '%s\n' "${GROUNDED_GIT_SOURCES[@]}")" \
LOCK_REVS="$LOCK_REVS" \
python3 - <<'PY'
import glob, hashlib, json, os, re, sys

# dir -> (url, grounded rev, grounded manifest sha256)
grounded = {}
for row in os.environ["GROUNDED"].splitlines():
    d, url, rev, sha = row.split("|")
    grounded[d] = (url, rev, sha)
lock_revs = dict(kv.split("=", 1) for kv in os.environ["LOCK_REVS"].split())

def sha256_file(path):
    h = hashlib.sha256()
    with open(path, "rb") as fh:
        for chunk in iter(lambda: fh.read(1 << 20), b""):
            h.update(chunk)
    return h.hexdigest()

errors = []

manifests = sorted(glob.glob("vendor/*/.cargo-checksum.json"))
if not manifests:
    print("::error::vendor-freshness: no vendor/*/.cargo-checksum.json found — "
          "is the vendor/ tree committed? Failing closed.", file=sys.stderr)
    sys.exit(1)

# ── (2) INTEGRITY ──────────────────────────────────────────────────────────
n_files = 0
for cj in manifests:
    crate_dir = os.path.dirname(cj)
    try:
        files = json.load(open(cj))["files"]
    except Exception as exc:                       # malformed manifest is itself a defect
        errors.append(f"{cj}: unreadable checksum manifest ({exc})")
        continue
    for rel, expected in files.items():
        path = os.path.join(crate_dir, rel)
        n_files += 1
        if not os.path.isfile(path):
            errors.append(f"{path}: MISSING (recorded in {cj})")
            continue
        actual = sha256_file(path)
        if actual != expected:
            errors.append(
                f"{path}: CONTENT MISMATCH\n"
                f"      recorded {expected}\n"
                f"      on disk  {actual}"
            )

# ── (3) REGISTRY PROVENANCE ────────────────────────────────────────────────
lock = open("Cargo.lock").read()
lock_ck = {}
for block in lock.split("[[package]]")[1:]:
    name = re.search(r'^name = "(.*)"$', block, re.M)
    ver = re.search(r'^version = "(.*)"$', block, re.M)
    ck = re.search(r'^checksum = "(.*)"$', block, re.M)
    if name and ver:
        lock_ck[(name.group(1), ver.group(1))] = ck.group(1) if ck else None

anchored = 0
unanchored = []
for cj in manifests:
    crate_dir = os.path.dirname(cj)
    toml = open(os.path.join(crate_dir, "Cargo.toml")).read()
    name = re.search(r'^name = "(.*)"$', toml, re.M)
    ver = re.search(r'^version = "(.*)"$', toml, re.M)
    if not (name and ver):
        errors.append(f"{crate_dir}/Cargo.toml: could not read name/version")
        continue
    key = (name.group(1), ver.group(1))
    pkg = json.load(open(cj)).get("package")
    want = lock_ck.get(key)
    if pkg is None or want is None:
        # No published tarball to anchor against — a git/path source.
        unanchored.append((os.path.basename(crate_dir), key))
    elif pkg != want:
        errors.append(
            f"{crate_dir}: PROVENANCE MISMATCH for {key[0]} {key[1]}\n"
            f"      .cargo-checksum.json package {pkg}\n"
            f"      Cargo.lock checksum          {want}"
        )
    else:
        anchored += 1

# The ONLY crates allowed to lack a registry anchor are the grounded git sources.
# A new unanchored source must fail closed rather than be silently exempted.
unexpected = [d for d, _ in unanchored if d not in grounded]
if unexpected:
    errors.append(
        "vendored crate(s) with NO offline provenance anchor and no grounding in "
        "this gate: " + ", ".join(sorted(unexpected)) + "\n"
        "      A git or path source cannot be checked against Cargo.lock (no published\n"
        "      tarball digest). Ground it in GROUNDED_GIT_SOURCES, or drop it."
    )

# ── (4) GIT-FORK PROVENANCE ────────────────────────────────────────────────
ok4 = []
for d, (url, rev, want_sha) in sorted(grounded.items()):
    manifest_path = os.path.join("vendor", d, ".cargo-checksum.json")
    lock_rev = lock_revs.get(d)
    if lock_rev is None:
        errors.append(
            f"{d}: grounded as a git source ({url}), but Cargo.lock no longer pins it\n"
            f"      from git. Drop its row from GROUNDED_GIT_SOURCES in\n"
            f"      ci/repro/vendor-freshness.sh, so the gate's exemptions match the lock."
        )
    elif not os.path.isfile(manifest_path):
        errors.append(f"{manifest_path}: MISSING — cannot verify git-source provenance")
    elif lock_rev != rev:
        errors.append(
            f"the {d} pin MOVED: Cargo.lock is at {lock_rev}, but this gate is\n"
            f"      grounded at {rev}. The vendored tree cannot be verified against a\n"
            f"      rev nobody has checked. Re-vendor, verify the tree against upstream at the\n"
            f"      new rev, then update its GROUNDED_GIT_SOURCES row in\n"
            f"      ci/repro/vendor-freshness.sh (see GROUNDING in its header)."
        )
    else:
        actual = sha256_file(manifest_path)
        if actual != want_sha:
            errors.append(
                f"{manifest_path}: GIT-SOURCE PROVENANCE MISMATCH\n"
                f"      grounded {want_sha}\n"
                f"      on disk  {actual}\n"
                f"      The vendored {d} tree is not the one grounded against upstream\n"
                f"      {rev}. This is the F-354 defect class: a tree vendored from the\n"
                f"      wrong rev is self-consistent and passes every other check."
            )
        else:
            ok4.append(f"{d}@{rev[:8]}")

if errors:
    print(f"::error::vendor-freshness: {len(errors)} content defect(s) in vendor/:", file=sys.stderr)
    for e in errors:
        print(f"  - {e}", file=sys.stderr)
    sys.exit(1)

print(f"vendor-freshness: (2/4) OK — {n_files} files across {len(manifests)} crates match their recorded sha256.")
print(f"vendor-freshness: (3/4) OK — {anchored} crates anchored to Cargo.lock checksums; "
      f"{len(unanchored)} git source(s) exempt by grounding ({', '.join(sorted(d for d, _ in unanchored))}).")
print(f"vendor-freshness: (4/4) OK — {', '.join(ok4)} match the trees grounded against upstream.")
PY
