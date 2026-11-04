#!/usr/bin/env python3
"""Classifier for .github/workflows/miniscript-fork-tripwire.yml (stdlib only).

Rule (architect miniscript-13-2, Q2). Exit 1 (red) iff the highest non-yanked,
non-prerelease crates.io miniscript V is above --baseline AND (V's src/ carries
all four fork-only markers OR V's major is above the baseline's). Otherwise
exit 0. It fails closed (exit 1) on an empty or unusable index, a failed
download, a checksum mismatch, and a broken positive control.

  --baseline V       highest release known NOT to carry the fork-only APIs
  --index FILE       crates.io sparse-index reply for miniscript (JSON lines)
  --control-dir DIR  the pinned rev's src/ (vendor/miniscript/src); must match
                     4/4 markers or the scan itself is broken
  --crate FILE       use this .crate instead of downloading V (offline tests)

Not covered: it cannot see master-only fixes that no release carries (e.g.
rust-miniscript #1046), so it does not say whether the pin is fresh.
"""
import argparse
import hashlib
import io
import json
import os
import re
import sys
import tarfile
import time
import urllib.request

CRATE_URL = "https://static.crates.io/crates/miniscript/miniscript-{v}.crate"

# (regex over src/**/*.rs, label). Order is the order names are printed in.
MARKERS = [
    (r"^\s*SortedMultiA\(Threshold<", "Terminal::SortedMultiA (#910)"),
    (r"^\s*SortedMulti\(Threshold<", "Terminal::SortedMulti (#915)"),
    (r"pub fn derive_at_index\b", "Descriptor::derive_at_index (#911)"),
    (r"pub fn into_definite\b", "Descriptor::into_definite (#911)"),
]


def key(v):
    # Release-only comparison; a pre-release sorts below its release and is
    # filtered out before this is used.
    core = v.split("-", 1)[0].split("+", 1)[0]
    return tuple(int(p) for p in core.split("."))


def scan(texts):
    """Return the labels of the markers found in any of the given sources."""
    texts = list(texts)  # a generator would be spent after the first marker
    found = []
    for rx, label in MARKERS:
        if any(re.search(rx, t, re.M) for t in texts):
            found.append(label)
    return found


def dir_texts(root):
    for dirpath, _, files in os.walk(root):
        for f in sorted(files):
            if f.endswith(".rs"):
                with open(os.path.join(dirpath, f), encoding="utf-8", errors="replace") as fh:
                    yield fh.read()


def crate_texts(data):
    """Return the text of every <dir>/src/**/*.rs in a .crate, read in memory."""
    out = []
    with tarfile.open(fileobj=io.BytesIO(data), mode="r:gz") as tf:
        for m in tf:
            parts = m.name.split("/")
            if m.isfile() and len(parts) >= 3 and parts[1] == "src" and m.name.endswith(".rs"):
                out.append(tf.extractfile(m).read().decode("utf-8", errors="replace"))
    return out


def fetch(version):
    url = CRATE_URL.format(v=version)
    last = None
    for attempt in range(3):
        try:
            with urllib.request.urlopen(url, timeout=60) as r:
                return r.read()
        except Exception as e:  # transport: retry, then fail closed
            last = e
            time.sleep(5 if attempt < 2 else 0)
    print("::error::could not download miniscript-%s.crate (%s). Failing closed." % (version, last))
    sys.exit(1)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--baseline", required=True)
    ap.add_argument("--index", required=True)
    ap.add_argument("--control-dir", required=True)
    ap.add_argument("--crate")
    a = ap.parse_args()

    # Step 2: positive control on the pinned rev.
    k = len(scan(dir_texts(a.control_dir)))
    if k != 4:
        print("::error::marker scan found %d/4 fork-only APIs in vendor/miniscript "
              "(the pinned rev); the scan or the vendor tree is broken, so a green "
              "would mean nothing. Failing closed." % k)
        sys.exit(1)

    # Step 3: index, failing closed on an empty or unusable reply.
    try:
        with open(a.index, encoding="utf-8") as fh:
            rows = [json.loads(l) for l in fh if l.strip()]
    except (OSError, ValueError) as e:
        print("::error::unreadable miniscript index (%s) — failing closed" % e)
        sys.exit(1)
    if not rows:
        print("::error::empty miniscript index response — failing closed")
        sys.exit(1)
    live = {r["vers"]: r for r in rows if not r.get("yanked") and "-" not in r["vers"]}
    if not live:
        print("::error::no non-yanked, non-prerelease miniscript version in the "
              "index — failing closed")
        sys.exit(1)
    top = max(live, key=key)

    print("baseline           : %s" % a.baseline)
    print("highest published  : %s" % top)
    print("pinned rev markers : 4/4 (vendor/miniscript)")

    # Step 4.
    if key(top) <= key(a.baseline):
        print("No release newer than the baseline. The [patch.crates-io] pin stays.")
        return 0

    # Step 5: download, verify against the index checksum, scan.
    data = open(a.crate, "rb").read() if a.crate else fetch(top)
    if hashlib.sha256(data).hexdigest() != live[top].get("cksum"):
        print("::error::could not verify miniscript-%s.crate against the crates.io "
              "index checksum. Failing closed." % top)
        return 1
    texts = crate_texts(data)
    if not texts:
        print("::error::miniscript-%s.crate has no src/*.rs files, so a scan of it "
              "would mean nothing. Failing closed." % top)
        return 1
    found = scan(texts)
    missing = [l for _, l in MARKERS if l not in found]
    new_major = key(top)[0] > key(a.baseline)[0]

    # Step 6.
    if len(found) == 4 or new_major:
        msg = ("::error::miniscript %s may let the [patch.crates-io] pin go: %d/4 "
               "fork-only APIs present (%s)" % (top, len(found), ", ".join(found)))
        if new_major:
            msg += "; %s starts a new major line" % top
        print(msg)
        print("NEXT STEPS:")
        print("  1. In a scratch worktree of every repo that carries the pin "
              "(mnemonic-toolkit and its fuzz/, descriptor-mnemonic, mnemonic-refugium "
              "and its scripts/factcheck, refugium-wallet, mnemonic-gui), delete "
              "[patch.crates-io], raise the miniscript requirement if %s is a new "
              "major, re-lock to %s and build (cargo check --workspace --all-targets "
              "--keep-going, as the 2026-11-03 recon did)." % (top, top))
        print("  2. If all build: work FOLLOWUP taproot-coverage-cycle-on-miniscript-gt-13-1-0, "
              "and re-derive mr-codec cost.rs's planner compensations against %s." % top)
        print("  3. If any fails: raise BASELINE to %s with a dated comment naming "
              "what is still missing." % top)
        return 1

    # Step 7.
    print("::notice::miniscript %s is newer than the baseline %s but lacks %s; the "
          "[patch.crates-io] pin stays." % (top, a.baseline, ", ".join(missing)))
    return 0


if __name__ == "__main__":
    sys.exit(main())
