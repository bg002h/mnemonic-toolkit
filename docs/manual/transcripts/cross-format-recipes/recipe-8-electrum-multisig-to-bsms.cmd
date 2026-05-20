mnemonic import-wallet --format electrum \
  --blob electrum-multisig-2of3-wsh.json --json \
  | mnemonic export-wallet --from-import-json - --format bsms \
  > coordinator.bsms.txt
