# Stegano Core Library

[![License: GPL v3](https://img.shields.io/badge/License-GPLv3-blue.svg)](https://www.gnu.org/licenses/gpl-3.0)
[![Build Status](https://travis-ci.org/steganogram/stegano-rs.svg?branch=main)](https://travis-ci.org/steganogram/stegano-rs)
[![codecov](https://codecov.io/gh/steganogram/stegano-rs/branch/main/graph/badge.svg)](https://codecov.io/gh/steganogram/stegano-rs)
[![LOC](https://tokei.rs/b1/github/steganogram/stegano-rs?category=code)](https://github.com/Aaronepower/tokei)

Implementation of [least significant bit steganography][lsb] for PNG images and WAV audio files in rust-lang.

Aims for compatibility to the [Stegano for windows version regarding image en-/decoding][1]
Rewrite of the core of the [original stegano.net tool][1]

[lsb]: https://youtu.be/ARDhkujNXrY?t=705

## Caution: No stable API yet
 
Changes on all API levels and the overall architecture can happen at any time.

## How to use it

[Checkout the stegano command line interface documentation][3]

## Architecture

![architecture overview](https://github.com/steganogram/stegano-rs/raw/main/stegano-core/docs/architecture-overview.png)

## License

- **[GNU GPL v3 license](https://www.gnu.org/licenses/gpl-3.0)**
- Copyright 2019 - 2020 © [Sven Assmann][2].

[1]: http://www.stegano.org
[2]: https://www.d34dl0ck.me
[3]: https://crates.io/crates/stegano-cli