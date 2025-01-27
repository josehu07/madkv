# list top-level just recipes
default:
    @just --list

# list all files as a tree
tree:
    tree . --filelimit 25

# sync repo to a remote (experimental)
rsync remote:
    rsync -aP --delete \
        --exclude .git/ \
        --exclude .DS_Store \
        --exclude .vscode/ \
        --exclude "*/__pycache__/" \
        --exclude "*/debug/" \
        --exclude "*/target/" \
        . "{{remote}}:~/madkv"

# common recipes
mod common 'justmods/common.just'

# project 1 recipes
mod p1 'justmods/proj1.just'
