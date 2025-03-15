#!/usr/bin/bash

id="$1"

mapper_name=madkv-"$id"
mount_point=/mnt/"$mapper_name"
dev_path=/dev/nvme"$id"n1

kill $(lsof -t "$mount_point")
umount "$mount_point"
dmsetup remove "$mapper_name"
dmsetup create "$mapper_name" --table "0 $(blockdev --getsz "$dev_path") linear $dev_path 0"
mkdir -p "$mount_point"
mount -O data=journal /dev/mapper/"$mapper_name" "$mount_point"
chmod -R ugoa+rw "$mount_point"
