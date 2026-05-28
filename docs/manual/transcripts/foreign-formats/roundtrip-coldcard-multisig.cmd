cp $FIXTURES_DIR/coldcard-ms-2of3-p2wsh-with-xfp.txt .
$MNEMONIC_BIN import-wallet --format coldcard-multisig \
  --blob coldcard-ms-2of3-p2wsh-with-xfp.txt --json > envelope.json
$MNEMONIC_BIN export-wallet --from-import-json envelope.json \
  --format coldcard > coldcard_ms_re.txt
diff coldcard-ms-2of3-p2wsh-with-xfp.txt coldcard_ms_re.txt
