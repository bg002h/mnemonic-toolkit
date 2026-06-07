#!/usr/bin/env python3
# tests/symbol-ref-check.py
#
# BLOCKING gate (lint.sh step 7/7) — kills silent source-line-ref drift in the
# technical manual. Two assertions over src/**/*.md (see
# design/SPEC_technical_manual_symbol_pin_lint.md and AUTHORING.md "Source
# citations"):
#
#   G1 — line-ref ban: no `file.rs:N` (or comma-list / range) and no bare
#        backtick `:N` continuation may appear anywhere (incl. mermaid/code
#        fences). Escape hatch: an explicit `<!-- lint-allow-lineref -->` marker
#        on the same or preceding line (for a genuine quoted rustc trace).
#   G2 — symbol existence: every `` `<path>.rs::<anchor>` `` token must resolve
#        to a source file and EVERY `::`-separated segment of <anchor> must exist
#        as a whole word in that file (`grep -wF` semantics). Colliding bare
#        basenames in non-authoritative chapters must be repo-qualified.
#
# Resolution mirrors the recon resolver (chapter->repo default + global
# suffix index over the 4 codec + 3 CLI crates src/+tests/, shallowest
# tiebreak), with authoritative-chapter trust and path-suffix collision
# detection. Sibling-repo source absent (bare CI) -> skip with a warning,
# never fail (mirrors api-surface-coverage.sh). Exit 1 on any failure.

import os
import re
import sys

# -------- args --------
SRC_DIR = ""
for arg in sys.argv[1:]:
    if arg.startswith("SRC_DIR="):
        SRC_DIR = arg[len("SRC_DIR="):]
    # MD_BIN/MK_BIN/MS_BIN/MNEMONIC_BIN accepted + ignored (parity with lint.sh)
if not SRC_DIR:
    HERE = os.path.dirname(os.path.abspath(__file__))
    SRC_DIR = os.path.abspath(os.path.join(HERE, "..", "src"))

# .../<workspace>/mnemonic-toolkit/docs/technical-manual/src -> <workspace>
WS = os.path.abspath(os.path.join(SRC_DIR, "..", "..", "..", ".."))

REPO_ROOTS = {
    "toolkit": os.path.join(WS, "mnemonic-toolkit/crates/mnemonic-toolkit"),
    "md":      os.path.join(WS, "descriptor-mnemonic/crates/md-codec"),
    "mk":      os.path.join(WS, "mnemonic-key/crates/mk-codec"),
    "ms":      os.path.join(WS, "mnemonic-secret/crates/ms-codec"),
    "md-cli":  os.path.join(WS, "descriptor-mnemonic/crates/md-cli"),
    "mk-cli":  os.path.join(WS, "mnemonic-key/crates/mk-cli"),
    "ms-cli":  os.path.join(WS, "mnemonic-secret/crates/ms-cli"),
}
CRATE_REPO = {"md-codec": "md", "mk-codec": "mk", "ms-codec": "ms",
              "mnemonic-toolkit": "toolkit", "md-cli": "md-cli",
              "mk-cli": "mk-cli", "ms-cli": "ms-cli"}

# Build the global index: (repo, abspath, relpath-from-crate-root) for every
# .rs under each crate's src/ + tests/. Absent repos are simply skipped.
INDEX = []
PRESENT_REPOS = set()
for repo, root in REPO_ROOTS.items():
    for sub in ("src", "tests"):
        d = os.path.join(root, sub)
        if not os.path.isdir(d):
            continue
        PRESENT_REPOS.add(repo)
        for dp, _, fns in os.walk(d):
            for fn in fns:
                if fn.endswith(".rs"):
                    ap = os.path.join(dp, fn)
                    rel = os.path.relpath(ap, root).replace(os.sep, "/")
                    INDEX.append((repo, ap, rel))

# Repos with no source tree in this checkout (bare CI checks out toolkit only).
ABSENT = [r for r in REPO_ROOTS if r not in PRESENT_REPOS]

# Chapters authoritatively ABOUT one repo: chapter default is trusted.
def authoritative_repo(base):
    for pre, repo in (("21-", "md"), ("22-", "mk"), ("23-", "ms"),
                      ("51-", "md"), ("52-", "mk"), ("53-", "ms"),
                      ("41-", "toolkit"), ("42-", "toolkit"), ("54-", "toolkit")):
        if base.startswith(pre):
            return repo
    return None  # catch-all / mixed-source chapter

REPO_DIR_PREFIX = ("descriptor-mnemonic/", "mnemonic-key/",
                   "mnemonic-secret/", "mnemonic-toolkit/")

def is_repo_qualified(pathpart):
    if "crates/" in pathpart:
        return True
    return any(pathpart.startswith(p) for p in REPO_DIR_PREFIX)

def suffix_matches(pathpart, repo_filter=None):
    """Index rows whose relpath suffix matches pathpart. repo_filter limits repo."""
    q = pathpart
    if q.startswith("src/") or q.startswith("tests/"):
        suf = q
    else:
        suf = q if q.startswith("/") else q  # subpath/basename
    out = []
    for repo, ap, rel in INDEX:
        if repo_filter and repo != repo_filter:
            continue
        reln = "/" + rel
        if reln.endswith("/" + suf) or rel == suf:
            out.append((repo, ap, rel))
    return out

def shallowest(hits):
    if not hits:
        return None
    mind = min(h[2].count("/") for h in hits)
    sh = [h for h in hits if h[2].count("/") == mind]
    return sh[0] if len(sh) == 1 else None

def resolve(pathpart, chapter):
    """Return (abspath, status). status: 'ok' | 'collision' | 'skip:<repo>' |
    'unresolved' | 'ambiguous'."""
    # explicit crates/<codec>/(src|tests)/...
    m = re.search(r'crates/([A-Za-z0-9_-]+)/((?:src|tests)/.+)$', pathpart)
    if m:
        codec, rest = m.group(1), m.group(2)
        repo = CRATE_REPO.get(codec)
        if repo and repo not in PRESENT_REPOS:
            return (None, "skip:" + repo)
        if repo:
            cand = os.path.join(REPO_ROOTS[repo], rest)
            return (cand, "ok") if os.path.isfile(cand) else (None, "unresolved")
        return (None, "unresolved")

    qualified = is_repo_qualified(pathpart)
    auth = authoritative_repo(chapter)

    # Authoritative chapter: trust its repo first.
    if auth and not qualified:
        if auth not in PRESENT_REPOS:
            return (None, "skip:" + auth)
        hit = shallowest(suffix_matches(pathpart, auth))
        if hit:
            return (hit[1], "ok")
        # fall through to global if not in the authoritative repo

    # Non-authoritative or fell through: collision check on path-suffix.
    if not qualified:
        repos_with = sorted({h[0] for h in suffix_matches(pathpart)})
        # restrict the "collision" notion to the 4 primary codecs (cli/toolkit
        # share many helper names but are not the ambiguity this rule targets)
        if not auth and len(repos_with) >= 2:
            return (None, "collision")
    hit = shallowest(suffix_matches(pathpart))
    if hit:
        return (hit[1], "ok")
    allhits = suffix_matches(pathpart)
    if len(allhits) > 1:
        return (None, "ambiguous")
    # Non-authoritative catch-all chapter: a bare basename may belong to any
    # repo. With siblings absent (bare CI) we cannot disprove that it lives in
    # an absent sibling, so skip rather than false-fail. Local runs (all repos
    # present, ABSENT empty) still fail on a genuinely-missing symbol — so this
    # never weakens the gate where it can resolve. (Restricted to non-auth +
    # unqualified: authoritative chapters and crates/-qualified refs stay
    # strict; see SPEC §Item-2a + design/agent-reports/...-r0-round1-review.md.)
    if not auth and not qualified and ABSENT:
        return (None, "skip:absent-sibling")
    return (None, "unresolved")

# -------- scan --------
fail = 0
def err(msg):
    global fail
    sys.stderr.write("[symbol-ref-check] FAIL: %s\n" % msg)
    fail = 1

def warn(msg):
    sys.stderr.write("[symbol-ref-check] WARN: %s\n" % msg)

ALLOW = "<!-- lint-allow-lineref -->"
LINEREF_RE = re.compile(r'[A-Za-z_][A-Za-z0-9_/.\-]*\.rs:[0-9]')
BARE_COLON_RE = re.compile(r'`:[0-9]+(?:-[0-9]+)?`')
TOKEN_RE = re.compile(r'`([A-Za-z_][A-Za-z0-9_/.\-]*\.rs)::([A-Za-z_][A-Za-z0-9_:]*)`')

def grep_word(sym, lines_text):
    return re.search(r'(?<![A-Za-z0-9_])' + re.escape(sym) + r'(?![A-Za-z0-9_])', lines_text) is not None

g2_checked = 0
g2_skipped = 0
for dp, _, fns in os.walk(SRC_DIR):
    for fn in sorted(fns):
        if not fn.endswith(".md"):
            continue
        path = os.path.join(dp, fn)
        rel = os.path.relpath(path, SRC_DIR)
        with open(path, encoding="utf-8", errors="replace") as fh:
            lines = fh.readlines()
        text_cache = {}
        for i, line in enumerate(lines, 1):
            allowed = ALLOW in line or (i >= 2 and ALLOW in lines[i - 2])
            # G1
            if not allowed:
                if LINEREF_RE.search(line):
                    err("%s:%d line-ref `%s` — use `file.rs::symbol` (AUTHORING Source citations)"
                        % (rel, i, LINEREF_RE.search(line).group(0)))
                bm = BARE_COLON_RE.search(line)
                if bm:
                    err("%s:%d bare `:N` continuation `%s` — pin a symbol anchor"
                        % (rel, i, bm.group(0)))
            # G2
            for tm in TOKEN_RE.finditer(line):
                pathpart, anchor = tm.group(1), tm.group(2)
                ap, status = resolve(pathpart, fn)
                if status.startswith("skip:"):
                    g2_skipped += 1
                    continue
                if status == "collision":
                    err("%s:%d `%s::%s` — ambiguous colliding basename; qualify the path "
                        "(e.g. add crates/<codec>/src/...)" % (rel, i, pathpart, anchor))
                    continue
                if ap is None:
                    err("%s:%d `%s::%s` — cannot resolve source file (%s)"
                        % (rel, i, pathpart, anchor, status))
                    continue
                if ap not in text_cache:
                    try:
                        with open(ap, encoding="utf-8", errors="replace") as sfh:
                            text_cache[ap] = sfh.read()
                    except OSError:
                        text_cache[ap] = None
                src_text = text_cache[ap]
                if src_text is None:
                    err("%s:%d `%s::%s` — source file unreadable" % (rel, i, pathpart, anchor))
                    continue
                g2_checked += 1
                for seg in anchor.split("::"):
                    if not grep_word(seg, src_text):
                        err("%s:%d `%s::%s` — segment `%s` not found in %s"
                            % (rel, i, pathpart, anchor, seg, os.path.relpath(ap, WS)))

if ABSENT:
    warn("sibling source absent (%s); %d sibling-repo refs skipped "
         "(codec G2 not enforced in bare CI; run `make lint` locally with all "
         "sibling repos present for full G2)" % (",".join(ABSENT), g2_skipped))

if fail:
    sys.stderr.write("[symbol-ref-check] FAILED\n")
    sys.exit(1)
sys.stdout.write("[symbol-ref-check] OK (%d `::` anchors checked, %d skipped)\n"
                 % (g2_checked, g2_skipped))
