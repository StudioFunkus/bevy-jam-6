#!/bin/bash

dir=${PWD##*/}
tmp_path="/mnt/c/temp/$dir"

mkdir -p $tmp_path

rsync -rq . $tmp_path --exclude-from=.gitignore

cd $tmp_path

powershell.exe -Command "cargo run $@"
