# GUI Forms reference {#gui-forms-reference}

This Part is the structural-render gallery for every GUI form in the m-format constellation. The per-subcommand chapters (Parts 4–7) keep their prose, per-flag reference, and worked example; the **generated form render** for each subcommand lives here, one section per form, reached by the `> **GUI form:**` cross-link at the top of each subcommand chapter. The four chapters split by tab: [`mnemonic`](#gui-forms-mnemonic) (32 forms), [`md`](#gui-forms-md) (10), [`ms`](#gui-forms-ms) (10), and [`mk`](#gui-forms-mk) (9).

## What a structural render shows

Each render is produced by the pinned headless `gui-render` (the egui-free `--no-default-features` build of `mnemonic-gui`) at manual build time and byte-pinned by the `verify-examples-gui` fidelity gate. Per form it lists every flag and positional with its **control kind** (text / number / checkbox / dropdown / path) and its **on-load default value**, in the order the GUI lays them out. Reading the tail of each row:

- `dropdown[a,b,c]` enumerates the allowed values; the trailing `-> <value>` is the on-load selection.
- `-> <unset>` / `-> <empty>` / `-> [ ] off` are the empty-number, empty-text, and unchecked-checkbox initial states.
- `[disabled]` marks a control the conditional-visibility engine has greyed out for the default field combination.
- `[ slot editor: N rows ]` appears for subcommands that accept the repeating `--slot` flag.

## Secret fields are masked

A flag the schema classifies secret renders its value as `<masked>` — never the real bytes — so every render reproduced in this manual carries no live key material. The worked examples in the subcommand chapters use only the canonical all-`abandon` BIP-39 test vector, which must never be funded.

## "(required)" is conditional-sourced

A `(required)` marker on a render comes from the conditional-visibility engine, not from a fixed clap attribute: such a flag is typically required only until a sibling input is supplied. A few forms mark a *set* of inputs `(required)` that resolve to an at-least-one or exactly-one constraint once you fill any one of them; the precise rule for each of those forms is stated in that form's own subcommand chapter (for example `mnemonic inspect`, `mnemonic repair`, and `ms encode`).
