#!/usr/bin/env bash
. ./lib.sh
options "{prompt:'Debug >',message:$(val "Current stack: $*"),fallback:{push:null}}"
raw "{text:'!!!!Pop',pop:1}"
raw "{text:'!!!!Pop 2',pop:2}"
raw "{text:'!!!!Jump to self',jump:'sample/debug.sh'}"
raw "{text:'!!!!Return',return:1}"
