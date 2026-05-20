cp $FIXTURES_DIR/bsms-shwsh-2of3.txt coordinator.bsms.txt
$MNEMONIC_BIN import-wallet --format bsms --blob coordinator.bsms.txt --json \
  | $MNEMONIC_BIN export-wallet --from-import-json - --format bitcoin-core \
  > core-import.json
cat core-import.json
