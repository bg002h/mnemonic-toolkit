"""Tests for check.py, the architecture guide link checker (stdlib only).

Run from the repository root:  python3 -B -m unittest discover -s docs/architecture -p test_check.py
"""
import os
import subprocess
import sys
import tempfile
import unittest

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import check  # noqa: E402


def write(path, text=""):
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, "w", encoding="utf-8") as f:
        f.write(text)


def git(cwd, *args):
    subprocess.run(
        ["git", "-c", "user.name=t", "-c", "user.email=t@example.invalid", *args],
        cwd=cwd, check=True, capture_output=True,
    )


def make_sibling(root, name, branch, files):
    """A sibling clone with `files` committed on `branch`."""
    repo = os.path.join(root, name)
    os.makedirs(repo)
    git(repo, "init", "-q", "-b", branch)
    for rel in files:
        write(os.path.join(repo, rel), "x\n")
    git(repo, "add", "-A")
    git(repo, "commit", "-q", "-m", "init")
    return repo


class Fixture(unittest.TestCase):
    def setUp(self):
        self._tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self._tmp.cleanup)
        self.root = os.path.join(self._tmp.name, "repo")
        self.siblings = os.path.join(self._tmp.name, "siblings")
        os.makedirs(self.siblings)
        self.docs = os.path.join(self.root, "docs", "architecture")
        os.makedirs(self.docs)

    def guide(self, text):
        write(os.path.join(self.docs, "README.md"), text)

    def run_check(self):
        return check.check(self.docs, self.root, [self.siblings])


class RelativeLinks(Fixture):
    def test_existing_relative_path_passes(self):
        write(os.path.join(self.root, "crates", "a", "lib.rs"))
        self.guide("See [a](../../crates/a/lib.rs) and [dir](../../crates/a/).\n")
        r = self.run_check()
        self.assertEqual(r.errors, [])
        self.assertEqual(r.checked, 2)

    def test_missing_relative_path_fails(self):
        self.guide("See [gone](../../crates/gone/lib.rs).\n")
        r = self.run_check()
        self.assertEqual(len(r.errors), 1)
        self.assertIn("crates/gone/lib.rs", r.errors[0])

    def test_fragment_is_not_validated_but_path_is(self):
        write(os.path.join(self.root, "docs", "m.md"))
        self.guide("[ok](../m.md#no-such-heading) [bad](../nope.md#x)\n")
        r = self.run_check()
        self.assertEqual(len(r.errors), 1)
        self.assertIn("nope.md", r.errors[0])

    def test_links_in_code_fences_are_ignored(self):
        self.guide("```\n[x](../../missing.rs)\n```\n")
        r = self.run_check()
        self.assertEqual(r.errors, [])
        self.assertEqual(r.checked, 0)


class SiblingLinks(Fixture):
    URL = "https://github.com/bg002h/other/tree/{b}/{p}"

    def test_existing_sibling_path_passes(self):
        make_sibling(self.siblings, "other", "main", ["crates/x/lib.rs"])
        self.guide("[x](%s)\n" % self.URL.format(b="main", p="crates/x"))
        r = self.run_check()
        self.assertEqual(r.errors, [])
        self.assertEqual(r.checked, 1)

    def test_missing_sibling_path_fails(self):
        make_sibling(self.siblings, "other", "main", ["crates/x/lib.rs"])
        self.guide("[x](%s)\n" % self.URL.format(b="main", p="crates/gone"))
        r = self.run_check()
        self.assertEqual(len(r.errors), 1)
        self.assertIn("crates/gone", r.errors[0])

    def test_missing_sibling_clone_is_skipped_with_a_note(self):
        self.guide("[x](%s)\n" % self.URL.format(b="main", p="crates/x"))
        r = self.run_check()
        self.assertEqual(r.errors, [])
        self.assertEqual(r.checked, 0)
        self.assertEqual(len(r.skipped), 1)
        self.assertIn("other", r.skipped[0])

    def test_sibling_is_checked_against_the_named_branch_not_the_checkout(self):
        repo = make_sibling(self.siblings, "other", "main", ["a.txt"])
        git(repo, "checkout", "-q", "-b", "feature")
        write(os.path.join(repo, "only-on-feature.txt"))
        git(repo, "add", "-A")
        git(repo, "commit", "-q", "-m", "f")
        self.guide("[x](%s)\n" % self.URL.format(b="main", p="only-on-feature.txt"))
        r = self.run_check()
        self.assertEqual(len(r.errors), 1)

    def test_link_naming_a_non_default_branch_fails(self):
        repo = make_sibling(self.siblings, "other", "main", ["a.txt"])
        git(repo, "update-ref", "refs/remotes/origin/main", "HEAD")
        git(repo, "update-ref", "refs/remotes/origin/dev", "HEAD")
        git(repo, "symbolic-ref", "refs/remotes/origin/HEAD", "refs/remotes/origin/main")
        self.guide("[x](%s)\n" % self.URL.format(b="dev", p="a.txt"))
        r = self.run_check()
        self.assertEqual(len(r.errors), 1)
        self.assertIn("default branch is main", r.errors[0])


class MainExitCodes(Fixture):
    def test_exit_zero_when_clean_and_nonzero_when_missing(self):
        write(os.path.join(self.root, "ok.txt"))
        self.guide("[ok](../../ok.txt)\n")
        argv = ["--repo-root", self.root, "--siblings", self.siblings]
        self.assertEqual(check.main(argv), 0)
        self.guide("[bad](../../missing.txt)\n")
        self.assertEqual(check.main(argv), 1)


if __name__ == "__main__":
    unittest.main()
