. ./lib.sh
a=${1:-0}
options '{"prompt":"Decrementor >","message":"Current value: '"$a"'"}'
row '{"text":"Decrement","push":[null,"'"$(("$a" - 1))"'"],"exec":"sleep 5 && echo '"$(("$a" - 1))"' > value.txt","fork":true}'
row '{"text":"Switch to incrementor","jump":"sample/incrementor.sh"}'
row '{"text":"Close","jump":null}'
