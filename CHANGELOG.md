# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.4.0](https://github.com/WilaHD/Impulsor-3000/compare/v0.3.0...v0.4.0) - 2026-08-09

### Added

- settings: copy into new template

### Other

- add windows installation guide if blocked by smartscreen

## [0.3.0](https://github.com/WilaHD/Impulsor-3000/compare/v0.2.2...v0.3.0) - 2026-05-06

### Added

- show version in settings ([#9](https://github.com/WilaHD/Impulsor-3000/pull/9))
- show version in settings ([#7](https://github.com/WilaHD/Impulsor-3000/pull/7))
- app version with git tag and git sha ([#5](https://github.com/WilaHD/Impulsor-3000/pull/5))
- semantic releases
- persistent dark light auto mode settings
- add icons and drag and drop for files
- add selection boxes
- add selection boxes
- add macos appicon
- *(lame)* add linux lame
- *(installer max)* add .app packaging
- initial macos arm64 support
- macos dynlibs for pdfium and lame
- *(lame)* add workflow to build lame lib
- feat(settings) add some padding so content is not directly attached on the border
- refactoring
- *(start)* exit button
- *(validation)* add full pdf form validation
- *(pdfium)* update lib

### Fixed

- workflow ([#10](https://github.com/WilaHD/Impulsor-3000/pull/10))
- *(wordpress)* recognize new line breaks
- workflow
- width and icons
- windows mac
- icon font file path
- *(macos)* new website path
- *(macos)* remove macos lib duplicates
- *(macos)* lame libary paths
- *(installer mac)* add zpipping to .app packaging
- *(mac arm64)* remove build lame and fix lame path
- *(mac arm64)* remove build lame and fix lame path
- *(lame)* add temporary trigger
- *(workflow)* makensis install with choco
- *(workflow)* build binaries
- *(banner)* render text as svg path
- *(lock)* format file lock rs file
- *(ui.rs)* cleanup and add element lifetime annotation

### Other

- Feat/show version in settings ([#6](https://github.com/WilaHD/Impulsor-3000/pull/6))
- setup dev env config
- some dev stuff
- mac install docs
- new steps for installation on mac
- macos installer
- *(workflow)* update versions
- *(setting)* inital settings with inital pdf form validation
- Correct whatsapp url
# Change Log

## v0.2.2 - 2025-02-10

### Added

- Ability to convert ogg recordings (mostly from Telegram) and mp4 into mp3 files
- Button to open input file

## v0.2.1 - 2025-01-27

### Added

- Asynchronous file selector (prevents that gnome detects a frozen application)
- Welcome screen
- Lock screen while selecting files
- Ability to convert m4a recordings (mostly from WhatsApp) into mp3 files

### Changed

- Updated libraries
    - iced
    - pdfium
    - rfd
    - ... and many more
- Reorganized external libraries
- Renamed build application into `impulsor3000(.exe)`

### Fixed

- AppImage runs now with just `./Impulsor-3000_x86-64.AppImage`

## v0.2.0 - 2024-06-09

### Added

- Simple UI with the Iced Framework for a better usability
- Linux AppImage (doesn't run very well, a `--appimage-mount` is necessary)

### Changed

- Remove CLI Output (should be added later with full cli support)

## v0.1.0 - 2024-05-16

### Added
Initial release of the Impulsor 3000.


## [Template]

```
### Breaking
- Note

### Changed
- Note

### Added
- Note

### Fixed
- Note

```