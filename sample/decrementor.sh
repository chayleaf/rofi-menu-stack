. ./lib.sh
a="$1"
options "{prompt:'Decrementor >',message:$(val "Current value: $a"),selection:null}"
row "{text:'Decrement',pop:1,push:$(val "$(("$a" - 1))"),exec:$(val "sleep 5 && echo $(("$a" - 1)) > value.txt"),fork:true}"
row "{text:'Switch to incrementor',goto:'sample/incrementor.sh'}"
row "{text:'Close',return:null}"
