# Stegano CLI

[![License: GPL v3](https://img.shields.io/badge/License-GPLv3-blue.svg)](https://www.gnu.org/licenses/gpl-3.0)
[![Build Status](https://travis-ci.org/steganogram/stegano-rs.svg?branch=main)](https://travis-ci.org/steganogram/stegano-rs)
[![codecov](https://codecov.io/gh/steganogram/stegano-rs/branch/main/graph/badge.svg)](https://codecov.io/gh/steganogram/stegano-rs)
[![LOC](https://tokei.rs/b1/github/steganogram/stegano-rs?category=code)](https://github.com/Aaronepower/tokei)

Implementation of [least significant bit steganography][lsb] for PNG images and WAV audio files in rust-lang.

Aims for compatibility to the [Stegano for windows version regarding image en-/decoding][1]

[lsb]: https://youtu.be/ARDhkujNXrY?t=705

## What is steganography?

In short, the art of hiding information in something (like a book, a image, a audio or even a video). 
[![speakerdeck](resources/plain/stegano-in-rust.jpeg)][slides]
You can find more information [on my slides][slides] or checkout [my talk on the rust meetup munich in june, 2020][meetup].

[slides]: https://speakerdeck.com/sassman/steganography-in-rust
[meetup]: https://youtu.be/ARDhkujNXrY?t=366

## Watch it in action

[![asciicast](https://asciinema.org/a/gNNTVcj6EZm3ZTaihZYoC7rfC.svg)](https://asciinema.org/a/gNNTVcj6EZm3ZTaihZYoC7rfC)

## Install

To install the stegano cli, you just need to run

```sh
❯ cargo install --force stegano-cli
```

(--force just makes it update to the latest `stegano-cli` if it's already installed)

*Note* the binary is called `stegano` (without `-cli`)

to verify if the installation went thru, you can run `which stegano` that should output similar to

```sh
$HOME/.cargo/bin/stegano
```

### AUR

`stegano` can be installed from available [AUR packages](https://aur.archlinux.org/packages/?O=0&SeB=b&K=stegano&outdated=&SB=n&SO=a&PP=50&do_Search=Go) using an [AUR helper](https://wiki.archlinux.org/index.php/AUR_helpers). For example,

```
yay -S stegano
```

## Usage

```sh
❯ stegano --help                                                                                                                                                                                                       10203  00:00:39 
Stegano CLI 0.4.0-beta1
Sven Assmann <sven.assmann.it@gmail.com>
Hiding secret data with steganography in PNG images and WAV audio files

USAGE:
    stegano [SUBCOMMAND]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

SUBCOMMANDS:
    help          Prints this message or the help of the given subcommand(s)
    hide          Hides data in PNG images and WAV audio files
    unveil        Unveils data from PNG images
    unveil-raw    Unveils raw data in PNG images
``` 

## Subcommands

### hide

```sh
❯ stegano hide --help                                                                                                                                                                                                  10204  00:00:44 
stegano-hide
Hides data in PNG images and WAV audio files

USAGE:
    stegano hide [FLAGS] [OPTIONS] --data <data file> --in <media file> --out <output image file>

FLAGS:
        --x-force-content-version-2    Experimental: enforce content version 2 encoding (for backwards compatibility)
    -h, --help                         Prints help information
    -V, --version                      Prints version information

OPTIONS:
    -d, --data <data file>           File(s) to hide in the image
    -i, --in <media file>            Media file such as PNG image or WAV audio file, used readonly.
    -m, --message <text message>     A text message that will be hidden
    -o, --out <output image file>    Final image will be stored as file
```

#### Example with am Image PNG file

Let's illustrate how to hide a file like `README.md`, inside an image `Base.png` and save it as `README.png`:

```sh
❯ stegano hide --data README.md --in resources/plain/carrier-iamge.png --out README.png
```

The final result is then contained in the image `README.png`.

*Pro TIP* you can hide multiple files at once

here I'm using the shorthand parameters (--data, -d), (--in, -i), (--out, -o)

```sh
❯ stegano hide \
  -i resources/plain/carrier-image.png \
  -d resources/secrets/Blah.txt \
     resources/secrets/Blah-2.txt \
  -o secret.png
```

*Hidden Feature* you can use a `.jpg` for input and save it as `.png`

```sh
❯ stegano hide \
  -i resources/NoSecret.jpg \
  -d resources/secrets/Blah.txt \
  -o secret.png
```

#### Example with an Audio WAV file

```sh
❯ stegano hide \
  -i resources/plain/carrier-audio.wav \
  -d resources/secrets/Blah.txt \
     resources/secrets/Blah-2.txt \
  -o secret.wav
```

#### Example Hide short messages

Now let's assume we want to hide just a little text message in `secret-text.png`. So we would run:

```sh
❯ stegano hide \
  -i resources/NoSecrets.jpg \
  -m 'This is a super secret message' \
  -o secret-text.png
```

### unveil

```sh
❯ stegano unveil --help                                                                                                                                                                                                10213  00:07:31 
stegano-unveil
Unveils data from PNG images

USAGE:
    stegano unveil --in <image source file> --out <output folder>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -i, --in <image source file>    Source image that contains secret data
    -o, --out <output folder>       Final data will be stored in that folder
```

#### Example unveil from an Image PNG file

Let's unveil the `README.md` that we've hidden (just above) in `README.png`

```sh
❯ stegano unveil --in README.png --out ./

❯ file README.md                                                                                                                                                                                                       10215  00:10:50 
README.md: UTF-8 Unicode text
```

#### Example unveil short messages

Now let's unveil the message from above `secret-text.png`. So we would run:

```sh
❯ stegano unveil \
  -i secret-text.png \
  -o message

❯ cat message/secret-message.txt
This is a super secret message
```

### unveil-raw

```sh
❯ stegano unveil-raw --help                                                                                                                                                                                            10216  00:10:58 
stegano-unveil-raw
Unveils raw data in PNG images

USAGE:
    stegano unveil-raw --in <image source file> --out <output file>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -i, --in <image source file>    Source image that contains secret data
    -o, --out <output file>         Raw data will be stored as binary file
```

#### Example unveil raw data

Let's unveil the raw data of the `README.md` that we've hidden just above in `README.png`

```sh
❯ stegano unveil-raw --in README.png --out README.bin
```

The file `README.bin` contains all raw binary data unfiltered decoded by the LSB decoding algorithm. 
That is for the curious people, and not so much interesting for regular usage.

## Contribute

To contribute to stegano-rs, please see open an issue on github and note that at 
this very time the architecture and the API is still in flux and might change. 

## License

- **[GNU GPL v3 license](https://www.gnu.org/licenses/gpl-3.0)**
- Copyright 2019 - 2020 © [Sven Assmann][2].

[1]: https://www.stegano.org/pages/downloads-en.html
[2]: https://www.d34dl0ck.me
[3]: https://en.wikipedia.org/wiki/Steganography