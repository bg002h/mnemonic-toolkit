PHRASE="abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about"
SHARES=$(printf %s "$PHRASE" | $MNEMONIC_BIN slip39 split --from phrase=- --group-threshold 1 --group 3,2 2>/dev/null)
mapfile -t SARR < <(printf "%s\n" "$SHARES" | grep .)
$MNEMONIC_BIN slip39 combine --share "${SARR[0]}" --share "${SARR[1]}" --to phrase --language english
