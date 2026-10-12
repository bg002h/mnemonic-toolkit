#!/usr/bin/env python3
"""docs-skip gate: proves the docs-only skip edits are EXACTLY the plan.

Usage:
  docs-skip-gate.py workflows <repo> <worktree> <base-ref>
  docs-skip-gate.py files <repo> <worktree>
  docs-skip-gate.py paths-check <repo> <worktree>
  docs-skip-gate.py release-gen <engrave-worktree> <repo>=<worktree>...
  docs-skip-gate.py self-test <scratch-dir>

`workflows`: builds the expected parsed YAML of every workflow from the base
(PLAN_ci_docs_skip_2026-10-11.md, D1-D5) and asserts each edited file equals
it, types included, with repeated mapping keys rejected; asserts the counts
equal the plan's table, that `git diff` removed no line except the ones the
plan replaces (toolkit rust.yml trigger lines, the replaced `if:` lines of
combined steps, engrave's detection step and its `steps.detect`
conditions), that `.gitattributes` gained exactly ATTRS and lost nothing,
and that the set of files differing from the base is exactly the planned set.
`files`: ci/docs-only.sh and ci/docs-only.test.sh are byte-identical to this
directory's docs-only.sh / docs-only.test.sh, ci/docs-only-paths.txt is
byte-identical to paths/<repo>.txt, and git resolves `eol=lf` for all three.
`workflows`, `files` and `release-gen` refuse a tree with unstaged changes or
untracked files, so what they certify is exactly what the commit will hold.
`paths-check`: runs the worktree's real ci/docs-only.sh, in pull_request
mode on a scratch repo, once per sample path in SAMPLES[repo], and asserts
each sample is classified as the plan says.
`release-gen`: runs the edited engrave generator in memory and asserts each
output is byte-identical to that repo's release.yml.
`self-test`: proves this gate goes GREEN on the plan and RED on mutations.

NOT COVERED: added comments, blank lines, quoting, key order and YAML 1.1
spellings (YAML parsing drops them; the reviewer reads the diff; the textual
self-test applies the plan line by line to one base, descriptor-mnemonic),
whether CHECK_NAME renders to the job's real check-run name (checked by hand
against the API, see the plan), and the guard's behaviour on real GitHub runners (the PR's own CI runs the guard on
every OS leg, and the post-merge probe in the plan exercises the skip).
"""
import copy
import hashlib
import os
import pathlib
import re
import shutil
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
        "CHECK_NAME": None,  # set per job: check_name()
        "GH_TOKEN": "${{ github.token }}",
    },
    "run": "bash ci/docs-only.sh",
}
PERMS = {"contents": "read", "checks": "read"}
# D5: the workflow files that gain a top-level `permissions:` block
TOP_PERMS = {("descriptor-mnemonic", "ci.yml"), ("mnemonic-toolkit", "rust.yml"), ("mnemonic-key", "ci.yml"),
             ("mnemonic-secret", "rust.yml")}
# the one guarded job whose check-run name is not its `name:` (GitHub appends the matrix values)
CHECK_NAME_OVERRIDE = {("mnemonic-toolkit", "rust.yml", "lib-cross-platform"):
                       "lib cross-platform check (${{ matrix.target }}, ${{ matrix.runs-on }})"}
ATTRS = [
    "# ci/docs-only*: LF on every OS (Git Bash cannot run a CRLF script).",
    "ci/docs-only.sh text eol=lf",
    "ci/docs-only.test.sh text eol=lf",
    "ci/docs-only-paths.txt text eol=lf",
]
CI_FILES = ["ci/docs-only.sh", "ci/docs-only.test.sh", "ci/docs-only-paths.txt"]
EXTRA_CHANGED = {"mnemonic-engrave": ["scripts/release-workflows/emit.py", "scripts/release-workflows/gen.py"]}
SELFTEST = {"name": "Self-test the docs-only guard", "shell": "bash", "run": "bash ci/docs-only.test.sh"}
SELFTEST_LINUX = {"name": "Self-test the docs-only guard", "if": "runner.os == 'Linux'", "shell": "bash",
                  "run": "bash ci/docs-only.test.sh"}

# repo -> {workflow file: {job: "self-test" | "guard" | "skip"}}; "*" = every other job without a job-level `if`
GUARDED = {
    "descriptor-mnemonic": {"ci.yml": {"clippy": "self-test", "*": "guard"}},
    "mnemonic-toolkit": {"rust.yml": {"clippy": "self-test", "g6-invariant": "skip", "*": "guard"}},
    "mnemonic-key": {"ci.yml": {"fmt": "self-test", "*": "guard"}},
    "mnemonic-secret": {"rust.yml": {"clippy": "self-test", "g6-invariant": "skip", "*": "guard"}},
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
    "mnemonic-toolkit": dict(jobs=9, steps=37, combined=3, selftests=2),
    "mnemonic-key": dict(jobs=6, steps=28, combined=2, selftests=2),
    "mnemonic-secret": dict(jobs=11, steps=35, combined=2, selftests=2),
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


def check_name(repo, fname, jn, job):
    o = CHECK_NAME_OVERRIDE.get((repo, fname, jn))
    if o:
        return o
    if "name" not in job:
        raise GateError(f"{fname}:{jn}: guarded job has no name: (plan: STOP)")
    if "strategy" in job and "${{" not in job["name"]:
        raise GateError(f"{fname}:{jn}: matrix job whose name: names no matrix value (plan: STOP)")
    return job["name"]


def guarded_step(name):
    g = copy.deepcopy(GUARD)
    g["env"]["CHECK_NAME"] = name
    return g


def guard_job(job, selftest, counts, where, name, ifs):
    steps = job["steps"]
    if not steps or not str(steps[0].get("uses", "")).startswith("actions/checkout@"):
        raise GateError(f"{where}: first step is not actions/checkout (plan: STOP)")
    for st in steps[1:]:
        if "if" in st:
            ifs.append(st["if"])
            st["if"] = f"({unwrap(st['if'])}) && {COND}"
            counts["combined"] += 1
        else:
            st["if"] = COND
        counts["steps"] += 1
    new = ([copy.deepcopy(selftest)] if selftest else []) + [guarded_step(name)]
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
    steps[i:i + 1] = [copy.deepcopy(SELFTEST), guarded_step(job["name"])]
    counts["jobs"] += 1
    counts["selftests"] += 1


def set_perms(holder, want_before, where):
    if holder.get("permissions") != want_before:
        raise GateError(f"{where}: permissions are {holder.get('permissions')!r}, plan expects {want_before!r} (plan: STOP)")
    holder["permissions"] = dict(PERMS)


def expected(repo, fname, old):
    """Returns (expected YAML, counts, the original `if:` text of every combined step)."""
    e = copy.deepcopy(old)
    counts = dict(jobs=0, steps=0, combined=0, selftests=0)
    ifs = []
    spec = GUARDED[repo].get(fname)
    if spec:
        for jn, job in e["jobs"].items():
            mode = spec.get(jn, spec.get("*"))
            if mode in (None, "skip") or ("if" in job and jn not in spec):
                continue
            if mode == "engrave":
                engrave_test(job, counts)
            else:
                guard_job(job, SELFTEST if mode == "self-test" else None, counts, f"{fname}:{jn}",
                          check_name(repo, fname, jn, job), ifs)
            if repo == "mnemonic-engrave":
                set_perms(job, {"contents": "read"}, f"{fname}:{jn}")
    if (repo, fname) in TOP_PERMS:
        set_perms(e, None, fname)
    if repo in GENERATED and fname == "release.yml":
        t = e["jobs"]["test"]
        guard_job(t, SELFTEST_LINUX, counts, "release.yml:test", check_name(repo, fname, "test", t), ifs)
        set_perms(t, None, "release.yml:test")
    if repo == "mnemonic-toolkit" and fname == "rust.yml":
        if "paths" not in e["on"]["push"]:
            raise GateError("rust.yml: push has no paths filter in base (plan: STOP)")
        del e["on"]["push"]["paths"]
    return e, counts, ifs


def allowed_removed(repo, fname, base_text, ifs=()):
    """The base lines the plan's edits remove: one `if:` line per combined step, plus D2/D3's lines."""
    lines = base_text.splitlines()
    rem, used = [], set()
    for o in ifs:
        forms = {f"if: {o}", f"if: '{o}'", f'if: "{o}"'}
        hit = [i for i, l in enumerate(lines) if l.strip() in forms and i not in used]
        if not hit:
            raise GateError(f"{fname}: no base line holds the combined step's `if: {o}`")
        used.add(hit[0])
        rem.append(lines[hit[0]])
    if repo == "mnemonic-toolkit" and fname == "rust.yml":
        return rem + list(TK_REMOVED)
    if repo == "mnemonic-engrave" and fname == "release.yml":
        lines = base_text.splitlines()
        start = lines.index("      - name: docs-only detection")
        end = lines.index("      - name: Install Rust toolchain") - 1  # the blank line before it stays
        rem += lines[start:end]
        rem += [l for l in lines if l.strip() in ("if: steps.detect.outputs.docs_only != 'true'",
                                                  "if: steps.detect.outputs.docs_only == 'true'")]
        return rem
    return rem


def unstaged(tree):
    """Lines of `git status` that make the worktree differ from the index (unstaged or untracked)."""
    st = git("-C", str(tree), "status", "--porcelain=v1", "--untracked-files=all")
    return [l for l in st.splitlines() if l[:2] == "??" or l[1] != " "]


def refuse_unstaged(tree):
    bad = unstaged(tree)
    for l in bad[:5]:
        print(f"FAIL: {tree}: not staged: {l}")
    return bool(bad)


def diff_lines(tree, base, path, sign):
    diff = git("-C", str(tree), "diff", "--cached", "-U0", base, "--", path)
    head = "+++" if sign == "+" else "---"
    return [l[1:] for l in diff.splitlines() if l.startswith(sign) and not l.startswith(head)]


def check_workflows(repo, tree, base):
    tree = pathlib.Path(tree)
    if refuse_unstaged(tree):
        print("GATE: RED")
        return False
    errs = []
    planned = set(CI_FILES) | {".gitattributes"} | set(EXTRA_CHANGED.get(repo, []))
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
            exp, counts, ifs = expected(repo, f, old)
            allowed[f] = sorted(allowed_removed(repo, f, base_text, ifs))
        except GateError as ex:
            errs.append(f"{f}: {ex}")
            continue
        for k in total:
            total[k] += counts[k]
        if not same(exp, old):
            planned.add(f".github/workflows/{f}")
        if not same(new, exp):
            errs.append(f"{f}: parsed YAML differs from base + planned edits")
    if total != EXPECTED_COUNTS[repo]:
        errs.append(f"counts {total} != plan table {EXPECTED_COUNTS[repo]}")
    for f in allowed:
        removed = sorted(diff_lines(tree, base, f".github/workflows/{f}", "-"))
        if removed != allowed[f]:
            extra = sorted(set(removed) - set(allowed[f]))
            missing = sorted(set(allowed[f]) - set(removed))
            errs.append(f"{f}: removed lines differ from the plan (unplanned: {extra[:3]}, not removed: {missing[:3]})")
    if diff_lines(tree, base, ".gitattributes", "-"):
        errs.append(".gitattributes: lines removed (the plan only appends)")
    if diff_lines(tree, base, ".gitattributes", "+") != ATTRS:
        errs.append(".gitattributes: added lines are not exactly the plan's four")
    changed = set(git("-C", str(tree), "diff", "--cached", "--name-only", base).split("\n")) - {""}
    if changed != planned:
        errs.append(f"changed files differ from the plan (unplanned: {sorted(changed - planned)}, "
                    f"not changed: {sorted(planned - changed)})")
    print(f"{repo}: files={len(names)} " + " ".join(f"{k}={v}" for k, v in total.items()))
    print(f"{repo}: changed files ({len(changed)}): {' '.join(sorted(changed))}")
    for e in errs:
        print("FAIL:", e)
    print("GATE: " + ("RED" if errs else "GREEN"))
    return not errs


def sha(p):
    return hashlib.sha256(pathlib.Path(p).read_bytes()).hexdigest()


def check_files(repo, tree):
    tree = pathlib.Path(tree)
    if refuse_unstaged(tree):
        print("GATE: RED")
        return False
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
    for line in git("-C", str(tree), "check-attr", "--cached", "eol", "--", *CI_FILES).splitlines():
        good = line.endswith(": eol: lf")
        ok &= good
        print(f"files {repo}: {line}{'' if good else '  (want eol: lf)'}")
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
            stub = d / "gh"
            stub.write_text("#!/usr/bin/env bash\necho completed/success\n")
            stub.chmod(0o755)
            e2 = dict(env, GITHUB_OUTPUT=str(out), DOCS_ONLY_PATHS=str(rules), GITHUB_EVENT_NAME="pull_request",
                      DEFAULT_BRANCH="main", GITHUB_BASE_REF="main", PR_BASE_SHA=base, CHECK_NAME="gate",
                      DOCS_ONLY_GH=str(stub), GITHUB_REPOSITORY="o/r")
            subprocess.run(["bash", str(guard)], cwd=w, env=e2, capture_output=True, text=True, check=True)
            got = out.read_text().strip()
            good = got == f"docs_only={'true' if want else 'false'}"
            ok &= good
            print(f"paths-check {repo}: {path} -> {got}{'' if good else '  WANT docs_only=' + str(want).lower()}")
    print("GATE: " + ("GREEN" if ok else "RED"))
    return ok


def check_release_gen(engrave_tree, pairs):
    if any(refuse_unstaged(t) for t in [engrave_tree] + [p.split("=", 1)[1] for p in pairs]):
        print("GATE: RED")
        return False
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


GUARD_TEXT = """\
      - name: Docs-only change? (skips the steps below when every changed file is documentation)
        id: docs
        shell: bash
        env:
          DEFAULT_BRANCH: ${{ github.event.repository.default_branch }}
          PR_BASE_SHA: ${{ github.event.pull_request.base.sha }}
          PUSH_BEFORE: ${{ github.event.before }}
          CHECK_NAME: %s
          GH_TOKEN: ${{ github.token }}
        run: bash ci/docs-only.sh"""
SELFTEST_TEXT = """\
      - name: Self-test the docs-only guard
        shell: bash
        run: bash ci/docs-only.test.sh"""
SELFTEST_LINUX_TEXT = """\
      - name: Self-test the docs-only guard
        if: runner.os == 'Linux'
        shell: bash
        run: bash ci/docs-only.test.sh"""
PERMS_TEXT = ["  contents: read", "  checks: read"]


def apply_text(text, jobs, top_perms):
    """The plan's D1/D4(release.yml)/D5 edits applied line by line, as an implementer would.
    jobs: {job id: (self-test text or None, job-level permissions?)}."""
    L = text.split("\n")
    ind = lambda l: len(l) - len(l.lstrip(" "))
    j0 = L.index("jobs:")
    starts = [i for i in range(j0 + 1, len(L)) if re.match(r"^  [A-Za-z0-9_-]+:\s*$", L[i])] + [len(L)]
    out = L[:j0] + (["permissions:"] + PERMS_TEXT + [""] if top_perms else []) + ["jobs:"]
    out += L[j0 + 1:starts[0]]
    for a, b in zip(starts, starts[1:]):
        blk = L[a:b]
        jn = blk[0].strip().rstrip(":")
        if jn not in jobs:
            out += blk
            continue
        selftest, jperms = jobs[jn]
        name = next(l.split("name: ", 1)[1] for l in blk if l.startswith("    name: "))
        si = blk.index("    steps:")
        st = [i for i in range(si + 1, len(blk)) if blk[i].startswith("      - ")] + [len(blk)]
        after, repl = {}, {}
        for k, (s0, s1) in enumerate(zip(st, st[1:])):
            content = [i for i in range(s0 + 1, s1) if blk[i].strip() and ind(blk[i]) >= 8]
            if k == 0:
                after[content[-1] if content else s0] = ((selftest + "\n") if selftest else "") + GUARD_TEXT % name
                continue
            iff = [i for i in content if ind(blk[i]) == 8 and blk[i].lstrip().startswith("if: ")]
            if iff:
                repl[iff[0]] = f"        if: ({unwrap(blk[iff[0]].split('if: ', 1)[1])}) && {COND}"
            else:
                after[s0] = f"        if: {COND}"
        for i, l in enumerate(blk):
            if jperms and l == "    strategy:":
                out += ["    permissions:"] + ["    " + x for x in PERMS_TEXT]
            out.append(repl.get(i, l))
            if i in after:
                out += after[i].split("\n")
    return "\n".join(out)


def _scratch_repo(repo, base, d):
    root = pathlib.Path("/scratch/code/shibboleth")
    subprocess.run(["git", "init", "-q", str(d)], check=True)
    arch = subprocess.run(["git", "-C", str(root / repo), "archive", base, ".github"], capture_output=True, check=True).stdout
    subprocess.run(["tar", "-x", "-C", str(d)], input=arch, check=True)
    ga = subprocess.run(["git", "-C", str(root / repo), "show", f"{base}:.gitattributes"], capture_output=True)
    if ga.returncode == 0:
        (d / ".gitattributes").write_bytes(ga.stdout)
    _commit(d, "base")
    (d / "ci").mkdir()
    for src, dst in ((HERE / "docs-only.sh", "ci/docs-only.sh"), (HERE / "docs-only.test.sh", "ci/docs-only.test.sh"),
                     (HERE / "paths" / f"{repo}.txt", "ci/docs-only-paths.txt")):
        shutil.copyfile(src, d / dst)
        (d / dst).chmod(0o755 if dst.endswith(".sh") else 0o644)
    ga = d / ".gitattributes"
    ga.write_text((ga.read_text() if ga.exists() else "") + "\n".join(ATTRS) + "\n")
    for f in EXTRA_CHANGED.get(repo, []):
        (d / f).parent.mkdir(parents=True, exist_ok=True)
        (d / f).write_text("stand-in for the plan's generator edit\n")


def _commit(d, msg):
    subprocess.run(["git", "-C", str(d), "add", "-A"], check=True)
    if msg:
        subprocess.run(["git", "-C", str(d), "-c", "user.email=g@t", "-c", "user.name=g", "commit", "-qm", msg], check=True)
    else:
        subprocess.run(["git", "-C", str(d), "update-index", "--chmod=+x", "ci/docs-only.sh", "ci/docs-only.test.sh"], check=True)


def _want(label, got, want):
    print(f"== self-test {label} (want {'GREEN' if want else 'RED'})")
    if got != want:
        print(f"SELF-TEST FAIL: {label}")
    return got == want


def self_test(scratch):
    """GREEN on the plan, RED on each mutation, for three bases dumped from expected()
    (the removed-lines check switched off, since a dump rewrites every line) and one
    base, descriptor-mnemonic, edited TEXTUALLY with apply_text(), where the real
    removed-lines check runs. It proves the checks fire; it cannot prove expected()
    encodes the plan, except where the textual applier and expected() agree."""
    global allowed_removed
    ok = True
    file_muts = {
        "a file outside the plan changed": lambda d: (d / "Cargo.toml").write_text("x\n"),
        ".gitattributes lost a rule": lambda d: (d / ".gitattributes").write_text(
            (d / ".gitattributes").read_text().replace(ATTRS[-1] + "\n", "")),
        "the guard script differs": lambda d: (d / "ci/docs-only.sh").write_text("#!/bin/sh\n"),
    }
    bases = {"mnemonic-toolkit": "origin/master", "mnemonic-engrave": "origin/master", "mnemonic-key": "origin/main"}
    for repo, base in bases.items():
        with tempfile.TemporaryDirectory(dir=scratch) as d:
            d = pathlib.Path(d)
            _scratch_repo(repo, base, d)
            wf = d / ".github/workflows"
            for p in wf.glob("*.yml"):
                old = load(p.read_text())
                exp, _, _ = expected(repo, p.name, old)
                if not same(exp, old):
                    p.write_text(yaml.safe_dump(exp, sort_keys=False, allow_unicode=True))
            _commit(d, None)
            real = allowed_removed
            allowed_removed = lambda r, f, t, i=(): sorted(diff_lines(d, "HEAD", f".github/workflows/{f}", "-"))
            try:
                ok &= _want(f"{repo}: plan applied", check_workflows(repo, d, "HEAD") and check_files(repo, d), True)
                green = {p: p.read_bytes() for p in d.rglob("*") if p.is_file() and ".git" not in p.parts}
                f = GUARDED[repo] and next(iter(GUARDED[repo])) or "release.yml"
                muts = {
                    "drop the guard step": lambda y: [j["steps"].remove(s) for j in y["jobs"].values() for s in list(j.get("steps", [])) if s.get("id") == "docs"][:1],
                    "one step loses its if": lambda y: [s.pop("if", None) for s in y["jobs"][next(j for j in y["jobs"] if any("if" in s for s in y["jobs"][j]["steps"]))]["steps"][-1:]],
                    "guard keyed on another output": lambda y: [s.__setitem__("if", s["if"].replace("docs_only != 'true'", "docs_only == 'false'")) for j in y["jobs"].values() for s in j.get("steps", []) if s.get("if") == COND][:1],
                    "guard env PUSH_BEFORE wrong": lambda y: [s["env"].__setitem__("PUSH_BEFORE", "${{ github.sha }}") for j in y["jobs"].values() for s in j.get("steps", []) if s.get("id") == "docs"][:1],
                    "guard CHECK_NAME wrong": lambda y: [s["env"].__setitem__("CHECK_NAME", "build") for j in y["jobs"].values() for s in j.get("steps", []) if s.get("id") == "docs"][:1],
                    "checks: read dropped": lambda y: [p.pop("checks") for p in [y.get("permissions")] + [j.get("permissions") for j in y["jobs"].values()] if isinstance(p, dict) and "checks" in p][:1],
                    "selftest dropped": lambda y: [j["steps"].remove(s) for j in y["jobs"].values() for s in list(j.get("steps", [])) if s.get("name") == "Self-test the docs-only guard"][:1],
                }
                for name, mut in list(muts.items()) + list(file_muts.items()):
                    for p, b in green.items():
                        p.write_bytes(b)
                    if name in muts:
                        y = load((wf / f).read_text())
                        before = copy.deepcopy(y)
                        mut(y)
                        if same(before, y):
                            print(f"SELF-TEST BROKEN: mutation {name!r} changed nothing in {repo} {f}")
                            ok = False
                            continue
                        (wf / f).write_text(yaml.safe_dump(y, sort_keys=False, allow_unicode=True))
                    else:
                        mut(d)
                    _commit(d, None)
                    ok &= _want(f"{repo} mutation: {name}", check_workflows(repo, d, "HEAD") and check_files(repo, d), False)
                for p, b in green.items():
                    p.write_bytes(b)
                _commit(d, None)
                (wf / f).write_text((wf / f).read_text() + "\n")
                ok &= _want(f"{repo} mutation: an unstaged edit", check_workflows(repo, d, "HEAD"), False)
            finally:
                allowed_removed = real
    # textual: descriptor-mnemonic, real removed-lines check
    repo = "descriptor-mnemonic"
    with tempfile.TemporaryDirectory(dir=scratch) as d:
        d = pathlib.Path(d)
        _scratch_repo(repo, "origin/main", d)
        wf = d / ".github/workflows"
        ci = wf / "ci.yml"
        ci.write_text(apply_text(ci.read_text(), {j: (SELFTEST_TEXT if j == "clippy" else None, False)
                                                  for j in load(ci.read_text())["jobs"]}, True))
        rel = wf / "release.yml"
        rel.write_text(apply_text(rel.read_text(), {"test": (SELFTEST_LINUX_TEXT, True)}, False))
        _commit(d, None)
        ok &= _want(f"{repo}: plan applied textually", check_workflows(repo, d, "HEAD") and check_files(repo, d), True)
        good = ci.read_text()
        ci.write_text(good.replace("    # Required contexts here: `cargo test (ubuntu-latest)` and `cargo clippy`.\n", "", 1))
        if ci.read_text() == good:
            print("SELF-TEST BROKEN: textual mutation found no comment line")
            ok = False
        _commit(d, None)
        ok &= _want(f"{repo} textual mutation: an unrelated comment line deleted", check_workflows(repo, d, "HEAD"), False)
        dup = f"        if: (matrix.musl_native) && {COND}\n"
        if dup not in good:
            print("SELF-TEST BROKEN: textual mutation found no combined if line")
            ok = False
        ci.write_text(good.replace(dup, dup + "        if: true\n", 1))
        _commit(d, None)
        ok &= _want(f"{repo} textual mutation: a repeated if key", check_workflows(repo, d, "HEAD"), False)
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
