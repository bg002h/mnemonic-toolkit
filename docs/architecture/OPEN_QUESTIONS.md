# Open questions and documentation drift

Questions still open for the constellation, each checked against the default branches
when this file was last edited. They are handoffs, not requirements. Delete an item
when it is resolved; link the change that resolved it in the commit, not here.

| Item and owner | What is unresolved | Consequence |
| --- | --- | --- |
| `mk` and `md` compatibility: `mk`, `md` | [`mk`'s](https://github.com/bg002h/mnemonic-key/tree/main/vendor/md-codec) `md-codec` dependency is older than `md`'s current release and rejects descriptor wire version 8 (non-NUMS taproot internal key), which [`md`](https://github.com/bg002h/descriptor-mnemonic/tree/main/crates/md-codec) and the toolkit accept | A new descriptor card does not necessarily decode in every sibling CLI. Decide when `mk` adopts a newer `md-codec` (it needs the miniscript git patch) |
| Toolkit codec lag: toolkit | The toolkit [pins older](../../crates/mnemonic-toolkit) `ms-codec` and `mk-codec` releases than the sibling repositories | The toolkit cannot read the hashlock-preimage `ms1` payload or the derived `mk1` chunk-set ID. Decide upgrades per contract |
| `me` consumer claim: `me`, Refugium | [`me`'s manifest](https://github.com/bg002h/mnemonic-engrave/tree/master/crates/me-cli) says a Refugium app links the `me` library, but the [wallet workspace](https://github.com/bg002h/refugium-wallet/tree/main) does not depend on it | Treat the dependency as intended, not real, until a consumer exists |
| Stale status prose: `ms`, toolkit, GUI | The `ms` [project notes](https://github.com/bg002h/mnemonic-secret/blob/master/CLAUDE.md) describe the original entropy-only scope with shares as future work; `wc-codec`'s [crate docs](../../crates/wc-codec) say RAID and the toolkit adapter are absent; the [GUI README](https://github.com/bg002h/mnemonic-gui/blob/master/README.md) says it does not interpret CLI output | Correct each when next maintaining it. The code is authoritative |
