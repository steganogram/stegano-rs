# Stegano App

[![Build Status](https://travis-ci.org/steganogram/cli.rs.svg?branch=master)](https://travis-ci.org/steganogram/cli.rs)
[![codecov](https://codecov.io/gh/steganogram/cli.rs/branch/master/graph/badge.svg)](https://codecov.io/gh/steganogram/cli.rs)

DISCLAIMER: DRAFT not production ready.

Implements LSB steganography for PNG image files in rust-lang. Aims for a command line version only.

Rewrite of the core of the [originally stegano.net tool][1]

## Usage: Hide

Let's assume we want to hide data of a file called `README.md`, into an image called `HelloWorld.png`, based on a image called `resources/HelloWorld_no_passwd_v2.x.png`. So we would run:

```sh
stegano hide \
 --data README.md \
 --in resources/HelloWorld_no_passwd_v2.x.png \
 --out HelloWorld.png
```

or by cargo

```sh
cargo run --bin stegano -- hide \
 --data README.md \
 --in resources/HelloWorld_no_passwd_v2.x.png \
 --out HelloWorld.png
```

TODO this is not yet implemented

Let's assume we want to hide a simple message like `Hello World!` in an existing png image (without touching the original image), so that we get a new png image that would contain the message.

So we would run:

```sh
cat "Hello World!" | stegano hide --in resources/HelloWorld_no_passwd_v2.x.png > HelloWorld.png
```

The final result is then contained in the image `HelloWorld.png`.

## Usage: Unveil

Let's unveil the `README.md` that we've hidden just above in `HelloWorld.png`

```sh
stegano unveil \
 --in HelloWorld.png \
 --out Secret.txt
```

or by cargo

```sh
cargo run --bin stegano -- unveil \
 --in HelloWorld.png \
 --out Secret.txt
```

## Usage: Unveil Raw data

Let's unveil the raw data of the `README.md` that we've hidden just above in `HelloWorld.png`

```sh
stegano unveil-raw \
 --in HelloWorld.png \
 --out Secret.bin
```

The file `Secret.bin` contains all raw data unfiltered decoded by the LSB decoding algorithm. That is for the curious people, and not so much interesting.

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

## TODOs

- investigate broken Version 2 content format
  - investigate ICSharpCode.SharpZipLib.Zip format that ends up being broken for unzip
    - http://mvdnes.github.io/rust-docs/zip-rs/zip/read/struct.ZipArchive.html
```sh
cargo run --bin stegano -- unveil-raw \
 --in resources/HelloWorld_no_passwd_with_attachment_v2.x.PNG \
 --out attachment.bin
dd if=attachment.bin of=attachment.zip bs=1 skip=1
zip -FF attachment.zip --out attachment-fixed.zip
unzip attachment-fixed.zip
```

- implement MessageContainer Version 2
  - first byte = 0x02
  - separate the parsing algorithms Version 1 and Version 2 somehow

## License

[![License: GPL v3](https://img.shields.io/badge/License-GPLv3-blue.svg)](https://www.gnu.org/licenses/gpl-3.0)

  - **[GNU GPL v3 license](https://www.gnu.org/licenses/gpl-3.0)**
  - Copyright 2019 © [Sven Assmann][2].

[1]: https://svenomenal.net/devel/steganoV2
[2]: https://www.d34dl0ck.me