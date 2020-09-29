# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## 🎯 [Unreleased]
### Added
- Proper error handling for exceeding carrier media capacity in core and cli

### Changed
- CHANGELOG.md follows now a [new format](https://keepachangelog.com/en/1.0.0/)
- CI pipeline is now on GitHub/Actions and travis-ci got retired

### Fixed
- Broken link in the CHANGELOG.md

## 🎼 [0.4.1] - 2020-09-23
### Fixed
- Broken link in the README.md of stegano-core

### Changed
- README.md of stegano-core is now better to understand

## 🎼 [0.4.0] - 2020-09-23
### Added
- **WAV Audio media file support - by [sassman], [pull/6]** 
    `stegano` has now support for input and output wav audio files (*.wav). This means that hiding secret messages and files are now **not** only possible for png image media files but also wav audio files in the same way. For example like this: 
    ```sh
    ❯ stegano hide \
        -i resources/plain/carrier-audio.wav \
        -d resources/secrets/Blah.txt \
           resources/secrets/Blah-2.txt \
        -o secret.wav
    ```
- **Arch Linux packages - by [orhun], [pull/10]**
    `stegano` can now be installed from available [AUR packages](https://aur.archlinux.org/packages/?O=0&SeB=b&K=stegano&outdated=&SB=n&SO=a&PP=50&do_Search=Go) using an [AUR helper](https://wiki.archlinux.org/index.php/AUR_helpers). For example like this:
    ```sh
    ❯ yay -S stegano
    ```

### Changed 
- **Update `stegano-core` to latest dependencies - by [sassman], [pull/2]**

[sassman]: https://github.com/sassman
[orhun]: https://github.com/orhun
[pull/6]: https://github.com/steganogram/stegano-rs/pull/6
[pull/10]: https://github.com/steganogram/stegano-rs/pull/10
[pull/2]: https://github.com/steganogram/stegano-rs/pull/2

## 🖼 [0.3.2] - 2020-07-10
### Added 
- Changelog based on conventional-commits

## 🖼 [0.3.1] - 2020-07-07
### Added
- Support for the legacy content version 2
- Support for hiding multiple files at once
- Benchmarks for image en/de-coding see `cargo bench`
- More end to end test cases
- Badges to README.md

### Changed
- Documentation of README.md is now better and more explicit
- Reduce redundancies of properties that can be extracted from `Cargo.toml`

### Fixed
- Typos in cli output and descriptions