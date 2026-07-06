# Journey 5 — Watch-only export

The final journey closes the loop: reconstruct the watch-only wallet
from the **shared `md1` card alone** — no seeds — and export it in the
shapes real wallets import. We take Journey 2's 2-of-3, pull its `md1`
chunks out of a `bundle --json` run, and feed them straight back into
`restore` to emit first a bare canonical descriptor and then a Bitcoin
Core `importdescriptors` payload. This is `Examples.pdf` section 4 —
the card-set-to-Bitcoin-Core path — expressed entirely in the GUI, with
the app chaining each step's output into the next.

> Everything in this journey is **public** watch-only material; no
> secret is typed.

## Bundle as JSON {#tut-j5-22-bundle-json}

The watch-only export starts by producing the `md1` card in a
machine-readable shape. On `bundle`, paste Journey 2's 2-of-3
descriptor into **`--descriptor`** and tick **`--json`**. The filled
form is below. The panel's standard output is the JSON bundle; its
`md1` array is exactly the card set the two restores below consume. This
one run stands in for `Examples.pdf`'s `bundle … --json | jq -r
".md1[]"` — the GUI chains the array straight into the next steps.

![GUI form (screenshot)](../figures/tutorial/tut-j5-22-bundle-json-form.png)

![Output pane after Run (screenshot)](../figures/tutorial/tut-j5-22-bundle-json-run.png)

**Output (stdout):**

```{.text include="tutorial/tut-j5-22-bundle-json.stdout.txt"}
(captured transcript — included at build time)
```

**Standard error (stderr):**

```{.text include="tutorial/tut-j5-22-bundle-json.stderr.txt"}
(captured transcript — included at build time)
```

**Exit code:**

```{.text include="tutorial/tut-j5-22-bundle-json.exit.txt"}
(captured transcript — included at build time)
```

## Restore to a descriptor {#tut-j5-23-restore-descriptor}

Reconstruct the wallet from that `md1` card and emit a **bare canonical
descriptor**. Select **Restore …**, set **Template** to **`(none)`** (an
`md1` restore needs no template — clearing the form's default `bip84`
avoids the single-sig-template refusal in `--md1` mode), set
**`--format`** to `descriptor`, and let the `md1` chunks chain in from
the bundle-JSON step's run. The filled form is below.

The panel returns `wsh(sortedmulti(2,…))#yjp7hj7w` on standard output —
the plain descriptor other wallets import — and the restore summary
(descriptor, first receive address, per-cosigner fingerprints,
`UNVERIFIED` reminder) on standard error. The first address matches
Journey 2's: same wallet, rebuilt from the card alone.

![GUI form (screenshot)](../figures/tutorial/tut-j5-23-restore-descriptor-form.png)

![Output pane after Run (screenshot)](../figures/tutorial/tut-j5-23-restore-descriptor-run.png)

**Output (stdout):**

```{.text include="tutorial/tut-j5-23-restore-descriptor.stdout.txt"}
(captured transcript — included at build time)
```

**Standard error (stderr):**

```{.text include="tutorial/tut-j5-23-restore-descriptor.stderr.txt"}
(captured transcript — included at build time)
```

**Exit code:**

```{.text include="tutorial/tut-j5-23-restore-descriptor.exit.txt"}
(captured transcript — included at build time)
```

## Restore for Bitcoin Core {#tut-j5-24-restore-core}

The same restore (Template stays `(none)`) with **`--format`** switched
to `bitcoin-core` emits
the ready-to-import `importdescriptors` array (external `.../0/*` and
change `.../1/*` requests, each with an `"active": true` range). The
filled form is below; the chained `md1` chunks are unchanged from the
bundle-JSON step.

Loading this into Bitcoin Core is an external step, run against your own
node and so not captured here: save the array, create a blank
descriptor wallet with `disable_private_keys=true`, and
`importdescriptors` the payload. Tips from `Examples.pdf`: `--timestamp
now` skips the rescan on a fresh wallet, `--range 0,4999` widens the gap
limit, and `--bitcoin-core-version 24` targets older Core. That closes
the loop — a wallet engraved to steel, reconstructed from one public
card, and handed to a full node, without a seed ever leaving its
owner.

![GUI form (screenshot)](../figures/tutorial/tut-j5-24-restore-core-form.png)

![Output pane after Run (screenshot)](../figures/tutorial/tut-j5-24-restore-core-run.png)

**Output (stdout):**

```{.text include="tutorial/tut-j5-24-restore-core.stdout.txt"}
(captured transcript — included at build time)
```

**Standard error (stderr):**

```{.text include="tutorial/tut-j5-24-restore-core.stderr.txt"}
(captured transcript — included at build time)
```

**Exit code:**

```{.text include="tutorial/tut-j5-24-restore-core.exit.txt"}
(captured transcript — included at build time)
```
