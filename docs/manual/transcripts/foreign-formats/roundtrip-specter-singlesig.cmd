cp $FIXTURES_DIR/specter-singlesig-p2wpkh.json .
$MNEMONIC_BIN import-wallet --format specter \
  --blob specter-singlesig-p2wpkh.json --json > envelope.json
$MNEMONIC_BIN export-wallet --from-import-json envelope.json \
  --format specter --wallet-name "Specter re-export" > specter_re.json
