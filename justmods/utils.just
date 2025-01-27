# list utils just recipes
default:
    @just --list utils

# build provided utilities (mode dev|rel)
build mode="rel":
    cargo build --workspace {{ if mode == "rel" { "-r" } else { "" } }}
