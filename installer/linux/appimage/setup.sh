#! /bin/sh

APPDIR="Impulsor-3000-x86_64.AppDir"

mkdir -p ${APPDIR}/pdfium/linux-x64

cp ../../../pdfium/linux-x64/libpdfium.so ${APPDIR}/pdfium/linux-x64/libpdfium.so
cp ../../../target/release/Impulsor-3000 ${APPDIR}/Impulsor-3000
cp ../../../imgs/logo.png ${APPDIR}/Impulsor-3000.png

cd ${APPDIR} && ln -s Impulsor-3000 AppRun