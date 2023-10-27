#!/usr/bin/env bash
. ./lib.sh
a="$1"
b="$(python3 -c "print($a-1)")"
options prompt "$(val "Decrementor >")" message "$(val "Current value: $a")" selection null
row "$(val "Decrement")" pop 1 push "$(val "$b")" exec "$(val "sleep 5 && echo $b > value.txt")" fork true
row "$(val "Switch to incrementor")" goto "$(val "sample/incrementor.sh")"
row "$(val "Close")" return null
