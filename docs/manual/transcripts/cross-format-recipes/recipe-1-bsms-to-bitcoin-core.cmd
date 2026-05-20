mnemonic import-wallet --format bsms --blob coordinator.bsms.txt --json \
  | mnemonic export-wallet --from-import-json - --format bitcoin-core \
  > core-import.json
