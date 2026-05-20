mnemonic import-wallet --format bsms --blob multisig.bsms --json \
  | mnemonic export-wallet --from-import-json - --format bip388 \
  > policy.json
