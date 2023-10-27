#!/usr/bin/env bash
. ./lib.sh
options prompt "$(val "Debug >")" message "$(val "Current stack: $*")" fallback "{push:null}"
row "$(val "!!!!Pop")" pop 1
row "$(val "!!!!Pop 2")" pop 2
row "$(val "!!!!Jump to self")" jump "$(val "sample/debug.sh")"
row "$(val "!!!!Return")" return 1
