cp $FIXTURES_DIR/sparrow-singlesig-p2wpkh.json .
$MNEMONIC_BIN import-wallet --format sparrow \
  --blob sparrow-singlesig-p2wpkh.json --json > envelope.json
$MNEMONIC_BIN export-wallet --from-import-json envelope.json \
  --format sparrow > sparrow_re.json
diff <(jq -S . sparrow-singlesig-p2wpkh.json) <(jq -S . sparrow_re.json)
