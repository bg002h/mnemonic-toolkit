#!/usr/bin/env python3
"""Link checker for the architecture guide (standard library only).

Every link in docs/architecture/*.md must resolve:

* A relative link must name an existing path in the working tree. A fragment
  (#heading) is stripped and not validated; only the path is checked.
* A link of the form https://github.com/bg002h/<repo>/(tree|blob)/<branch>/<path>
  is checked against the sibling clone <siblings>/<repo> with `git ls-tree`,
  on origin/<branch> when the clone has it, else on the local <branch>. When
  the clone records origin's default branch, the link must name that branch.
  A missing clone is skipped with a note. Nothing is fetched, so a stale clone
  gives a stale answer; fetch it first.
* Other URLs, mailto links and pure fragments are not checked.

Links inside fenced code blocks and inline code spans are ignored.

Usage (from anywhere):
    python3 docs/architecture/check.py [--repo-root DIR] [--siblings DIR ...]

The default sibling directory is /scratch/code/shibboleth. Exit status: 0 all
checked links resolve, 1 at least one is missing, 2 bad usage.
"""
import argparse
import collections
import os
import re
import subprocess
import sys
import urllib.parse

DEFAULT_SIBLINGS = "/scratch/code/shibboleth"
GITHUB = re.compile(
    r"^https://github\.com/bg002h/([A-Za-z0-9._-]+)/(?:tree|blob)/([^/]+)(?:/(.*))?$"
)
INLINE_LINK = re.compile(r"!?\[[^\]]*\]\(\s*<?([^)\s>]+)>?(?:\s+\"[^\"]*\")?\s*\)")
REF_DEF = re.compile(r"^\s{0,3}\[[^\]]+\]:\s*<?(\S+?)>?(?:\s+.*)?$")
FENCE = re.compile(r"^\s{0,3}(```|~~~)")
CODE_SPAN = re.compile(r"`[^`\n]*`")

Result = collections.namedtuple("Result", "errors skipped checked")


def links_in(text):
    """Yield (line_number, target) for each link outside code."""
    fence = None
    for number, line in enumerate(text.splitlines(), 1):
        m = FENCE.match(line)
        if m:
            if fence is None:
                fence = m.group(1)
            elif m.group(1) == fence:
                fence = None
            continue
        if fence is not None:
            continue
        line = CODE_SPAN.sub(lambda s: " " * len(s.group(0)), line)
        for m in INLINE_LINK.finditer(line):
            yield number, m.group(1)
        m = REF_DEF.match(line)
        if m:
            yield number, m.group(1)


def git(clone, *args):
    return subprocess.run(
        ["git", "-C", clone, *args], capture_output=True, text=True
    )


def find_clone(repo, sibling_roots):
    for root in sibling_roots:
        candidate = os.path.join(root, repo)
        if os.path.exists(os.path.join(candidate, ".git")):
            return candidate
    return None


def check_sibling(clone, branch, path):
    """Return an error string, or None when the path exists on the branch."""
    ref = None
    for candidate in ("refs/remotes/origin/" + branch, "refs/heads/" + branch):
        if git(clone, "rev-parse", "--verify", "-q", candidate).returncode == 0:
            ref = candidate
            break
    if ref is None:
        return "branch %s not found in the sibling clone" % branch
    head = git(clone, "symbolic-ref", "-q", "refs/remotes/origin/HEAD")
    if head.returncode == 0:
        default = head.stdout.strip().rsplit("/", 1)[-1]
        if default != branch:
            return "link names branch %s but the default branch is %s" % (branch, default)
    if not path:
        return None
    out = git(clone, "ls-tree", "-z", "--name-only", ref, "--", path)
    if out.returncode != 0:
        return "git ls-tree failed: %s" % out.stderr.strip()
    if path not in out.stdout.split("\0"):
        return "no such path on %s" % branch
    return None


def check(docs_dir, repo_root, sibling_roots):
    errors, skipped, checked = [], [], 0
    repo_root = os.path.realpath(repo_root)
    for name in sorted(os.listdir(docs_dir)):
        if not name.endswith(".md"):
            continue
        file = os.path.join(docs_dir, name)
        shown = os.path.relpath(file, repo_root)
        with open(file, encoding="utf-8") as f:
            text = f.read()
        for number, target in links_in(text):
            where = "%s:%d" % (shown, number)
            if target.startswith("#") or target.startswith("mailto:"):
                continue
            m = GITHUB.match(target)
            if m:
                repo, branch, path = m.group(1), m.group(2), m.group(3) or ""
                path = urllib.parse.unquote(path.split("#")[0].split("?")[0]).rstrip("/")
                clone = find_clone(repo, sibling_roots)
                if clone is None:
                    skipped.append("%s: %s (no sibling clone of %s)" % (where, target, repo))
                    continue
                checked += 1
                problem = check_sibling(clone, branch, path)
                if problem:
                    errors.append("%s: %s: %s" % (where, target, problem))
                continue
            if re.match(r"^[A-Za-z][A-Za-z0-9+.-]*:", target):
                continue  # some other URL scheme: not checked
            path = urllib.parse.unquote(target.split("#")[0].split("?")[0])
            if not path:
                continue
            checked += 1
            if path.startswith("/"):
                errors.append("%s: %s: use a path relative to this file" % (where, target))
                continue
            resolved = os.path.realpath(os.path.join(docs_dir, path))
            if os.path.commonpath([resolved, repo_root]) != repo_root:
                errors.append("%s: %s: leaves the repository" % (where, target))
            elif not os.path.exists(resolved):
                errors.append("%s: %s: no such path in the working tree" % (where, target))
    return Result(errors, skipped, checked)


def main(argv=None):
    here = os.path.dirname(os.path.abspath(__file__))
    parser = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    parser.add_argument("--repo-root", default=os.path.dirname(os.path.dirname(here)))
    parser.add_argument("--siblings", action="append", default=None, metavar="DIR",
                        help="directory holding sibling clones (repeatable)")
    try:
        args = parser.parse_args(argv)
    except SystemExit as e:
        return 2 if e.code else 0
    docs_dir = os.path.join(args.repo_root, "docs", "architecture")
    if not os.path.isdir(docs_dir):
        print("check: no such directory: %s" % docs_dir, file=sys.stderr)
        return 2
    result = check(docs_dir, args.repo_root, args.siblings or [DEFAULT_SIBLINGS])
    for note in result.skipped:
        print("skipped: " + note)
    for error in result.errors:
        print("FAIL: " + error)
    print("%s: %d links checked, %d skipped"
          % ("FAIL" if result.errors else "PASS", result.checked, len(result.skipped)))
    return 1 if result.errors else 0


if __name__ == "__main__":
    sys.exit(main())
