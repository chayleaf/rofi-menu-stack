#!/usr/bin/env bash
. ./lib.sh
i="$1"
nxt="$2"
cur="$3"
options prompt "$(val "Fibonacci >")" message "$(val "Value #$i: $cur")"
row "$(val "Next")" pop 3 push "$(vals "$nxt" "$(python3 -c "print($cur+$nxt)")" "$(("$i" + 1))")"
row "$(val "Switch to incrementor")" pop 2 goto "$(val "sample/incrementor.sh")"
row "$(val "Close")" return null
