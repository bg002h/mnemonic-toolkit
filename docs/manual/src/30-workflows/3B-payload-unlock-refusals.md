# When the SeedHammer II refuses a payload

A Sealed Payload is what [`me seal`](#me-seal) writes and what the machine's Sealed Payload
program reads. Its secret section is encrypted, so opening it on the device asks
for the passphrase the host printed when it sealed. That much is the ordinary
path. (A `sysw` payload from [`me sysw pack`](#me-sysw-pack) is a different container, read by
Load Payload; nothing on this page happens to it.)

This section is about the other one: the machine reads the payload, derives the
key, and then refuses a record inside it. The wording it uses is precise, and
one of its instructions has a consequence the screen has no room to spell out.

## What the refusal says

The device names the record, its kind, and what to do:

```text
Record 1 is a hashlock preimage, not a seed. This payload cannot be unlocked here.
Nothing was opened. Remove that record -- and any others like it --
(records count from 0) on the host and seal the payload again.
```

Three parts of that are load-bearing.

**"Nothing was opened."** The refusal happens after the key derivation, not
before it, so the wait is real and the outcome is still nothing. No record from
the payload reached the screen, and none is held.

**"records count from 0."** The index is the position in the record list you
handed `me seal`, counting from zero. Read it as a 1-based position and you
delete the record above the one the machine refused, which on a mixed payload is
often a seed.

**"and any others like it."** The admission pass returns on the FIRST refused
record, so a payload carrying two of them is refused twice, once per round. The
index moves between rounds: removing record 1 renumbers everything after it. If
you apply the second round's number to your original listing, you delete a
record the machine never complained about.

## The part with no room on the screen: sealing again changes the passphrase

The instruction ends with "seal the payload again", and that is not the same
payload with one record missing. **Every seal generates a fresh passphrase.**
The card you wrote down for the first attempt does not open the second one.

Measured, sealing the same records twice:

```text
$ me seal --seal-secret --in records.txt --out payload.uf2
passphrase — write this down and store it APART from the machine:

    enable sunset squeeze segment wife now cattle media rhythm chat tomato solar

$ me seal --seal-secret --in records.txt --out payload.uf2
passphrase — write this down and store it APART from the machine:

    carry enemy reason execute breeze swarm session spare clean return claim erosion
```

So the sequence after a refusal is: remove the record on the host, seal again,
**write down the new passphrase**, and load the new payload. Destroy the old
card once the new payload has opened — keeping both invites typing the stale one
into a machine that will take about half a minute to tell you it was wrong.

## Which records get refused

The payload's public section takes wallet material; its secret section takes
seed material. A record that is neither is refused rather than carried, and the
kinds the machine names are the ones an operator is most likely to have put
there on purpose:

| the machine says | what you gave it | where it belongs |
|---|---|---|
| a hashlock preimage, not a seed | an `ms1` kind-`0x03` string | its own plate, cut from the composer or from `me sysw pack --pack-preimage` |
| not a format this machine reads | an `mr1` string, a card of a Refugium kit | a `sysw` payload: `me sysw pack` takes it as a public record, and `me seal` refuses it |
| not a format this machine reads | anything else the profile does not admit | nowhere in a payload; re-encode it |

A preimage is the common case and the reason the wording is careful: it is
bearer spend material, and a machine that quietly carried it inside a wallet
backup would be doing something the operator did not ask for.
