# Installer

- Windows
  - [NSIS Installer](../../installer/windows/README.md)
- Linux
  - [AppImage](../../installer/linux/appimage/README.md)
- MacOS
  - Additional installation steps after download if the app can't be launched. Please run the following steps in the terminal
    1. switch to the directory where you downloaded and saved the `Impulsor-3000.app`
    2. `xattr -dr com.apple.quarantine Impulsor-3000.app`
    3. Re-sign the bundled libraries and executable first:
       `codesign --force --sign - --timestamp=none Impulsor-3000.app/Contents/Resources/libs/lame/mac-arm64/libmp3lame.dylib`
       `codesign --force --sign - --timestamp=none Impulsor-3000.app/Contents/Resources/libs/pdfium/mac-arm64/libpdfium.dylib`
       `codesign --force --sign - --timestamp=none Impulsor-3000.app/Contents/MacOS/impulsor3000`
    4. Re-sign the app bundle:
       `codesign --force --sign - --timestamp=none Impulsor-3000.app`
    5. Verify the result:
       `codesign --verify --deep --strict --verbose=2 Impulsor-3000.app`
