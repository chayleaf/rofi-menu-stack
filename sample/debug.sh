. ./lib.sh
options "{prompt:'Debug >',message:$(val "Current stack: $*"),fallback:{push:null}}"
row "{text:'!!!!Pop',pop:1}"
row "{text:'!!!!Pop 2',pop:2}"
row "{text:'!!!!Jump to self',jump:'sample/debug.sh'}"
row "{text:'!!!!Return',return:1}"
