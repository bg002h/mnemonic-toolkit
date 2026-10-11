#!/usr/bin/env python3
"""ci-hardening gate: proves a workflow diff contains ONLY the planned edits.

Usage: ci-hardening-gate.py <repo-dir> <base-ref> [--toolkit]
Compares every .github/workflows/*.yml at <base-ref> with the working tree.

For each file it (1) asserts the planned edits are present, (2) strips them
from the new YAML, and (3) asserts what remains is identical to the base YAML.
Planned edits (PLAN_ci_hardening_2026-10-11.md):
  T  every job with `runs-on` has job-level `timeout-minutes: 60`
  X  every step whose name starts "Install pandoc + texlive" has
     step-level `timeout-minutes: 15`
  C  every workflow with a pull_request trigger and no base top-level
     `concurrency` gets exactly CONCURRENCY below
  K  (--toolkit only) the cache/PATH steps named in K_STEPS, and
     `CARGO_INSTALL_ROOT: ${{ runner.temp }}/sibling-clis` in the env of
     steps whose `run` contains `cargo install`

NOT COVERED: comments, key order, quoting style and blank lines (YAML parsing
drops them) -- the reviewer reads the textual diff for those. It does not
check the K steps' contents (the reviewer does) and it does not run the
workflows; CI on the PR does that.
"""
import copy
import pathlib
import subprocess
import sys

import yaml

CONCURRENCY = {
    "group": "${{ github.workflow }}-${{ github.event_name == 'pull_request' && github.event.pull_request.number || github.run_id }}",
    "cancel-in-progress": True,
}
K_STEPS = {
    "Restore cached sibling CLIs",
    "Put cached sibling CLIs on PATH",
    "Restore cached GUI-pinned CLI tier",
    "Restore cached GUI-pinned gui-render",
}
INSTALL_ROOT = "${{ runner.temp }}/sibling-clis"


def load(text):
    y = yaml.safe_load(text)
    if True in y:  # PyYAML parses the bare key `on` as boolean True
        y["on"] = y.pop(True)
    return y


def triggers(on):
    if isinstance(on, dict):
        return set(on)
    if isinstance(on, list):
        return set(on)
    return {on}


def main(argv):
    repo, base = pathlib.Path(argv[0]), argv[1]
    is_toolkit = "--toolkit" in argv[2:]
    names = subprocess.run(
        ["git", "-C", repo, "ls-tree", "--name-only", f"{base}:.github/workflows/"],
        capture_output=True, text=True, check=True).stdout.split()
    cur = sorted(p.name for p in (repo / ".github/workflows").iterdir())
    errs = []
    counts = {"T": 0, "X": 0, "C": 0, "K-steps": 0, "K-env": 0}
    if sorted(names) != cur:
        errs.append(f"workflow file set changed: base={sorted(names)} now={cur}")
    for f in names:
        if not f.endswith((".yml", ".yaml")):
            continue
        old = load(subprocess.run(
            ["git", "-C", repo, "show", f"{base}:.github/workflows/{f}"],
            capture_output=True, text=True, check=True).stdout)
        new = load((repo / ".github/workflows" / f).read_text())
        strip = copy.deepcopy(new)
        if "pull_request" in triggers(old.get("on")) and "concurrency" not in old:
            if strip.get("concurrency") != CONCURRENCY:
                errs.append(f"{f}: top-level concurrency missing or not exactly the planned block: {strip.get('concurrency')!r}")
            else:
                counts["C"] += 1
            strip.pop("concurrency", None)
        for jn, job in (strip.get("jobs") or {}).items():
            old_job = (old.get("jobs") or {}).get(jn) or {}
            if "runs-on" in job:
                if job.get("timeout-minutes") != 60:
                    errs.append(f"{f}: job {jn}: timeout-minutes is {job.get('timeout-minutes')!r}, want 60")
                else:
                    counts["T"] += 1
                if "timeout-minutes" not in old_job:
                    job.pop("timeout-minutes", None)
            elif "timeout-minutes" in job:
                errs.append(f"{f}: job {jn} calls a reusable workflow and must not get timeout-minutes")
            if "steps" not in job:
                continue
            kept = []
            for st in job["steps"]:
                name = st.get("name", "")
                if name.startswith("Install pandoc + texlive"):
                    if st.get("timeout-minutes") != 15:
                        errs.append(f"{f}: job {jn} step {name!r}: timeout-minutes is {st.get('timeout-minutes')!r}, want 15")
                    else:
                        counts["X"] += 1
                    st.pop("timeout-minutes", None)
                if name in K_STEPS:
                    if not is_toolkit:
                        errs.append(f"{f}: K step {name!r} outside mnemonic-toolkit")
                    counts["K-steps"] += 1
                    continue
                env = st.get("env")
                if env and "CARGO_INSTALL_ROOT" in env:
                    if not is_toolkit or env["CARGO_INSTALL_ROOT"] != INSTALL_ROOT or "cargo install" not in st.get("run", ""):
                        errs.append(f"{f}: job {jn} step {name!r}: unexpected CARGO_INSTALL_ROOT use")
                    else:
                        counts["K-env"] += 1
                    env.pop("CARGO_INSTALL_ROOT")
                    if not env:
                        st.pop("env")
                kept.append(st)
            job["steps"] = kept
        if strip != old:
            errs.append(f"{f}: changes beyond the planned edits remain after stripping them")
    print(f"{repo}: files={len(names)} " + " ".join(f"{k}={v}" for k, v in counts.items()))
    for e in errs:
        print("FAIL:", e)
    print("GATE: " + ("RED" if errs else "GREEN"))
    return 1 if errs else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
