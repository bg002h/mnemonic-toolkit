mnemonic import-wallet --format sparrow \
  --blob sparrow-multisig-2of3-p2wsh-sortedmulti.json --json \
  | mnemonic export-wallet --from-import-json - --format bsms \
  > coordinator.bsms.txt
