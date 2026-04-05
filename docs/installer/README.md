# Installer

- Windows
  - [NSIS Installer](../../installer/windows/README.md)
- Linux
  - [AppImage](../../installer/linux/appimage/README.md)
- MacOS
  - Additional installation steps after download
    - ` codesign --force --deep --sign - Impulsor-3000.app`
      - Sometimes the signing of all libs and executables in it is also required
    - `xattr -dr com.apple.quarantine Impulsor-3000.app`
