#!/usr/bin/env bash
. ./lib.sh
a="$1"
b="$(python3 -c "print($a-1)")"
options "{prompt:'Decrementor >',message:$(val "Current value: $a"),selection:null}"
raw "{text:'Decrement',pop:1,push:$(val "$b"),exec:$(val "sleep 5 && echo $b > value.txt"),fork:true}"
raw "{text:'Switch to incrementor',goto:'sample/incrementor.sh'}"
raw "{text:'Close',return:null}"
