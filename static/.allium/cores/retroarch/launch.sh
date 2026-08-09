#!/bin/sh
# RetroArch wrapper: derive the SD root from this script's location
# (.allium/cores/retroarch/launch.sh -> SD root), so it works regardless of
# the mount point case (/mnt/sdcard on Minime, /mnt/SDCARD upstream).
DIR=$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)/RetroArch
HOME="$DIR" LD_PRELOAD=libpadsp.so exec "$DIR/retroarch" -v -L "$DIR/.retroarch/cores/$1_libretro.so" "$2"
