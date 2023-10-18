. ./lib.sh
options "{'prompt':'Debug >','message':$(quote "Current stack: $*"),'fallback':{'pop':1,'push':null}}"
row "{'text':'!!!!Push \"a\"','push':'a'}"
row "{'text':'!!!!Pop','pop':1}"
row "{'text':'!!!!Close','return':null}"
