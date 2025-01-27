# list top-level just recipes
default:
    @just --list

# list all files as a tree
tree:
    tree . --filelimit 20 -I '.git|debug|target|ycsb'

# sync repo to a remote (experimental)
rsync remote:
    rsync -aP --delete \
        --exclude .git/ \
        --exclude .DS_Store \
        --exclude .vscode/ \
        --exclude "*/__pycache__/" \
        --exclude "*/debug/" \
        --exclude "*/target/" \
        --exclude "ycsb/" \
        . "{{remote}}:~/madkv"

# common utils recipes
mod utils 'justmods/utils.just'

# project 1 recipes
mod p1 'justmods/proj1.just'
