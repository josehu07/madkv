set working-directory := '..'

# list project 1 just recipes
default:
    @just --list p1 --unsorted

# ensure a subdir under 'tmp/' exists
tmpdir subdir:
    mkdir -p "/tmp/madkv-p1/{{subdir}}"

# build your executables in release mode
build:
    # FIXME: your build commands here

# run your KV store server
server listen="0.0.0.0:3777":
    # FIXME: your server run command here, listening on addr for new clients

# run you KV store client in stdin/out interface mode
client server="127.0.0.1:3777":
    # FIXME: your client run command here, connecting to server at addr

# run your KV store testcase client 1
test1 server="127.0.0.1:3777":
    # FIXME: your testcase client 1 run command here
    #        for testcases, feel free to reuse 'just p1::client' if you
    #        designed your clients this way, or invoke different executables

# run your KV store testcase client 2
test2 server="127.0.0.1:3777":
    # FIXME: your testcase client 2 run command here

# run your KV store testcase client 3
test3 server="127.0.0.1:3777":
    # FIXME: your testcase client 3 run command here

# run your KV store testcase client 4
test4 server="127.0.0.1:3777":
    # FIXME: your testcase client 4 run command here

# run your KV store testcase client 5
test5 server="127.0.0.1:3777":
    # FIXME: your testcase client 5 run command here

# kill all processes of your KV store system
kill:
    # FIXME: your kill commands here

# NOTE: feel free to add more recipes as you see fit...
#       also feel free to add extra parameters to the recipes as you see fit,
#       but don't change the existing parameters

# run fuzz testing (wip)
fuzz server="127.0.0.1:3777": (tmpdir "fuzz")
    # TODO: to be announced

# run YCSB benchmarking (wip)
bench server="127.0.0.1:3777": (tmpdir "bench")
    # TODO: to be announced

# generate .md report template from existing results (wip)
report:
    # TODO: to be announced
