DESC='wsh(andor(pkh(@0),after(12000000),or_i(and_v(v:pkh(@1),older(4032)),and_v(v:pkh(@2),older(32768)))))'
export SEED_0='abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about'
export SEED_1='legal winner thank year wave sausage worth useful legal winner thank yellow'
export SEED_2='letter advice cage absurd amount doctor acoustic avoid letter advice cage above'
$MNEMONIC_BIN bundle --network mainnet --account 0 \
  --descriptor "$DESC" \
  --language english \
  --slot '@0.phrase=@env:SEED_0' \
  --slot '@1.phrase=@env:SEED_1' \
  --slot '@2.phrase=@env:SEED_2'
