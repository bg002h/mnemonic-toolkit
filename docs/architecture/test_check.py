"""Regression tests for the offline architecture checker; synthetic Git data only."""
import copy
import hashlib
import importlib.util
from pathlib import Path
import subprocess
import tempfile
import unittest


spec = importlib.util.spec_from_file_location("architecture_check", Path(__file__).with_name("check.py"))
check = importlib.util.module_from_spec(spec)
spec.loader.exec_module(check)


class CheckerTests(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory(prefix="architecture-check-")
        self.addCleanup(self.tmp.cleanup)
        self.workspace = Path(self.tmp.name)
        self.repo = self.workspace / "mnemonic-toolkit"
        self.repo.mkdir()
        self.git("init", "-q", "-b", "master")
        self.git("config", "user.name", "Architecture fixture")
        self.git("config", "user.email", "fixture@example.invalid")
        self.git("config", "commit.gpgsign", "false")
        hooks = self.workspace / "empty-hooks"
        hooks.mkdir()
        self.git("config", "core.hooksPath", str(hooks))
        self.write("Cargo.toml", '[package]\nname = "mnemonic-toolkit"\nversion = "0.0.0"\n')
        self.write("Cargo.lock", 'version = 4\n[[package]]\nname = "mnemonic-toolkit"\nversion = "0.0.0"\n')
        self.write("src/lib.rs", "// Synthetic source fixture.\n")
        self.git("add", "Cargo.toml", "Cargo.lock", "src/lib.rs")
        self.git("commit", "-qm", "Source baseline")
        self.base = self.git("rev-parse", "HEAD")
        self.git("update-ref", "refs/remotes/origin/master", self.base)
        self.git("symbolic-ref", "refs/remotes/origin/HEAD", "refs/remotes/origin/master")
        self.branch = "docs/constellation-architecture"
        self.git("switch", "-qc", self.branch)
        self.write("docs/architecture/README.md", "# Guide\n\n## Details\nFixture.\n")
        self.write("docs/architecture-extra/README.md", "# Outside the allowed prefix\n")
        self.git("add", "docs")
        self.git("commit", "-qm", "Documentation snapshot")
        self.docs_rev = self.git("rev-parse", "HEAD")
        self.source = self.repo / "docs/architecture/README.md"
        self.row = {
            "repository": "mnemonic-toolkit", "default_branch": "master",
            "source_commit": self.base, "url": "https://github.com/bg002h/mnemonic-toolkit",
            "packages": [{"name": "mnemonic-toolkit", "version": "0.0.0",
                          "manifest": "Cargo.toml", "explicit_binary_declarations": []}],
            "patches": {},
            "file_sha256": {p: hashlib.sha256((self.repo / p).read_bytes()).hexdigest()
                            for p in ("Cargo.toml", "Cargo.lock")},
            "resolved_packages": [{"name": "mnemonic-toolkit", "version": "0.0.0",
                                   "source": "workspace"}],
        }
        self.inventory = {
            "documentation_branch": self.branch, "repositories": [self.row], "boundaries": [],
            "documentation_refs": [{"repository": "mnemonic-toolkit", "commit": self.docs_rev,
                                    "paths": ["docs/architecture"]}],
        }
        check.errors.clear()

    def git(self, *args, input=None):
        return subprocess.run(["git", "-C", str(self.repo), *args], input=input,
                              capture_output=True, text=True, check=True).stdout.strip()

    def write(self, path, body):
        target = self.repo / path
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_text(body)

    def url(self, kind, path, rev=None):
        return f"https://github.com/bg002h/mnemonic-toolkit/{kind}/{rev or self.branch}/{path}"

    def result(self, function, *args):
        check.errors.clear()
        try:
            function(*args)
        except ValueError as exc:
            check.errors.append(str(exc))
        return list(check.errors)

    def link(self, target):
        return self.result(check.check_link, self.source, target, self.workspace, self.inventory)

    def test_valid_inventory(self):
        self.assertEqual([], self.result(check.check_inventory, self.row, self.workspace))

    def test_wrong_default_branch(self):
        self.row["default_branch"] = "wrong-default"
        self.assertTrue(self.result(check.check_inventory, self.row, self.workspace))

    def test_missing_cached_default(self):
        self.git("symbolic-ref", "--delete", "refs/remotes/origin/HEAD")
        self.assertTrue(self.result(check.check_inventory, self.row, self.workspace))

    def test_disconnected_source_with_identical_manifest_and_lock(self):
        tree = self.git("rev-parse", self.base + "^{tree}")
        self.row["source_commit"] = self.git("commit-tree", tree, input="Disconnected snapshot\n")
        self.assertTrue(self.result(check.check_inventory, self.row, self.workspace))

    def test_branch_link_kind_must_match(self):
        for kind, path in [("tree", "docs/architecture/README.md"), ("blob", "docs/architecture")]:
            with self.subTest(kind=kind):
                self.assertTrue(self.link(self.url(kind, path)))

    def test_untracked_then_staged_file_and_directory(self):
        self.write("docs/staged/new.md", "# Staged document\n")
        targets = [self.url("blob", "docs/staged/new.md"), self.url("tree", "docs/staged")]
        for target in targets:
            with self.subTest(state="untracked", target=target):
                self.assertTrue(self.link(target))
        self.git("add", "docs/staged/new.md")
        for target in targets:
            with self.subTest(state="staged", target=target):
                self.assertEqual([], self.link(target))

    def test_directory_fragments_are_rejected(self):
        for target in [".#guide", self.url("tree", "docs/architecture#guide"),
                       self.url("tree", "docs/architecture#guide", self.docs_rev)]:
            with self.subTest(target=target):
                self.assertTrue(self.link(target))

    def test_valid_tracked_file_and_directory(self):
        for kind, path in [("blob", "docs/architecture/README.md"), ("tree", "docs/architecture")]:
            with self.subTest(kind=kind):
                self.assertEqual([], self.link(self.url(kind, path)))

    def test_heading_and_line_fragments(self):
        for prefix in ["README.md", self.url("blob", "docs/architecture/README.md"),
                       self.url("blob", "docs/architecture/README.md", self.docs_rev)]:
            with self.subTest(prefix=prefix):
                self.assertEqual([], self.link(prefix + "#details"))
                self.assertEqual([], self.link(prefix + "#L1-L3"))
                self.assertTrue(self.link(prefix + "#missing"))
                self.assertTrue(self.link(prefix + "#L999"))

    def test_encoded_delimiters_remain_filename_data(self):
        prefixes = ["README.md", self.url("blob", "docs/architecture/README.md"),
                    self.url("blob", "docs/architecture/README.md", self.docs_rev)]
        for prefix in prefixes:
            with self.subTest(prefix=prefix):
                self.assertEqual([], self.link(prefix+"#L1"))
                self.assertTrue(self.link(prefix+"%23L1"))
        for name in ("README.md#L1", "README.md?query"):
            self.write("docs/architecture/"+name, "# Actual file\n")
        self.git("add", "docs/architecture")
        self.git("commit", "-qm", "Literal delimiter filenames")
        rev = self.git("rev-parse", "HEAD")
        self.inventory["documentation_refs"].append({"repository": "mnemonic-toolkit",
            "commit": rev, "paths": ["docs/architecture"]})
        for encoded in ("README.md%23L1", "README.md%3Fquery"):
            for target in (encoded, self.url("blob", "docs/architecture/"+encoded),
                           self.url("blob", "docs/architecture/"+encoded, rev)):
                with self.subTest(target=target):
                    self.assertEqual([], self.link(target+"#actual-file"))
        self.assertEqual([], self.link("README.md#%64etails"))

    def test_configured_default_documentation_branch_links(self):
        self.inventory["documentation_branch"] = "master"
        self.assertEqual([], self.link(self.url("blob", "docs/architecture/README.md", "master")))

    def test_sha_shaped_branch_cannot_bypass_immutable_validation(self):
        self.inventory["documentation_refs"] = []
        fake = "0123456789abcdef0123456789abcdef01234567"
        for rev in [self.docs_rev, self.docs_rev.upper(), fake, fake.upper(),
                    "0123456789aBcDeF0123456789AbCdEf01234567"]:
            with self.subTest(rev=rev):
                self.inventory["documentation_branch"] = rev
                self.assertTrue(self.link(self.url("blob", "src/lib.rs", rev)))

    def test_glob_filename_requires_its_own_index_entry(self):
        self.write("docs/a.md", "# Tracked\n")
        self.git("add", "docs/a.md")
        self.write("docs/[ab].md", "# Literal filename\n")
        target = self.url("blob", "docs/[ab].md")
        self.assertTrue(self.link(target))
        self.git("--literal-pathspecs", "add", "docs/[ab].md")
        self.assertEqual([], self.link(target))

    def test_symlinks_are_not_implicit_files_or_trees(self):
        for name, destination, kind in [("alias", "architecture", "tree"),
                                        ("alias.md", "architecture/README.md", "blob")]:
            with self.subTest(kind=kind):
                path = self.repo / "docs" / name
                path.symlink_to(destination)
                self.git("add", "docs/" + name)
                self.assertTrue(self.link(self.url(kind, "docs/" + name)))

    def test_directory_replacing_indexed_file_requires_staging(self):
        path = "docs/architecture/README.md"
        (self.repo / path).unlink()
        self.write(path + "/scratch.md", "# Untracked child\n")
        target = self.url("tree", path)
        self.assertTrue(self.link(target))
        self.git("add", "--", path)
        self.assertEqual([], self.link(target))

    def test_approved_documentation_snapshot(self):
        self.assertEqual([], self.link(self.url("blob", "docs/architecture/README.md", self.docs_rev)))

    def test_documentation_snapshot_does_not_admit_other_source_or_prefix(self):
        for path in ["src/lib.rs", "Cargo.toml", "docs/architecture-extra/README.md"]:
            with self.subTest(path=path):
                self.assertTrue(self.link(self.url("blob", path, self.docs_rev)))
        self.assertEqual([], self.link(self.url("blob", "src/lib.rs", self.base)))

    def test_undeclared_documentation_revision(self):
        self.inventory["documentation_refs"] = []
        self.assertTrue(self.link(self.url("blob", "docs/architecture/README.md", self.docs_rev)))

    def test_exact_file_documentation_allowance(self):
        self.inventory["documentation_refs"][0]["paths"] = ["docs/architecture/README.md"]
        self.assertEqual([], self.link(self.url("blob", "docs/architecture/README.md", self.docs_rev)))
        self.assertTrue(self.link(self.url("tree", "docs/architecture", self.docs_rev)))

    def test_documentation_inventory_rejects_invalid_records(self):
        self.assertEqual([], self.result(check.check_documentation_refs, self.inventory, self.workspace))
        original = copy.deepcopy(self.inventory)
        for mutation in ("duplicate", "unknown-repo", "bad-sha", "escape", "empty", "missing-path", "source-path"):
            with self.subTest(mutation=mutation):
                self.inventory = copy.deepcopy(original)
                row = self.inventory["documentation_refs"][0]
                if mutation == "duplicate":
                    self.inventory["documentation_refs"].append(copy.deepcopy(row))
                elif mutation == "unknown-repo":
                    row["repository"] = "unknown"
                elif mutation == "bad-sha":
                    row["commit"] = "invalid"
                elif mutation == "escape":
                    row["paths"] = ["docs/../src"]
                elif mutation == "empty":
                    row["paths"] = []
                elif mutation == "missing-path":
                    row["paths"] = ["docs/missing"]
                else:
                    row["paths"] = ["src"]
                self.assertTrue(self.result(check.check_documentation_refs, self.inventory, self.workspace))


if __name__ == "__main__":
    unittest.main()
