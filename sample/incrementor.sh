. ./lib.sh
a=${1:-0}
options '{"prompt":"Incrementor >","message":"Current value: '"$a"'","selection":null}'
row '{"text":"Increment","push":[null,"'"$(("$a" + 1))"'"],"exec":"sleep 5 && echo '"$(("$a" + 1))"' > value.txt","fork":true}'
row '{"text":"Switch to decrementor","jump":"sample/decrementor.sh"}'
row '{"text":"Close","jump":null}'
