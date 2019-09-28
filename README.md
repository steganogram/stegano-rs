# Stegano App

DISCLAIMER: DRAFT not ready to be used.

Implements LSB steganography for PNG image files in rust-lang. Aims for a command line version only.

Rewrite of the core of the [originally stegano.net tool][1]

## Hiding Things Usage

Let's assume we want to hide data of a file called `README.md`, into an image called `HelloWorld.png`, based on a image called `resources/HelloWorld_no_passwd_v2.x.png`. So we would run:

```bash
stegano hide \
 --data README.md \
 --in resources/HelloWorld_no_passwd_v2.x.png \
 --out HelloWorld.png
```

TODO this is not yet implemented

Let's assume we want to hide a simple message like `Hello World!` in an existing png image (without touching the original image), so that we get a new png image that would contain the message.

So we would run:

```bash
cat "Hello World!" | stegano hide --in resources/HelloWorld_no_passwd_v2.x.png > HelloWorld.png
```

The final result is then contained in the image `HelloWorld.png`.

## License

[![License: GPL v3](https://img.shields.io/badge/License-GPLv3-blue.svg)](https://www.gnu.org/licenses/gpl-3.0)

- **[GNU GPL v3 license](https://www.gnu.org/licenses/gpl-3.0)**
- Copyright 2019 © [Sven Assmann][2].

[1]: https://svenomenal.net/devel/steganoV2
[2]: https://www.d34dl0ck.me
