DESC='wsh(andor(pkh(@0),after(12000000),or_i(and_v(v:pkh(@1),older(4032)),and_v(v:pkh(@2),older(32768)))))'

$MNEMONIC_BIN bundle --network mainnet --account 0 \
  --descriptor "$DESC" \
  --language english \
  --slot '@0.phrase=abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about' \
  --slot '@1.phrase=legal winner thank year wave sausage worth useful legal winner thank yellow' \
  --slot '@2.phrase=letter advice cage absurd amount doctor acoustic avoid letter advice cage above' \
  --json > inheritance-bundle.json

$MNEMONIC_BIN verify-bundle --network mainnet --account 0 \
  --descriptor "$DESC" \
  --slot '@0.phrase=abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about' \
  --slot '@1.phrase=legal winner thank year wave sausage worth useful legal winner thank yellow' \
  --slot '@2.phrase=letter advice cage absurd amount doctor acoustic avoid letter advice cage above' \
  --bundle-json inheritance-bundle.json
