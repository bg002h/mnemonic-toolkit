# Help-icons and deep-links into this manual

Every dropdown, every NodeValueComposite flag, every TaggedOrIndexed
flag, and every repeating-field flag in the GUI renders with a `?`
help-icon button next to its label or selector. Each per-subcommand
ComboBox also gets one `?` button immediately to its right.
Clicking any of these buttons opens this manual in your default
browser at the anchor for that exact UI element. This chapter walks
through the three button classes, shows the URL composition rule,
and explains how to point the buttons at a self-hosted mirror of
the manual.

The button affordance itself is small — a gray-background ASCII
`?` (`U+003F`, NOT the fullwidth-question-mark codepoint `U+FF1F`
or the question-mark emoji `U+2753`). It stays out of the way of
the form widgets but is reachable by mouse or by tab-keyboard
navigation.

## Three placement classes (Option C — selective placement)

The GUI does NOT render a `?` button next to every form widget.
Doing so would put roughly 200 buttons on screen across the four
tabs and produce visual chaos. Instead, the v1.0 cycle ships
**Option C** from the design plan: 91 total buttons split across
three classes, each class chosen because its semantics are not
self-evident from the widget alone.

### Class 1 — per-subcommand (28 buttons)

One `?` button immediately to the right of the subcommand-selector
ComboBox at the top of every form. Click → opens this manual at
the section for that subcommand. Example: with the **mnemonic** tab
active and **Convert (between formats)** selected, clicking the `?`
opens `…/manual-gui/#mnemonic-convert`.

```text
Pinned: mnemonic 0.13.0  |  subcommand: convert ▾  [?]
                                                   ↑
                         per-subcommand `?` button
```

There are 28 of these total: 10 (mnemonic) + 8 (md) + 5 (ms) + 5
(mk), one per `SubcommandSchema` entry across the four tabs.

### Class 2 — per-enumerated-flag (43 buttons)

One `?` button immediately to the right of every flag label whose
value-set is enumerated: `FlagKind::Dropdown`, `FlagKind::NodeValueComposite`,
`FlagKind::TaggedOrIndexed`. Click → opens this manual at the
section for that flag. The class includes any flag where the user
must pick from a list — `--from`, `--to`, `--network`, `--template`,
`--language`, etc.

```text
--from    [ phrase ▾ ]  [?]   ← per-NodeValueComposite `?` button
--to      [ ms1 ▾ ]     [?]   ← per-Dropdown `?` button
```

There are 43 of these total: 36 Dropdown occurrences + 6
NodeValueComposite occurrences + 1 TaggedOrIndexed occurrence
(the latter is `export-wallet --taproot-internal-key` with the
`nums` tag).

### Class 3 — per-repeating-field-flag (20 buttons)

One `?` button next to the label for every repeating flag — the
ones that take multiple values across multiple form rows. The
canonical example is the slot editor's `Slot rows:` label, which
covers the `--slot` repeating flag:

```text
Slot rows:  [?]                ← per-`--slot` `?` button
  @0  [ xpub ▾ ]  [?]   [ ... ]  ← also a per-Dropdown `?` on the
                                    subkey selector
  @1  [ fingerprint ▾ ]  [?]  [ ... ]
```

The 20 buttons in this class cover 11 distinct flag names:
`--slot`, `--ms1`, `--mk1`, `--md1`, `--to`, `--share`, `--group`
(in the mnemonic tab); `--key`, `--fingerprint` (in the md tab);
`--policy-id-stub`, `--from-md1` (in the mk tab). Repeating flags
get explicit per-flag buttons because cardinality, ordering, and
the optional `-` stdin-sentinel are non-obvious — the click-through
deep-link is the fastest way to surface those rules.

### What deliberately does NOT get a button

Plain-text fields (`--passphrase`, `--xpub`, `--policy-id-stub`
when used as a single value, etc.) and boolean checkboxes (`--json`,
`--force-chunked`, etc.) do NOT get a `?` button. Their hover
tooltip — populated from the upstream CLI's clap-derive `help`
string — is the help affordance. To find detailed prose on a
plain-text or boolean flag, navigate to its parent subcommand via
the per-subcommand `?` button and read down the page. The
`gui-help-icon-per-flag-affordance` FOLLOWUP in the GUI repo
tracks the option to extend `?` coverage to every flag if user
feedback surfaces gaps.

## URL composition rule

The URLs the buttons open follow a deterministic formula
implemented at `mnemonic-gui/src/help/url.rs`:

```text
base URL  = MANUAL_BASE_URL  (default https://bg002h.github.io/mnemonic-toolkit/manual-gui/)
subcmd    = base URL + "#" + tab-binary-name + "-" + kebab(subcommand)
flag      = subcmd + "-" + flag-name-stripped-of-leading-dashes
variant   = flag + "-" + kebab(variant-value)
```

Three example outputs:

| Click target | Generated URL |
|---|---|
| `mnemonic convert` (subcommand) | `…/manual-gui/#mnemonic-convert` |
| `mnemonic convert --from` (flag) | `…/manual-gui/#mnemonic-convert-from` |
| `mnemonic convert --from phrase` (variant) | `…/manual-gui/#mnemonic-convert-from-phrase` |

`kebab(...)` lowercases its input, replaces every non-alphanumeric
ASCII run with a single `-`, and strips trailing `-`. In practice
this is a no-op on real subcommand names (`bundle`, `convert`,
`seed-xor-split`, etc., all already kebab-shaped) — but it is
applied unconditionally for safety. Flag names are already
constrained to lowercase-ASCII plus `-`, so the only transformation
on a flag is stripping the leading `--`.

This formula is bidirectionally enforced. The GUI's
`tests/manual_anchor_coverage.rs` cell asserts that every URL the
GUI can generate corresponds to a real anchor in the rendered
manual; this manual's `tests/lint.sh::gui-schema-coverage` phase
asserts the converse, that every flag / variant / repeating-flag
in the GUI's schema has a matching `id="…"` in the rendered HTML.
Neither side can drift without breaking CI on at least one of the
two repos.

## Re-pointing the buttons at a self-hosted mirror

The `?` buttons resolve URLs at runtime against the constant
`MANUAL_BASE_URL` baked into the binary at compile time. If you
build the GUI from source for an air-gapped or self-hosted
environment, override the constant by setting the
`MNEMONIC_GUI_MANUAL_BASE_URL` environment variable at build time:

```sh
MNEMONIC_GUI_MANUAL_BASE_URL=https://docs.example.internal/m-format-gui/ \
  cargo install --locked --git https://github.com/bg002h/mnemonic-gui.git mnemonic-gui
```

The trailing `/` on the URL matters — the formula appends
`#<anchor>` directly without inserting a path separator. A runtime
flag (`--manual-base-url`) is NOT yet wired; that work is tracked
at FOLLOWUP `gui-manual-base-url-runtime-override` in the GUI
repo.

## When clicking a `?` does nothing

A few harmless reasons a click might not appear to do anything:

- Your default browser is closed and has not been launched in this
  session. Most operating systems start the browser before opening
  the URL, but the wake-up may take a few seconds.
- The GUI is running in a containerised desktop environment that
  blocks `xdg-open` (or the equivalent macOS / Windows hand-off).
  Run the GUI directly from your host desktop, or use the
  always-on `Preview:` line in the central form to copy the argv
  and consult this manual via your usual tooling.
- You are running a development build with no network access. The
  `?` buttons resolve to the `https://bg002h.github.io/...` URL
  by default; without network the page will not load until you
  re-build with `MNEMONIC_GUI_MANUAL_BASE_URL` pointed at a
  reachable mirror.

## End of tour

You have launched the GUI, navigated tabs, filled and run a
non-secret form, walked through the run-confirm modal with the
canonical test vector, and seen the three classes of `?`
help-icons. The next part (chapters 40 / 50 / 60 / 70) is the
per-tab reference — every subcommand of every CLI documented to
the depth the help-icons deep-link to.
