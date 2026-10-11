#!/usr/bin/env python3
"""docs-skip gate: proves the docs-only skip edits are EXACTLY the plan.

Usage:
  docs-skip-gate.py workflows <repo> <worktree> <base-ref>
  docs-skip-gate.py files <repo> <worktree>
  docs-skip-gate.py paths-check <repo> <worktree>
  docs-skip-gate.py release-gen <engrave-worktree> <repo>=<worktree>...
  docs-skip-gate.py self-test <scratch-dir>

`workflows`: builds the expected parsed YAML of every workflow from the base
(PLAN_ci_docs_skip_2026-10-11.md, D1-D4) and asserts each edited file equals
it, types included, with repeated mapping keys rejected; asserts the counts
equal the plan's table, and that `git diff` removed no line except the ones
the plan replaces (toolkit rust.yml trigger lines, engrave's detection step
and its `steps.detect` conditions).
`files`: ci/docs-only.sh and ci/docs-only.test.sh are byte-identical to this
directory's docs-only.sh / docs-only.test.sh, and ci/docs-only-paths.txt is
byte-identical to paths/<repo>.txt.
`paths-check`: runs the worktree's real ci/docs-only.sh, in pull_request
mode on a scratch repo, once per sample path in SAMPLES[repo], and asserts
each sample is classified as the plan says.
`release-gen`: runs the edited engrave generator in memory and asserts each
output is byte-identical to that repo's release.yml.
`self-test`: proves this gate goes GREEN on the plan and RED on mutations.

NOT COVERED: added comments, blank lines, quoting, key order and YAML 1.1
spellings (YAML parsing drops them; the reviewer reads the diff), and the
guard's behaviour on real GitHub runners (the PR's own CI runs the guard on
every OS leg, and the post-merge probe in the plan exercises the skip).
"""
import copy
import hashlib
import os
import pathlib
import subprocess
import sys
import tempfile

import yaml

HERE = pathlib.Path(__file__).resolve().parent
COND = "steps.docs.outputs.docs_only != 'true'"
GUARD = {
    "name": "Docs-only change? (skips the steps below when every changed file is documentation)",
    "id": "docs",
    "shell": "bash",
    "env": {
        "DEFAULT_BRANCH": "${{ github.event.repository.default_branch }}",
        "PR_BASE_SHA": "${{ github.event.pull_request.base.sha }}",
        "PUSH_BEFORE": "${{ github.event.before }}",
    },
    "run": "bash ci/docs-only.sh",
}
SELFTEST = {"name": "Self-test the docs-only guard", "shell": "bash", "run": "bash ci/docs-only.test.sh"}
SELFTEST_LINUX = {"name": "Self-test the docs-only guard", "if": "runner.os == 'Linux'", "shell": "bash",
                  "run": "bash ci/docs-only.test.sh"}

# repo -> {workflow file: {job: "self-test" | "guard"}}; None = every job without a job-level `if`
GUARDED = {
    "descriptor-mnemonic": {"ci.yml": {"clippy": "self-test", "*": "guard"}},
    "mnemonic-toolkit": {"rust.yml": {"clippy": "self-test", "*": "guard"}},
    "mnemonic-key": {"ci.yml": {"fmt": "self-test", "*": "guard"}},
    "mnemonic-secret": {"rust.yml": {"clippy": "self-test", "*": "guard"}},
    "mnemonic-engrave": {"release.yml": {"test": "engrave", "picotool-probe": "guard"}},
    "mnemonic-transaction": {},
}
GENERATED = {"descriptor-mnemonic", "mnemonic-toolkit", "mnemonic-key", "mnemonic-secret", "mnemonic-transaction"}

TK_REMOVED = [
    "    # agents, not the human. The `paths:` filter below is UNCHANGED and",
    "    # deliberate: a staged commit must touch a gated path to build, which every",
    "    # code change does.",
    "    paths:",
    "      - 'crates/**'",
    "      # `.gitattributes` is gated because it CHANGES CHECKED-OUT BYTES and so can",
    "      # change test outcomes. Measured 2026-08-27: the shared conformance corpus",
    "      # lacked an `eol=lf` rule, so Windows checkouts CRLF-converted it and three",
    "      # windows-latest jobs went RED on a byte-level hash while every other",
    "      # platform stayed green. A commit fixing that touches no gated path under",
    "      # the old filter, so the fix itself could not earn its required contexts.",
    "      - '.gitattributes'",
    "      - 'Cargo.toml'",
    "      - 'Cargo.lock'",
    "      - '.github/workflows/rust.yml'",
    "      # The install-sh-harnesses job tests scripts/install.sh; without these a",
    "      # push that changes only the installer never ran its own harnesses.",
    "      - 'scripts/install.sh'",
    "      - 'scripts/install-*.test.sh'",
    "    # green (unchanged code) — the cost is CI minutes, not correctness. The",
    "    # push-side path filter above keeps code-push efficiency; docs-only DIRECT",
    "    # pushes to master are covered by admin bypass (enforce_admins:false).",
]

EXPECTED_COUNTS = {  # guarded jobs, steps given COND, of which combined with an existing if, self-test steps
    "descriptor-mnemonic": dict(jobs=7, steps=30, combined=2, selftests=2),
    "mnemonic-toolkit": dict(jobs=10, steps=40, combined=3, selftests=2),
    "mnemonic-key": dict(jobs=6, steps=28, combined=2, selftests=2),
    "mnemonic-secret": dict(jobs=12, steps=37, combined=2, selftests=2),
    "mnemonic-engrave": dict(jobs=2, steps=18, combined=0, selftests=1),
    "mnemonic-transaction": dict(jobs=1, steps=5, combined=0, selftests=1),
}

SAMPLES = {  # (path, expected docs_only) -- classified by the real guard
    "descriptor-mnemonic": [("design/SPEC_x.md", True), ("design/agent-reports/r.md", True), ("docs/a.md", True),
                            ("bip/bip-x.mediawiki", True), ("README.md", True), ("CHANGELOG.md", True),
                            ("design/display-grouping-vectors.tsv", False), ("design/display-grouping-vectors.tsv.sha256", False),
                            ("crates/md-cli/README.md", False), ("vendor/x/README.md", False), ("ci/docs-only-paths.txt", False),
                            (".github/workflows/ci.yml", False), ("Cargo.lock", False)],
    "mnemonic-toolkit": [("design/SPEC_x.md", True), ("docs/manual/src/a.md", True), ("CHANGELOG.md", True),
                         ("README.md", False), ("design/SPEC_convert_v0_6.md", False),
                         ("design/PLAN_v0_26_0_three_way_merge.md", False), ("design/display-grouping-vectors.tsv", False),
                         ("crates/mnemonic-toolkit/README.md", False), (".examples-build/Examples.md", False),
                         ("vendor/x/README.md", False), ("rust-toolchain.toml", False), ("ci/docs-only.sh", False)],
    "mnemonic-key": [("design/SPEC_x.md", True), ("docs/a.md", True), ("bip/bip-mnemonic-key.mediawiki", True),
                     ("README.md", True), ("design/SPEC_mk_v0_1.md", False), ("design/display-grouping-vectors.tsv", False),
                     ("design/display-grouping-vectors.tsv.sha256", False), ("crates/mk-cli/CHANGELOG.md", False),
                     ("vendor/x/README.md", False)],
    "mnemonic-secret": [("design/SPEC_x.md", True), ("docs/verify-reproducibility.md", True), ("MIGRATION.md", True),
                        ("design/display-grouping-vectors.tsv", False), ("design/display-grouping-vectors.tsv.sha256", False),
                        ("crates/ms-cli/README.md", False), ("vendor/x/README.md", False)],
    "mnemonic-engrave": [("design/SPEC_x.md", True), ("design/agent-reports/r.md", True), ("README.md", True),
                         ("design/hardware/retail-otp.json", False), ("design/hardware/captures/a.txt", False),
                         ("docs/a.md", False), ("scripts/refugium-otp.sh", False), ("flake.lock", False)],
    "mnemonic-transaction": [("design/SPEC_mt_v0_1.md", True), ("README.md", True), ("design/vectors/mt1_v1_vectors.json", True),
                             ("crates/mt-codec/src/test_vectors/mt1_v1.json", False), ("scripts/check-provenance.sh", False)],
}


class GateError(Exception):
    pass


class _Unique(yaml.SafeLoader):
    """SafeLoader that rejects a repeated key in any mapping."""


def _unique_mapping(loader, node, deep=False):
    seen = set()
    for k, _ in node.value:
        key = repr(loader.construct_object(k, deep=deep))
        if key in seen:
            raise GateError(f"duplicate key {key} at line {k.start_mark.line + 1}")
        seen.add(key)
    return loader.construct_mapping(node, deep)


_Unique.add_constructor(yaml.resolver.BaseResolver.DEFAULT_MAPPING_TAG, _unique_mapping)


def load(text):
    y = yaml.load(text, Loader=_Unique)
    if True in y:
        y["on"] = y.pop(True)
    return y


def same(a, b):
    if type(a) is not type(b):
        return False
    if isinstance(a, dict):
        return a.keys() == b.keys() and all(same(a[k], b[k]) for k in a)
    if isinstance(a, list):
        return len(a) == len(b) and all(same(x, y) for x, y in zip(a, b))
    return a == b


def git(*args):
    return subprocess.run(["git", *args], capture_output=True, text=True, check=True).stdout


def unwrap(expr):
    e = expr.strip()
    if e.startswith("${{") and e.endswith("}}"):
        e = e[3:-2].strip()
    return e


def guard_job(job, selftest, counts, where):
    steps = job["steps"]
    if not steps or not str(steps[0].get("uses", "")).startswith("actions/checkout@"):
        raise GateError(f"{where}: first step is not actions/checkout (plan: STOP)")
    for st in steps[1:]:
        if "if" in st:
            st["if"] = f"({unwrap(st['if'])}) && {COND}"
            counts["combined"] += 1
        else:
            st["if"] = COND
        counts["steps"] += 1
    new = ([copy.deepcopy(selftest)] if selftest else []) + [copy.deepcopy(GUARD)]
    steps[1:1] = new
    counts["jobs"] += 1
    counts["selftests"] += 1 if selftest else 0


def engrave_test(job, counts):
    steps = job["steps"]
    idx = [i for i, s in enumerate(steps) if s.get("name") == "docs-only detection"]
    if len(idx) != 1:
        raise GateError("engrave test: 'docs-only detection' step not found exactly once (plan: STOP)")
    i = idx[0]
    for st in steps[i + 1:]:
        if "if" in st:
            if "steps.detect.outputs.docs_only" not in st["if"]:
                raise GateError(f"engrave test: step {st.get('name')!r} has an unexpected if")
            st["if"] = st["if"].replace("steps.detect.outputs.docs_only", "steps.docs.outputs.docs_only")
            if st["if"] == COND:
                counts["steps"] += 1
    steps[i:i + 1] = [copy.deepcopy(SELFTEST), copy.deepcopy(GUARD)]
    counts["jobs"] += 1
    counts["selftests"] += 1


def expected(repo, fname, old):
    e = copy.deepcopy(old)
    counts = dict(jobs=0, steps=0, combined=0, selftests=0)
    spec = GUARDED[repo].get(fname)
    if spec:
        for jn, job in e["jobs"].items():
            mode = spec.get(jn, spec.get("*"))
            if mode is None or ("if" in job and jn not in spec):
                continue
            if mode == "engrave":
                engrave_test(job, counts)
            else:
                guard_job(job, SELFTEST if mode == "self-test" else None, counts, f"{fname}:{jn}")
    if repo in GENERATED and fname == "release.yml":
        guard_job(e["jobs"]["test"], SELFTEST_LINUX, counts, "release.yml:test")
    if repo == "mnemonic-toolkit" and fname == "rust.yml":
        if "paths" not in e["on"]["push"]:
            raise GateError("rust.yml: push has no paths filter in base (plan: STOP)")
        del e["on"]["push"]["paths"]
    return e, counts


def allowed_removed(repo, fname, base_text):
    if repo == "mnemonic-toolkit" and fname == "rust.yml":
        return list(TK_REMOVED)
    if repo == "mnemonic-engrave" and fname == "release.yml":
        lines = base_text.splitlines()
        start = lines.index("      - name: docs-only detection")
        end = lines.index("      - name: Install Rust toolchain") - 1  # the blank line before it stays
        rem = lines[start:end]
        rem += [l for l in lines if l.strip() in ("if: steps.detect.outputs.docs_only != 'true'",
                                                  "if: steps.detect.outputs.docs_only == 'true'")]
        return rem
    return []


def check_workflows(repo, tree, base):
    tree = pathlib.Path(tree)
    errs = []
    names = sorted(git("-C", str(tree), "ls-tree", "--name-only", f"{base}:.github/workflows/").split())
    cur = sorted(p.name for p in (tree / ".github/workflows").iterdir())
    if names != cur:
        errs.append(f"workflow file set changed: base={names} now={cur}")
    total = dict(jobs=0, steps=0, combined=0, selftests=0)
    allowed = {}
    for f in names:
        if not f.endswith((".yml", ".yaml")):
            continue
        base_text = git("-C", str(tree), "show", f"{base}:.github/workflows/{f}")
        try:
            old = load(base_text)
            new = load((tree / ".github/workflows" / f).read_text())
            exp, counts = expected(repo, f, old)
        except GateError as ex:
            errs.append(f"{f}: {ex}")
            continue
        for k in total:
            total[k] += counts[k]
        if not same(new, exp):
            errs.append(f"{f}: parsed YAML differs from base + planned edits")
        allowed[f] = sorted(allowed_removed(repo, f, base_text))
    if total != EXPECTED_COUNTS[repo]:
        errs.append(f"counts {total} != plan table {EXPECTED_COUNTS[repo]}")
    for f in allowed:
        diff = git("-C", str(tree), "diff", "-U0", base, "--", f".github/workflows/{f}")
        removed = sorted(l[1:] for l in diff.splitlines() if l.startswith("-") and not l.startswith("---"))
        if removed != allowed[f]:
            extra = sorted(set(removed) - set(allowed[f]))
            missing = sorted(set(allowed[f]) - set(removed))
            errs.append(f"{f}: removed lines differ from the plan (unplanned: {extra[:3]}, not removed: {missing[:3]})")
    print(f"{repo}: files={len(names)} " + " ".join(f"{k}={v}" for k, v in total.items()))
    for e in errs:
        print("FAIL:", e)
    print("GATE: " + ("RED" if errs else "GREEN"))
    return not errs


def sha(p):
    return hashlib.sha256(pathlib.Path(p).read_bytes()).hexdigest()


def check_files(repo, tree):
    tree = pathlib.Path(tree)
    pairs = [(HERE / "docs-only.sh", tree / "ci/docs-only.sh"),
             (HERE / "docs-only.test.sh", tree / "ci/docs-only.test.sh"),
             (HERE / "paths" / f"{repo}.txt", tree / "ci/docs-only-paths.txt")]
    ok = True
    for want, have in pairs:
        good = have.is_file() and sha(want) == sha(have)
        ok &= good
        print(f"files {repo}: {have.relative_to(tree)} {'identical' if good else 'MISSING OR DIFFERENT'}")
    for p in ("ci/docs-only.sh", "ci/docs-only.test.sh"):
        mode = git("-C", str(tree), "ls-files", "-s", p).split()[:1]
        good = mode == ["100755"]
        ok &= good
        print(f"files {repo}: {p} mode {mode[0] if mode else 'untracked'}{'' if good else ' (want 100755, staged)'}")
    print("GATE: " + ("GREEN" if ok else "RED"))
    return ok


def check_paths(repo, tree):
    guard = pathlib.Path(tree) / "ci/docs-only.sh"
    rules = pathlib.Path(tree) / "ci/docs-only-paths.txt"
    ok = True
    with tempfile.TemporaryDirectory(dir=os.environ.get("GATE_SCRATCH")) as d:
        d = pathlib.Path(d)
        env = dict(PATH=os.environ["PATH"], HOME=str(d), GIT_CONFIG_GLOBAL=str(d / "gc"), GIT_CONFIG_NOSYSTEM="1")
        run = lambda *a, cwd=None: subprocess.run(a, cwd=cwd, env=env, capture_output=True, text=True, check=True).stdout
        run("git", "config", "--global", "user.email", "g@t")
        run("git", "config", "--global", "user.name", "gate")
        run("git", "config", "--global", "protocol.file.allow", "always")
        run("git", "init", "-q", "--bare", "-b", "main", str(d / "o.git"))
        run("git", "-C", str(d / "o.git"), "config", "uploadpack.allowAnySHA1InWant", "true")
        run("git", "init", "-q", "-b", "main", str(d / "w"))
        w = d / "w"
        (w / "seed").write_text("seed\n")
        run("git", "add", "-A", cwd=w)
        run("git", "commit", "-qm", "seed", cwd=w)
        base = run("git", "rev-parse", "HEAD", cwd=w).strip()
        run("git", "remote", "add", "origin", f"file://{d / 'o.git'}", cwd=w)
        run("git", "push", "-q", "origin", "HEAD:main", cwd=w)
        for path, want in SAMPLES[repo]:
            run("git", "checkout", "-q", "-B", "s", base, cwd=w)
            p = w / path
            p.parent.mkdir(parents=True, exist_ok=True)
            p.write_text("sample\n")
            run("git", "add", "-A", cwd=w)
            run("git", "commit", "-qm", path, cwd=w)
            out = d / "out"
            out.write_text("")
            e2 = dict(env, GITHUB_OUTPUT=str(out), DOCS_ONLY_PATHS=str(rules), GITHUB_EVENT_NAME="pull_request",
                      DEFAULT_BRANCH="main", PR_BASE_SHA=base)
            subprocess.run(["bash", str(guard)], cwd=w, env=e2, capture_output=True, text=True, check=True)
            got = out.read_text().strip()
            good = got == f"docs_only={'true' if want else 'false'}"
            ok &= good
            print(f"paths-check {repo}: {path} -> {got}{'' if good else '  WANT docs_only=' + str(want).lower()}")
    print("GATE: " + ("GREEN" if ok else "RED"))
    return ok


def check_release_gen(engrave_tree, pairs):
    rw = pathlib.Path(engrave_tree) / "scripts/release-workflows"
    src = (rw / "emit.py").read_text()
    marker = "\nfor repo, kw in REPOS.items():\n"
    if src.count(marker) != 1:
        print("FAIL: emit.py write loop not found exactly once")
        print("GATE: RED")
        return False
    ns = {}
    sys.path.insert(0, str(rw))
    cwd = os.getcwd()
    os.chdir(rw)
    try:
        exec(compile(src.split(marker)[0], str(rw / "emit.py"), "exec"), ns)
    finally:
        os.chdir(cwd)
    want = set(ns["REPOS"])
    got = {p.split("=", 1)[0] for p in pairs}
    if want != got:
        print(f"FAIL: generator repos {sorted(want)} != given {sorted(got)}")
        print("GATE: RED")
        return False
    ok = True
    for pair in pairs:
        repo, tree = pair.split("=", 1)
        kw = ns["REPOS"][repo]
        b = kw["bin_"]
        text = ns["gen"](why_subset=ns["WHY"][b], smoke=ns["SMOKE"].get(b, ns["MD_SMOKE"]),
                         test_setup=ns["TEST_SETUP"].get(b, ""), test_os=ns["TEST_OS"].get(b, "[ubuntu-latest]"), **kw)
        same_text = text == (pathlib.Path(tree) / ".github/workflows/release.yml").read_text()
        ok &= same_text
        print(f"release-gen {repo}: {'identical' if same_text else 'DIFFERS'}")
    print("GATE: " + ("GREEN" if ok else "RED"))
    return ok


def self_test(scratch):
    """GREEN on expected() output written back as YAML; RED on each mutation.
    The removed-lines check is exercised separately: a dumped file rewrites
    every line, so here it is switched off and tested on its own."""
    ok = True
    bases = {"mnemonic-toolkit": "origin/master", "mnemonic-engrave": "origin/master", "mnemonic-key": "origin/main"}
    root = pathlib.Path("/scratch/code/shibboleth")
    for repo, base in bases.items():
        with tempfile.TemporaryDirectory(dir=scratch) as d:
            d = pathlib.Path(d)
            subprocess.run(["git", "init", "-q", str(d)], check=True)
            arch = subprocess.run(["git", "-C", str(root / repo), "archive", base, ".github"], capture_output=True, check=True).stdout
            subprocess.run(["tar", "-x", "-C", str(d)], input=arch, check=True)
            subprocess.run(["git", "-C", str(d), "add", "-A"], check=True)
            subprocess.run(["git", "-C", str(d), "-c", "user.email=g@t", "-c", "user.name=g", "commit", "-qm", "b"], check=True)
            wf = d / ".github/workflows"
            for p in wf.glob("*.yml"):
                exp, _ = expected(repo, p.name, load(p.read_text()))
                p.write_text(yaml.safe_dump(exp, sort_keys=False, allow_unicode=True))
            global allowed_removed
            real = allowed_removed
            allowed_removed = lambda r, f, t: sorted(l[1:] for l in git("-C", str(d), "diff", "-U0", "HEAD", "--", f".github/workflows/{f}").splitlines() if l.startswith("-") and not l.startswith("---"))
            try:
                print(f"== self-test {repo}: plan applied (want GREEN)")
                ok &= check_workflows(repo, d, "HEAD")
                green = {p.name: p.read_text() for p in wf.glob("*.yml")}
                f = GUARDED[repo] and next(iter(GUARDED[repo])) or "release.yml"
                muts = {
                    "drop the guard step": lambda y: y["jobs"][next(iter(y["jobs"]))]["steps"].pop(1 if "test" not in y["jobs"] else 1),
                    "one step loses its if": lambda y: [s.pop("if", None) for s in y["jobs"][next(j for j in y["jobs"] if any("if" in s for s in y["jobs"][j]["steps"]))]["steps"][-1:]],
                    "guard keyed on another output": lambda y: [s.__setitem__("if", s["if"].replace("docs_only != 'true'", "docs_only == 'false'")) for j in y["jobs"].values() for s in j.get("steps", []) if s.get("if") == COND][:1],
                    "guard env PUSH_BEFORE wrong": lambda y: [s["env"].__setitem__("PUSH_BEFORE", "${{ github.sha }}") for j in y["jobs"].values() for s in j.get("steps", []) if s.get("id") == "docs"][:1],
                    "selftest dropped": lambda y: [j["steps"].remove(s) for j in y["jobs"].values() for s in list(j.get("steps", [])) if s.get("name") == "Self-test the docs-only guard"][:1],
                }
                for name, mut in muts.items():
                    for p in wf.glob("*.yml"):
                        p.write_text(green[p.name])
                    y = load((wf / f).read_text())
                    before = copy.deepcopy(y)
                    mut(y)
                    if same(before, y):
                        print(f"SELF-TEST BROKEN: mutation {name!r} changed nothing in {repo} {f}")
                        ok = False
                        continue
                    (wf / f).write_text(yaml.safe_dump(y, sort_keys=False, allow_unicode=True))
                    print(f"== self-test {repo} mutation: {name} (want RED)")
                    if check_workflows(repo, d, "HEAD"):
                        print(f"SELF-TEST FAIL: {name!r} passed")
                        ok = False
            finally:
                allowed_removed = real
    print("SELF-TEST: " + ("PASS" if ok else "FAIL"))
    return ok


if __name__ == "__main__":
    a = sys.argv[1:]
    modes = {
        ("workflows", 4): lambda: check_workflows(a[1], a[2], a[3]),
        ("files", 3): lambda: check_files(a[1], a[2]),
        ("paths-check", 3): lambda: check_paths(a[1], a[2]),
        ("self-test", 2): lambda: self_test(a[1]),
    }
    if a[:1] == ["release-gen"] and len(a) >= 3:
        sys.exit(0 if check_release_gen(a[1], a[2:]) else 1)
    fn = modes.get((a[0] if a else "", len(a)))
    if not fn:
        print(__doc__)
        sys.exit(2)
    sys.exit(0 if fn() else 1)
