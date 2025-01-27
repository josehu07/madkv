# list common just recipes
default:
    @just --list common

# build provided utilities (mode dev|rel)
build mode="dev":
    cargo build {{ if mode == "rel" { "-r" } else { "" } }}
