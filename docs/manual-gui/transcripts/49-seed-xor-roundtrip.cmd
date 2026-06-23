PHRASE="abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about"
SHARES=$(printf %s "$PHRASE" | $MNEMONIC_BIN seed-xor split --from phrase=- --shares 3 --deterministic-from-master 2>/dev/null)
ARGS=(); while IFS= read -r s; do [ -n "$s" ] && ARGS+=(--share "phrase=$s"); done <<< "$SHARES"
$MNEMONIC_BIN seed-xor combine "${ARGS[@]}" --shares 3
