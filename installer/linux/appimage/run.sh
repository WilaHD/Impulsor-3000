#! /bin/sh

export APPDIR="$(dirname "$(readlink -f "${0}")")"
export LD_LIBRARY_PATH="${APPDIR}/libs/lame/linux-x64${LD_LIBRARY_PATH:+:${LD_LIBRARY_PATH}}"
cd $APPDIR
./impulsor3000
