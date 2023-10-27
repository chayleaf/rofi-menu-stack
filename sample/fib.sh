#!/usr/bin/env bash
. ./lib.sh
i="$1"
nxt="$2"
cur="$3"
options "{prompt:'Fibonacci >',message:$(val "Value #$i: $cur")}"
raw "{text:'Next',pop:3,push:$(vals "$nxt" "$(python3 -c "print($cur+$nxt)")" "$(("$i" + 1))")}"
raw "{text:'Switch to incrementor',pop:2,goto:'sample/incrementor.sh'}"
raw "{text:'Close',return:null}"
