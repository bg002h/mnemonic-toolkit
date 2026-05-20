mnemonic import-wallet --format jade \
  --blob jade-multisig-2of3-p2wsh.json --json \
  | mnemonic export-wallet --from-import-json - --format bsms \
  > coordinator.bsms.txt
