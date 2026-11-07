#! /bin/sh

export APPDIR="$(dirname "$(readlink -f "${0}")")"
cd $APPDIR
./impulsor3000
