# Stegano Core Library

DISCLAIMER: not production ready, core changes can be done at any time.

Implements LSB steganography for PNG image files in rust-lang.

Rewrite of the core of the [originally stegano.net tool][1]

## How to use it

[Checkout Stegano CLI to see it in Action][3]

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

[1]: http://www.stegano.org
[2]: https://www.d34dl0ck.me
[3]: https://github.com/steganogram/cli.stegano.org