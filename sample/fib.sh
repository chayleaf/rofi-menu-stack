. ./lib.sh
i="$1"
nxt="$2"
cur="$3"
options "{'prompt':'Fibonacci >','message':$(quote "Value #$i: $cur")}"
row "{'text':'Next','pop':3,'push':[$(quote "$nxt"), $(quote "$(python3 -c "print($cur+$nxt)")"), $(quote "$(("$i" + 1))")]}"
row "{'text':'Switch to incrementor','pop':2,'goto':'sample/incrementor.sh'}"
row "{'text':'Close','return':null}"
