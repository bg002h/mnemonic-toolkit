cp $FIXTURES_DIR/bsms-2line-sortedmulti-2of3.txt multisig.bsms
$MNEMONIC_BIN import-wallet --format bsms --blob multisig.bsms --json \
  | $MNEMONIC_BIN export-wallet --from-import-json - --format bip388 \
  > policy.json
cat policy.json
