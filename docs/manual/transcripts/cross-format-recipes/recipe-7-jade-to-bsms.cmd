cp $FIXTURES_DIR/jade-multisig-2of3-p2wsh.json .
$MNEMONIC_BIN import-wallet --format jade \
  --blob jade-multisig-2of3-p2wsh.json --json \
  | $MNEMONIC_BIN export-wallet --from-import-json - --format bsms \
  > coordinator.bsms.txt
cat coordinator.bsms.txt
