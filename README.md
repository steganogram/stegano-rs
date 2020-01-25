# Stegano App

[![Build Status](https://travis-ci.org/steganogram/cli.rs.svg?branch=master)](https://travis-ci.org/steganogram/cli.rs)
[![codecov](https://codecov.io/gh/steganogram/cli.rs/branch/master/graph/badge.svg)](https://codecov.io/gh/steganogram/cli.rs)

DISCLAIMER: DRAFT not production ready.

Implements LSB steganography for PNG image files in rust-lang. Aims for a command line version only.

Rewrite of the core of the [originally stegano.net tool][1]

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
cargo run --bin stegano -- hide \
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
cargo run --bin stegano -- unveil \
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

## Technical Details

### Stegano Header (Content Version 4)

| Size in Byte  |     Meaning      | Example Data |
|---------------|:----------------:|-------------:|
| 1             | Format Version   | 1, 2, 4      |
| 4 (BigEndian) | Payload Size (p) | 1634         |
|-------------------------------------------------|
| p             | Payload          |              |


### Architecture

Overview about the used components:

LSBCodec(Image):
 - impl Read
 - impl Write
 - PNG LSB

Message()
 - Header
 - Files
 - Text
 - of<LSBReader>
 - into([u8])

RawMessage(LSBReader)
 - all data from Reader
 - of<LSBReader>
 - into([u8])

## License

[![License: GPL v3](https://img.shields.io/badge/License-GPLv3-blue.svg)](https://www.gnu.org/licenses/gpl-3.0)

  - **[GNU GPL v3 license](https://www.gnu.org/licenses/gpl-3.0)**
  - Copyright 2019 © [Sven Assmann][2].

[1]: https://svenomenal.net/devel/steganoV2
[2]: https://www.d34dl0ck.me