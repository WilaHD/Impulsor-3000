#! /bin/sh

APPDIR="Impulsor-3000-x86_64.AppDir"

mkdir -p ${APPDIR}/libs/lame/linux-x64
mkdir -p ${APPDIR}/libs/pdfium/linux-x64

cp ../Impulsor-3000.desktop ${APPDIR}/Impulsor-3000.desktop 
cp ../../../libs/lame/linux-x64/libmp3lame.so   ${APPDIR}/libs/lame/linux-x64/libmp3lame.so
cp ../../../libs/pdfium/linux-x64/libpdfium.so  ${APPDIR}/libs/pdfium/linux-x64/libpdfium.so
cp ../../../target/release/impulsor3000         ${APPDIR}/impulsor3000
cp ../../../imgs/logo.png                       ${APPDIR}/impulsor3000.png
cp run.sh                                       ${APPDIR}/run.sh

chmod u+x ${APPDIR}/impulsor3000
chmod u+x ${APPDIR}/run.sh
cd ${APPDIR} && ln -s ./run.sh ./AppRun
