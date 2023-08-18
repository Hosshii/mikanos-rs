#!/bin/sh

set -eux

SCRIPT_DIR=$(cd $(dirname $0); pwd)
DISK_IMG=${SCRIPT_DIR}/disk.img
MNT_POINT=${SCRIPT_DIR}/mnt
EFI_FILE=${SCRIPT_DIR}/../../target/x86_64-unknown-uefi/release/bootloader.efi


CUR_DIR=$(pwd)

OVMF_CODE=${SCRIPT_DIR}/OVMF_CODE.fd
OVMF_VARS=${SCRIPT_DIR}/OVMF_VARS.fd

cargo build --release


mkdir -p $MNT_POINT
qemu-img create -f raw $DISK_IMG 200M
mkfs.fat -n 'MIKAN OS' -s 2 -f 2 -R 32 -F 32 $DISK_IMG

sudo mount -o loop $DISK_IMG $MNT_POINT
sudo mkdir -p $MNT_POINT/EFI/BOOT
sudo cp $EFI_FILE $MNT_POINT/EFI/BOOT/BOOTX64.EFI
sudo umount $MNT_POINT

qemu-system-x86_64 \
    -drive if=pflash,file=$OVMF_CODE \
    -drive if=pflash,file=$OVMF_VARS \
    -hda $DISK_IMG \
    -monitor stdio
