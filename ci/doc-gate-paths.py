#!/usr/bin/env python3
"""ci/doc-gate-paths.py -- derive the set of repo paths a doc gate depends on.

    usage: python3 ci/doc-gate-paths.py [--rev REV]... <book-dir>...
           (run inside the repo; default REV is HEAD)
    prints one repo-relative path per line; a directory ends in "/".

A doc gate (docs/manual, docs/technical-manual, ...) executes and includes more
than its own directory: the books share files by SYMLINK
(docs/technical-manual/tests/verify-examples.sh -> ../../manual/tests/...) and
by RELATIVE REFERENCE (docs/quickstart/.cspell.json imports
../manual/.cspell.json; docs/quickstart/Makefile reads ../manual/transcripts).
Hand-maintained path lists missed exactly these (review I-1), so the set is
DERIVED, to a fixed point:

  1. every book dir given;
  2. the LEXICAL target of every symlink under a watched directory: the
     symlink's stored text resolved against the symlink's own directory, with
     NO existence check (review fix1 NEW-1: requiring the target to exist
     meant a commit deleting docs/manual/tests/verify-examples.sh un-watched
     it for every book that reached it by symlink -- the deletion erased the
     evidence in the very commit that needed catching). A target that exists
     nowhere is watched both as a file and as a directory;
  3. every `../`-relative path written in a non-Markdown text file (a
     Makefile, script, filter, lint config) under a watched directory that
     resolves, from that file's own directory, to a path that EXISTS in the
     tree being read, outside the watched set. A path that resolves to an
     ANCESTOR of the file (the Makefile's `$(DIR)/../..` repo-root idiom) is
     a traversal base, not an include, and is skipped -- otherwise every book
     would watch the whole repository.

THE TREE IS READ FROM GIT, NOT THE WORKING DIRECTORY, and the guard derives the
set from BOTH the diff's base and HEAD and takes the union (--rev twice). Step 3
still needs existence, so a referenced file deleted in HEAD is caught through
the base tree, where it still exists; a retargeted symlink contributes its old
target (base) and its new one (head).

Markdown is not scanned for step 3: a `../` in prose or a link is a reference
for the reader, not a file the build reads (measured: scanning it pulled
LICENSE, a codex32 PDF and all of docs/manual into manual-gui through README
and agent-report prose). Transcript includes are resolved through
$TRANSCRIPTS_DIR by the Lua filter, never by a relative path in the source.

Over-inclusion is the safe direction (a gate runs when it need not); the
derivation never removes anything. `$(TOOLKIT_ROOT)/<path>` in a scanned file
is resolved from the repo root (fix1 self-check: manual-gui's lint runs
$(TOOLKIT_ROOT)/docs/tools/render-mermaid-cache.py, which neither its old
`paths:` filter nor its --also watched). What it still CANNOT see is a
dependency spelled any other indirect way (a shell variable, a computed path);
each workflow's explicit `--also` regex keeps those (crates/, Cargo.*).
A book dir that does not exist in a rev contributes its own prefix only (a
deleted book is still watched).
"""
import posixpath
import re
import subprocess
import sys

REL = re.compile(r"(?:\.\./)+[A-Za-z0-9_.][A-Za-z0-9_./-]*")
# The books' Makefiles all define TOOLKIT_ROOT := $(abspath $(<BOOK>_DIR)/../..),
# i.e. the repo root, and reach repo files through it (the mermaid cache tool,
# the wallet_import fixtures, Cargo.toml). Those are resolved from the repo
# root. WORKSPACE_ROOT is the directory ABOVE the repo (sibling repos): out of
# scope for a path filter, so not followed.
ROOTED = re.compile(r"\$\(TOOLKIT_ROOT\)/([A-Za-z0-9_.][A-Za-z0-9_./-]*)")


class Tree:
    """A git tree: files (path -> (mode, sha)) and the set of directories."""

    def __init__(self, rev):
        out = subprocess.run(
            ["git", "ls-tree", "-r", "-z", "--full-tree", rev],
            check=True, capture_output=True,
        ).stdout.decode("utf-8", "surrogateescape")
        self.files = {}
        self.dirs = {""}
        for rec in out.split("\0"):
            if not rec:
                continue
            meta, path = rec.split("\t", 1)
            mode, typ, sha = meta.split()
            if typ != "blob":
                continue  # submodules: not followed
            self.files[path] = (mode, sha)
            d = posixpath.dirname(path)
            while d not in self.dirs:
                self.dirs.add(d)
                d = posixpath.dirname(d)
        self._blobs = {}

    def blob(self, sha):
        if sha not in self._blobs:
            self._blobs[sha] = subprocess.run(
                ["git", "cat-file", "blob", sha], check=True, capture_output=True
            ).stdout
        return self._blobs[sha]

    def is_link(self, p):
        return p in self.files and self.files[p][0] == "120000"

    def exists(self, p):
        return p in self.files or p in self.dirs

    def walk(self, d):
        """Every file path under directory d."""
        pre = d + "/"
        return [p for p in self.files if p.startswith(pre)]


def norm(p):
    p = posixpath.normpath(p)
    return "" if p == "." else p


def inside_repo(p):
    return p != "" and p != ".." and not p.startswith("../")


def derive(tree, books, watched):
    """Add to watched (path -> kind in {"file","dir","both"}) to a fixed point."""
    queue = []

    def covered(p):
        for w, kind in watched.items():
            if p == w or (kind in ("dir", "both") and p.startswith(w + "/")):
                return True
        return False

    def add(p, kind):
        if not inside_repo(p):
            return
        if covered(p):
            # Still scan it in THIS tree: a path already watched from another
            # rev must contribute this rev's symlinks and references too.
            if p not in scanned:
                queue.append(p)
            return
        watched[p] = kind
        queue.append(p)

    scanned = set()
    for b in books:
        add(norm(b.rstrip("/")), "dir")

    while queue:
        w = queue.pop()
        if w in scanned:
            continue
        scanned.add(w)
        paths = tree.walk(w) if w in tree.dirs else ([w] if w in tree.files else [])
        for p in paths:
            here = posixpath.dirname(p)
            if tree.is_link(p):
                target = tree.blob(tree.files[p][1]).decode("utf-8", "surrogateescape")
                t = norm(posixpath.join(here, target))
                if t in tree.dirs:
                    add(t, "dir")
                elif t in tree.files:
                    add(t, "file")
                else:
                    add(t, "both")  # dangling here: watch it as either
                continue
            if p.endswith(".md"):
                continue
            data = tree.blob(tree.files[p][1])
            if b"\0" in data[:8192]:
                continue
            for m in REL.findall(data.decode("utf-8", "replace")):
                t = norm(posixpath.join(here, m.rstrip(".")))
                if not tree.exists(t):
                    continue
                if here == t or here.startswith(t + "/"):
                    continue  # an ancestor: a traversal base, not an include
                add(t, "dir" if t in tree.dirs else "file")
            for m in ROOTED.findall(data.decode("utf-8", "replace")):
                t = norm(m.rstrip("."))
                if tree.exists(t):
                    add(t, "dir" if t in tree.dirs else "file")


def main(argv):
    revs, books = [], []
    it = iter(argv)
    for a in it:
        if a == "--rev":
            revs.append(next(it))
        else:
            books.append(a)
    if not books:
        sys.exit("usage: doc-gate-paths.py [--rev REV]... <book-dir>...")
    watched = {}
    for rev in revs or ["HEAD"]:
        derive(Tree(rev), books, watched)
    out = set()
    for w, kind in watched.items():
        if kind in ("file", "both"):
            out.add(w)
        if kind in ("dir", "both"):
            out.add(w + "/")
    # de-duplicate: drop anything under a watched directory
    dirs = [o for o in out if o.endswith("/")]
    for o in sorted(out):
        if any(o != d and o.startswith(d) for d in dirs):
            continue
        print(o)


if __name__ == "__main__":
    main(sys.argv[1:])
