cp $FIXTURES_DIR/electrum-standard-bip84-mainnet.json .
$MNEMONIC_BIN import-wallet --format electrum \
  --blob electrum-standard-bip84-mainnet.json --json > envelope.json
$MNEMONIC_BIN export-wallet --from-import-json envelope.json \
  --format electrum > electrum_re.json
