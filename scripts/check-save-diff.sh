#!/usr/bin/env bash

set -eo

i="$1"

file1="/tmp/01_$i.json"
file2="/tmp/02_$i.json"

cat $file1 | sort | jq -c . > /tmp/tmp_01.json
cat $file2 | sort | jq -c . > /tmp/tmp_02.json

meld /tmp/tmp_01.json /tmp/tmp_02.json
