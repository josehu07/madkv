#!/usr/bin/bash

id="$1"
dev_path=/dev/nvme"$id"n1
mapper_name=madkv-"$id"
mount_point=/mnt/"$mapper_name"

kill $(lsof -t "$mount_point")
umount "$mount_point"
dmsetup remove "$mapper_name"
mkfs.ext4 -F -O fast_commit "$dev_path"
