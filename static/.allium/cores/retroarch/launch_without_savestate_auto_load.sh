#!/bin/sh
# RetroArch wrapper that disables savestate auto-load.  Derives the SD root
# from this script's location, same as launch.sh.
DIR=$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)/RetroArch
if [ -f "$DIR/.retroarch/retroarch.cfg" ]; then
    cp "$DIR/.retroarch/retroarch.cfg" "/tmp/retroarch.cfg"
    sed -i 's/savestate_auto_load = "true"/savestate_auto_load = "false"/g' "/tmp/retroarch.cfg"
fi
HOME="$DIR" LD_PRELOAD=libpadsp.so exec "$DIR/retroarch" -v -L "$DIR/.retroarch/cores/$1_libretro.so" "$2" -c /tmp/retroarch.cfg
