#!/usr/bin/env bash
# ci/code-docs-check.test.sh -- scenario tests for ci/code-docs-check.sh.
#
# Builds a throwaway git repository per row and runs the check in it. Needs
# only git and bash 4. Exit 0 = every row passed.
set -eu
here=$(cd "$(dirname "$0")" && pwd)
check=$here/code-docs-check.sh
tmp=$(mktemp -d "${TMPDIR:-/tmp}/code-docs-test.XXXXXX")
trap 'rm -rf "$tmp"' EXIT
export GIT_CONFIG_GLOBAL=$tmp/gitconfig GIT_CONFIG_NOSYSTEM=1
git config --global user.email test@example.invalid
git config --global user.name code-docs-test

pass=0 fail=0
# fresh: start a new repository holding the standard rules and docs files.
fresh() {
  rm -rf "$tmp/r"; mkdir -p "$tmp/r/ci" "$tmp/r/design/sub" "$tmp/r/crates/x/src" "$tmp/r/scripts"
  cd "$tmp/r"; git init -q
  printf '%s\n' '+ ^design/' '+ ^[^/]+\.md$' '- ^design/protected\.md$' > ci/docs-only-paths.txt
  echo spec > design/SPEC_a.md; echo p > design/protected.md; echo r > design/sub/README.md
  echo r > crates/x/README.md; echo n > design/cli; echo f > FOLLOWUPS.md
}
put() { # put <file> <line>: append a line to a file
  mkdir -p "$(dirname "$1")"; printf '%s\n' "$2" >> "$1"
}
row() { # row <name> <want ok|fail> [<text the output must contain>]
  git add -A > /dev/null; git commit -qm row > /dev/null 2>&1 || true
  set +e; got=$(bash "$check" 2>&1); rc=$?; set -e
  if { [ "$2" = ok ] && [ "$rc" -eq 0 ]; } || { [ "$2" = fail ] && [ "$rc" -eq 1 ]; }; then
    if [ -n "${3:-}" ] && ! grep -qF -- "$3" <<<"$got"; then
      fail=$((fail + 1)); echo "FAIL $1: output lacks '$3':"; echo "$got"; return
    fi
    pass=$((pass + 1)); echo "ok   $1 ($(tail -1 <<<"$got"))"
  else
    fail=$((fail + 1)); echo "FAIL $1: exit $rc, want $2:"; echo "$got"
  fi
}

fresh; put crates/x/src/lib.rs 'fn f() {}'
row "nothing names a docs file" ok "code-docs check: ok"
fresh; put crates/x/src/lib.rs 'const S: &str = include_str!("../../../design/SPEC_a.md");'
row "full path in a .rs string" fail "design/SPEC_a.md is named at crates/x/src/lib.rs:1"
fresh; put scripts/a.py 'open("SPEC_a.md")'
row "unique basename in a .py" fail "design/SPEC_a.md is named at scripts/a.py:1"
fresh; put crates/x/src/lib.rs 'let p = "README.md";'
row "shared basename alone is not a match" ok
fresh; put crates/x/src/lib.rs 'let p = "design/sub/README.md";'
row "full path of a docs file with a shared basename" fail "design/sub/README.md is named at"
fresh; put crates/x/src/lib.rs 'let a = "x_SPEC_a.md"; let b = "SPEC_a.md_old"; let c = "v2SPEC_a.md-x";'
row "a name inside a longer name is not a match" ok
fresh; put crates/x/src/lib.rs 'let a = "x_SPEC_a.md"; let b = "(SPEC_a.md)";'
row "a later bounded occurrence on the same line matches" fail "crates/x/src/lib.rs:1"
fresh; put crates/x/src/lib.rs 'let p = "design/SPEC_a.md.bak";'
row "a trailing dot is a boundary" fail "crates/x/src/lib.rs:1"
fresh; put scripts/a.sh 'echo cli'
row "basename without a dot is not a match" ok
fresh; echo d > design/data.tsv; put crates/x/tests/t.rs 'std::fs::write(dir.join("data.tsv"), "x");'
row "unique basename of a non-.md docs file alone is not a match" ok
fresh; echo d > design/data.tsv; put crates/x/tests/t.rs 'include_str!("../../../design/data.tsv")'
row "full path of a non-.md docs file matches" fail "design/data.tsv is named at crates/x/tests/t.rs:1"
fresh; put scripts/a.sh 'cat design/cli'
row "full path of a docs file without a dot" fail "design/cli is named at scripts/a.sh:1"
fresh; put crates/x/src/lib.rs '    // see design/SPEC_a.md'; put crates/x/src/lib.rs '/// SPEC_a.md'; put crates/x/src/lib.rs '//! SPEC_a.md'
row "whole-line // comments in .rs are skipped" ok
fresh; put crates/x/src/lib.rs 'fn f() {} // SPEC_a.md'
row "trailing comment in .rs still matches" fail "crates/x/src/lib.rs:1"
fresh; put scripts/a.sh '  # SPEC_a.md'; put scripts/a.py '# SPEC_a.md'; put .github/workflows/w.yml '# SPEC_a.md'
put x.toml '# SPEC_a.md'; put flake.nix '  # SPEC_a.md'
row "whole-line # comments in .sh .py .yml .toml .nix are skipped" ok
fresh; put .github/workflows/ci.yml "    paths-ignore: ['design/SPEC_a.md', 'x/**']"; put .github/workflows/t.yaml "  paths: ['design/SPEC_a.md']"
row "a one-line paths or paths-ignore filter in a workflow is skipped" ok
fresh; put .github/workflows/ci.yml "    paths-ignore: ['x/**']"; put .github/workflows/ci.yml '      - run: cat design/SPEC_a.md'
row "another line of the same workflow still matches" fail ".github/workflows/ci.yml:2"
fresh; put .github/workflows/ci.yml "    paths-ignore:"; put .github/workflows/ci.yml "      - 'design/SPEC_a.md'"
row "a block-style filter entry still matches" fail ".github/workflows/ci.yml:2"
fresh; put scripts/conf.yml "paths-ignore: ['design/SPEC_a.md']"
row "a filter-shaped line outside .github/workflows still matches" fail "scripts/conf.yml:1"
fresh; put scripts/a.sh 'echo x # SPEC_a.md'
row "trailing # comment still matches" fail "scripts/a.sh:1"
fresh; put crates/x/Cargo.toml 'readme = "FOLLOWUPS.md"'
row "a bare toml readme key is skipped" ok
fresh; put crates/x/Cargo.toml 'include = ["FOLLOWUPS.md"]'
row "another toml line matches" fail "FOLLOWUPS.md is named at crates/x/Cargo.toml:1"
fresh; put crates/x/src/lib.rs 'include_str!("../../../design/protected.md")'
row "a protected (- rule) file is code, not docs" ok
fresh; put vendor/y/src/lib.rs 'include_str!("SPEC_a.md")'
row "vendor/ is not scanned" ok
fresh; put scripts/gen.py 'open("design/SPEC_a.md")'; printf '%s\n' '# not run by CI' '^scripts/gen\.py$' > ci/code-docs-exempt.txt
row "an exempt file is not scanned" ok
fresh; put scripts/gen.py 'open("design/SPEC_a.md")'; printf '%s\n' '^scripts/other\.py$' > ci/code-docs-exempt.txt
row "exempting another file does not exempt this one" fail "scripts/gen.py:1"
fresh; printf '%s\n' '^scripts/(' > ci/code-docs-exempt.txt; put scripts/a.sh 'true'
row "bad exempt pattern fails" fail "bad pattern in ci/code-docs-exempt.txt"
fresh; put design/notes.md 'see SPEC_a.md'; put README.md 'see design/SPEC_a.md'
row "docs files are not scanned" ok
fresh; put design/tools/gen.py 'open("design/SPEC_a.md")'
row "a code-typed file under a docs rule is docs, not scanned" ok
fresh; put Makefile 'cat design/SPEC_a.md'; put sub/justfile 'x: cat SPEC_a.md'; put a.go 's := "SPEC_a.md"'
row "Makefile, justfile and .go are scanned" fail "code-docs check: 3 docs file reference(s)"
fresh; put 'scripts/with space.sh' 'cat SPEC_a.md'
row "file name with a space" fail "scripts/with space.sh:1"
fresh; put notes.txt 'SPEC_a.md'; put data.json '"SPEC_a.md"'
row "other file types are not scanned" ok
fresh; rm ci/docs-only-paths.txt
row "missing rules file fails" fail "cannot read ci/docs-only-paths.txt"
fresh; echo 'nonsense' >> ci/docs-only-paths.txt
row "unparseable rules line fails" fail "unparseable line in ci/docs-only-paths.txt: nonsense"
fresh; printf '%s\n' '- ^x$' > ci/docs-only-paths.txt
row "rules without + fail" fail "no + rules"
fresh; printf '%s\n' '+ ^design/' '+ ^bip/(' > ci/docs-only-paths.txt; put crates/x/src/lib.rs 'fn f() {}'
row "bad + pattern fails" fail "bad pattern in ci/docs-only-paths.txt: ^bip/("
fresh; printf '%s\n' '+ ^design/' '- ^x(' > ci/docs-only-paths.txt; put crates/x/src/lib.rs 'fn f() {}'
row "bad - pattern fails" fail "bad pattern in ci/docs-only-paths.txt: ^x("
fresh; cp "$check" ci/code-docs-check.sh; put ci/docs-only.sh 'cat design/SPEC_a.md'
put ci/docs-only.test.sh 'SPEC_a.md'; put ci/code-docs-check.test.sh 'SPEC_a.md'
row "the shared ci scripts are not scanned" ok
fresh; put crates/x/src/lib.rs 'include_str!("../../../design/SPEC_a.md")'; put scripts/a.py 'open("SPEC_a.md")'
row "every hit is reported" fail "code-docs check: 2 docs file reference(s)"
fresh; put scripts/a.py 'open("SPEC_a.md")'
row "the advice allows a reword only for a line that does not read the file" fail "only when the line names the file without reading it"
fresh; mkdir -p "$tmp/r/design"; echo x > "$tmp/r/design/untracked.md"; put scripts/a.sh 'cat design/untracked.md'
git rm -q --cached design/untracked.md 2> /dev/null || true; echo design/untracked.md > .gitignore
row "untracked files are not docs" ok

cd "$here"
echo "code-docs-check.test.sh: $pass passed, $fail failed"
[ "$fail" -eq 0 ]
