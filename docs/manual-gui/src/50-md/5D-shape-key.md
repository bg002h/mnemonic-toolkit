# `md shape-key` {#md-shape-key}

\index{md shape-key}Print a policy's **shape key**: the canonical,
coordinator-independent summary that md's coordinator verdict table is
keyed by. It has three fields — the template, the key partitions and
the key-path kind — separated by the control character `U+001F` (unit
separator), which most terminals draw as nothing and `cat -v` shows as
`^_`.

Two input modes, exactly one of which you fill: the `[PHRASES]`
positional (the `md1` strings of **one** card) **or**
[`--descriptor`](#md-shape-key-descriptor). The conditional-visibility
engine (`form::conditional::md_shape_key`) marks both **Required**
while neither has a value and greys `--descriptor` once the positional
has one.

The inputs are public, so the run-confirm modal does not fire — with
one exception: a descriptor holding a **private** key. See
[`--descriptor`](#md-shape-key-descriptor).

> **GUI form:** see [GUI Forms › md › shape-key](#gui-form-md-shape-key).

## `--descriptor` {#md-shape-key-descriptor}

Text widget. A multipath (`<0;1>`) BIP-380 descriptor instead of a
card: a wallet's own export, or the output of
[`md descriptor`](#md-descriptor). Xpub parent fingerprints are not
compared (a card cannot carry one).

**Public keys only.** md refuses a descriptor that holds an `xprv`
(`md: shape-key: not a descriptor: public keys must be …`, exit 2),
but by then the key would already have been on screen. So the GUI
checks the field's content: when any key-shaped part of it reads as a
private extended key (`xprv`, `tprv`, …), the field is masked, never
saved with the session, shown as `••••` in the Preview and the confirm
dialog, and the run asks for confirmation. The **Copy command**
buttons then read *"— reveals secret"*: md takes this field only on
the command line, so a copied command would carry the key.

## Positional `[PHRASES]`

One or more `md1` strings of **one** card. Repeating, optional at the
clap level; mutually exclusive with `--descriptor`.

## Worked example — the shape key of a single-sig descriptor

1. **md** tab; pick **Shape Key (card or descriptor -> shape key)**.
2. Leave the positional empty; paste into `--descriptor`:
   `wpkh([73c5da0a/84'/0'/0']xpub6CatWdiZi…VMrjPC7PW6V/<0;1>/*)`
   (the all-`abandon` BIP-84 account xpub).
3. **Run**.

stdout is one line, `wpkh(@0/<0;1>/*)` + `U+001F` + `[[[0]]][[0]]` +
`U+001F` + `NotTaproot`; stderr is empty; exit `0`.

## Refusals

| Trigger | Refusal |
|---|---|
| Neither `[PHRASES]` nor `--descriptor` | clap, exit 2 (the GUI marks both Required first) |
| Both | clap `cannot be used with`, exit 2 (the GUI greys `--descriptor`) |
| A private key in `--descriptor` | `md: shape-key: not a descriptor: public keys must be …`, exit 2 |
