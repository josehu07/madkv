# list top-level just recipes
default:
    @just --list --unsorted

# list all files as a tree
tree:
    tree . -I '.git|.venv|__pycache__|debug|target|ycsb|backer.*'

# sync repo to a remote (experimental)
rsync remote:
    rsync -aP --delete \
        --exclude .git/ \
        --exclude .venv/ \
        --exclude .DS_Store \
        --exclude .vscode/ \
        --exclude "sumgen/__pycache__/" \
        --exclude "debug/" \
        --exclude "target/" \
        --exclude "ycsb/" \
        --exclude "backer.*/" \
        --exclude "report/" \
        . "{{remote}}:~/madkv"

# fetch a file/dir from remote (experimental)
fetch remote path:
    rsync -aP {{remote}}:~/madkv/{{path}} .

# common utils recipes
mod utils 'justmod/utils.just'

# project 1 recipes
mod p1 'justmod/proj1.just'

# project 2 recipes
mod p2 'justmod/proj2.just'

# project 3 recipes
mod p3 'justmod/proj3.just'
