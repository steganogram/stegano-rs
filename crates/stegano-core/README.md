# Stegano Core Library

[![License: GPL v3](https://img.shields.io/badge/License-GPLv3-blue.svg)](https://www.gnu.org/licenses/gpl-3.0)

[![Build](https://github.com/steganogram/stegano-rs/actions/workflows/build.yml/badge.svg)](https://github.com/steganogram/stegano-rs/actions/workflows/build.yml)

[![codecov](https://codecov.io/gh/steganogram/stegano-rs/branch/main/graph/badge.svg)](https://codecov.io/gh/steganogram/stegano-rs)

Implementation of [least significant bit steganography](https://youtu.be/ARDhkujNXrY?t=705) for PNG images and WAV audio files in rust-lang.

Aims for compatibility with [stegano for windows for image en-/decoding](https://apps.microsoft.com/detail/9p6xh5xr280v?ocid=webpdpshare)
Rewrite of the core of the [original stegano.net tool](http://web.archive.org/web/20160925025634/http://svenomenal.net/devel/steganoV2)

## Caution: No stable API yet

Changes on all API levels and the overall architecture can happen at any time.

## How to use it

[Checkout the stegano API docs](https://docs.rs/stegano-core/latest/stegano_core/)

## Architecture

![architecture overview](https://github.com/steganogram/stegano-rs/raw/main/stegano-core/docs/architecture-overview.png)

## License

- **[GNU GPL v3 license](https://www.gnu.org/licenses/gpl-3.0)**
- Copyright 2019 - 2024 © [Sven Kanoldt](https://www.d34dl0ck.me).
