# ci-docs-skip fold C1 implementation report

All six PRs folded (one commit each, pushed, no force). Every gate GREEN, release-gen identical for all five repos, actionlint empty before and after, all CI checks pass or skipping, PRs still draft.

Gate output is the tail of each command (final lines, incl. GATE: GREEN). actionlint before = run on HEAD~1's release.yml via stdin (empty, rc=0); after = run in worktree (empty, rc=0).

## descriptor-mnemonic (PR #37)
- new head: acaa0c86cdc1bb1c8fa737539fe31e519f543901
- CI tally (head verified = PR headRefOid):  32 pass, 2 skipping,  (0 fail; counts include push+pull_request runs)
```
diff --git a/.github/workflows/release.yml b/.github/workflows/release.yml
index dab3df0e..866d558d 100644
--- a/.github/workflows/release.yml
+++ b/.github/workflows/release.yml
@@ -113,10 +113,11 @@ jobs:
         with:
           ref: ${{ github.event.inputs.tag || github.ref }}
       - name: Self-test the docs-only guard
-        if: runner.os == 'Linux'
+        if: runner.os == 'Linux' && github.event.inputs.tag == ''
         shell: bash
         run: bash ci/docs-only.test.sh
       - name: Docs-only change? (skips the steps below when every changed file is documentation)
+        if: github.event.inputs.tag == ''
         id: docs
         shell: bash
         env:
```
## mnemonic-toolkit (PR #103)
- new head: 924e0a335d97439b3048b82e43c49ea0d0ddcbb1
- CI tally (head verified = PR headRefOid):  48 pass, 12 skipping,  (0 fail; counts include push+pull_request runs)
```
diff --git a/.github/workflows/release.yml b/.github/workflows/release.yml
index e454be52..b7362d32 100644
--- a/.github/workflows/release.yml
+++ b/.github/workflows/release.yml
@@ -112,10 +112,11 @@ jobs:
         with:
           ref: ${{ github.event.inputs.tag || github.ref }}
       - name: Self-test the docs-only guard
-        if: runner.os == 'Linux'
+        if: runner.os == 'Linux' && github.event.inputs.tag == ''
         shell: bash
         run: bash ci/docs-only.test.sh
       - name: Docs-only change? (skips the steps below when every changed file is documentation)
+        if: github.event.inputs.tag == ''
         id: docs
         shell: bash
         env:
```
## mnemonic-key (PR #13)
- new head: 654d1f9da2713bc5df31d2eed14999780c026f56
- CI tally (head verified = PR headRefOid):  38 pass, 4 skipping,  (0 fail; counts include push+pull_request runs)
```
diff --git a/.github/workflows/release.yml b/.github/workflows/release.yml
index a0e1e2c..98e1c83 100644
--- a/.github/workflows/release.yml
+++ b/.github/workflows/release.yml
@@ -108,10 +108,11 @@ jobs:
         with:
           ref: ${{ github.event.inputs.tag || github.ref }}
       - name: Self-test the docs-only guard
-        if: runner.os == 'Linux'
+        if: runner.os == 'Linux' && github.event.inputs.tag == ''
         shell: bash
         run: bash ci/docs-only.test.sh
       - name: Docs-only change? (skips the steps below when every changed file is documentation)
+        if: github.event.inputs.tag == ''
         id: docs
         shell: bash
         env:
```
## mnemonic-secret (PR #12)
- new head: 02702991332e29767a12bc589530515c497e6654
- CI tally (head verified = PR headRefOid):  34 pass, 2 skipping,  (0 fail; counts include push+pull_request runs)
```
diff --git a/.github/workflows/release.yml b/.github/workflows/release.yml
index 877b623..e6dc169 100644
--- a/.github/workflows/release.yml
+++ b/.github/workflows/release.yml
@@ -116,10 +116,11 @@ jobs:
         with:
           ref: ${{ github.event.inputs.tag || github.ref }}
       - name: Self-test the docs-only guard
-        if: runner.os == 'Linux'
+        if: runner.os == 'Linux' && github.event.inputs.tag == ''
         shell: bash
         run: bash ci/docs-only.test.sh
       - name: Docs-only change? (skips the steps below when every changed file is documentation)
+        if: github.event.inputs.tag == ''
         id: docs
         shell: bash
         env:
```
## mnemonic-transaction (PR #3)
- new head: 8719a87d76dc6a99d3eaf12983bb96f2f7ed7dbc
- CI tally (head verified = PR headRefOid):  14 pass, 2 skipping,  (0 fail; counts include push+pull_request runs)
```
diff --git a/.github/workflows/release.yml b/.github/workflows/release.yml
index 9aee698..7eee3a5 100644
--- a/.github/workflows/release.yml
+++ b/.github/workflows/release.yml
@@ -105,10 +105,11 @@ jobs:
         with:
           ref: ${{ github.event.inputs.tag || github.ref }}
       - name: Self-test the docs-only guard
-        if: runner.os == 'Linux'
+        if: runner.os == 'Linux' && github.event.inputs.tag == ''
         shell: bash
         run: bash ci/docs-only.test.sh
       - name: Docs-only change? (skips the steps below when every changed file is documentation)
+        if: github.event.inputs.tag == ''
         id: docs
         shell: bash
         env:
```
## mnemonic-engrave (PR #19)
- new head: 22ef304fe559bb059b9ccecd8221ba614f9ef774
- CI tally (head verified = PR headRefOid):  16 pass, 2 skipping,  (0 fail; counts include push+pull_request runs)
```
diff --git a/scripts/release-workflows/gen.py b/scripts/release-workflows/gen.py
index 1e4ece1f..cca228e0 100644
--- a/scripts/release-workflows/gen.py
+++ b/scripts/release-workflows/gen.py
@@ -139,10 +139,11 @@ jobs:
         with:
           ref: ${{{{ github.event.inputs.tag || github.ref }}}}
       - name: Self-test the docs-only guard
-        if: runner.os == 'Linux'
+        if: runner.os == 'Linux' && github.event.inputs.tag == ''
         shell: bash
         run: bash ci/docs-only.test.sh
       - name: Docs-only change? (skips the steps below when every changed file is documentation)
+        if: github.event.inputs.tag == ''
         id: docs
         shell: bash
         env:
```
## Gate lines (identical for every repo; per-repo output in session)
- descriptor-mnemonic: files GREEN, paths-check GREEN, workflows GREEN
- mnemonic-toolkit: files GREEN, paths-check GREEN, workflows GREEN
- mnemonic-key: files GREEN, paths-check GREEN, workflows GREEN
- mnemonic-secret: files GREEN, paths-check GREEN, workflows GREEN
- mnemonic-transaction: files GREEN, paths-check GREEN, workflows GREEN
- mnemonic-engrave: files GREEN, paths-check GREEN, workflows GREEN
- mnemonic-engrave release-gen: GATE: GREEN; descriptor-mnemonic, mnemonic-secret, mnemonic-key, mnemonic-transaction, mnemonic-toolkit all identical
