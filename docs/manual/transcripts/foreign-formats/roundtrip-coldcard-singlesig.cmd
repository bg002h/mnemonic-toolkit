cp $FIXTURES_DIR/coldcard-singlesig-bip84-mainnet.json .
$MNEMONIC_BIN import-wallet --format coldcard \
  --blob coldcard-singlesig-bip84-mainnet.json --json > envelope.json
$MNEMONIC_BIN export-wallet --from-import-json envelope.json \
  --format coldcard > coldcard_re.json
