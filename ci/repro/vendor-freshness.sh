#!/usr/bin/env bash
# vendor/ freshness and CONTENT gate. SHARED FILE: byte-identical in
# mnemonic-toolkit, mnemonic-refugium, mnemonic-key, mnemonic-secret and
# descriptor-mnemonic, pinned by its sha256 in each of them. Edit it in all
# five at once. Spec: mnemonic-toolkit design/SPEC_vendor_freshness_shared.md.
#
# Usage, from anywhere in the repo: bash ci/repro/vendor-freshness.sh
# No arguments. VF_SOURCES names the data file (default
# ci/repro/vendor-git-sources.txt; a relative path is taken from the repo root).
# Exit 0 = every check passed; 1 = a refusal or any check red.
#
# Four checks, in ascending cost. Any one red fails the gate.
#   (1) RESOLUTION: the committed vendor/ satisfies Cargo.lock under the release
#       build's --offline --locked source replacement, run under an EMPTY,
#       throwaway CARGO_HOME (F-675: a warm ~/.cargo/git hid a missing stanza).
#   (2) INTEGRITY: every vendored file matches the sha256 its crate's
#       .cargo-checksum.json records.
#   (3) REGISTRY PROVENANCE: every crates.io crate's `package` digest equals the
#       `checksum` Cargo.lock pins, and the crates with no such anchor are
#       exactly the data file's rows.
#   (4) GIT-SOURCE PROVENANCE: each data-file row's vendored tree is re-derived
#       from its upstream repo at the rev Cargo.lock pins: the same file set as
#       its .cargo-checksum.json, every file but Cargo.toml byte-identical to
#       upstream, Cargo.toml's package name and version equal to upstream's,
#       and the manifest as cargo reads it (`cargo metadata`: features, normal
#       and build dependencies, lib and build-script targets, edition, links,
#       [package.metadata] and the CARGO_PKG_* fields) equal to upstream's.
#       Needs network access to each row's host; (1)-(3) are offline.
#
# WHY (2)-(4): F-354. Check (1) was green for two months over a vendor tree
# vendored from the WRONG REV; it compares resolution, never bytes, and that
# tree was self-consistent, so (2) passes it too. Only (4) sees it.
#
# The data file: one row per vendored git source,
#   <vendor dir>|<url exactly as in Cargo.lock>|<crate dir in upstream, or .>
# `#` lines and blank lines are ignored. A git source in Cargo.lock with no
# row, and a row whose url Cargo.lock does not pin, both fail closed.
#
# NOT covered: bit-for-bit reproducibility (repro-drift, the release repro
# gate); files on disk that a registry crate's .cargo-checksum.json does not
# list (check (4) catches them for git sources only); in a git source's
# Cargo.toml, what a dependent's build never reads (dev-dependencies, example,
# test, bench and bin targets, [lints], [profile], keywords and the like);
# upstream files cargo vendor left out under include/exclude (a lib target or
# build script they would add shows in the manifest comparison).
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$REPO_ROOT"

python3 -I - "$@" <<'PY'
import glob, hashlib, json, os, re, shutil, subprocess, sys, tempfile

def err(msg):
    print(f"::error::vendor-freshness: {msg}", file=sys.stderr)

def refuse(msg):
    err(msg)
    sys.exit(1)

if len(sys.argv) > 1:
    refuse(f"takes no arguments; got: {' '.join(sys.argv[1:])}")

SOURCES = os.environ.get("VF_SOURCES") or "ci/repro/vendor-git-sources.txt"
if not os.path.isfile(SOURCES):
    refuse(f"data file {SOURCES} not found. Every repo keeps one; it holds only "
           f"comments when Cargo.lock has no git source.")

# ── the data file ───────────────────────────────────────────────────────────
ROW = re.compile(r"^([A-Za-z0-9_.+-]+)\|(https://[^|\s]+)\|(\.|[A-Za-z0-9_.+-]+(?:/[A-Za-z0-9_.+-]+)*)$")
rows = []          # (lineno, dir, url, sub)
seen = {}
for n, line in enumerate(open(SOURCES, encoding="utf-8").read().splitlines(), 1):
    if line == "" or line.startswith("#"):
        continue
    m = ROW.match(line)
    if not m or ".." in m.group(3).split("/") or m.group(1) in (".", ".."):
        refuse(f"{SOURCES}:{n}: malformed row '{line}'. A row is "
               f"<vendor dir>|<url as in Cargo.lock>|<crate dir in upstream>: three "
               f"fields, no spaces, an https:// url, and a crate dir that is '.' or a "
               f"relative path without '..'.")
    d, url, sub = m.groups()
    if d in seen:
        refuse(f"{SOURCES}:{n}: vendor dir '{d}' already has a row (line {seen[d]}).")
    seen[d] = n
    rows.append((n, d, url, sub))

# ── the lock's git sources ──────────────────────────────────────────────────
lock = open("Cargo.lock", encoding="utf-8").read()
GIT = re.compile(r'^source = "git\+([^?"#]+)\?rev=([0-9a-f]{40})#([0-9a-f]{40})"$')
lock_rev = {}      # url -> rev
for line in lock.splitlines():
    if not line.startswith('source = "git+'):
        continue
    m = GIT.match(line)
    if not m or m.group(2) != m.group(3):
        refuse(f"Cargo.lock has a git source not pinned by a 40-hex rev: "
               f"{line[len('source = '):]}. Pin it with rev = \"<40-hex>\" or take "
               f"it from crates.io. Failing closed.")
    url, rev = m.group(1), m.group(2)
    if url in lock_rev and lock_rev[url] != rev:
        a, b = sorted((lock_rev[url], rev))
        refuse(f"Cargo.lock pins {url} at more than one rev ({a}, {b}). This gate "
               f"re-derives one rev per url. Failing closed.")
    lock_rev[url] = rev

row_urls = {url for _, _, url, _ in rows}
for url in sorted(lock_rev):
    if url not in row_urls:
        refuse(f"Cargo.lock pins git source {url}?rev={lock_rev[url]}, which has no "
               f"row in {SOURCES}. A git source has no published tarball digest, so "
               f"only check (4) can verify its vendored bytes. Add a row "
               f"(<vendor dir>|{url}|<crate dir in upstream>) or drop the source.")
for n, d, url, _ in rows:
    if url not in lock_rev:
        refuse(f"{SOURCES}:{n}: row '{d}' names {url}, which Cargo.lock does not pin "
               f"from git. Drop the row, so the gate's exemptions match the lock.")

for tool in ("cargo", "git"):
    if shutil.which(tool) is None:
        refuse(f"{tool} not found on PATH.")

os.makedirs("target", exist_ok=True)
WORK = tempfile.mkdtemp(prefix="vendor-freshness.", dir="target")
try:
    # ── (1) RESOLUTION ──────────────────────────────────────────────────────
    cfg = ["--config", 'source.crates-io.replace-with="vendored-sources"']
    for url, rev in sorted(lock_rev.items()):
        key = f'source."git+{url}?rev={rev}"'
        cfg += ["--config", f'{key}.git="{url}"',
                "--config", f'{key}.rev="{rev}"',
                "--config", f'{key}.replace-with="vendored-sources"']
    cfg += ["--config", 'source.vendored-sources.directory="vendor"']
    pins = " ".join(f"{d}={lock_rev[url]}" for _, d, url, _ in sorted(rows, key=lambda r: r[1])) or "none"
    print(f"vendor-freshness: (1/4) resolving Cargo.lock against committed vendor/ "
          f"(offline, locked, empty CARGO_HOME; git sources: {pins}) ...", flush=True)
    home = os.path.join(WORK, "cargo-home")
    os.mkdir(home)
    env = dict(os.environ, CARGO_HOME=os.path.abspath(home))
    r = subprocess.run(["cargo", "metadata", "--format-version", "1", "--locked", "--offline", *cfg],
                       env=env, stdout=subprocess.PIPE)
    if r.returncode != 0:
        refuse("vendor/ is out of sync with Cargo.lock: the --offline --locked release "
               "build cannot resolve a dependency from the committed vendor/ tree. Run "
               "'cargo vendor --locked vendor/' and commit the result. This is the "
               "toolkit v0.74.0 release failure class, caught at PR time.")
    print("vendor-freshness: (1/4) OK -- vendor/ satisfies Cargo.lock.", flush=True)
    resolved = r.stdout  # the package graph, read again by check (4)

    # ── (2) INTEGRITY ───────────────────────────────────────────────────────
    def sha256_file(path):
        h = hashlib.sha256()
        with open(path, "rb") as fh:
            for chunk in iter(lambda: fh.read(1 << 20), b""):
                h.update(chunk)
        return h.hexdigest()

    errors = []
    manifests = sorted(glob.glob("vendor/*/.cargo-checksum.json"))
    if not manifests:
        refuse("no vendor/*/.cargo-checksum.json found: is the vendor/ tree "
               "committed? Failing closed.")
    for root, dirs, files in os.walk("vendor"):
        for n in dirs + files:
            if os.path.islink(os.path.join(root, n)):
                errors.append(f"{os.path.join(root, n)}: SYMLINK in vendor/ (cargo vendor "
                              f"writes none; the checks above and below walk real files only)")
    loaded = {}
    n_files = 0
    for cj in manifests:
        crate_dir = os.path.dirname(cj)
        try:
            loaded[cj] = json.load(open(cj))
            files = loaded[cj]["files"]
        except Exception as exc:
            errors.append(f"{cj}: unreadable checksum manifest ({exc})")
            continue
        for rel, expected in files.items():
            path = os.path.join(crate_dir, rel)
            n_files += 1
            if not os.path.isfile(path):
                errors.append(f"{path}: MISSING (recorded in {cj})")
                continue
            actual = sha256_file(path)
            if actual != expected:
                errors.append(f"{path}: CONTENT MISMATCH\n"
                              f"      recorded {expected}\n"
                              f"      on disk  {actual}")

    # ── (3) REGISTRY PROVENANCE ─────────────────────────────────────────────
    lock_ck = {}
    for block in lock.split("[[package]]")[1:]:
        name = re.search(r'^name = "(.*)"$', block, re.M)
        ver = re.search(r'^version = "(.*)"$', block, re.M)
        ck = re.search(r'^checksum = "(.*)"$', block, re.M)
        if name and ver:
            lock_ck[(name.group(1), ver.group(1))] = ck.group(1) if ck else None
    row_dirs = {d for _, d, _, _ in rows}
    anchored = 0
    unanchored = []
    for cj in manifests:
        if cj not in loaded:
            continue
        crate_dir = os.path.dirname(cj)
        toml = open(os.path.join(crate_dir, "Cargo.toml"), encoding="utf-8").read()
        name = re.search(r'^name = "(.*)"$', toml, re.M)
        ver = re.search(r'^version = "(.*)"$', toml, re.M)
        if not (name and ver):
            errors.append(f"{crate_dir}/Cargo.toml: could not read name/version")
            continue
        key = (name.group(1), ver.group(1))
        pkg = loaded[cj].get("package")
        want = lock_ck.get(key)
        if pkg is None or want is None:
            unanchored.append(os.path.basename(crate_dir))
        elif pkg != want:
            errors.append(f"{crate_dir}: PROVENANCE MISMATCH for {key[0]} {key[1]}\n"
                          f"      .cargo-checksum.json package {pkg}\n"
                          f"      Cargo.lock checksum          {want}")
        else:
            anchored += 1
    unexpected = sorted(d for d in unanchored if d not in row_dirs)
    if unexpected:
        errors.append("vendored crate(s) with NO registry anchor and no row in "
                      f"{SOURCES}: {', '.join(unexpected)}\n"
                      "      A git or path source cannot be checked against Cargo.lock (no\n"
                      "      published tarball digest). Add a row for it, or drop it.")
    if errors:
        err(f"{len(errors)} content defect(s) in vendor/:")
        for e in errors:
            print(f"  - {e}", file=sys.stderr)
        sys.exit(1)
    print(f"vendor-freshness: (2/4) OK -- {n_files} files across {len(manifests)} crates "
          f"match their recorded sha256.")
    print(f"vendor-freshness: (3/4) OK -- {anchored} crates anchored to Cargo.lock checksums; "
          f"{len(unanchored)} git source(s) exempt by a row in {SOURCES} "
          f"({', '.join(sorted(unanchored)) or 'none'}).", flush=True)

    # ── (4) GIT-SOURCE PROVENANCE ───────────────────────────────────────────
    if not rows:
        print("vendor-freshness: (4/4) OK -- Cargo.lock has no git source; nothing to re-derive.")
        sys.exit(0)
    print(f"vendor-freshness: (4/4) re-deriving {len(rows)} git source(s) from upstream at "
          f"the Cargo.lock rev ...", flush=True)
    # The caller's GIT_* variables (GIT_DIR from a hook, say) never reach git.
    genv = {k: v for k, v in os.environ.items() if not k.startswith("GIT_")}
    genv.update(GIT_TERMINAL_PROMPT="0", GIT_CONFIG_NOSYSTEM="1", GIT_CONFIG_GLOBAL=os.devnull)

    def git(*args):
        return subprocess.run(["git", *args], env=genv, stdout=subprocess.PIPE,
                              stderr=subprocess.DEVNULL, text=True)

    def version_of(toml):
        m = re.search(r'^version = "([^"]+)"', toml, re.M)
        return m.group(1) if m else "none"

    def name_of(toml):
        m = re.search(r'^name = "([^"]+)"', toml, re.M)
        return m.group(1) if m else "none"

    # The manifest as cargo reads it, limited to what a dependent's build
    # compiles or sees. cargo vendor may drop dev-dependencies and example,
    # test, bench and bin targets; that build never reads them.
    BUILT = {"lib", "rlib", "dylib", "cdylib", "staticlib", "proc-macro", "custom-build"}
    PKG_FIELDS = ("authors", "description", "edition", "features", "homepage", "license",
                  "license_file", "links", "metadata", "readme", "repository", "rust_version")
    DEP_FIELDS = ("name", "rename", "req", "kind", "target", "optional",
                  "uses_default_features", "features")

    def shape(p):
        root = os.path.dirname(p["manifest_path"])
        s = {f: p.get(f) for f in PKG_FIELDS}
        s["dependencies"] = sorted(json.dumps({f: x.get(f) for f in DEP_FIELDS}, sort_keys=True)
                                   for x in p["dependencies"] if x.get("kind") != "dev")
        s["targets"] = sorted(json.dumps({"kind": t["kind"], "crate_types": t["crate_types"],
                                          "name": t["name"], "edition": t["edition"],
                                          "src_path": os.path.relpath(t["src_path"], root)},
                                         sort_keys=True)
                              for t in p["targets"] if BUILT & set(t["kind"]))
        return s

    def by_dir(meta):
        return {os.path.realpath(os.path.dirname(p["manifest_path"])): p
                for p in json.loads(meta)["packages"]}

    vpkgs = by_dir(resolved)
    # CARGO_HOME at the checkouts' parent: cargo's workspace search stops there,
    # so an upstream crate outside any workspace never meets this repo's.
    uenv = dict(env, CARGO_HOME=os.path.abspath(os.path.join(WORK, "upstream")))

    CAP = 20
    bad4 = []
    ok4 = []
    for _, d, url, sub in sorted(rows, key=lambda r: r[1]):
        rev = lock_rev[url]
        vend = os.path.join("vendor", d)
        cj = os.path.join(vend, ".cargo-checksum.json")
        if not os.path.isfile(cj):
            bad4.append(f"vendor/{d}: missing")
            continue
        up = os.path.join(WORK, "upstream", d)
        os.makedirs(up)
        if (git("init", "-q", up).returncode != 0
                or git("-C", up, "fetch", "-q", "--depth", "1", url, rev).returncode != 0
                or git("-C", up, "-c", "advice.detachedHead=false", "checkout", "-q", "FETCH_HEAD").returncode != 0):
            bad4.append(f"vendor/{d}: cannot fetch {url} at {rev}")
            continue
        if git("-C", up, "rev-parse", "HEAD").stdout.strip() != rev:
            bad4.append(f"vendor/{d}: fetched HEAD is not {rev}")
            continue
        crate = os.path.join(up, sub)
        if not os.path.isfile(os.path.join(crate, "Cargo.toml")):
            bad4.append(f"vendor/{d}: {sub} is not a crate directory in {url} at {rev}")
            continue
        listed = set(json.load(open(cj))["files"])
        on_disk = set()
        for root, _, files in os.walk(vend):
            for f in files:
                rel = os.path.relpath(os.path.join(root, f), vend)
                if rel != ".cargo-checksum.json":
                    on_disk.add(rel)
        found = []
        if on_disk != listed:
            diff = sorted(on_disk ^ listed)
            found.append(f"file set differs from .cargo-checksum.json: {', '.join(diff[:10])}")
        for rel in sorted(on_disk - {"Cargo.toml"}):
            upstream = os.path.join(crate, rel)
            if not os.path.isfile(upstream):
                found.append(f"{rel}: not in upstream at {rev[:8]}")
            elif open(upstream, "rb").read() != open(os.path.join(vend, rel), "rb").read():
                found.append(f"{rel}: differs from upstream at {rev[:8]}")
        vt = open(os.path.join(vend, "Cargo.toml"), encoding="utf-8").read()
        ut = open(os.path.join(crate, "Cargo.toml"), encoding="utf-8").read()
        if name_of(vt) != name_of(ut):
            found.append(f"Cargo.toml: package name {name_of(vt)} is not upstream's {name_of(ut)}")
        if version_of(vt) != version_of(ut) or version_of(ut) == "none":
            found.append(f"Cargo.toml: version {version_of(vt)} is not upstream's {version_of(ut)}")
        um = subprocess.run(["cargo", "metadata", "--no-deps", "--format-version", "1", "--offline",
                             "--manifest-path", os.path.join(crate, "Cargo.toml")],
                            env=uenv, capture_output=True, text=True)
        vp = vpkgs.get(os.path.realpath(vend))
        upk = by_dir(um.stdout).get(os.path.realpath(crate)) if um.returncode == 0 else None
        if vp is None:
            found.append("Cargo.toml: not a package in check (1)'s resolution")
        elif upk is None:
            why = (um.stderr.strip().splitlines() or ["no package there"])[0] if um.returncode else "no package there"
            found.append(f"Cargo.toml: cargo metadata cannot read upstream's {sub}/Cargo.toml at {rev[:8]} ({why})")
        else:
            vs, us = shape(vp), shape(upk)
            found += [f"Cargo.toml: '{f}' differs from upstream at {rev[:8]}" for f in sorted(vs) if vs[f] != us[f]]
        if found:
            bad4 += [f"vendor/{d}: {f}" for f in found[:CAP]]
            if len(found) > CAP:
                bad4.append(f"vendor/{d}: ... and {len(found) - CAP} more")
        else:
            print(f"vendor-freshness: (4/4) vendor/{d} matches {url} at {rev[:8]} "
                  f"({len(on_disk)} files).")
            ok4.append(f"{d}@{rev[:8]}")
    if bad4:
        err(f"(4/4) {len(bad4)} git-source defect(s): a vendored git source is not "
            f"upstream's tree at the Cargo.lock rev. A tree vendored from another rev "
            f"passes (1)-(3); only this check sees it (F-354).")
        for b in bad4:
            print(f"  - {b}", file=sys.stderr)
        sys.exit(1)
    print(f"vendor-freshness: (4/4) OK -- {', '.join(ok4)} re-derived from upstream.")
finally:
    shutil.rmtree(WORK, ignore_errors=True)
PY
