# Stegano CLI

DISCLAIMER: not production ready, core changes can be done at any time.

Implements LSB steganography for PNG image files in rust-lang.

Aims for compatibility to the [Stegano for windows version][1]

## Watch it in action

[![asciicast](https://asciinema.org/a/gNNTVcj6EZm3ZTaihZYoC7rfC.svg)](https://asciinema.org/a/gNNTVcj6EZm3ZTaihZYoC7rfC)

## Usage: Hide

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

## Usage: Unveil

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

## Usage: Unveil Raw data

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

[1]: http://www.stegano.org
[2]: https://www.d34dl0ck.me