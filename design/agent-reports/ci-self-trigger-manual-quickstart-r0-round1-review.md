# R0 Architect Review — SPEC_ci_manual_quickstart_self_trigger.md — Round 1

> Reviewer had Read/Grep/WebFetch; parent persists. Source basis: HEAD `beab477` (= SPEC source SHA).

## Verdict: 1 Important / 0 Critical — NOT GREEN

One blocking finding (wrong line citations + a false grep-proof claim in the audit artifact). Everything else verified-correct. Path to GREEN is mechanical.

## Critical
None.

## Important

**I1 — All four Item-1 insertion-point citations are wrong; §3.2 claims a "grep proof" not performed.**
Verified against `manual.yml`/`quickstart.yml` at `beab477`:

| SPEC claim | Actual | Correct insertion |
|---|---|---|
| manual.yml push.paths "after :6 render-mermaid" | `:6` = header comment; render-mermaid at **:13** (push.paths 12–13) | after **:13** |
| manual.yml PR.paths "after :12" | `:12` = `docs/manual/**` in the **push** block; PR paths 18–19 | after **:19** |
| quickstart.yml push.paths "after :6" | `:6` = header comment; push.paths 12–13 | after **:13** |
| quickstart.yml PR.paths "after :14" | `:14` = `tags:` in the **push** block; PR paths 18–21 | after **:21** |

All four count into header-comment lines / the wrong block. (The SPEC's *mirror-pattern* citations are correct: technical-manual self-path push :23 / PR :32 verified.) Important-not-Minor in THIS repo: §3.2 asserts "Grep proof: each … now has exactly 2 self-path refs" + line 36 "actionlint-validated," yet four ungrepped wrong citations + a false grep-proof claim would ship in the persisted audit artifact — against the repo's active line-ref-drift convention + grep-verify-at-write-time rule (CLAUDE.md). The semantic target (add self-path to push.paths + PR.paths in both files) is unambiguous so an implementer can't misimplement, but the SPEC must not be committed with false location claims.
**Fix:** replace the four line numbers with :13/:19 (manual) and :13/:21 (quickstart); perform the grep before claiming "grep proof," or downgrade §3.2 to a post-implementation verification step. Re-confirm + re-dispatch.

## Minor
**M1 — Counter-reading.** An implementer following Item 1's prose lands correctly regardless of the numbers; absent the line-ref-drift convention + the self-claimed grep proof, I1 would be Minor. The combination lifts it to Important.

## Verified-correct
- **Current paths LACK the self-ref** (gap real): manual.yml push (12–13)/PR (18–19) = docs/manual + render-mermaid only; quickstart.yml push (12–13)/PR (18–21) = docs/quickstart + 2 docs/manual + render-mermaid. Neither lists its own `.github/workflows/<self>.yml`.
- **Mirror pattern confirmed** (self-path in BOTH push+PR): technical-manual push :23/PR :32, rust push :20/PR :26, manual-gui push :14/PR :20.
- **Heavy-cost flag accurate:** manual.yml = pandoc + full texlive-* + lychee + 3 `cargo install --git` sibling CLIs + `cargo build --bin mnemonic` + `make audit` + `make pdf`; quickstart.yml = pandoc + texlive + lychee + cargo install mk-cli + `make lint` + `make pdf`. Both multi-minute. "Rare edits, accepted" framing correct.
- **@v5 actions present (validation gap real):** manual checkout@v5 :30, setup-node@v5 :52, upload-artifact@v5 :120; quickstart checkout@v5 :32, setup-node@v5 :53, upload-artifact@v5 :86.
- **Purely additive:** adding a `paths` entry only widens WHEN the workflow fires (OR-set); no job/step/working-directory/checkout-nesting change → no gate WS-derivation effect; only 4 list entries change. Cannot break an existing run.
- **Self-validating claim TRUE** (resolved against authoritative GitHub docs): plain `pull_request` (unlike `pull_request_target`/`schedule`/`workflow_run`) is NOT subject to the "workflow must exist on default branch" rule; it runs against the **merge ref**, evaluating `paths:` from the merge-ref workflow file. So the landing PR's edited self-path is in effect for the triggering PR → manual+quickstart fire on that very PR, exercising checkout@v5+setup-node@v5+upload-artifact@v5 on real runners. (`pull_request_target` exists precisely because plain `pull_request` uses the merge ref — proves the behavior.)
- **Tag/push non-interference confirmed:** `paths` AND-ed with `branches:` for branch pushes (adding an entry only enlarges the OR-set); `tags:` block independent of `paths`; the `if: startsWith(github.ref,'refs/tags/…')` release path unaffected.
- **Cost framing endorsed:** both push+PR self-path = parity (divergent trigger conventions are themselves a footgun); PR-only would leave direct-to-master edits unvalidated; an actionlint-only side-job can't exercise the v5 actions on real runners (which is the gap). Reject alternatives.
- **Item 2 correct + v5-precondition still unmet:** the self-trigger sub-item is the trailing clause of `ci-actions-catch-up-to-latest-majors` (FOLLOWUPS ~:2350); dropping it + noting extraction is right. This cycle gives manual/quickstart their first live @v5 **PR** run, but the highest-risk v8 download-artifact change is in the **tag-gated** release job PRs can't reach → the release-path v5-stability precondition stays unmet → Cycle B (v6/v7/v8 + SHA-pin) correctly deferred.
- **Disposition/locksteps:** no-bump/no-tag correct (CI-only YAML, binary byte-identical); no clap/CLI/codec surface → no schema_mirror/manual-mirror/sibling companion.

### Single action to GREEN
Correct the four citations (manual :13/:19, quickstart :13/:21) + reconcile §3.2's grep-proof claim. Re-confirm.
Sources: GitHub Docs "Events that trigger workflows"; community discussion #26795.
