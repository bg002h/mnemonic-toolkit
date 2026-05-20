cp $FIXTURES_DIR/electrum-multisig-2of3-wsh.json .
$MNEMONIC_BIN import-wallet --format electrum \
  --blob electrum-multisig-2of3-wsh.json --json \
  | $MNEMONIC_BIN export-wallet --from-import-json - --format bsms \
  > coordinator.bsms.txt
cat coordinator.bsms.txt
