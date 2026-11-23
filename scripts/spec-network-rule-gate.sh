#!/usr/bin/env bash
# spec-network-rule-gate.sh -- machine checks for design/SPEC_network_rule.md.
#
# usage: scripts/spec-network-rule-gate.sh [SPEC] [--no-red]
#   SPEC defaults to design/SPEC_network_rule.md (relative to the repo root).
#   --no-red skips check 1 (the RED rows' "today" legs), e.g. on a machine
#   without the installed binaries.
# exit 0 = every check passed; 1 = a check failed (each failure is a FAIL line).
#
# What it checks:
#  1. RED rows: every table row of the spec whose first cell is a row id
#     R<unit>.<n> (unit 1..8 or 8a..8h) and which has 10 cells is RUN against the
#     INSTALLED binaries (mnemonic, md, mk, mr, me, mt on PATH) from the fixture
#     directory it names (design/network-rule/<dir>), with the stdin file it
#     names, and its "today" leg is checked: exit code equal; stdout matches the
#     "today stdout has" ERE; stderr matches "today stderr has"; stderr does NOT
#     match "today stderr lacks" ("-" = no check). The "after" leg is not run
#     (the code does not exist yet); its exit cell must be a number.
#  2. Cites: every {{repo@rev:path:LINE}} "quote" in the spec: the repo is a
#     checkout under $SHIB (default /scratch/code/shibboleth), the rev exists,
#     and line LINE of path at rev contains quote as a fixed string.
#  3. Text ids: every TX-/GUI-/MG-/SH- id used anywhere in the spec or in
#     design/network-rule/vectors.tsv is defined (a line starting "- **ID**"),
#     and every defined id is used at least once outside its definition line.
#     Prints the counts.
#  4. vectors.tsv: header exact; every row has 9 fields; enumerated fields hold
#     allowed values; ids unique; class/exit/override consistent (hard -> 8 and
#     override never helps; soft -> 8 without override, 0 with; none/no-source
#     -> 0); a tool with no --network never gets a text that names a
#     --network remedy (TX-H1, TX-H3, TX-H4, TX-H5, TX-N, TX-T).
#  5. Exit code 8: re-greps every CLI's exit-code definitions and the toolkit
#     manual's exit tables at the revs the spec's "Exit-code grep" line names,
#     and fails if any of them already uses 8 as an exit code. Patterns:
#     ExitCode::from(8), process::exit(8), exit_code(8), EXIT_* / *EXIT* consts
#     = 8 anywhere; in error.rs, exit_codes.rs and main.rs also a match arm
#     `=> 8,` and any `[pub] const NAME: u8|i32 = 8;`; manual table rows `| 8 |`.
#
# What it does NOT cover (a reviewer's job):
#  * the "after" legs of the RED rows (they describe code not yet written);
#  * whether a text reads well, or whether a decision is right;
#  * whether the per-arm table is complete (arms nobody listed are invisible);
#  * any GUI or SeedHammer behaviour (no row runs a GUI or the device);
#  * the mt testnet4 row against a real node (the fake bitcoin-cli only);
#  * protocol facts (BIP/SLIP/Core) beyond the cites listed in the spec;
#  * cites whose quote is a fixed string that also matches a different meaning.
set -u
cd "$(git -C "$(dirname "$0")" rev-parse --show-toplevel)" || exit 1
SPEC=design/SPEC_network_rule.md
RUN_RED=1
for a in "$@"; do
  case "$a" in
    --no-red) RUN_RED=0 ;;
    *) SPEC=$a ;;
  esac
done
VEC=${NETRULE_VECTORS:-design/network-rule/vectors.tsv}   # override only to show the gate RED on a planted table
SHIB=${SHIB:-/scratch/code/shibboleth}
FAILS=0
fail() { echo "FAIL: $*"; FAILS=$((FAILS + 1)); }
[ -f "$SPEC" ] || { echo "FAIL: no spec at $SPEC"; exit 1; }
ROOT=$PWD

# ---------- 1. RED rows ----------
strip() { local s="$1"; s="${s#"${s%%[![:space:]]*}"}"; s="${s%"${s##*[![:space:]]}"}"; s="${s#\`}"; s="${s%\`}"; printf '%s' "$s"; }
if [ "$RUN_RED" = 1 ]; then
  export X=xpub6CatWdiZiodmUeTDp8LT5or8nmbKNcuyvz7WyksVFkKB4RHwCD3XyuvPEbvqAQY3rAPshWcMLoP2fMFMKHPJ4ZeZXYVUhLv1VMrjPC7PW6V
  export T=tpubDC8msFGeGuwnKG9Upg7DM2b4DaRqg3CUZa5g8v2SRQ6K4NSkxUgd7HsL2XVWbVm39yBA4LAxysQAm397zwQSQoQgewGiYZqrA9DsP4zbQ1M
  export P="abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about"
  export T1='[704c7836/84h/1h/0h]tpubDC8msFGeGuwnKG9Upg7DM2b4DaRqg3CUZa5g8v2SRQ6K4NSkxUgd7HsL2XVWbVm39yBA4LAxysQAm397zwQSQoQgewGiYZqrA9DsP4zbQ1M'
  export T2='[97139860/84h/1h/0h]tpubDC9Go1KDateW3gS8VXZ6DD1Xu7PgoTdPcf1MX9Z6qVLiHbaeDJ78swPyuQ8YQY19QjtrzkfkZSXwqCcb7XArtid1iLq8Vy55Ydfm4giZh6X'
  export X1='[11111111/48h/0h/0h/2h]xpub661MyMwAqRbcEZVB4dScxMAdx6d4nFc9nvyvH3v4gJL378CSRZiYmhRoP7mBy6gSPSCYk6SzXPTf3ND1cZAceL7SfJ1Z3GC8vBgp2epUt13'
  export X2='[97139860/48h/0h/0h/2h]xpub6DnEBNkSJKBYQmsbhS1sP9cNdtU5c9PLFGCjTJmxicxc13WB8zNNGQazabQpyFAGW5bV9tMko4uBxDxjUKL6dSAcx1tEbgEHtgSqyRsekh6'
  export X1c1='[11111111/48h/1h/0h/2h]xpub661MyMwAqRbcEZVB4dScxMAdx6d4nFc9nvyvH3v4gJL378CSRZiYmhRoP7mBy6gSPSCYk6SzXPTf3ND1cZAceL7SfJ1Z3GC8vBgp2epUt13'
  # md1 strings of a bundle file as --md1 arguments; a bundle's mk1 lines L1..L2 as --cosigner @N=.
  mdargs() { grep '^md1' "$1" | sed 's/^/--md1 /' | tr '\n' ' '; }
  cos() { sed -n "$2,$3p" "$1" | sed "s/^/--cosigner @$4=/" | tr '\n' ' '; }
  export -f mdargs cos
  OUT=$(mktemp -d "${TMPDIR:-/tmp}/netrule-gate.XXXXXX")
  export W="$OUT/w"; mkdir -p "$W"   # rows write their output files (me --out, --manifest) here
  nrows=0
  while IFS= read -r line; do
    IFS='|' read -r _ c1 c2 c3 c4 c5 c6 c7 c8 c9 c10 rest <<<"$line"
    [ -n "${c10// /}" ] || continue
    id=$(strip "$c1"); dir=$(strip "$c2"); sin=$(strip "$c3"); cmd=$(strip "$c4")
    rc_want=$(strip "$c5"); out_has=$(strip "$c6"); err_has=$(strip "$c7"); err_lacks=$(strip "$c8"); after_rc=$(strip "$c9")
    nrows=$((nrows + 1))
    [ -d "design/network-rule/$dir" ] || { fail "$id: no fixture dir design/network-rule/$dir"; continue; }
    case "$after_rc" in ''|*[!0-9]*) fail "$id: after exit cell '$after_rc' is not a number" ;; esac
    if [ "$sin" = "-" ]; then sinf=/dev/null; else sinf="$ROOT/design/network-rule/$dir/$sin"; fi
    (cd "design/network-rule/$dir" && timeout 300 bash -c "$cmd" <"$sinf" >"$OUT/o" 2>"$OUT/e")
    rc=$?
    ok=1
    [ "$rc" = "$rc_want" ] || { fail "$id: exit $rc, the spec says today $rc_want"; ok=0; }
    if [ "$out_has" != "-" ] && ! grep -q -E -- "$out_has" "$OUT/o"; then fail "$id: stdout lacks /$out_has/ (stdout L1: $(head -c 160 "$OUT/o" | head -1))"; ok=0; fi
    if [ "$err_has" != "-" ] && ! grep -q -E -- "$err_has" "$OUT/e"; then fail "$id: stderr lacks /$err_has/ (stderr L1: $(head -c 160 "$OUT/e" | head -1))"; ok=0; fi
    if [ "$err_lacks" != "-" ] && grep -q -E -- "$err_lacks" "$OUT/e"; then fail "$id: stderr matches /$err_lacks/, which the spec says is absent today"; ok=0; fi
    [ $ok = 1 ] && echo "ok   $id today rc=$rc"
  done < <(grep -E '^\| *`?R[1-8][a-h]?\.[0-9]+`? *\|' "$SPEC")
  rm -r -- "$OUT"
  echo "RED rows run: $nrows"
  [ "$nrows" -gt 0 ] || fail "no RED rows found in $SPEC"
else
  echo "RED rows: skipped (--no-red)"
fi

# ---------- 2. cites ----------
ncite=0
while IFS= read -r c; do
  ref=${c#\{\{}; ref=${ref%%\}\}*}
  quote=${c#*\}\} \"}; quote=${quote%\"}
  repo=${ref%%@*}; rest=${ref#*@}; rev=${rest%%:*}; rest=${rest#*:}; path=${rest%:*}; ln=${rest##*:}
  ncite=$((ncite + 1))
  d="$SHIB/$repo"
  [ -d "$d" ] || { fail "cite $ref: no checkout $d"; continue; }
  git -C "$d" cat-file -e "$rev^{commit}" 2>/dev/null || { fail "cite $ref: rev $rev not in $repo"; continue; }
  got=$(git -C "$d" show "$rev:$path" 2>/dev/null | sed -n "${ln}p")
  case "$got" in
    *"$quote"*) : ;;
    *) fail "cite $ref: line $ln does not contain \"$quote\" (it reads: $(printf '%s' "$got" | cut -c1-120))" ;;
  esac
done < <(grep -o -E '\{\{[a-z-]+@[0-9a-f]{7,40}:[^:}]+:[0-9]+\}\} "[^"]+"' "$SPEC")
echo "cites checked: $ncite"
[ "$ncite" -gt 0 ] || fail "no cites found"

# ---------- 3. text ids ----------
IDRE='\b(TX|GUI|MG|SH)-[A-Z0-9]+(-[A-Z0-9]+)*\b'
defs=$(grep -o -E '^- \*\*(TX|GUI|MG|SH)-[A-Z0-9]+(-[A-Z0-9]+)*\*\*' "$SPEC" | sed -E 's/^- \*\*//; s/\*\*$//' | sort)
dupdefs=$(printf '%s\n' "$defs" | uniq -d)
[ -z "$dupdefs" ] || fail "text ids defined twice: $(echo $dupdefs)"
used=$( { grep -v -E '^- \*\*(TX|GUI|MG|SH)-' "$SPEC"; [ -f "$VEC" ] && cut -f8 "$VEC" | tail -n +2; } | grep -o -E "$IDRE" | sort -u)
ndef=$(printf '%s\n' "$defs" | grep -c .)
nused=$(printf '%s\n' "$used" | grep -c .)
for u in $used; do printf '%s\n' "$defs" | grep -q -x -- "$u" || fail "text id $u is used but not defined"; done
for d in $defs; do printf '%s\n' "$used" | grep -q -x -- "$d" || fail "text id $d is defined but never used"; done
echo "text ids: defined $ndef, used $nused"

# ---------- 4. vectors.tsv ----------
if [ ! -f "$VEC" ]; then fail "no $VEC"; else
  HDR=$'id\ttool\thas_network_flag\trun_network\tsources\toverride\tclass\ttext\texit'
  [ "$(head -1 "$VEC")" = "$HDR" ] || fail "vectors.tsv header differs from the schema"
  nvec=0
  while IFS=$'\t' read -r id tool hasflag net src ovr cls txt ex extra; do
    nvec=$((nvec + 1))
    [ -z "${extra:-}" ] && [ -n "$ex" ] || { fail "vector $id: not 9 fields"; continue; }
    case "$tool" in mnemonic|md|mk|ms|mr|mr-sitting|me|mt|seedhammer|mr-gui|mnemonic-gui|any) ;; *) fail "vector $id: tool '$tool'";; esac
    case "$hasflag" in yes|no) ;; *) fail "vector $id: has_network_flag '$hasflag'";; esac
    case "$net" in main|signet|testnet|regtest|none) ;; *) fail "vector $id: run_network '$net'";; esac
    [ "$hasflag" = no ] && [ "$net" != none ] && fail "vector $id: a tool with no --network has run_network none"
    case "$ovr" in 0|1) ;; *) fail "vector $id: override '$ovr'";; esac
    case "$cls" in hard|soft|hard+soft|none|no-source) ;; *) fail "vector $id: class '$cls'";; esac
    case "$cls:$ovr:$ex" in hard:?:8|hard+soft:?:8|soft:0:8|soft:1:0|none:?:0|no-source:?:0) ;; *) fail "vector $id: class $cls with override $ovr cannot exit $ex";; esac
    for s in $(printf '%s' "$src" | tr ';' ' '); do
      case "$s" in -|key=main|key=test|hrp=bc|hrp=tb|hrp=bcrt|b58=main|b58=test|wif=main|wif=test|core=main|core=test|core=testnet4|core=signet|core=regtest|stated=main|stated=test|stated=signet|stated=regtest|stated2=main|stated2=test|coin=0|coin=1|path=other|reads=kept|reads=seed|out=secret|plate=dup-other|plate=only-other) ;;
        *) fail "vector $id: source '$s'";; esac
    done
    # Texts whose every form names a --network remedy; a tool with no --network never prints them.
    if [ "$hasflag" = no ]; then
      for t in $(printf '%s' "$txt" | tr ',' ' '); do
        case "$t" in TX-H1|TX-H3|TX-H4|TX-H5|TX-N|TX-T) fail "vector $id: a tool with no --network gets $t, which names a --network remedy";; esac
      done
    fi
  done < <(tail -n +2 "$VEC")
  dupv=$(tail -n +2 "$VEC" | cut -f1 | sort | uniq -d)
  [ -z "$dupv" ] || fail "vector ids repeated: $(echo $dupv)"
  echo "vectors: $nvec rows"
fi

# ---------- 5. exit code 8 is free ----------
G8=$(grep -o -E '^Exit-code grep revs: .*' "$SPEC" | head -1)
[ -n "$G8" ] || fail "spec has no 'Exit-code grep revs:' line"
n8=0
for pair in ${G8#Exit-code grep revs: }; do
  repo=${pair%%@*}; rev=${pair#*@}; d="$SHIB/$repo"
  [ -d "$d" ] || { fail "exit-8 grep: no checkout $d"; continue; }
  hits=$(git -C "$d" grep -n -E 'ExitCode::from\(8\)|process::exit\(8\)|exit_code\(8\)|EXIT_[A-Z_]+: *(u8|i32) *= *8;|const [A-Z_]*EXIT[A-Z_]*: *(u8|i32) *= *8;|^\| *`8` *\|' "$rev" -- ':!vendor' ':!**/vendor/**' ':!fuzz' 2>/dev/null)
  hits="$hits$(git -C "$d" grep -n -E '=> *8,|^ *(pub )?const [A-Z_]+: *(u8|i32) *= *8;' "$rev" -- '**/error.rs' '**/exit_codes.rs' '**/main.rs' ':!vendor' ':!**/vendor/**' ':!fuzz' 2>/dev/null)"
  if [ "$repo" = mnemonic-toolkit ]; then
    # The manual's CLI pages hold every tool's exit table (mnemonic, md, ms, mk, mr, mr-sitting).
    hits="$hits$(git -C "$d" grep -n -E '^\| *`?8`? *\|' "$rev" -- 'docs/manual/src/40-cli-reference/*.md' 2>/dev/null)"
  fi
  if [ -n "$hits" ]; then fail "exit-8 grep: $repo@$rev already uses 8: $(printf '%s' "$hits" | head -3)"; fi
  n8=$((n8 + 1))
done
echo "exit-8 grep: $n8 repos, 8 free in all"

if [ "$FAILS" -gt 0 ]; then echo "GATE RED: $FAILS failure(s)"; exit 1; fi
echo "GATE GREEN"
