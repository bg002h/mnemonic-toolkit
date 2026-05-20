mnemonic import-wallet --format specter \
  --blob specter-singlesig-p2wpkh.json --json \
  | mnemonic export-wallet --from-import-json - --format bitcoin-core \
  > core-import.json
