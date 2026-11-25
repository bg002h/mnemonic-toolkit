# usage: keys.sh COIN VER -> prints keyA keyB for seats at 270028'/COIN'/0' and /1'
A="abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about"
B="legal winner thank year wave sausage worth useful legal winner thank yellow"
python3 -I b32.py "$A" "270028'/$1'/0'" $2
python3 -I b32.py "$B" "270028'/$1'/1'" $2
