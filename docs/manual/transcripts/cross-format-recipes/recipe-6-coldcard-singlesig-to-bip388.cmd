mnemonic import-wallet --format coldcard \
  --blob coldcard-singlesig-bip84-mainnet.json --json \
  | mnemonic export-wallet --from-import-json - --format bip388 \
  > policy.json
