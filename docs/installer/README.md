# Installer

## Windows
- User
  - Additional installation steps after download (and installation) if Smart App Control blocks the execution of the program
    - Open the Windows-Search and enter "Windows Powershell"
    - Select the Windows Powershell entry and click on "Run as Administrator"
    - Enter the following command and submit with `ENTER`
      ```powershell
      Unblock-File -Path 'C:\Program Files\Impulsor-3000\impulsor3000.exe'
      ```

- Developer
  - [NSIS Installer](../../installer/windows/README.md)

## Linux
- Developer
  - [AppImage](../../installer/linux/appimage/README.md)

## MacOS
- User
  - Additional installation steps after download if the app can't be launched. Please run the following steps in the terminal
    1. Switch to the directory where you downloaded and saved the `Impulsor-3000.app`
    2. Run `xattr -dr com.apple.quarantine Impulsor-3000.app`
    3. Re-sign the bundled libraries and executable first:
        - `codesign --force --sign - --timestamp=none Impulsor-3000.app/Contents/Resources/libs/lame/mac-arm64/libmp3lame.dylib`
        - `codesign --force --sign - --timestamp=none Impulsor-3000.app/Contents/Resources/libs/pdfium/mac-arm64/libpdfium.dylib`
        - `codesign --force --sign - --timestamp=none Impulsor-3000.app/Contents/MacOS/impulsor3000`
    4. Re-sign the app bundle:
        - `codesign --force --sign - --timestamp=none Impulsor-3000.app`
    5. (optional) Verify the result:
        - `codesign --verify --deep --strict --verbose=2 Impulsor-3000.app`
