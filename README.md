# Steganogram (cli)

DISCLAIMER: DRAFT not ready to be used.

Implements LSB steganography for PNG image files in rust-lang. Aims for a command line version only.

Rewrite of the core of the [originally stegano.net tool][1]

[1]: https://svenomenal.net/devel/steganoV2

## Usage

```bash
stegano.rs -m my-image.png -i secret-data.txt -o my-secret-image.png
```

## License

[![License: GPL v3](https://img.shields.io/badge/License-GPLv3-blue.svg)](https://www.gnu.org/licenses/gpl-3.0)

- **[GNU GPL v3 license](https://www.gnu.org/licenses/gpl-3.0)**
- Copyright 2019 © Sven Aßmann.
