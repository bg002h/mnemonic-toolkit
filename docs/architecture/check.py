#!/usr/bin/env python3
"""Check the architecture snapshot and its Markdown targets; Python 3.11+.

Read-only, no network/build/runtime checks. Markdown support covers inline links
used by these documents, local heading fragments, and GitHub blob/tree links.
Remote branch links are checked against local deliverables, not by HTTP.
Architectural semantics and instruction loading still need independent review.
"""
import argparse
import hashlib
import json
from pathlib import Path
import re
import subprocess
import sys
import tomllib
from urllib.parse import unquote, urlsplit

REPOSITORIES = {
    "mnemonic-secret", "mnemonic-key", "descriptor-mnemonic",
    "mnemonic-toolkit", "mnemonic-transaction", "mnemonic-engrave", "mnemonic-gui",
}
PACKAGES = {
    "ms-codec", "mk-codec", "md-codec", "mt-codec", "wc-codec",
    "mnemonic-toolkit", "mnemonic-engrave", "mnemonic-io-lib", "miniscript",
}
REQUIRED_DOCS = {
    "README.md", "SOURCE_MAP.md", "OPEN_QUESTIONS.md",
    "WORKFLOW.md", "WORKSPACE_AGENTS.md",
}
OPTIONAL_SNAPSHOT_FILES = {
    "rust-toolchain.toml", "pinned-upstream.toml", ".gitmodules", "preview/go.mod",
}
LINK = re.compile(r"!?\[[^\]\n]*\]\(([^\s)]+)\)")
errors = []
counts = {"repositories": 0, "hashes": 0, "markdown": 0, "links": 0}


def require(condition, message):
    if not condition:
        errors.append(message)


def git(repo, *args):
    result = subprocess.run(
        ["git", "-C", str(repo), *args], capture_output=True, check=False
    )
    if result.returncode:
        raise ValueError(result.stderr.decode().strip())
    return result.stdout


def snapshot(repo, rev, path):
    return git(repo, "show", f"{rev}:{path}")


def present(repo, rev, path):
    result = subprocess.run(
        ["git", "-C", str(repo), "cat-file", "-e", f"{rev}:{path}"],
        capture_output=True, check=False,
    )
    return result.returncode == 0


def headings(body):
    # GitHub-style anchors for the simple headings used in this guide.
    return {
        re.sub(r"[^\w\- ]", "", line.lstrip("# ").strip().lower()).replace(" ", "-")
        for line in body.splitlines() if line.startswith("#")
    }


def check_inventory(row, workspace):
    name, rev = row["repository"], row["source_commit"]
    repo = workspace / name
    require(bool(re.fullmatch(r"[0-9a-f]{40}", rev)), f"{name}: invalid source SHA")
    git(repo, "cat-file", "-e", rev + "^{commit}")
    root = tomllib.loads(snapshot(repo, rev, "Cargo.toml").decode())
    manifests = (
        ["Cargo.toml"] if "package" in root
        else [m + "/Cargo.toml" for m in root["workspace"]["members"]]
    )
    expected_packages = []
    for path in manifests:
        manifest = tomllib.loads(snapshot(repo, rev, path).decode())
        package = manifest["package"]
        version = package["version"]
        if isinstance(version, dict):
            version = root["workspace"]["package"]["version"]
        expected_packages.append({
            "name": package["name"], "version": version, "manifest": path,
            "binaries": [item["name"] for item in manifest.get("bin", [])],
        })
    require(row["packages"] == expected_packages, f"{name}: package manifest mismatch")
    require(row["patches"] == root.get("patch", {}), f"{name}: patch mismatch")
    expected_files = set(manifests) | {"Cargo.toml", "Cargo.lock"}
    expected_files |= {p for p in OPTIONAL_SNAPSHOT_FILES if present(repo, rev, p)}
    require(set(row["file_sha256"]) == expected_files, f"{name}: snapshot file set mismatch")
    for path, digest in row["file_sha256"].items():
        require(hashlib.sha256(snapshot(repo, rev, path)).hexdigest() == digest,
                f"{name}/{path}: snapshot hash mismatch")
        current = repo / path
        require(current.is_file() and hashlib.sha256(current.read_bytes()).hexdigest() == digest,
                f"{name}/{path}: worktree differs from documented baseline")
        counts["hashes"] += 1
    lock = tomllib.loads(snapshot(repo, rev, "Cargo.lock").decode())
    selected = [
        {"name": p["name"], "version": p["version"], "source": p.get("source", "workspace")}
        for p in lock["package"] if p["name"] in PACKAGES
    ]
    require(row["resolved_packages"] == selected, f"{name}: resolved dependency mismatch")
    require(row["url"] == f"https://github.com/bg002h/{name}", f"{name}: repository URL mismatch")
    counts["repositories"] += 1


def check_link(source, target, workspace, inventory):
    parsed = urlsplit(unquote(target))
    if parsed.scheme:
        if parsed.netloc != "github.com":
            raise ValueError(f"external URL needs separate verification: {target}")
        match = re.fullmatch(r"/bg002h/([^/]+)/(blob|tree)/(.+)", parsed.path)
        if not match:
            raise ValueError(f"unsupported GitHub target: {target}")
        name, kind, rest = match.groups()
        branch = inventory["documentation_branch"]
        if rest.startswith(branch + "/"):
            require(name in REPOSITORIES, f"{source}: unknown documentation repository {name}")
            path = workspace / name / rest[len(branch) + 1:]
            require(path.exists(), f"{source}: missing documentation branch target {target}")
            if parsed.fragment and path.is_file():
                require(parsed.fragment in headings(path.read_text()),
                        f"{source}: missing heading {target}")
            return
        parts = rest.split("/", 1)
        rev, path = parts[0], parts[1] if len(parts) == 2 else ""
        require(bool(re.fullmatch(r"[0-9a-f]{40}", rev)), f"{source}: source link is not immutable: {target}")
        rows = inventory["repositories"] + inventory["boundaries"]
        record = next((r for r in rows if r["repository"] == name), None)
        require(record is not None and record.get("source_commit") == rev,
                f"{source}: source link outside recorded baseline: {target}")
        repo = workspace / name
        if not (repo / ".git").exists():
            repo = workspace.parent / name  # boundary-only original checkouts
        obj = f"{rev}:{path}" if path else rev + "^{tree}"
        actual_kind = git(repo, "cat-file", "-t", obj).decode().strip()
        require(actual_kind == kind, f"{source}: {kind} link points at {actual_kind}: {target}")
        if parsed.fragment:
            blob = git(repo, "show", obj).decode()
            line = re.fullmatch(r"L([0-9]+)(?:-L([0-9]+))?", parsed.fragment)
            if line:
                require(0 < int(line[1]) <= int(line[2] or line[1]) <= len(blob.splitlines()),
                        f"{source}: invalid source line fragment: {target}")
            else:
                require(parsed.fragment in headings(blob), f"{source}: missing source heading: {target}")
    else:
        path = source.parent / parsed.path if parsed.path else source
        require(path.exists(), f"{source}: missing local target {target}")
        if parsed.fragment and path.is_file():
            require(parsed.fragment in headings(path.read_text()), f"{source}: missing local heading {target}")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--workspace", type=Path, default=Path(__file__).resolve().parents[3])
    parser.add_argument("--inventory", type=Path, default=Path(__file__).with_name("inventory.json"))
    parser.add_argument("--extra-markdown", type=Path, action="append", default=[])
    args = parser.parse_args()
    workspace = args.workspace.resolve()
    inventory = json.loads(args.inventory.read_text())
    require(inventory["schema_version"] == 1, "unsupported inventory schema")
    rows = inventory["repositories"]
    require(len(rows) == len(REPOSITORIES) and {r["repository"] for r in rows} == REPOSITORIES,
            "inventory must contain exactly the seven scoped repositories")
    require(inventory["documentation_branch"] == "docs/constellation-architecture",
            "unexpected documentation branch; update checker when integration changes navigation")
    for row in rows:
        try:
            check_inventory(row, workspace)
        except (ValueError, OSError, KeyError) as exc:
            errors.append(f"{row.get('repository')}: {exc}")
    arch = workspace / "mnemonic-toolkit/docs/architecture"
    for name in REQUIRED_DOCS:
        require((arch / name).is_file(), f"missing required guide: {name}")
    docs = list(arch.glob("*.md")) + [workspace / n / "AGENTS.md" for n in sorted(REPOSITORIES)]
    docs += [workspace / "mnemonic-toolkit/design/PLAN_constellation_architecture_guide.md"]
    docs += args.extra_markdown
    for source in docs:
        try:
            body = source.read_text()
            if source.name == "AGENTS.md":
                require(len(body.encode()) < 8192, f"{source}: entry instructions exceed 8 KiB")
                require("CLAUDE.md" in body and "WORKFLOW.md" in body, f"{source}: missing instruction route")
            for target in LINK.findall(body):
                try:
                    check_link(source, target, workspace, inventory)
                except (ValueError, OSError) as exc:
                    errors.append(f"{source}: {exc}")
                counts["links"] += 1
            require(not re.search(r"[ \t]+$", body, re.M), f"{source}: trailing whitespace")
            require(body.endswith("\n"), f"{source}: missing final newline")
            counts["markdown"] += 1
        except OSError as exc:
            errors.append(str(exc))
    for message in errors:
        print("FAIL:", message, file=sys.stderr)
    print(("FAIL" if errors else "PASS") + ": " + json.dumps(counts, sort_keys=True))
    print("Coverage: inventory Git objects, current manifest/lock drift, selected pins, inline Markdown targets.")
    print("Excluded: remote HTTP/branch existence, full Markdown/Mermaid parsing, runtime tests and architectural semantics.")
    return bool(errors)


if __name__ == "__main__":
    try:
        sys.exit(main())
    except (ValueError, OSError, KeyError, TypeError) as exc:
        print(f"FAIL: invalid inventory or invocation: {exc}", file=sys.stderr)
        sys.exit(1)
