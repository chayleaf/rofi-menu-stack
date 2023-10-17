. ./lib.sh
options '{"prompt":"Debug >","message":"Current stack: '"$*"'","fallback":{"push":[null,0]}}'
row '{"text":"!!!!Push \"a\"","push":"a"}'
row '{"text":"!!!!Pop","push":null}'
row '{"text":"!!!!Close","jump":null}'
