#!/usr/bin/env python3
"""ci/doc-gate-paths.py -- derive the set of repo paths a doc gate depends on.

    usage: python3 ci/doc-gate-paths.py <book-dir>...     (run from the repo root)
    prints one repo-relative path per line; a directory ends in "/".

A doc gate (docs/manual, docs/technical-manual, ...) executes and includes more
than its own directory: the books share files by SYMLINK
(docs/technical-manual/tests/verify-examples.sh -> ../../manual/tests/...) and
by RELATIVE REFERENCE (docs/quickstart/.cspell.json imports
../manual/.cspell.json; docs/quickstart/Makefile reads ../manual/transcripts).
Hand-maintained path lists missed exactly these (review I-1, 2026-09-23), so the
set is DERIVED from the checked-out tree, to a fixed point:

  1. every book dir given;
  2. the resolved target of every symlink found under a watched directory
     (a directory target is watched recursively; a file target exactly);
  3. every `../`-relative path written in a non-Markdown text file (a
     Makefile, script, filter, lint config) under a watched
     directory that resolves, from that file's own directory, to an EXISTING
     path inside the repo but outside the watched set. A path that resolves to
     an ANCESTOR of the file (e.g. the Makefile's `$(DIR)/../..` repo-root
     idiom) is a traversal base, not an include, and is skipped -- otherwise
     every book would watch the whole repository.

Markdown is not scanned for step 3: a `../` in prose or a link is a
reference for the reader, not a file the build reads (measured: scanning it
pulled LICENSE, a codex32 PDF and all of docs/manual into manual-gui through
README and agent-report prose). Transcript includes are resolved through
$TRANSCRIPTS_DIR by the Lua filter, never by a relative path in the source.

Over-inclusion is the safe direction (a gate runs when it need not); the
derivation never removes anything. What it CANNOT see is a dependency spelled
through a make/shell variable rooted at the repo (`$(TOOLKIT_ROOT)/crates/...`,
`docs/tools/render-mermaid-cache.py`); those stay in each workflow's explicit
`--also` regex, and ci/doc-gate-guard.test.sh pins the ones known today.
"""
import os
import re
import sys

REL = re.compile(r"(?:\.\./)+[A-Za-z0-9_.][A-Za-z0-9_./-]*")


def is_text(path):
    try:
        with open(path, "rb") as f:
            return b"\0" not in f.read(8192)
    except OSError:
        return False


def main(books):
    root = os.path.realpath(".")
    watched = {}  # repo-relative path -> is_dir

    def inside_repo(r):
        return r != ".." and not r.startswith("../") and r != "."

    def covered(r):
        for w, isdir in watched.items():
            if r == w or (isdir and r.startswith(w + "/")):
                return True
        return False

    queue = []

    def add(r, isdir):
        if not inside_repo(r) or covered(r):
            return
        watched[r] = isdir
        queue.append(r)  # a directory is walked; a file is scanned itself

    for b in books:
        b = b.rstrip("/")
        if not os.path.isdir(b):
            sys.exit(f"doc-gate-paths: {b} is not a directory")
        add(os.path.relpath(os.path.abspath(b), root), True)

    def scan_links(p):
        if os.path.islink(p):
            t = os.path.realpath(p)
            if os.path.exists(t):
                add(os.path.relpath(t, root), os.path.isdir(t))

    def scan_refs(p):
        if os.path.islink(p) or p.endswith(".md") or not is_text(p):
            return
        try:
            text = open(p, encoding="utf-8", errors="replace").read()
        except OSError:
            return
        here = os.path.dirname(os.path.realpath(p))
        for m in REL.findall(text):
            t = os.path.normpath(os.path.join(here, m.rstrip(".")))
            if not os.path.exists(t):
                continue
            if here == t or here.startswith(t + os.sep):
                continue  # an ancestor: a traversal base, not an include
            add(os.path.relpath(os.path.realpath(t), root), os.path.isdir(t))

    while queue:
        d = queue.pop()
        if not os.path.isdir(d):
            scan_refs(d)
            continue
        for dirpath, dirnames, filenames in os.walk(d):
            # never walk build output
            dirnames[:] = [x for x in dirnames if x not in ("build", ".git")]
            for name in dirnames + filenames:
                scan_links(os.path.join(dirpath, name))
            for name in filenames:
                scan_refs(os.path.join(dirpath, name))

    # Final de-duplication: an entry added before its enclosing directory.
    for w in sorted(watched):
        if any(o != w and watched[o] and w.startswith(o + "/") for o in watched):
            continue
        print(w + ("/" if watched[w] else ""))


if __name__ == "__main__":
    main(sys.argv[1:])
