set working-directory := '..'

# list project 1 just recipes
default:
    @just --list p1 --unsorted

# build your executables in release mode
build:
    # FIXME: your build commands here

# run your KV store server
server listen="0.0.0.0:3777":
    # FIXME: your server run command here, listening on addr for new clients

# run you KV store client in stdin/out interface mode
client server="127.0.0.1:3777":
    # FIXME: your client run command here, connecting to server at addr

# run your KV store testcase 1 client
test1 server="127.0.0.1:3777":
    # FIXME: your testcase 1 client run command here
    #        for testcases, feel free to reuse 'just p1::client' if you
    #        designed your clients this way, or invoke different executables

# run your KV store testcase 2 client
test2 server="127.0.0.1:3777":
    # FIXME: your testcase 2 client run command here

# run your KV store testcase 3 client
test3 server="127.0.0.1:3777":
    # FIXME: your testcase 3 client run command here

# run your KV store testcase 4 client(s)
test4 server="127.0.0.1:3777":
    # FIXME: your testcase 4 client(s) run commands here
    #        for multi-client testcases, this recipe should run multiple
    #        concurrent client processes; you may write an additional runner
    #        utility or a scripting layer to manage multiple processes

# run your KV store testcase 5 client(s)
test5 server="127.0.0.1:3777":
    # FIXME: your testcase 5 client(s) run commands here

# kill all processes of your KV store system
kill:
    # FIXME: your kill commands here
    #        make sure that it kills all server, clients, and any extra helper
    #        processes of your system

# NOTE: feel free to add more recipes as you see fit...
#       also feel free to add extra parameters to the recipes as you see fit,
#       but don't change the existing parameters

# ensure a subdir under 'tmp/' exists
tmpdir subdir:
    mkdir -p "/tmp/madkv-p1/{{subdir}}"

# run a student-provided testcase (wip)
testcase num server="127.0.0.1:3777": (tmpdir "tests")
    just "p1::test{{num}}" > "/tmp/madkv-p1/tests/test{{num}}.log"
    just p1::kill

# run a fuzz testing scenario (wip)
fuzz nclis conflict server="127.0.0.1:3777": (tmpdir "fuzz")
    # TODO: to be announced
    just p1::kill

# run a YCSB benchmark workload (wip)
bench nclis wload server="127.0.0.1:3777": (tmpdir "bench")
    # TODO: to be announced
    just p1::kill

# generate .md report template from existing results (wip)
report:
    # TODO: to be announced
