cp $FIXTURES_DIR/sparrow-multisig-2of3-p2wsh-sortedmulti.json .
$MNEMONIC_BIN import-wallet --format sparrow \
  --blob sparrow-multisig-2of3-p2wsh-sortedmulti.json --json \
  | $MNEMONIC_BIN export-wallet --from-import-json - --format bsms \
  > coordinator.bsms.txt
cat coordinator.bsms.txt
