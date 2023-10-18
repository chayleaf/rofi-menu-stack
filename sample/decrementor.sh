#!/usr/bin/env bash
. ./lib.sh
a="$1"
b="$(python3 -c "print($a-1)")"
options "{prompt:'Decrementor >',message:$(val "Current value: $a"),selection:null}"
row "{text:'Decrement',pop:1,push:$(val "$b"),exec:$(val "sleep 5 && echo $b > value.txt"),fork:true}"
row "{text:'Switch to incrementor',goto:'sample/incrementor.sh'}"
row "{text:'Close',return:null}"
