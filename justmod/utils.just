set working-directory := '..'

# list utils just recipes
default:
    @just --list utils --unsorted

# build provided utilities (mode dev|rel)
build mode="rel":
    cargo build --workspace {{ if mode == "rel" { "-r" } else { "" } }}

# fetch the YCSB benchmark if haven't
ycsb:
    #!/usr/bin/env bash
    if [ -d "ycsb" ]; then
        echo "YCSB benchmark already fetched at ycsb/"
    else
        curl -O --location https://github.com/brianfrankcooper/YCSB/releases/download/0.17.0/ycsb-0.17.0.tar.gz
        tar xfvz ycsb-0.17.0.tar.gz
        mv ycsb-0.17.0 ycsb
        rm ycsb-0.17.0.tar.gz
    fi
