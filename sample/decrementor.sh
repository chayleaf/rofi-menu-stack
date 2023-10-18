. ./lib.sh
a=${1:-0}
options "{'prompt':'Decrementor >','message':$(quote "Current value: $a"),'selection':null}"
row "{'text':'Decrement','pop':1,'push':$(quote "$(("$a" - 1))"),'exec':$(quote "sleep 5 && echo $(("$a" - 1)) > value.txt"),'fork':true}"
row "{'text':'Switch to incrementor','return':1,'jump':'sample/incrementor.sh'}"
row "{'text':'Close','return':null}"
