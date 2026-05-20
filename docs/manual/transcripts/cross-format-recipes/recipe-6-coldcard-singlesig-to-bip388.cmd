cp $FIXTURES_DIR/coldcard-singlesig-bip84-mainnet.json .
$MNEMONIC_BIN import-wallet --format coldcard \
  --blob coldcard-singlesig-bip84-mainnet.json --json \
  | $MNEMONIC_BIN export-wallet --from-import-json - --format bip388 \
  > policy.json
cat policy.json
