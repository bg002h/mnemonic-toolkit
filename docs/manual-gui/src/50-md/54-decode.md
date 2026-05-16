# `md decode` {#md-decode}

Decode one or more `md1` strings into the canonical BIP-388
wallet-policy template — terser than [`md inspect`](#md-inspect),
intended for piping the template string into other tooling.

## `--json` {#md-decode-json}

Boolean. Emit JSON output. Default off.

## Positional `strings`

One or more `md1` strings. Required, repeating.

## Worked example

1. **md** tab; pick **Decode (md1 → template)**.
2. Paste the canonical first md1 into `strings`:

   ```text
   md1zsxdspqqqpm6jzzqqvqz6qu79mg9p2sgfff6p2eph8wftp5uf6gqnlgzqqqnymv0
   ```

3. Click **Run**.

The output panel renders the canonical template string on stdout
(typically `wpkh(@0/<0;1>/*)` for a single-sig BIP-84 bundle).
Use this output as input to [`md verify`](#md-verify) or to
external descriptor-aware tooling.

## Refusals

| Trigger | Refusal |
|---|---|
| No positional `strings` provided | clap-level `required` error |
| Any positional that does not parse as `md1` | md1-decode error per `md-cli` |
