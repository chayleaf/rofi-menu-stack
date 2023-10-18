. ./lib.sh
a=${1:-0}
options "{'prompt':'Incrementor >','message':$(quote "Current value: $a"),'selection':null}"
row "{'text':'Increment','pop':1,'push':$(quote "$(("$a" + 1))"),'exec':$(quote "sleep 5 && echo $(("$a" + 1)) > value.txt"),'fork':true}"
row "{'text':'Switch to decrementor','return':1,'jump':'sample/decrementor.sh'}"
row "{'text':'Close','return':null}"
