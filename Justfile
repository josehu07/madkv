# list top-level just recipes
default:
    @just --list --unsorted

# list all files as a tree
tree:
    tree . -I '.git|debug|target|ycsb'

# sync repo to a remote (experimental)
rsync remote:
    rsync -aP --delete \
        --exclude .git/ \
        --exclude .DS_Store \
        --exclude .vscode/ \
        --exclude "scripts/__pycache__/" \
        --exclude "debug/" \
        --exclude "target/" \
        --exclude "ycsb/" \
        . "{{remote}}:~/madkv"

# fetch a file/dir from remote (experimental)
fetch remote path:
    rsync -aP {{remote}}:~/madkv/{{path}} .

# common utils recipes
mod utils 'justmod/utils.just'

# project 1 recipes
mod p1 'justmod/proj1.just'
