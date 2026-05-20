cp $FIXTURES_DIR/specter-singlesig-p2wpkh.json .
$MNEMONIC_BIN import-wallet --format specter \
  --blob specter-singlesig-p2wpkh.json --json \
  | $MNEMONIC_BIN export-wallet --from-import-json - --format bitcoin-core \
  > core-import.json
cat core-import.json
