# Stegano CLI

Implements LSB steganography for PNG image files in rust-lang.

Aims for compatibility to the [Stegano for windows version][1]

## Watch it in action

[![asciicast](https://asciinema.org/a/gNNTVcj6EZm3ZTaihZYoC7rfC.svg)](https://asciinema.org/a/gNNTVcj6EZm3ZTaihZYoC7rfC)

## Quick Start

### Install

To install the stegano cli, you just need to run 

```bash
cargo install --force stegano-cli
```

(--force just makes it update to the latest stegano-cli if it's already installed)

*Note* the binary is called `stegano` (without `-cli`)

to verify if the installation went thru, you can run `which stegano` that should output similar to 
```bash
$HOME/.cargo/bin/stegano
```

### Hide data

Let's assume we want to hide data of a file called `README.md`, into an image called `HelloWorld.png`, based on a image called `resources/with_attachment/Blah.txt.png`. So we would run:

```sh
stegano hide \
 --data README.md \
 --in resources/Base.png \
 --out README.png
```

or by cargo

```sh
cargo run -- hide \
 --data README.md \
 --in resources/Base.png \
 --out README.png
```

The final result is then contained in the image `README.png`.
 
### Unveil data

Let's unveil the `README.md` that we've hidden just above in `README.png`

```sh
stegano unveil \
 --in README.png \
 --out README-2.md
```

or by cargo

```sh
cargo run -- unveil \
 --in README.png \
 --out README-2.md
```

### Unveil Raw data

Let's unveil the raw data of the `README.md` that we've hidden just above in `README.png`

```sh
stegano unveil-raw \
 --in README.png \
 --out README.bin
```

The file `README.bin` contains all raw data unfiltered decoded by the LSB decoding algorithm. That is for the curious people, and not so much interesting.

## License

[![License: GPL v3](https://img.shields.io/badge/License-GPLv3-blue.svg)](https://www.gnu.org/licenses/gpl-3.0)

  - **[GNU GPL v3 license](https://www.gnu.org/licenses/gpl-3.0)**
  - Copyright 2019 © [Sven Assmann][2].

[1]: https://www.stegano.org/pages/downloads-en.html
[2]: https://www.d34dl0ck.me