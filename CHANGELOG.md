# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.3](https://github.com/Tetsuya81/minimum-viewer/compare/v0.1.2...v0.1.3) - 2026-03-02

### Fixed

- fix ci.yml

### Other

- Fix ci.yml
- Change the OS used in CI from macOS to Linux
- add self-hosted runner compose template

## [0.1.2](https://github.com/Tetsuya81/minimum-viewer/compare/v0.1.1...v0.1.2) - 2026-03-01

### Added

- Add markdown viewer command

### Fixed

- Validate selection is a file before opening markdown viewer

## [0.1.1](https://github.com/Tetsuya81/minimum-viewer/compare/v0.1.0...v0.1.1) - 2026-02-26

### Added

- Add cp command for copying files and directories ([#55](https://github.com/Tetsuya81/minimum-viewer/pull/55))

### Other

- bump release-plz binary version for git_only support
- configure release-plz for private git-only versioning
- harden release-plz workflow permissions and action pinning
- Fix Wayland clipboard fallback to try xclip/xsel when wl-copy fails
- Add yank command to copy selected entry path to clipboard
- Fix space-in-path parsing and add env_lock for test stability
- Fix cd with no args to go to home directory and prefill path on Tab completion
- Fix
