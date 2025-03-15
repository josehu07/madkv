#!/usr/bin/bash

id="$1"
mapper_name=madkv-"$id"

dmsetup wipe_table "$mapper_name"
